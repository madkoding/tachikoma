//! =============================================================================
//! HTTP Handlers Module
//! =============================================================================
//! Axum HTTP handlers for the voice service API endpoints.
//! =============================================================================

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum::body::Body;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::audio_effects::SAMPLE_RATE;
use crate::config::EffectsConfig;
use crate::opus_encoder::{encode_pcm_to_opus, resample_stereo, OPUS_SAMPLE_RATE};
use crate::text_cleaner::{clean_text_for_speech, split_into_sentences};
use crate::voice_engine::VoiceEngine;

/// Application state shared across handlers
pub struct AppState {
    pub voice_engine: Arc<VoiceEngine>,
}

// =============================================================================
// Request/Response DTOs
// =============================================================================

/// Synthesis request
#[derive(Debug, Deserialize)]
pub struct SynthesizeRequest {
    pub text: String,
    #[serde(default = "default_voice")]
    pub voice: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default = "default_pitch_shift")]
    pub pitch_shift: f32,
    #[serde(default = "default_robot_effect")]
    pub robot_effect: bool,
}

fn default_voice() -> String {
    "es_MX-claude-high".to_string()
}

fn default_speed() -> f32 {
    1.05  // Tachikoma: ligeramente rápido (entusiasmo)
}

fn default_pitch_shift() -> f32 {
    6.0  // Tachikoma: voz aguda/infantil (+6 semitonos)
}

fn default_robot_effect() -> bool {
    true
}

/// Status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub enabled: bool,
    pub engine: String,
    pub sample_rate: u32,
    pub available_voices: Vec<String>,
    pub default_voice: String,
}

/// Health response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub piper_installed: bool,
    pub voices_count: usize,
}

/// Voices response
#[derive(Debug, Serialize)]
pub struct VoicesResponse {
    pub voices: Vec<String>,
    pub default: String,
}

/// Root response
#[derive(Debug, Serialize)]
pub struct RootResponse {
    pub service: String,
    pub status: String,
    pub engine: String,
    pub voices: usize,
}

/// Streaming audio event
#[derive(Debug, Serialize)]
pub struct AudioEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub detail: String,
}

// =============================================================================
// Handlers
// =============================================================================

/// Root endpoint
pub async fn root(State(state): State<Arc<AppState>>) -> Json<RootResponse> {
    let voices = state.voice_engine.list_voices().await;
    Json(RootResponse {
        service: "voice".to_string(),
        status: "running".to_string(),
        engine: "piper-tts".to_string(),
        voices: voices.len(),
    })
}

/// Health check endpoint
pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let is_ready = state.voice_engine.is_ready().await;
    let voices = state.voice_engine.list_voices().await;
    
    Json(HealthResponse {
        status: if is_ready { "healthy" } else { "degraded" }.to_string(),
        model_loaded: !voices.is_empty(),
        piper_installed: is_ready || !voices.is_empty(),
        voices_count: voices.len(),
    })
}

/// Get service status
pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    let voices = state.voice_engine.list_voices().await;
    
    Json(StatusResponse {
        enabled: !voices.is_empty(),
        engine: "piper-tts".to_string(),
        sample_rate: SAMPLE_RATE,
        available_voices: voices,
        default_voice: state.voice_engine.default_voice().to_string(),
    })
}

/// List available voices
pub async fn list_voices(State(state): State<Arc<AppState>>) -> Json<VoicesResponse> {
    let voices = state.voice_engine.list_voices().await;
    
    Json(VoicesResponse {
        voices,
        default: state.voice_engine.default_voice().to_string(),
    })
}

/// Synthesize text to speech
pub async fn synthesize(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SynthesizeRequest>,
) -> Response {
    let text = request.text.trim();

    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Text cannot be empty".to_string(),
            }),
        )
            .into_response();
    }

    // Clean text for speech
    let cleaned_text = clean_text_for_speech(text);

    if cleaned_text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Text is empty after cleaning".to_string(),
            }),
        )
            .into_response();
    }

    // Truncate if too long
    let final_text = if cleaned_text.len() > 5000 {
        warn!("Text truncated to 5000 characters");
        &cleaned_text[..5000]
    } else {
        &cleaned_text
    };

    // Build effects config from request
    let mut effects = EffectsConfig::default();
    effects.speed = request.speed;
    effects.pitch_shift = request.pitch_shift;
    effects.robot_effect = request.robot_effect;

    info!(
        "🎤 Synthesizing {} chars with voice '{}'",
        final_text.len(),
        request.voice
    );

    // Synthesize
    match state
        .voice_engine
        .synthesize(final_text, Some(&request.voice), &effects)
        .await
    {
        Ok(wav_bytes) => {
            info!("✅ Generated {} bytes of audio", wav_bytes.len());

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "audio/wav")
                .header("Content-Disposition", "inline; filename=speech.wav")
                .body(Body::from(wav_bytes))
                .unwrap()
        }
        Err(e) => {
            error!("Synthesis error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    detail: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Synthesize text to speech with streaming (SSE)
pub async fn synthesize_stream(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SynthesizeRequest>,
) -> Response {
    let text = clean_text_for_speech(request.text.trim());

    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Text cannot be empty".to_string(),
            }),
        )
            .into_response();
    }

    let sentences = split_into_sentences(&text);
    let total_sentences = sentences.len();

    info!(
        "🎤 Streaming {} sentences with voice '{}'",
        total_sentences, request.voice
    );

    // Build effects config from request
    let mut effects = EffectsConfig::default();
    effects.speed = request.speed;
    effects.pitch_shift = request.pitch_shift;
    effects.robot_effect = request.robot_effect;

    let voice = request.voice.clone();
    let voice_engine = state.voice_engine.clone();

    // Create SSE stream
    let stream = stream::iter(sentences.into_iter().enumerate())
        .then(move |(index, sentence)| {
            let voice_engine = voice_engine.clone();
            let voice = voice.clone();
            let effects = effects.clone();

            async move {
                match voice_engine
                    .synthesize(&sentence, Some(&voice), &effects)
                    .await
                {
                    Ok(wav_bytes) => {
                        let audio_b64 = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &wav_bytes,
                        );

                        let event = AudioEvent {
                            event_type: "audio".to_string(),
                            index: Some(index),
                            total: Some(total_sentences),
                            data: Some(audio_b64),
                        };

                        format!("data: {}\n\n", serde_json::to_string(&event).unwrap())
                    }
                    Err(e) => {
                        error!("Error synthesizing sentence {}: {}", index, e);
                        String::new()
                    }
                }
            }
        })
        .chain(stream::once(async {
            let event = AudioEvent {
                event_type: "done".to_string(),
                index: None,
                total: None,
                data: None,
            };
            format!("data: {}\n\n", serde_json::to_string(&event).unwrap())
        }));

    let body = Body::from_stream(stream.map(|s| Ok::<_, Infallible>(s)));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(body)
        .unwrap()
}

/// Synthesize text to speech with Opus streaming (SSE)
/// Returns Opus encoded audio chunks for ~10x smaller payloads than WAV
pub async fn synthesize_opus_stream(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SynthesizeRequest>,
) -> Response {
    let text = clean_text_for_speech(request.text.trim());

    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                detail: "Text cannot be empty".to_string(),
            }),
        )
            .into_response();
    }

    let sentences = split_into_sentences(&text);
    let total_sentences = sentences.len();

    info!(
        "🎤 Streaming Opus {} sentences with voice '{}'",
        total_sentences, request.voice
    );

    // Build effects config from request
    let mut effects = EffectsConfig::default();
    effects.speed = request.speed;
    effects.pitch_shift = request.pitch_shift;
    effects.robot_effect = request.robot_effect;

    let voice = request.voice.clone();
    let voice_engine = state.voice_engine.clone();

    // Create SSE stream with Opus encoding
    let stream = stream::iter(sentences.into_iter().enumerate())
        .then(move |(index, sentence)| {
            let voice_engine = voice_engine.clone();
            let voice = voice.clone();
            let effects = effects.clone();

            async move {
                // Get raw samples instead of WAV
                match voice_engine
                    .synthesize_raw(&sentence, Some(&voice), &effects)
                    .await
                {
                    Ok(samples) => {
                        // Resample stereo from voice engine rate (44100) to Opus rate (48000)
                        let resampled = resample_stereo(&samples, SAMPLE_RATE, OPUS_SAMPLE_RATE);
                        
                        // Encode to Opus (stereo)
                        match encode_pcm_to_opus(&resampled) {
                            Ok(opus_bytes) => {
                                let audio_b64 = base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    &opus_bytes,
                                );

                                let event = AudioEvent {
                                    event_type: "audio".to_string(),
                                    index: Some(index),
                                    total: Some(total_sentences),
                                    data: Some(audio_b64),
                                };

                                format!("data: {}\n\n", serde_json::to_string(&event).unwrap())
                            }
                            Err(e) => {
                                error!("Opus encoding error for sentence {}: {}", index, e);
                                String::new()
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error synthesizing sentence {}: {}", index, e);
                        String::new()
                    }
                }
            }
        })
        .chain(stream::once(async {
            let event = AudioEvent {
                event_type: "done".to_string(),
                index: None,
                total: None,
                data: None,
            };
            format!("data: {}\n\n", serde_json::to_string(&event).unwrap())
        }));

    let body = Body::from_stream(stream.map(|s| Ok::<_, Infallible>(s)));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("X-Audio-Format", "opus")
        .body(body)
        .unwrap()
}
