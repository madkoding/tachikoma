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
use models::ImageState;

pub struct AppState {
    pub config: Config,
    pub image_state: RwLock<ImageState>,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_image=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let port = config.port;

    let state = Arc::new(AppState {
        config,
        image_state: RwLock::new(ImageState::new()),
        http_client: reqwest::Client::new(),
    });

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("🖼️ Neuro Image Service running on port {}", port);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("  ▸ Gallery:   GET /api/images");
    info!("  ▸ Generate:  POST /api/images/generate");
    info!("  ▸ Albums:    GET/POST /api/images/albums");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    axum::serve(listener, app).await.unwrap();
}
