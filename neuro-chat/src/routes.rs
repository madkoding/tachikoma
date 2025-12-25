//! =============================================================================
//! API Routes
//! =============================================================================

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Health & System
        .route("/health", get(handlers::health_check))
        .route("/models", get(handlers::list_models))
        
        // Chat
        .route("/chat", post(handlers::send_message))
        .route("/chat/stream", post(handlers::stream_message))
        
        // Conversations
        .route("/chat/conversations", get(handlers::list_conversations))
        .route("/chat/conversations/:id", get(handlers::get_conversation))
        .route("/chat/conversations/:id", delete(handlers::delete_conversation));

    Router::new()
        .nest("/api", api_routes)
        .with_state(state)
}
