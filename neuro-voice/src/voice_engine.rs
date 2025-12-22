//! =============================================================================
//! Voice Engine Module
//! =============================================================================
//! Text-to-speech synthesis using Piper TTS binary.
//! Handles voice discovery, synthesis, and audio processing.
//! =============================================================================

use anyhow::{anyhow, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;
use std::io::Cursor;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use crate::audio_effects::{
    apply_pitch_shift, apply_robot_effect_chain, 
    mono_to_stereo_dual_voice, StereoBuffer,
    SAMPLE_RATE,
};
use crate::config::{EffectsConfig, PiperConfig, VoiceModel};

/// Piper outputs 16-bit PCM at 22050 Hz
#[allow(dead_code)]
const PIPER_OUTPUT_RATE: u32 = 22050;

/// Voice Engine state
pub struct VoiceEngine {
    config: PiperConfig,
    voices: Arc<RwLock<HashMap<String, VoiceModel>>>,
    piper_available: Arc<RwLock<bool>>,
}

impl VoiceEngine {
    /// Create a new VoiceEngine instance
    pub fn new(config: PiperConfig) -> Self {
        Self {
            config,
            voices: Arc::new(RwLock::new(HashMap::new())),
            piper_available: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the voice engine
    pub async fn initialize(&self) -> Result<()> {
        info!("🎙️ Initializing Voice Engine with Piper TTS...");

        // Check if Piper binary exists and is executable
        if !self.check_piper_installed().await {
            warn!(
                "⚠️ Piper binary not found at {}. Voice synthesis will fail.",
                self.config.binary_path.display()
            );
            return Ok(());
        }

        *self.piper_available.write().await = true;
        info!("✅ Piper TTS binary found and working");

        // Discover available voices
        self.discover_voices().await;

        let voices = self.voices.read().await;
        if voices.is_empty() {
            warn!("⚠️ No voice models found in {}. Please download Piper models.", 
                  self.config.models_dir.display());
        } else {
            info!("✅ Voice Engine ready with {} voices!", voices.len());
        }

        Ok(())
    }

    /// Check if Piper binary is installed and working
    async fn check_piper_installed(&self) -> bool {
        if !self.config.binary_path.exists() {
            error!("Piper binary not found at {}", self.config.binary_path.display());
            return false;
        }

        match Command::new(&self.config.binary_path)
            .arg("--help")
            .output()
            .await
        {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to run Piper: {}", e);
                false
            }
        }
    }

    /// Discover available voice models in the models directory
    async fn discover_voices(&self) {
        let mut voices = self.voices.write().await;
        voices.clear();

        if !self.config.models_dir.exists() {
            warn!(
                "Models directory not found: {}",
                self.config.models_dir.display()
            );
            return;
        }

        // Look for .onnx files
        if let Ok(entries) = std::fs::read_dir(&self.config.models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "onnx").unwrap_or(false) {
                    let voice_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    let config_path = path.with_extension("onnx.json");

                    if config_path.exists() {
                        info!("✅ Found voice: {}", voice_name);
                        voices.insert(
                            voice_name.clone(),
                            VoiceModel {
                                name: voice_name,
                                model_path: path,
                                config_path,
                            },
                        );
                    }
                }
            }
        }

        info!("📢 Total voices available: {}", voices.len());
    }

    /// Check if the engine is ready for synthesis
    pub async fn is_ready(&self) -> bool {
        let piper_ok = *self.piper_available.read().await;
        let has_voices = !self.voices.read().await.is_empty();
        piper_ok && has_voices
    }

    /// Get list of available voices
    pub async fn list_voices(&self) -> Vec<String> {
        self.voices.read().await.keys().cloned().collect()
    }

    /// Get the default voice name
    pub fn default_voice(&self) -> &str {
        &self.config.default_voice
    }

    /// Synthesize text to WAV audio
    ///
    /// # Arguments
    /// * `text` - Text to synthesize
    /// * `voice` - Voice name to use (optional, uses default if not provided)
    /// * `effects` - Audio effects configuration
    ///
    /// # Returns
    /// WAV audio bytes
    pub async fn synthesize(
        &self,
        text: &str,
        voice: Option<&str>,
        effects: &EffectsConfig,
    ) -> Result<Vec<u8>> {
        if !self.is_ready().await {
            return Err(anyhow!("Voice engine not ready. No Piper binary or voice models."));
        }

        let voices = self.voices.read().await;
        
        // Select voice
        let voice_name = voice.unwrap_or(&self.config.default_voice);
        let voice_model = if let Some(model) = voices.get(voice_name) {
            model
        } else if let Some((_, model)) = voices.iter().next() {
            warn!("Voice '{}' not found, using '{}'", voice_name, model.name);
            model
        } else {
            return Err(anyhow!("No voice models available"));
        };

        debug!(
            "🎤 Synthesizing {} chars with voice '{}'",
            text.len(),
            voice_model.name
        );

        // Run Piper synthesis (outputs at 22050 Hz)
        let raw_audio = self.run_piper_synthesis(text, voice_model, effects.speed).await?;

        debug!("🔊 Generated {} raw audio samples at 22050 Hz", raw_audio.len());

        // Upsample from 22050 Hz to 44100 Hz for better quality effects
        let upsampled_audio = Self::upsample_2x(&raw_audio);
        debug!("📈 Upsampled to {} samples at 44100 Hz", upsampled_audio.len());

        // Apply pitch shift if needed
        let pitched_audio = if effects.pitch_shift.abs() > 0.01 {
            debug!("🎵 Applying pitch shift: {} semitones", effects.pitch_shift);
            apply_pitch_shift(&upsampled_audio, effects.pitch_shift)
        } else {
            upsampled_audio
        };

        // Apply robot effect chain if enabled
        let processed_audio = if effects.robot_effect {
            debug!("🤖 Applying robot effect chain");
            apply_robot_effect_chain(
                &pitched_audio,
                effects.highpass_cutoff,
                effects.chorus_wet,
                effects.chorus_voices,
                effects.flanger_rate,
                effects.flanger_depth,
                effects.flanger_feedback,
                effects.flanger_wet,
                effects.ring_freq,
                effects.ring_wet,
                effects.reverb_wet,
                effects.reverb_decay,
                effects.reverb_room_size,
            )
        } else {
            pitched_audio
        };

        // Apply stereo dual-voice effect
        // Creates illusion of two voices alternating between L/R channels
        debug!("🔊 Applying stereo dual-voice effect");
        let stereo = mono_to_stereo_dual_voice(
            &processed_audio,
            8.0,   // 8ms delay between channels (Haas effect range)
            25.0,  // 25 cents detune for "second voice" (subtle pitch difference)
            0.8,   // 0.8 Hz swap rate (voices trade channels ~every 1.25 seconds)
        );

        // Convert to stereo WAV
        let wav_bytes = self.audio_to_wav_stereo(&stereo)?;

        debug!("✅ Generated {} bytes of stereo WAV audio at 44100 Hz", wav_bytes.len());

        Ok(wav_bytes)
    }

    /// Run Piper TTS synthesis via subprocess
    async fn run_piper_synthesis(
        &self,
        text: &str,
        voice: &VoiceModel,
        speed: f32,
    ) -> Result<Vec<f32>> {
        // Length scale is inverse of speed
        let length_scale = 1.0 / speed;

        let mut child = Command::new(&self.config.binary_path)
            .arg("--model")
            .arg(&voice.model_path)
            .arg("--config")
            .arg(&voice.config_path)
            .arg("--output-raw")
            .arg("--length-scale")
            .arg(length_scale.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Write text to stdin
        let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let text_bytes = text.as_bytes().to_vec();
        
        tokio::spawn(async move {
            if let Err(e) = stdin.write_all(&text_bytes).await {
                error!("Failed to write to Piper stdin: {}", e);
            }
        });

        // Wait for output with timeout
        let output = timeout(
            Duration::from_secs(self.config.timeout_secs),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| anyhow!("Piper synthesis timed out"))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Piper error: {}", stderr);
            return Err(anyhow!("Piper synthesis failed: {}", stderr));
        }

        // Convert raw PCM to f32 samples
        // Piper outputs 16-bit signed PCM at 22050 Hz mono
        let raw_bytes = output.stdout;
        let num_samples = raw_bytes.len() >> 1;  // Más rápido que / 2
        let mut samples = Vec::with_capacity(num_samples);
        
        // Constante pre-calculada para normalización (1/32768)
        const INV_32768: f32 = 1.0 / 32768.0;

        for chunk in raw_bytes.chunks_exact(2) {
            let sample_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
            // Multiplicar por recíproco es más rápido que dividir
            samples.push(sample_i16 as f32 * INV_32768);
        }

        Ok(samples)
    }

    /// Upsample audio from Piper rate (22050 Hz) to target rate (44100 Hz)
    /// Uses linear interpolation for simplicity - exact 2x ratio
    fn upsample_2x(samples: &[f32]) -> Vec<f32> {
        let len = samples.len();
        if len == 0 {
            return Vec::new();
        }
        
        let mut upsampled = Vec::with_capacity(len << 1);  // Más rápido que * 2
        let last_idx = len - 1;
        
        for i in 0..last_idx {
            let current = samples[i];
            let next = samples[i + 1];
            
            // Original sample
            upsampled.push(current);
            // Interpolated sample (midpoint): optimizado como (a + b) * 0.5
            upsampled.push((current + next) * 0.5);
        }
        
        // Último sample: sin interpolación (o duplicar)
        let last = samples[last_idx];
        upsampled.push(last);
        upsampled.push(last);
        
        upsampled
    }

    /// Convert f32 audio samples to WAV bytes (mono)
    #[allow(dead_code)]
    fn audio_to_wav(&self, audio: &[f32]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);

        let spec = WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::new(cursor, spec)?;

        for &sample in audio {
            // Convert f32 [-1.0, 1.0] to i16
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;
        Ok(buffer)
    }

    /// Convert stereo audio to WAV bytes
    fn audio_to_wav_stereo(&self, stereo: &StereoBuffer) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);

        let spec = WavSpec {
            channels: 2,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::new(cursor, spec)?;
        
        // Constantes pre-calculadas para conversión
        const SCALE: f32 = 32767.0;
        const MIN_VAL: f32 = -32768.0;
        const MAX_VAL: f32 = 32767.0;

        // Write interleaved stereo samples (L, R, L, R, ...)
        // Usar iteradores zip para mejor rendimiento
        for (&left, &right) in stereo.left.iter().zip(stereo.right.iter()) {
            let left_i16 = (left * SCALE).clamp(MIN_VAL, MAX_VAL) as i16;
            let right_i16 = (right * SCALE).clamp(MIN_VAL, MAX_VAL) as i16;
            writer.write_sample(left_i16)?;
            writer.write_sample(right_i16)?;
        }

        writer.finalize()?;
        Ok(buffer)
    }

    /// Synthesize text and return raw processed audio samples (f32)
    /// Used for encoding to other formats like Opus
    pub async fn synthesize_raw(
        &self,
        text: &str,
        voice: Option<&str>,
        effects: &EffectsConfig,
    ) -> Result<Vec<f32>> {
        if !self.is_ready().await {
            return Err(anyhow!("Voice engine not ready. No Piper binary or voice models."));
        }

        let voices = self.voices.read().await;
        
        // Select voice
        let voice_name = voice.unwrap_or(&self.config.default_voice);
        let voice_model = if let Some(model) = voices.get(voice_name) {
            model
        } else if let Some((_, model)) = voices.iter().next() {
            warn!("Voice '{}' not found, using '{}'", voice_name, model.name);
            model
        } else {
            return Err(anyhow!("No voice models available"));
        };

        debug!(
            "🎤 Synthesizing (raw) {} chars with voice '{}'",
            text.len(),
            voice_model.name
        );

        // Run Piper synthesis (outputs at 22050 Hz)
        let raw_audio = self.run_piper_synthesis(text, voice_model, effects.speed).await?;

        debug!("🔊 Generated {} raw audio samples at 22050 Hz", raw_audio.len());

        // Upsample from 22050 Hz to 44100 Hz for better quality effects
        let upsampled_audio = Self::upsample_2x(&raw_audio);
        debug!("📈 Upsampled to {} samples at 44100 Hz", upsampled_audio.len());

        // Apply pitch shift if needed
        let pitched_audio = if effects.pitch_shift.abs() > 0.01 {
            debug!("🎵 Applying pitch shift: {} semitones", effects.pitch_shift);
            apply_pitch_shift(&upsampled_audio, effects.pitch_shift)
        } else {
            upsampled_audio
        };

        // Apply robot effect chain if enabled
        let processed_audio = if effects.robot_effect {
            debug!("🤖 Applying robot effect chain");
            apply_robot_effect_chain(
                &pitched_audio,
                effects.highpass_cutoff,
                effects.chorus_wet,
                effects.chorus_voices,
                effects.flanger_rate,
                effects.flanger_depth,
                effects.flanger_feedback,
                effects.flanger_wet,
                effects.ring_freq,
                effects.ring_wet,
                effects.reverb_wet,
                effects.reverb_decay,
                effects.reverb_room_size,
            )
        } else {
            pitched_audio
        };

        // Apply stereo dual-voice effect and return interleaved samples
        debug!("🔊 Applying stereo dual-voice effect (raw)");
        let stereo = mono_to_stereo_dual_voice(
            &processed_audio,
            8.0,   // 8ms delay between channels
            25.0,  // 25 cents detune
            0.8,   // 0.8 Hz swap rate
        );

        // Return interleaved stereo (L, R, L, R, ...)
        Ok(stereo.interleave())
    }

    /// Get the current sample rate
    #[allow(dead_code)]
    pub fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
}

impl Clone for VoiceEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            voices: self.voices.clone(),
            piper_available: self.piper_available.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_to_wav() {
        let config = PiperConfig::default();
        let engine = VoiceEngine::new(config);
        
        let audio = vec![0.0f32; 1000];
        let wav = engine.audio_to_wav(&audio).unwrap();

        // WAV header is 44 bytes minimum
        assert!(wav.len() >= 44);
        // Check RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[tokio::test]
    async fn test_voice_engine_new() {
        let config = PiperConfig::default();
        let engine = VoiceEngine::new(config);
        
        // Should not be ready without initialization
        assert!(!engine.is_ready().await);
    }
}
