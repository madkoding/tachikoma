//! =============================================================================
//! Tachikoma-Checklists Microservice
//! =============================================================================
//! Independent microservice for managing checklists.
//! Uses tachikoma-backend as data layer via HTTP client.
//! =============================================================================

mod backend_client;
mod config;
mod handlers;
mod models;
mod routes;

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
                .unwrap_or_else(|_| "tachikoma_checklists=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    
    info!("🗒️  Tachikoma-Checklists Microservice");
    info!("================================");
    info!("Port: {}", config.port);
    info!("Backend URL: {}", config.backend_url);

    // Create backend client
    let client = BackendClient::new(&config);
    info!("✅ Backend client initialized");

    // Create app state
    let state = Arc::new(AppState { client, config: config.clone() });

    // Build router
    let app = routes::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🚀 Server listening on {}", addr);
    info!("  ▸ Health: GET /health");
    info!("  ▸ API: /api/checklists/*");
    
    axum::serve(listener, app).await?;

    Ok(())
}
