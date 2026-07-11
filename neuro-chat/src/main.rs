use std::sync::Arc;
use anyhow::Result;
use axum::http::header;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use neuro_common::{init_tracing, serve};

mod config;
mod db;
mod handlers;
mod models;
mod routes;
mod backend_client;
mod memory_client;

pub use config::Config;
pub use db::Database;
pub use backend_client::BackendLlmClient;
pub use memory_client::MemoryClient;

pub struct AppState {
    pub db: Database,
    pub llm_client: BackendLlmClient,
    pub memory_client: MemoryClient,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("info,tachikoma_chat=debug");

    info!("Starting Chat service...");

    let config = Config::from_env();
    info!("Backend URL: {}", config.backend_url);

    let db = Database::connect(&config).await?;
    db.initialize_schema().await?;

    let llm_client = BackendLlmClient::new(&config.backend_url);
    let memory_client = MemoryClient::new(&config.memory_service_url);

    let state = Arc::new(AppState {
        db,
        llm_client,
        memory_client,
        config: config.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([
            header::CONTENT_TYPE,
            header::CACHE_CONTROL,
            header::CONNECTION,
        ]);

    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("{}:{}", config.host, config.port);
    serve(&addr, app, "Chat").await?;
    Ok(())
}