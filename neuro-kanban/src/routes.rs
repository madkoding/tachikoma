use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
        // Health
        .route("/health", get(handlers::health_check))
        // Boards
        .route("/kanban/boards", get(handlers::list_boards))
        .route("/kanban/boards", post(handlers::create_board))
        .route("/kanban/boards/:board_id", get(handlers::get_board))
        .route("/kanban/boards/:board_id", patch(handlers::update_board))
        .route("/kanban/boards/:board_id", delete(handlers::delete_board))
        // Columns
        .route("/kanban/boards/:board_id/columns", post(handlers::create_column))
        .route("/kanban/boards/:board_id/columns/:column_id", patch(handlers::update_column))
        .route("/kanban/boards/:board_id/columns/:column_id", delete(handlers::delete_column))
        .route("/kanban/boards/:board_id/columns/:column_id/reorder", put(handlers::reorder_column))
        // Cards
        .route("/kanban/boards/:board_id/columns/:column_id/cards", post(handlers::create_card))
        .route("/kanban/boards/:board_id/columns/:column_id/cards/:card_id", patch(handlers::update_card))
        .route("/kanban/boards/:board_id/columns/:column_id/cards/:card_id", delete(handlers::delete_card))
        .route("/kanban/boards/:board_id/columns/:column_id/cards/:card_id/move", put(handlers::move_card));

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
