//! =============================================================================
//! Neuro-Pomodoro Microservice
//! =============================================================================
//! Pomodoro timer service for productivity tracking.
//! Uses in-memory storage for sessions and settings.
//! =============================================================================

mod config;
mod handlers;
mod memory_store;
mod models;
mod routes;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::memory_store::MemoryStore;

/// Application state shared across handlers
pub struct AppState {
    pub store: MemoryStore,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_pomodoro=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    
    info!("🍅 Neuro-Pomodoro Microservice");
    info!("================================");
    info!("Port: {}", config.port);

    // Create in-memory store
    let store = MemoryStore::new();
    info!("✅ In-memory store initialized");

    // Create app state
    let state = Arc::new(AppState { store, config: config.clone() });

    // Build router
    let app = routes::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🚀 Server listening on {}", addr);
    info!("  ▸ Health: GET /health");
    info!("  ▸ API: /api/pomodoro/*");
    
    axum::serve(listener, app).await?;

    Ok(())
}
