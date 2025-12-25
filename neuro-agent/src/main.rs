//! =============================================================================
//! NEURO-OS Agent Service - Main Entry Point
//! =============================================================================
//! Microservice for agent tools: web search and safe command execution.
//! =============================================================================

use std::sync::Arc;
use anyhow::Result;
use axum::http::header;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod handlers;
mod routes;
mod searxng;
mod executor;

pub use config::Config;
pub use searxng::SearxngClient;
pub use executor::CommandExecutor;

/// Application state shared across handlers
pub struct AppState {
    pub searxng: SearxngClient,
    pub executor: CommandExecutor,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,neuro_agent=debug".into()),
        )
        .init();

    info!("🤖 Starting NEURO-OS Agent Service...");

    // Load configuration
    let config = Config::from_env();
    info!("Configuration loaded");

    // Create clients
    let searxng = SearxngClient::new(&config.searxng_url);
    let executor = CommandExecutor::new();

    // Create app state
    let state = Arc::new(AppState {
        searxng,
        executor,
        config: config.clone(),
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([
            header::CONTENT_TYPE,
        ]);

    // Build router
    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("🚀 Agent service listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
