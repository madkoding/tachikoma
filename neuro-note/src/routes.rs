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
        // Notes
        .route("/api/notes", get(handlers::list_notes))
        .route("/api/notes", post(handlers::create_note))
        .route("/api/notes/search", get(handlers::search_notes))
        .route("/api/notes/:id", get(handlers::get_note))
        .route("/api/notes/:id", put(handlers::update_note))
        .route("/api/notes/:id", delete(handlers::delete_note))
        // Folders
        .route("/api/notes/folders", get(handlers::list_folders))
        .route("/api/notes/folders", post(handlers::create_folder))
        .route("/api/notes/folders/:id", get(handlers::get_folder))
        .route("/api/notes/folders/:id", put(handlers::update_folder))
        .route("/api/notes/folders/:id", delete(handlers::delete_folder))
        .layer(cors)
        .with_state(state)
}
