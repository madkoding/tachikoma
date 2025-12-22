//! =============================================================================
//! Voice Synthesis Handlers
//! =============================================================================
//! HTTP handlers for text-to-speech synthesis endpoints.
//! =============================================================================

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::infrastructure::services::VoiceSynthesisRequest;
use crate::AppState;

/// =============================================================================
/// Voice Synthesis Response
/// =============================================================================
#[derive(serde::Serialize)]
pub struct VoiceStatusResponse {
    pub enabled: bool,
    pub model: String,
    pub sample_rate: u32,
    pub available_voices: Vec<String>,
    pub current_voice: String,
}

/// =============================================================================
/// Check Voice Status
/// =============================================================================
/// Returns whether voice synthesis is available.
/// 
/// # Endpoint
/// `GET /api/voice/status`
/// =============================================================================
pub async fn voice_status(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let enabled = state.voice_engine.is_ready().await;
    let voices = state.voice_engine.list_voices().await;
    
    Json(VoiceStatusResponse {
        enabled,
        model: "kokoro-v1.0".to_string(),
        sample_rate: 24000,
        available_voices: voices,
        current_voice: "af_bella".to_string(),
    })
}

/// =============================================================================
/// Synthesize Voice (WAV)
/// =============================================================================
/// Converts text to speech and returns a WAV file.
/// 
/// # Endpoint
/// `POST /api/voice/synthesize`
/// 
/// # Request Body
/// ```json
/// {
///     "text": "Hola, soy Tachikoma",
///     "streaming": false,
///     "voice": "af_bella"
/// }
/// ```
/// =============================================================================
pub async fn synthesize_voice(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VoiceSynthesisRequest>,
) -> Response {
    info!("🎤 Voice synthesis request: {} chars", request.text.len());

    // Check if voice engine is ready
    if !state.voice_engine.is_ready().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Voice synthesis not available",
                "message": "Kokoro model not loaded. Please download the model files."
            })),
        )
            .into_response();
    }

    // Validate input
    if request.text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Empty text",
                "message": "Text cannot be empty"
            })),
        )
            .into_response();
    }

    // Limit text length
    let text = if request.text.len() > 5000 {
        &request.text[..5000]
    } else {
        &request.text
    };

    // Synthesize audio
    match state.voice_engine.synthesize(text).await {
        Ok(audio_bytes) => {
            debug!("✅ Voice synthesized: {} bytes", audio_bytes.len());

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "audio/wav")
                .header(header::CONTENT_LENGTH, audio_bytes.len())
                .header(
                    header::CONTENT_DISPOSITION,
                    "inline; filename=\"speech.wav\"",
                )
                .body(Body::from(audio_bytes))
                .unwrap()
        }
        Err(e) => {
            error!("❌ Voice synthesis failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Synthesis failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// =============================================================================
/// Synthesize Voice Streaming (sentence by sentence)
/// =============================================================================
/// Converts text to speech, splitting by sentences and streaming each as SSE.
/// 
/// # Endpoint
/// `POST /api/voice/stream`
/// 
/// # Request Body
/// ```json
/// {
///     "text": "Hola, este es un texto largo que se dividirá en oraciones.",
///     "streaming": true
/// }
/// ```
/// 
/// # Response
/// Server-Sent Events with audio chunks as base64.
/// =============================================================================
pub async fn stream_voice(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VoiceSynthesisRequest>,
) -> Response {
    info!("🎤 Voice streaming request: {} chars", request.text.len());

    // Check if voice engine is ready
    if !state.voice_engine.is_ready().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Voice synthesis not available"
            })),
        )
            .into_response();
    }

    // Split text into sentences
    let sentences: Vec<String> = request.text
        .split(|c| c == '.' || c == '!' || c == '?' || c == '\n')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    // Create async stream that synthesizes each sentence
    let voice_engine = state.voice_engine.clone();
    let stream = async_stream::stream! {
        for sentence in sentences {
            match voice_engine.synthesize(&sentence).await {
                Ok(audio_bytes) => {
                    let base64_audio = base64_encode(&audio_bytes);
                    let event = format!(
                        "data: {}\n\n",
                        serde_json::json!({
                            "type": "audio",
                            "data": base64_audio,
                            "format": "wav",
                            "text": sentence
                        })
                    );
                    yield Ok::<_, std::convert::Infallible>(event);
                }
                Err(e) => {
                    let event = format!(
                        "data: {}\n\n",
                        serde_json::json!({
                            "type": "error",
                            "message": e.to_string()
                        })
                    );
                    yield Ok(event);
                }
            }
        }
        
        // Send completion event
        let done_event = format!(
            "data: {}\n\n",
            serde_json::json!({
                "type": "done"
            })
        );
        yield Ok(done_event);
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Base64 encode helper
fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    let mut encoder = base64_encoder(&mut buf);
    encoder.write_all(data).unwrap();
    drop(encoder);
    String::from_utf8(buf).unwrap()
}

fn base64_encoder(writer: &mut Vec<u8>) -> impl std::io::Write + '_ {
    struct Base64Writer<'a>(&'a mut Vec<u8>);
    
    impl<'a> std::io::Write for Base64Writer<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            
            for chunk in buf.chunks(3) {
                let b0 = chunk[0] as usize;
                let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
                let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
                
                self.0.push(ALPHABET[b0 >> 2]);
                self.0.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)]);
                
                if chunk.len() > 1 {
                    self.0.push(ALPHABET[((b1 & 0x0F) << 2) | (b2 >> 6)]);
                } else {
                    self.0.push(b'=');
                }
                
                if chunk.len() > 2 {
                    self.0.push(ALPHABET[b2 & 0x3F]);
                } else {
                    self.0.push(b'=');
                }
            }
            
            Ok(buf.len())
        }
        
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    
    Base64Writer(writer)
}
