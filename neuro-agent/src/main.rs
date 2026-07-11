use std::sync::Arc;
use anyhow::Result;
use axum::http::header;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use neuro_common::{init_tracing, serve};

mod config;
mod handlers;
mod routes;
mod searxng;
mod executor;

pub use config::Config;
pub use searxng::SearxngClient;
pub use executor::CommandExecutor;

pub struct AppState {
    pub searxng: SearxngClient,
    pub executor: CommandExecutor,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("info,tachikoma_agent=debug");

    info!("Starting Agent service...");

    let config = Config::from_env();

    let searxng = SearxngClient::new(&config.searxng_url);
    let executor = CommandExecutor::new();

    let state = Arc::new(AppState {
        searxng,
        executor,
        config: config.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([header::CONTENT_TYPE]);

    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("{}:{}", config.host, config.port);
    serve(&addr, app, "Agent").await?;
    Ok(())
}