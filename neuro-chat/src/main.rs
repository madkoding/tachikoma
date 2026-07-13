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
mod backend_client;
mod memory_client;

pub use config::Config;
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

    let db = Database::connect(
        &config.database_url,
        &config.database_user,
        &config.database_pass,
        &config.database_ns,
        &config.database_db,
    ).await?;

    let schema = vec![
        "DEFINE TABLE conversation SCHEMAFULL",
        "DEFINE FIELD title ON conversation TYPE option<string>",
        "DEFINE FIELD created_at ON conversation TYPE datetime",
        "DEFINE FIELD updated_at ON conversation TYPE datetime",
        "DEFINE FIELD archived ON conversation TYPE bool DEFAULT false",
        "DEFINE FIELD message_count ON conversation TYPE int DEFAULT 0",
        "DEFINE TABLE chat_message SCHEMAFULL",
        "DEFINE FIELD conversation_id ON chat_message TYPE string",
        "DEFINE FIELD role ON chat_message TYPE string",
        "DEFINE FIELD content ON chat_message TYPE string",
        "DEFINE FIELD model ON chat_message TYPE option<string>",
        "DEFINE FIELD tokens ON chat_message TYPE option<int>",
        "DEFINE FIELD metadata ON chat_message TYPE object",
        "DEFINE FIELD created_at ON chat_message TYPE datetime",
        "DEFINE INDEX message_conversation_idx ON chat_message FIELDS conversation_id",
        "DEFINE INDEX message_created_idx ON chat_message FIELDS created_at",
    ];
    db.initialize_schema(&schema).await?;

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