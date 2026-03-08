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
use models::CalendarState;

pub struct AppState {
    pub config: Config,
    pub calendar_state: RwLock<CalendarState>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tachikoma_calendar=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let port = config.port;

    let state = Arc::new(AppState {
        config,
        calendar_state: RwLock::new(CalendarState::new()),
    });

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("🗓️ Tachikoma Calendar Service running on port {}", port);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("  ▸ Events:    GET/POST /api/calendar/events");
    info!("  ▸ Reminders: GET/POST /api/calendar/reminders");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    axum::serve(listener, app).await.unwrap();
}
