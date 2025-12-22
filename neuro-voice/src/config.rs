//! =============================================================================
//! Voice Service Configuration
//! =============================================================================
//! Configuration structures for the Piper TTS voice service.
//! =============================================================================

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8100),
        }
    }
}

/// Piper TTS configuration
#[derive(Debug, Clone)]
pub struct PiperConfig {
    /// Path to piper binary
    pub binary_path: PathBuf,
    /// Path to models directory
    pub models_dir: PathBuf,
    /// Default voice name
    pub default_voice: String,
    /// Max text length
    pub max_text_length: usize,
    /// Synthesis timeout in seconds
    pub timeout_secs: u64,
}

impl Default for PiperConfig {
    fn default() -> Self {
        Self {
            binary_path: PathBuf::from(
                std::env::var("PIPER_BIN").unwrap_or_else(|_| "/app/piper/piper".to_string())
            ),
            models_dir: PathBuf::from(
                std::env::var("MODELS_DIR").unwrap_or_else(|_| "/app/models".to_string())
            ),
            default_voice: std::env::var("DEFAULT_VOICE")
                .unwrap_or_else(|_| "es_MX-claude-high".to_string()),
            max_text_length: 5000,
            timeout_secs: 30,
        }
    }
}

/// Audio effects configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    /// Speech rate multiplier (1.0 = normal)
    pub speed: f32,
    /// Pitch shift in semitones
    pub pitch_shift: f32,
    /// Enable robotic effect chain
    pub robot_effect: bool,
    
    // High-pass filter settings
    pub highpass_cutoff: f32,
    
    // Chorus settings
    pub chorus_wet: f32,
    pub chorus_voices: usize,
    
    // Flanger settings  
    pub flanger_rate: f32,
    pub flanger_depth: f32,
    pub flanger_feedback: f32,
    pub flanger_wet: f32,
    
    // Ring modulation settings
    pub ring_freq: f32,
    pub ring_wet: f32,
    
    // Reverb settings
    pub reverb_wet: f32,
    pub reverb_decay: f32,
    pub reverb_room_size: f32,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        // =======================================================================
        // TACHIKOMA VOICE PROFILE
        // =======================================================================
        // Características de la voz Tachikoma de Ghost in the Shell:
        // - Voz aguda/infantil (high-pitched, childlike)
        // - Personalidad curiosa, alegre y activa
        // - Tono metálico pero no amenazante
        // - Como si hablara desde dentro de un cuerpo robótico pequeño
        // =======================================================================
        Self {
            // Velocidad normal para máxima claridad
            speed: 1.0,
            
            // Pitch 0: la cadena de efectos aplica +1 internamente a la capa procesada
            // La voz principal queda sin pitch shift para claridad
            pitch_shift: 0.0,
            
            // Activar efectos robóticos (doble capa: dry + wet con delay)
            robot_effect: true,
            
            // High-pass según spec (150-200Hz) - elimina graves innecesarios
            highpass_cutoff: 180.0,
            
            // Chorus sutil (15-20% mix)
            chorus_wet: 0.18,
            chorus_voices: 2,
            
            // Flanger muy sutil (movimiento ligero, como circuitos)
            flanger_rate: 0.3,
            flanger_depth: 0.08,
            flanger_feedback: 0.1,
            flanger_wet: 0.08,
            
            // Ring modulation: se mantiene por compatibilidad, pero la cadena actual
            // prioriza vocoder. Si se vuelve a usar, estos valores son sutiles.
            ring_freq: 80.0,
            ring_wet: 0.0,
            
            // Reverb muy ligera (5-10% mix, room pequeño 10-15%, decay 0.3-0.5s)
            reverb_wet: 0.08,
            reverb_decay: 0.4,
            reverb_room_size: 0.12,
        }
    }
}

/// Voice model information
#[derive(Debug, Clone, Serialize)]
pub struct VoiceModel {
    pub name: String,
    pub model_path: PathBuf,
    pub config_path: PathBuf,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub piper: PiperConfig,
    pub effects: EffectsConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig::default(),
            piper: PiperConfig::default(),
            effects: EffectsConfig::default(),
        }
    }
}
