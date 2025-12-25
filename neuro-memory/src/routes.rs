//! =============================================================================
//! API Routes
//! =============================================================================

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Health
        .route("/health", get(handlers::health_check))
        
        // Memories CRUD
        .route("/memories", get(handlers::list_memories))
        .route("/memories", post(handlers::create_memory))
        .route("/memories/search", post(handlers::search_memories))
        .route("/memories/:id", get(handlers::get_memory))
        .route("/memories/:id", patch(handlers::update_memory))
        .route("/memories/:id", delete(handlers::delete_memory))
        
        // Relations
        .route("/memories/:id/relations", get(handlers::get_memory_relations))
        .route("/memories/:id/related", get(handlers::get_related_memories))
        .route("/memories/relations", post(handlers::create_relation))
        .route("/memories/:from_id/relations/:to_id", delete(handlers::delete_relation))
        
        // Graph Admin
        .route("/admin/graph/stats", get(handlers::get_graph_stats))
        .route("/admin/graph/export", get(handlers::export_graph))
        .route("/admin/graph/events", get(handlers::subscribe_graph_events));

    Router::new()
        .nest("/api", api_routes)
        .with_state(state)
}
