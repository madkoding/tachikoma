//! =============================================================================
//! Neuro-Checklists Microservice
//! =============================================================================
//! Independent microservice for managing checklists with SurrealDB backend.
//! =============================================================================

mod config;
mod db;
mod handlers;
mod models;
mod routes;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::Database;

/// Application state shared across handlers
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_checklists=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    
    info!("🗒️  Neuro-Checklists Microservice");
    info!("================================");
    info!("Port: {}", config.port);
    info!("Database: {}", config.database_url);

    // Connect to database
    let db = Database::connect(&config).await?;
    info!("✅ Connected to SurrealDB (namespace: checklists)");

    // Create app state
    let state = Arc::new(AppState { db, config: config.clone() });

    // Build router
    let app = routes::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🚀 Server listening on {}", addr);
    info!("  ▸ Health: GET /health");
    info!("  ▸ Checklists: /api/checklists/*");

    axum::serve(listener, app).await?;

    Ok(())
}
