use std::sync::Arc;
use anyhow::Result;
use axum::http::header;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use neuro_common::{init_tracing, serve, db::Database};

mod config;
mod handlers;
mod models;
mod routes;

pub use config::Config;

pub struct AppState {
    pub db: Database,
    pub backend_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("info,tachikoma_memory=debug");

    info!("Starting Memory service...");

    let config = Config::from_env();

    let db = Database::connect(
        &config.database_url,
        &config.database_user,
        &config.database_pass,
        &config.database_ns,
        &config.database_db,
    ).await?;

    let schema = vec![
        "DEFINE TABLE memory SCHEMAFULL",
        "DEFINE FIELD content ON memory TYPE string",
        "DEFINE FIELD vector ON memory TYPE array<float>",
        "DEFINE FIELD memory_type ON memory TYPE string",
        "DEFINE FIELD metadata ON memory TYPE object",
        "DEFINE FIELD created_at ON memory TYPE datetime",
        "DEFINE FIELD updated_at ON memory TYPE datetime",
        "DEFINE FIELD access_count ON memory TYPE int DEFAULT 0",
        "DEFINE FIELD importance_score ON memory TYPE float DEFAULT 0.5",
        "DEFINE INDEX memory_type_idx ON memory FIELDS memory_type",
        "DEFINE INDEX memory_created_idx ON memory FIELDS created_at",
        "DEFINE INDEX memory_importance_idx ON memory FIELDS importance_score",
        "DEFINE TABLE related_to SCHEMAFULL",
        "DEFINE FIELD in ON related_to TYPE record<memory>",
        "DEFINE FIELD out ON related_to TYPE record<memory>",
        "DEFINE FIELD relation_type ON related_to TYPE string",
        "DEFINE FIELD confidence ON related_to TYPE float DEFAULT 1.0",
        "DEFINE FIELD created_at ON related_to TYPE datetime",
        "DEFINE FIELD metadata ON related_to TYPE object",
    ];
    db.initialize_schema(&schema).await?;

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