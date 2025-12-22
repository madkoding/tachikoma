//! =============================================================================
//! Voice Engine - Kokoro-82M TTS with Hybrid Effects
//! =============================================================================
//! Text-to-speech synthesis using Kokoro-82M ONNX model with dual-channel
//! processing: Channel A (human voice) + Channel B (robotic effects).
//! =============================================================================

use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use std::collections::HashMap;

/// =============================================================================
/// Voice Configuration
/// =============================================================================
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Path to Kokoro ONNX model file
    pub model_path: String,
    /// Path to voices NPZ file (voices.bin)
    pub voices_path: String,
    /// Voice name to use (e.g., "af_bella", "am_adam")
    pub voice_name: String,
    /// Sample rate for output audio (default: 24000)
    pub sample_rate: u32,
    /// Volume for human channel A (0.0 - 1.0)
    pub channel_a_volume: f32,
    /// Volume for robotic channel B (0.0 - 1.0)
    pub channel_b_volume: f32,
    /// Bitcrusher bit depth for robotic effect
    pub bitcrusher_bits: u8,
    /// Enable/disable robotic effects
    pub enable_effects: bool,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            model_path: "models/kokoro-v1.0.onnx".to_string(),
            voices_path: "models/voices.bin".to_string(),
            voice_name: "af_bella".to_string(),
            sample_rate: 24000,
            channel_a_volume: 1.0,
            channel_b_volume: 0.0,  // Disabled robotic effects
            bitcrusher_bits: 8,
            enable_effects: false,  // Clean voice by default
        }
    }
}

/// =============================================================================
/// Kokoro Phoneme Vocabulary
/// =============================================================================
/// Based on Kokoro's phoneme set for Spanish/English
const VOCAB: &[(&str, i64)] = &[
    ("$", 0), ("_", 1), (" ", 2), ("!", 3), ("'", 4), ("(", 5), (")", 6), (",", 7),
    ("-", 8), (".", 9), (":", 10), (";", 11), ("?", 12), ("a", 13), ("b", 14),
    ("c", 15), ("d", 16), ("e", 17), ("f", 18), ("g", 19), ("h", 20), ("i", 21),
    ("j", 22), ("k", 23), ("l", 24), ("m", 25), ("n", 26), ("o", 27), ("p", 28),
    ("q", 29), ("r", 30), ("s", 31), ("t", 32), ("u", 33), ("v", 34), ("w", 35),
    ("x", 36), ("y", 37), ("z", 38), ("æ", 39), ("ç", 40), ("ð", 41), ("ø", 42),
    ("ħ", 43), ("ŋ", 44), ("œ", 45), ("ǀ", 46), ("ǁ", 47), ("ǂ", 48), ("ʃ", 49),
    ("ʈ", 50), ("ʉ", 51), ("ʊ", 52), ("ʌ", 53), ("ʒ", 54), ("ʔ", 55), ("ʰ", 56),
    ("ˈ", 57), ("ˌ", 58), ("ː", 59), ("̃", 60), ("̩", 61), ("θ", 62), ("ᵻ", 63),
    ("ⁿ", 64), ("ɐ", 65), ("ɑ", 66), ("ɒ", 67), ("ɔ", 68), ("ɕ", 69), ("ə", 70),
    ("ɚ", 71), ("ɛ", 72), ("ɜ", 73), ("ɟ", 74), ("ɡ", 75), ("ɢ", 76), ("ɣ", 77),
    ("ɪ", 78), ("ɫ", 79), ("ɬ", 80), ("ɭ", 81), ("ɯ", 82), ("ɲ", 83), ("ɳ", 84),
    ("ɹ", 85), ("ɾ", 86), ("ʀ", 87), ("ʁ", 88), ("ʂ", 89), ("ʎ", 90), ("ʐ", 91),
    ("ʑ", 92), ("ˑ", 93), ("ɨ", 94), ("ɤ", 95), ("ɵ", 96), ("ʝ", 97),
];

/// =============================================================================
/// Voice Engine - Main TTS Engine
/// =============================================================================
pub struct VoiceEngine {
    session: Arc<Mutex<Option<Session>>>,
    config: VoiceConfig,
    voice_embedding: Arc<RwLock<Option<Vec<f32>>>>,
    initialized: Arc<RwLock<bool>>,
    vocab_map: std::collections::HashMap<String, i64>,
    voices_cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

impl VoiceEngine {
    /// Create a new VoiceEngine instance
    pub fn new(config: VoiceConfig) -> Self {
        let vocab_map: std::collections::HashMap<String, i64> = 
            VOCAB.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        
        Self {
            session: Arc::new(Mutex::new(None)),
            config,
            voice_embedding: Arc::new(RwLock::new(None)),
            initialized: Arc::new(RwLock::new(false)),
            vocab_map,
            voices_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the ONNX model and load voice embedding
    pub async fn initialize(&self) -> Result<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            debug!("VoiceEngine already initialized");
            return Ok(());
        }

        info!("🎤 Initializing Voice Engine with Kokoro-82M...");

        // Check if model file exists
        if !Path::new(&self.config.model_path).exists() {
            warn!("⚠️ Kokoro model not found at {}. Voice synthesis disabled.", self.config.model_path);
            warn!("📥 Download from: https://github.com/thewh1teagle/kokoro-onnx/releases");
            warn!("📥 Place kokoro-v1.0.onnx in the models/ directory");
            return Ok(());
        }

        // Load ONNX session
        let model_path = self.config.model_path.clone();
        let session_result = tokio::task::spawn_blocking(move || {
            Session::builder()?
                .with_optimization_level(GraphOptimizationLevel::Level3)?
                .with_intra_threads(4)?
                .commit_from_file(&model_path)
        })
        .await?;

        match session_result {
            Ok(session) => {
                *self.session.lock().await = Some(session);
                info!("✅ Kokoro ONNX model loaded successfully");
            }
            Err(e) => {
                warn!("⚠️ Failed to load Kokoro model: {}. Voice synthesis disabled.", e);
                return Ok(());
            }
        }

        // Load voices from NPZ file
        if Path::new(&self.config.voices_path).exists() {
            match tokio::fs::read(&self.config.voices_path).await {
                Ok(voices_data) => {
                    match self.parse_voices_npz(&voices_data) {
                        Ok(voices_map) => {
                            let voice_count = voices_map.len();
                            let voice_names: Vec<_> = voices_map.keys().cloned().collect();
                            *self.voices_cache.write().await = voices_map;
                            
                            // Set default voice embedding
                            if let Some(embedding) = self.voices_cache.read().await.get(&self.config.voice_name) {
                                *self.voice_embedding.write().await = Some(embedding.clone());
                                info!("✅ Voices loaded: {} voices. Using '{}'", voice_count, self.config.voice_name);
                            } else {
                                // Try first available voice
                                if let Some(first_name) = voice_names.first() {
                                    if let Some(embedding) = self.voices_cache.read().await.get(first_name) {
                                        *self.voice_embedding.write().await = Some(embedding.clone());
                                        info!("✅ Voices loaded: {} voices. Using '{}' (default not found)", voice_count, first_name);
                                    }
                                }
                            }
                            debug!("Available voices: {:?}", voice_names);
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse voices NPZ: {}. Using default.", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Failed to read voices file: {}. Using default.", e);
                }
            }
        } else {
            warn!("⚠️ Voices file not found: {}. Using default voice.", self.config.voices_path);
            warn!("📥 Download from: https://github.com/thewh1teagle/kokoro-onnx/releases");
        }

        *initialized = true;
        info!("🎙️ Voice Engine ready!");
        Ok(())
    }

    /// Check if the engine is ready for synthesis
    pub async fn is_ready(&self) -> bool {
        let session = self.session.lock().await;
        session.is_some()
    }

    /// Synthesize text to audio bytes (WAV format)
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        self.synthesize_with_voice(text, &self.config.voice_name.clone()).await
    }

    /// Synthesize text with a specific voice
    pub async fn synthesize_with_voice(&self, text: &str, voice: &str) -> Result<Vec<u8>> {
        if !self.is_ready().await {
            return Err(anyhow!("Voice engine not initialized or model not loaded"));
        }

        // Set the voice if different from current
        if voice != self.config.voice_name {
            self.set_voice(voice).await?;
        }

        debug!("🗣️ Synthesizing: '{}' with voice '{}'", text, voice);

        // Get language from voice prefix (first char)
        let lang = voice.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "a".to_string());

        // Convert text to tokens (via phonemes)
        let tokens = self.text_to_tokens(text, &lang);
        debug!("📝 Generated {} tokens", tokens.len());

        // Run inference
        let raw_audio = self.run_inference(&tokens).await?;
        debug!("🔊 Generated {} audio samples", raw_audio.len());

        // Apply hybrid effects
        let processed_audio = if self.config.enable_effects {
            self.apply_hybrid_effects(&raw_audio)
        } else {
            raw_audio
        };

        // Convert to WAV bytes
        let wav_bytes = self.audio_to_wav(&processed_audio)?;

        debug!("✅ Synthesized {} bytes of audio", wav_bytes.len());
        Ok(wav_bytes)
    }

    /// Get list of available voices
    pub async fn list_voices(&self) -> Vec<String> {
        self.voices_cache.read().await.keys().cloned().collect()
    }

    /// Set voice by name
    pub async fn set_voice(&self, voice_name: &str) -> Result<()> {
        let voices = self.voices_cache.read().await;
        if let Some(embedding) = voices.get(voice_name) {
            *self.voice_embedding.write().await = Some(embedding.clone());
            info!("🎤 Switched to voice: {}", voice_name);
            Ok(())
        } else {
            Err(anyhow!("Voice '{}' not found. Available: {:?}", 
                voice_name, voices.keys().collect::<Vec<_>>()))
        }
    }

    // ==========================================================================
    // Private Methods
    // ==========================================================================

    /// Parse voices NPZ file (ZIP containing .npy files for each voice)
    fn parse_voices_npz(&self, data: &[u8]) -> Result<HashMap<String, Vec<f32>>> {
        let mut voices = HashMap::new();
        let cursor = Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            // Extract voice name from filename (e.g., "af_bella.npy" -> "af_bella")
            if let Some(voice_name) = name.strip_suffix(".npy") {
                let mut npy_data = Vec::new();
                file.read_to_end(&mut npy_data)?;
                
                match self.parse_npy_voice(&npy_data) {
                    Ok(embedding) => {
                        debug!("Loaded voice '{}' with {} values", voice_name, embedding.len());
                        voices.insert(voice_name.to_string(), embedding);
                    }
                    Err(e) => {
                        warn!("Failed to parse voice '{}': {}", voice_name, e);
                    }
                }
            }
        }
        
        Ok(voices)
    }

    /// Parse a single .npy voice file
    /// NPY format: magic + version + header_len + header + data
    fn parse_npy_voice(&self, data: &[u8]) -> Result<Vec<f32>> {
        if data.len() < 10 {
            return Err(anyhow!("NPY file too small"));
        }

        // Check magic number "\x93NUMPY"
        if &data[0..6] != b"\x93NUMPY" {
            return Err(anyhow!("Invalid NPY magic number"));
        }

        // Version
        let _major = data[6];
        let _minor = data[7];

        // Header length (2 bytes for v1.0, 4 bytes for v2.0)
        let mut cursor = Cursor::new(data);
        cursor.seek(SeekFrom::Start(8))?;
        let header_len = cursor.read_u16::<LittleEndian>()? as usize;
        
        // Data starts after magic(6) + version(2) + header_len(2) + header
        let data_start = 10 + header_len;
        
        if data_start >= data.len() {
            return Err(anyhow!("Invalid NPY header"));
        }

        // Parse the data - shape is (510, 1, 256) so total = 130560 floats
        // But we only need first 256 values for the voice style
        let data_slice = &data[data_start..];
        let num_floats = data_slice.len() / 4;
        
        // For Kokoro, we take average or first slice - using first 256 values
        let style_dim = 256;
        let mut embedding = Vec::with_capacity(style_dim);
        let mut float_cursor = Cursor::new(data_slice);
        
        for _ in 0..style_dim.min(num_floats) {
            embedding.push(float_cursor.read_f32::<LittleEndian>()?);
        }

        Ok(embedding)
    }

    /// Parse voice embedding from binary file (legacy format)
    fn parse_voice_embedding(&self, data: &[u8]) -> Result<Vec<f32>> {
        if data.len() % 4 != 0 {
            return Err(anyhow!("Invalid voice embedding file size: {} bytes", data.len()));
        }

        let mut embedding = Vec::with_capacity(data.len() / 4);
        let mut cursor = Cursor::new(data);

        while cursor.position() < data.len() as u64 {
            embedding.push(cursor.read_f32::<LittleEndian>()?);
        }

        Ok(embedding)
    }

    /// Convert text to phonemes using espeak-ng
    fn text_to_phonemes(&self, text: &str, lang: &str) -> Result<String> {
        use std::process::Command;
        
        // Map voice prefix to espeak language code
        let espeak_lang = match lang {
            l if l.starts_with("a") => "en-us", // American English
            l if l.starts_with("b") => "en-gb", // British English
            l if l.starts_with("e") => "es",    // Spanish
            l if l.starts_with("f") => "fr",    // French
            l if l.starts_with("j") => "ja",    // Japanese
            l if l.starts_with("z") => "zh",    // Chinese
            l if l.starts_with("h") => "hi",    // Hindi
            l if l.starts_with("i") => "it",    // Italian
            l if l.starts_with("p") => "pt",    // Portuguese
            _ => "en-us",
        };

        let output = Command::new("espeak-ng")
            .args(["-q", "--ipa", "-v", espeak_lang, text])
            .output()
            .map_err(|e| anyhow!("Failed to run espeak-ng: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!("espeak-ng failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Replace newlines with spaces and normalize
        let phonemes = String::from_utf8_lossy(&output.stdout)
            .replace('\n', " ")
            .replace('\r', "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        debug!("📝 Phonemes: {}", phonemes);
        Ok(phonemes)
    }

    /// Convert text to token IDs for Kokoro (via phonemes)
    fn text_to_tokens(&self, text: &str, lang: &str) -> Vec<i64> {
        // First convert text to phonemes using espeak-ng
        let phonemes = match self.text_to_phonemes(text, lang) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to convert to phonemes: {}. Using raw text.", e);
                text.to_lowercase()
            }
        };

        let mut tokens: Vec<i64> = vec![0]; // Start token ($)
        let mut unknown_chars: Vec<char> = Vec::new();

        for ch in phonemes.chars() {
            let ch_str = ch.to_string();
            if let Some(&token_id) = self.vocab_map.get(&ch_str) {
                tokens.push(token_id);
            } else if ch.is_whitespace() {
                tokens.push(2); // space token
            } else if ch == '\'' || ch == 'ˈ' || ch == 'ˌ' {
                // Stress markers - try to map them
                if let Some(&token_id) = self.vocab_map.get("ˈ") {
                    tokens.push(token_id);
                }
            } else {
                // Track unknown characters for debugging
                if !unknown_chars.contains(&ch) {
                    unknown_chars.push(ch);
                }
                // Try to map similar characters
                let mapped = match ch {
                    'ɪ' | 'i' => self.vocab_map.get("i").or(self.vocab_map.get("ɪ")),
                    'ə' | 'e' => self.vocab_map.get("e").or(self.vocab_map.get("ə")),
                    'ʊ' | 'u' => self.vocab_map.get("u").or(self.vocab_map.get("ʊ")),
                    'ɔ' | 'o' => self.vocab_map.get("o").or(self.vocab_map.get("ɔ")),
                    'ɑ' | 'a' => self.vocab_map.get("a").or(self.vocab_map.get("ɑ")),
                    'ʃ' => self.vocab_map.get("ʃ"),
                    'ʒ' => self.vocab_map.get("ʒ"),
                    'θ' => self.vocab_map.get("θ"),
                    'ð' => self.vocab_map.get("ð"),
                    'ŋ' => self.vocab_map.get("ŋ"),
                    'ʧ' => self.vocab_map.get("t"),  // approximate tʃ
                    'ʤ' => self.vocab_map.get("d"),  // approximate dʒ
                    _ => None,
                };
                if let Some(&token_id) = mapped {
                    tokens.push(token_id);
                }
            }
        }

        if !unknown_chars.is_empty() {
            debug!("⚠️ Unknown phonemes: {:?}", unknown_chars);
        }

        tokens.push(0); // End token ($)
        debug!("🔢 Generated {} tokens from '{}' phonemes", tokens.len(), phonemes);
        tokens
    }

    /// Run ONNX inference to generate audio
    async fn run_inference(&self, tokens: &[i64]) -> Result<Vec<f32>> {
        let mut session_guard = self.session.lock().await;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| anyhow!("Session not initialized"))?;

        let voice_embedding_guard = self.voice_embedding.read().await;

        // Prepare input tensors using ort 2.0 API
        let seq_len = tokens.len();
        let tokens_vec: Vec<i64> = tokens.to_vec();
        
        // Voice/style embedding - Kokoro expects 256 values
        let style_dim = 256;
        let voice_vec: Vec<f32> = voice_embedding_guard
            .as_ref()
            .map(|v| {
                if v.len() >= style_dim {
                    v[..style_dim].to_vec()
                } else {
                    let mut padded = v.clone();
                    padded.resize(style_dim, 0.0);
                    padded
                }
            })
            .unwrap_or_else(|| vec![0.0f32; style_dim]);

        // Get input/output names from session
        let input_names: Vec<String> = session.inputs.iter()
            .map(|i| i.name.clone())
            .collect();
        let output_names: Vec<String> = session.outputs.iter()
            .map(|o| o.name.clone())
            .collect();

        debug!("Model inputs: {:?}", input_names);
        debug!("Model outputs: {:?}", output_names);

        // Create Tensors using ort 2.0 from_array with (shape, data) tuple
        // tokens: shape [1, seq_len]
        let tokens_tensor = Tensor::from_array(([1, seq_len], tokens_vec))?;
        // style: shape [1, 256]  
        let style_tensor = Tensor::from_array(([1, style_dim], voice_vec))?;
        // speed: shape [1]
        let speed_tensor = Tensor::from_array(([1_usize], vec![1.0f32]))?;

        // Run the model - ort::inputs! returns Vec directly, no ? needed
        let inputs = ort::inputs![
            "tokens" => tokens_tensor,
            "style" => style_tensor,
            "speed" => speed_tensor,
        ];
        
        let outputs = session.run(inputs)?;

        // Extract audio from first output
        let output_name = output_names.first()
            .map(|s| s.as_str())
            .unwrap_or("audio");
        
        let audio_output = outputs
            .get(output_name)
            .or_else(|| outputs.get("audio"))
            .ok_or_else(|| anyhow!("No audio output found in model response"))?;

        // Extract tensor data using try_extract_tensor which returns (shape, &[T])
        let (shape, audio_slice) = audio_output.try_extract_tensor::<f32>()?;
        debug!("Output audio shape: {:?}", shape);
        
        Ok(audio_slice.to_vec())
    }

    /// Apply hybrid dual-channel effects (human + robotic)
    fn apply_hybrid_effects(&self, audio: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(audio.len());
        let mut phase = 0.0f32;
        let ring_mod_freq = 150.0; // Hz for metallic effect
        let sample_rate = self.config.sample_rate as f32;

        for &sample in audio {
            // Channel A: Clean human voice (full volume)
            let channel_a = sample * self.config.channel_a_volume;

            // Channel B: Robotic effects
            let robotic = self.apply_robotic_effects(sample, phase);
            let channel_b = robotic * self.config.channel_b_volume;

            // Mix both channels
            let mixed = (channel_a + channel_b).clamp(-1.0, 1.0);
            output.push(mixed);

            // Update phase for ring modulation
            phase += ring_mod_freq / sample_rate;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }

        output
    }

    /// Apply robotic effects to a single sample
    fn apply_robotic_effects(&self, sample: f32, phase: f32) -> f32 {
        // 1. Bitcrusher effect (reduce bit depth)
        let bits = self.config.bitcrusher_bits as f32;
        let levels = 2.0f32.powf(bits);
        let crushed = (sample * levels).round() / levels;

        // 2. Soft clipping / saturation
        let saturated = (crushed * 2.0).tanh() * 0.7;

        // 3. Ring modulation for metallic/robotic timbre
        let ring_mod = (phase * std::f32::consts::TAU).sin();
        let modulated = saturated * (0.7 + 0.3 * ring_mod);

        modulated
    }

    /// Convert audio samples to WAV bytes
    fn audio_to_wav(&self, audio: &[f32]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.config.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::new(cursor, spec)?;

        for &sample in audio {
            // Convert f32 [-1.0, 1.0] to i16
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;
        Ok(buffer)
    }
}

impl Clone for VoiceEngine {
    fn clone(&self) -> Self {
        Self {
            session: self.session.clone(),
            config: self.config.clone(),
            voice_embedding: self.voice_embedding.clone(),
            initialized: self.initialized.clone(),
            vocab_map: self.vocab_map.clone(),
            voices_cache: self.voices_cache.clone(),
        }
    }
}

/// =============================================================================
/// Voice Synthesis Request DTO
/// =============================================================================
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VoiceSynthesisRequest {
    pub text: String,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default = "default_voice")]
    pub voice: String,
}

fn default_voice() -> String {
    "af_bella".to_string()
}

/// =============================================================================
/// Tests
/// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_to_tokens() {
        let engine = VoiceEngine::new(VoiceConfig::default());
        let tokens = engine.text_to_tokens("hello", "a");
        
        // Should have start token, phonemes, end token
        assert!(tokens.len() >= 3); // At minimum: start + something + end
        assert_eq!(tokens[0], 0); // Start ($)
        assert_eq!(tokens[tokens.len() - 1], 0); // End ($)
    }

    #[test]
    fn test_hybrid_effects() {
        let engine = VoiceEngine::new(VoiceConfig::default());
        let audio = vec![0.5f32; 100];
        let processed = engine.apply_hybrid_effects(&audio);
        
        assert_eq!(processed.len(), audio.len());
        // All samples should be within [-1, 1]
        for sample in &processed {
            assert!(*sample >= -1.0 && *sample <= 1.0);
        }
    }

    #[test]
    fn test_audio_to_wav() {
        let engine = VoiceEngine::new(VoiceConfig::default());
        let audio = vec![0.0f32; 1000];
        let wav = engine.audio_to_wav(&audio).unwrap();
        
        // WAV header is 44 bytes minimum
        assert!(wav.len() >= 44);
        // Check RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn test_robotic_effects() {
        let engine = VoiceEngine::new(VoiceConfig::default());
        
        // Test that effects don't exceed [-1, 1]
        let sample = 0.8f32;
        let result = engine.apply_robotic_effects(sample, 0.5);
        assert!(result >= -1.0 && result <= 1.0);
    }
}
