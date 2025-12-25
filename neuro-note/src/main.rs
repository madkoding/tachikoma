use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod models;
mod routes;

use config::Config;
use models::NotesState;

pub struct AppState {
    pub config: Config,
    pub notes_state: RwLock<NotesState>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_note=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let port = config.port;

    let state = Arc::new(AppState {
        config,
        notes_state: RwLock::new(NotesState::new()),
    });

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("📝 Neuro Note Service running on port {}", port);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("  ▸ Notes:     GET/POST /api/notes");
    info!("  ▸ Folders:   GET/POST /api/notes/folders");
    info!("  ▸ Search:    GET /api/notes/search");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    axum::serve(listener, app).await.unwrap();
}
