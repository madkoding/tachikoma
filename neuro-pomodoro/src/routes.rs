//! API Routes

use axum::{
    routing::{get, post, patch},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
        // Timer state
        .route("/pomodoro/state", get(handlers::get_timer_state))
        // Sessions
        .route("/pomodoro/sessions", post(handlers::start_session))
        .route("/pomodoro/sessions", get(handlers::get_today_sessions))
        .route("/pomodoro/sessions/:id", patch(handlers::update_session))
        .route("/pomodoro/sessions/:id/complete", post(handlers::complete_session))
        .route("/pomodoro/sessions/:id/cancel", post(handlers::cancel_session))
        .route("/pomodoro/sessions/:id/pause", post(handlers::pause_session))
        .route("/pomodoro/sessions/:id/resume", post(handlers::resume_session))
        // Settings
        .route("/pomodoro/settings", get(handlers::get_settings))
        .route("/pomodoro/settings", post(handlers::save_settings))
        // Stats
        .route("/pomodoro/stats", get(handlers::get_stats))
        .route("/pomodoro/stats/daily", get(handlers::get_daily_stats))
        .route("/pomodoro/stats/weekly", get(handlers::get_weekly_stats));

    // Compose final router
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
