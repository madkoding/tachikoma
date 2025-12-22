//! =============================================================================
//! Voice Service - Main Entry Point
//! =============================================================================
//! Ultra-fast text-to-speech service using Piper TTS with robotic effects.
//! Built with Axum for high performance HTTP serving.
//! =============================================================================

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod audio_effects;
mod config;
mod handlers;
mod opus_encoder;
mod text_cleaner;
mod voice_engine;

use config::AppConfig;
use handlers::AppState;
use voice_engine::VoiceEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,voice_service=debug".into()),
        )
        .with_target(true)
        .init();

    info!("🎙️ Voice Service (Piper TTS) starting...");

    // Load configuration
    let config = AppConfig::from_env();
    info!("📋 Configuration loaded");
    info!("   - Host: {}:{}", config.server.host, config.server.port);
    info!("   - Piper binary: {}", config.piper.binary_path.display());
    info!("   - Models dir: {}", config.piper.models_dir.display());
    info!("   - Default voice: {}", config.piper.default_voice);

    // Initialize voice engine
    let voice_engine = Arc::new(VoiceEngine::new(config.piper));
    voice_engine.initialize().await?;

    // Create application state
    let app_state = Arc::new(AppState { voice_engine });

    // Build router
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_status))
        .route("/voices", get(handlers::list_voices))
        .route("/synthesize", post(handlers::synthesize))
        .route("/synthesize/stream", post(handlers::synthesize_stream))
        .route("/synthesize/opus", post(handlers::synthesize_opus_stream))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("🚀 Voice Service listening on http://{}", bind_addr);
    info!("📚 API endpoints:");
    info!("   - Root:      GET  /");
    info!("   - Health:    GET  /health");
    info!("   - Status:    GET  /status");
    info!("   - Voices:    GET  /voices");
    info!("   - Synth:     POST /synthesize");
    info!("   - Stream:    POST /synthesize/stream");
    info!("   - Opus:      POST /synthesize/opus");

    axum::serve(listener, app).await?;

    Ok(())
}
