use std::sync::Arc;
use tracing::info;

use neuro_common::{init_tracing, serve};

mod config;
mod handlers;
mod memory_store;
mod models;
mod routes;

use crate::config::Config;
use crate::memory_store::MemoryStore;

pub struct AppState {
    pub store: MemoryStore,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing("tachikoma_pomodoro=info,tower_http=debug");

    let config = Config::from_env(3010);
    info!("Pomodoro service | port={}", config.port);

    let store = MemoryStore::new();
    let state = Arc::new(AppState { store, config: config.clone() });
    let app = routes::create_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    serve(&addr, app, "Pomodoro").await?;
    Ok(())
}