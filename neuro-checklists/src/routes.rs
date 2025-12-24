//! API Routes

use axum::{
    routing::{delete, get, patch, post},
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
        // Checklists
        .route("/checklists", get(handlers::list_checklists))
        .route("/checklists", post(handlers::create_checklist))
        .route("/checklists/import", post(handlers::import_from_markdown))
        .route("/checklists/:id", get(handlers::get_checklist))
        .route("/checklists/:id", patch(handlers::update_checklist))
        .route("/checklists/:id", delete(handlers::delete_checklist))
        // Checklist Items
        .route("/checklists/:id/items", post(handlers::add_item))
        .route("/checklists/:checklist_id/items/:item_id", patch(handlers::update_item))
        .route("/checklists/:checklist_id/items/:item_id", delete(handlers::delete_item))
        .route("/checklists/:checklist_id/items/:item_id/toggle", post(handlers::toggle_item));

    // Compose final router
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
