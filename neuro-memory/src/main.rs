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

pub use config::Config;
pub use db::Database;

pub struct AppState {
    pub db: Database,
    pub backend_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("info,tachikoma_memory=debug");

    info!("Starting Memory service...");

    let config = Config::from_env();

    let db = Database::connect(&config).await?;
    db.initialize_schema().await?;

    let state = Arc::new(AppState {
        db,
        backend_url: config.backend_url.clone(),
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
    serve(&addr, app, "Memory").await?;
    Ok(())
}