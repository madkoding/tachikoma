use std::sync::Arc;
use tracing::info;

use neuro_common::{init_tracing, serve};

mod backend_client;
mod config;
mod handlers;
mod models;
mod routes;

use crate::backend_client::BackendClient;
use crate::config::Config;

pub struct AppState {
    pub client: BackendClient,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing("tachikoma_checklists=info,tower_http=debug");

    let config = Config::from_env(3001);
    info!("Checklists service | port={} backend={}", config.port, config.backend_url);

    let client = BackendClient::new(&config);
    let state = Arc::new(AppState { client, config: config.clone() });
    let app = routes::create_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    serve(&addr, app, "Checklists").await?;
    Ok(())
}