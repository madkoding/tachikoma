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
        // Images
        .route("/api/images", get(handlers::list_images))
        .route("/api/images", post(handlers::create_image))
        .route("/api/images/generate", post(handlers::generate_image))
        .route("/api/images/styles", get(handlers::list_styles))
        .route("/api/images/:id", get(handlers::get_image))
        .route("/api/images/:id", put(handlers::update_image))
        .route("/api/images/:id", delete(handlers::delete_image))
        // Albums
        .route("/api/images/albums", get(handlers::list_albums))
        .route("/api/images/albums", post(handlers::create_album))
        .route("/api/images/albums/:id", get(handlers::get_album))
        .route("/api/images/albums/:id", put(handlers::update_album))
        .route("/api/images/albums/:id", delete(handlers::delete_album))
        .layer(cors)
        .with_state(state)
}
