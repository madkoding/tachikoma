//! =============================================================================
//! API Routes
//! =============================================================================
//! Defines all HTTP routes and creates the Axum router.
//! =============================================================================

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::infrastructure::api::handlers;
use crate::infrastructure::api::middleware::{logging_middleware, request_id_middleware};
use crate::AppState;

/// =============================================================================
/// Create the API router
/// =============================================================================
/// Builds the complete Axum router with all routes and middleware.
/// =============================================================================
pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build routes
    let api_routes = Router::new()
        // Health & System
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::readiness_check))
        .route("/live", get(handlers::liveness_check))
        .route("/models", get(handlers::list_models))
        .route("/system/info", get(handlers::system_info))
        
        // Chat
        .route("/chat", post(handlers::send_message))
        .route("/chat/stream", post(handlers::stream_message))
        .route("/chat/conversations", get(handlers::list_conversations))
        .route("/chat/conversations/:id", get(handlers::get_conversation))
        .route("/chat/conversations/:id", delete(handlers::delete_conversation))
        
        // Voice Synthesis
        .route("/voice/status", get(handlers::voice_status))
        .route("/voice/synthesize", post(handlers::synthesize_voice))
        .route("/voice/stream", post(handlers::stream_voice))
        
        // Memories
        .route("/memories", get(handlers::list_memories))
        .route("/memories", post(handlers::create_memory))
        .route("/memories/search", post(handlers::search_memories))
        .route("/memories/:id", get(handlers::get_memory))
        .route("/memories/:id", patch(handlers::update_memory))
        .route("/memories/:id", delete(handlers::delete_memory))
        .route("/memories/:id/relations", get(handlers::get_memory_relations))
        .route("/memories/:id/related", get(handlers::get_related_memories))
        .route("/memories/relations", post(handlers::create_relation))
        .route("/memories/:from_id/relations/:to_id", delete(handlers::delete_relation))
        
        // Graph Admin
        .route("/admin/graph/stats", get(handlers::get_graph_stats))
        .route("/admin/graph/export", get(handlers::export_graph))
        
        // Agent Tools
        .route("/agent/search", post(handlers::web_search))
        .route("/agent/search/categories", get(handlers::get_search_categories))
        .route("/agent/execute", post(handlers::execute_command))
        .route("/agent/commands", get(handlers::get_allowed_commands));

    // Compose final router
    Router::new()
        .nest("/api", api_routes)
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(logging_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
