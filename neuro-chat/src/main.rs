//! =============================================================================
//! NEURO-OS Chat Service - Main Entry Point
//! =============================================================================
//! Microservice for chat interactions with LLM, streaming responses,
//! and conversation management.
//! =============================================================================

use std::sync::Arc;
use anyhow::Result;
use axum::http::header;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod db;
mod handlers;
mod models;
mod routes;
mod ollama;
mod memory_client;

pub use config::Config;
pub use db::Database;
pub use ollama::OllamaClient;
pub use memory_client::MemoryClient;

/// Application state shared across handlers
pub struct AppState {
    pub db: Database,
    pub ollama: OllamaClient,
    pub memory_client: MemoryClient,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,neuro_chat=debug".into()),
        )
        .init();

    info!("💬 Starting NEURO-OS Chat Service...");

    // Load configuration
    let config = Config::from_env();
    info!("Configuration loaded");

    // Connect to database
    let db = Database::connect(&config).await?;
    info!("✅ Connected to SurrealDB");

    // Initialize schema
    db.initialize_schema().await?;
    info!("✅ Database schema initialized");

    // Create clients
    let ollama = OllamaClient::new(&config.ollama_url);
    let memory_client = MemoryClient::new(&config.memory_service_url);

    // Create app state
    let state = Arc::new(AppState {
        db,
        ollama,
        memory_client,
        config: config.clone(),
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([
            header::CONTENT_TYPE,
            header::CACHE_CONTROL,
            header::CONNECTION,
        ]);

    // Build router
    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("🚀 Chat service listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
