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
use std::io::{self, Write};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod audio_effects;
mod config;
mod handlers;
mod opus_encoder;
mod text_cleaner;
mod voice_engine;

use config::AppConfig;
use handlers::AppState;
use voice_engine::VoiceEngine;

// ANSI color codes for terminal output
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

/// Print a startup step with progress
fn print_step(step: u8, total: u8, message: &str) {
    let progress = "█".repeat(step as usize) + &"░".repeat((total - step) as usize);
    print!("\r{CYAN}[{progress}]{RESET} {DIM}({step}/{total}){RESET} {message}");
    io::stdout().flush().ok();
}

/// Print a completed step
fn print_done(step: u8, total: u8, message: &str) {
    let progress = "█".repeat(step as usize) + &"░".repeat((total - step) as usize);
    println!("\r{CYAN}[{progress}]{RESET} {GREEN}✓{RESET} {message}");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize logging (quiet mode - only warnings)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_target(false)
        .compact()
        .init();

    // Pretty startup banner
    println!("\n{CYAN}{BOLD}╔═══════════════════════════════════════════════════════════╗{RESET}");
    println!("{CYAN}{BOLD}║{RESET}            {MAGENTA}🎙️ Voice Service (Piper TTS){RESET}                 {CYAN}{BOLD}║{RESET}");
    println!("{CYAN}{BOLD}╚═══════════════════════════════════════════════════════════╝{RESET}\n");

    const TOTAL_STEPS: u8 = 4;

    // Load configuration
    print_step(1, TOTAL_STEPS, "Loading configuration...");
    let config = AppConfig::from_env();
    print_done(1, TOTAL_STEPS, "Configuration loaded");

    // Initialize voice engine
    print_step(2, TOTAL_STEPS, "Initializing Piper TTS engine...");
    let voice_engine = Arc::new(VoiceEngine::new(config.piper.clone()));
    voice_engine.initialize().await?;
    print_done(2, TOTAL_STEPS, "Piper TTS engine ready");

    // Create application state
    print_step(3, TOTAL_STEPS, "Creating application state...");
    let app_state = Arc::new(AppState { voice_engine });
    print_done(3, TOTAL_STEPS, "Application state ready");

    // Build router
    print_step(4, TOTAL_STEPS, "Building HTTP router...");
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
    print_done(4, TOTAL_STEPS, "HTTP router ready");

    // Start server
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    println!("\n{GREEN}{BOLD}✓ Voice Service ready!{RESET}");
    println!("{DIM}─────────────────────────────────────────────────────────{RESET}");
    println!("  {CYAN}▸{RESET} Server:     {YELLOW}http://{bind_addr}{RESET}");
    println!("  {CYAN}▸{RESET} Piper:      {DIM}{}{RESET}", config.piper.binary_path.display());
    println!("  {CYAN}▸{RESET} Voice:      {DIM}{}{RESET}", config.piper.default_voice);
    println!("  {CYAN}▸{RESET} Synthesize: {DIM}POST /synthesize{RESET}");
    println!("  {CYAN}▸{RESET} Stream:     {DIM}POST /synthesize/opus{RESET}");
    println!("{DIM}─────────────────────────────────────────────────────────{RESET}\n");

    axum::serve(listener, app).await?;

    Ok(())
}
