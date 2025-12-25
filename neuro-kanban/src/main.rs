mod config;
mod models;
mod handlers;
mod routes;
mod backend_client;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::models::KanbanState;

pub struct AppState {
    pub config: Config,
    pub kanban: RwLock<KanbanState>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_kanban=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    let port = config.port;

    // Initialize state
    let state = Arc::new(AppState {
        config,
        kanban: RwLock::new(KanbanState::default()),
    });

    // Build router
    let app = routes::create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind port");

    info!("🗂️ NEURO-OS Kanban Service running on port {}", port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
