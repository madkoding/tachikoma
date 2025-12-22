//! =============================================================================
//! Voice Engine - HTTP Client for Voice Service
//! =============================================================================
//! Proxies TTS requests to the Docker voice-service container running Kokoro.
//! =============================================================================

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// =============================================================================
/// Voice Configuration
/// =============================================================================
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// URL of the voice service
    pub service_url: String,
    /// Default voice name
    pub voice_name: String,
    /// Sample rate (for reference, actual rate from service)
    pub sample_rate: u32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            service_url: std::env::var("VOICE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8100".to_string()),
            voice_name: "af_bella".to_string(),
            sample_rate: 24000,
        }
    }
}

/// =============================================================================
/// Voice Service Status
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceStatus {
    pub enabled: bool,
    pub model: String,
    pub sample_rate: u32,
    pub available_voices: Vec<String>,
    pub current_voice: String,
}

/// =============================================================================
/// Synthesis Request
/// =============================================================================
#[derive(Debug, Clone, Serialize)]
pub struct SynthesizeRequest {
    pub text: String,
    pub voice: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
}

#[allow(dead_code)]
fn default_speed() -> f32 {
    1.0
}

/// =============================================================================
/// Voice Synthesis Request DTO (for API handlers)
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceSynthesisRequest {
    pub text: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub streaming: bool,
    #[serde(default = "default_voice")]
    #[allow(dead_code)]
    pub voice: String,
}

fn default_voice() -> String {
    "af_bella".to_string()
}

/// =============================================================================
/// Voice Engine - HTTP Client
/// =============================================================================
pub struct VoiceEngine {
    client: Client,
    config: VoiceConfig,
    cached_voices: Arc<tokio::sync::RwLock<Option<Vec<String>>>>,
}

impl VoiceEngine {
    /// Create a new VoiceEngine instance
    pub fn new(config: VoiceConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            cached_voices: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Initialize the voice engine (check connection to service)
    pub async fn initialize(&self) -> Result<()> {
        info!("🎙️ Connecting to Voice Service at {}", self.config.service_url);
        
        match self.check_health().await {
            Ok(_) => {
                info!("✅ Voice Service connected!");
                // Cache available voices
                if let Ok(status) = self.get_status().await {
                    let mut cache = self.cached_voices.write().await;
                    *cache = Some(status.available_voices);
                }
                Ok(())
            }
            Err(e) => {
                warn!("⚠️ Voice Service not available: {}. Voice features disabled.", e);
                Ok(()) // Don't fail startup if voice service is unavailable
            }
        }
    }

    /// Check health of voice service
    async fn check_health(&self) -> Result<()> {
        let url = format!("{}/health", self.config.service_url);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Voice service unhealthy: {}", response.status()))
        }
    }

    /// Get voice service status
    pub async fn get_status(&self) -> Result<VoiceStatus> {
        let url = format!("{}/status", self.config.service_url);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let status: VoiceStatus = response.json().await?;
            Ok(status)
        } else {
            Err(anyhow!("Failed to get status: {}", response.status()))
        }
    }

    /// Check if the engine is ready for synthesis
    pub async fn is_ready(&self) -> bool {
        self.check_health().await.is_ok()
    }

    /// Get current voice name
    pub fn current_voice(&self) -> &str {
        &self.config.voice_name
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Synthesize text to audio bytes (WAV format)
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        self.synthesize_with_voice(text, &self.config.voice_name.clone()).await
    }

    /// Synthesize text with a specific voice
    pub async fn synthesize_with_voice(&self, text: &str, voice: &str) -> Result<Vec<u8>> {
        let url = format!("{}/synthesize", self.config.service_url);
        
        let request = SynthesizeRequest {
            text: text.to_string(),
            voice: voice.to_string(),
            speed: 1.0,
        };

        debug!("🗣️ Requesting synthesis: '{}' with voice '{}'", text, voice);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            debug!("✅ Received {} bytes of audio", bytes.len());
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!("Synthesis failed: {}", error_text))
        }
    }

    /// Get list of available voices
    pub async fn list_voices(&self) -> Vec<String> {
        // Return cached voices if available
        if let Some(voices) = self.cached_voices.read().await.as_ref() {
            return voices.clone();
        }

        // Fetch from service
        match self.get_status().await {
            Ok(status) => {
                let mut cache = self.cached_voices.write().await;
                *cache = Some(status.available_voices.clone());
                status.available_voices
            }
            Err(_) => Vec::new()
        }
    }

    /// Set voice by name (for compatibility)
    pub async fn set_voice(&self, _voice_name: &str) -> Result<()> {
        // Voice is passed per-request, this is a no-op
        Ok(())
    }
}

impl Clone for VoiceEngine {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            cached_voices: self.cached_voices.clone(),
        }
    }
}

/// =============================================================================
/// Tests
/// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VoiceConfig::default();
        assert_eq!(config.voice_name, "af_bella");
        assert_eq!(config.sample_rate, 24000);
    }

    #[test]
    fn test_synthesize_request() {
        let request = SynthesizeRequest {
            text: "Hello".to_string(),
            voice: "af_bella".to_string(),
            speed: 1.0,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("af_bella"));
    }
}
