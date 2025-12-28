//! =============================================================================
//! Neuro-Kanban Microservice
//! =============================================================================
//! Independent microservice for managing Kanban boards.
//! Uses neuro-backend as data layer via HTTP client.
//! =============================================================================

mod config;
mod models;
mod handlers;
mod routes;
mod backend_client;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::backend_client::BackendClient;
use crate::config::Config;

/// Application state shared across handlers
pub struct AppState {
    pub client: BackendClient,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_kanban=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    let port = config.port;

    info!("🗂️  Neuro-Kanban Microservice");
    info!("================================");
    info!("Port: {}", config.port);
    info!("Backend URL: {}", config.backend_url);

    // Create backend client
    let client = BackendClient::new(&config);
    info!("✅ Backend client initialized");

    // Create app state
    let state = Arc::new(AppState { client, config });

    // Build router
    let app = routes::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("🚀 Server listening on {}", addr);
    info!("  ▸ Health: GET /api/health");
    info!("  ▸ API: /api/kanban/*");

    axum::serve(listener, app).await?;

    Ok(())
}
