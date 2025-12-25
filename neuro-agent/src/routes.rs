//! =============================================================================
//! Routes Configuration
//! =============================================================================

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;
use crate::handlers;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/api/health", get(handlers::health_check))
        
        // Agent tools
        .route("/api/agent/search", post(handlers::web_search))
        .route("/api/agent/execute", post(handlers::execute_command))
        .route("/api/agent/commands", get(handlers::list_allowed_commands))
        
        // State
        .with_state(state)
}
