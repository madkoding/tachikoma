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
        // Documents
        .route("/api/docs", get(handlers::list_documents))
        .route("/api/docs", post(handlers::create_document))
        .route("/api/docs/search", get(handlers::search_documents))
        .route("/api/docs/stats", get(handlers::get_storage_stats))
        .route("/api/docs/:id", get(handlers::get_document))
        .route("/api/docs/:id", put(handlers::update_document))
        .route("/api/docs/:id", delete(handlers::delete_document))
        // Folders
        .route("/api/docs/folders", get(handlers::list_folders))
        .route("/api/docs/folders", post(handlers::create_folder))
        .route("/api/docs/folders/:id", get(handlers::get_folder_contents))
        .route("/api/docs/folders/:id", put(handlers::update_folder))
        .route("/api/docs/folders/:id", delete(handlers::delete_folder))
        .layer(cors)
        .with_state(state)
}
