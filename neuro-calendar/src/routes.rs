use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers;
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        // Events
        .route("/api/calendar/events", get(handlers::list_events))
        .route("/api/calendar/events", post(handlers::create_event))
        .route("/api/calendar/events/today", get(handlers::get_today_events))
        .route("/api/calendar/events/:id", get(handlers::get_event))
        .route("/api/calendar/events/:id", put(handlers::update_event))
        .route("/api/calendar/events/:id", delete(handlers::delete_event))
        // Reminders
        .route("/api/calendar/reminders", get(handlers::list_pending_reminders))
        .route("/api/calendar/reminders/:id/dismiss", post(handlers::dismiss_reminder))
        .layer(cors)
        .with_state(state)
}
