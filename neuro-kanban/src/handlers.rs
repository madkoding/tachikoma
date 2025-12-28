//! =============================================================================
//! Kanban Handlers
//! =============================================================================
//! HTTP handlers that proxy requests to neuro-backend data layer.
//! =============================================================================

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "neuro-kanban",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListBoardsParams {
    #[serde(default)]
    pub include_archived: bool,
}

// =============================================================================
// Board Handlers
// =============================================================================

/// List all boards (summaries)
pub async fn list_boards(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListBoardsParams>,
) -> Result<Json<Vec<BoardSummary>>, StatusCode> {
    match state.client.get_all_boards(params.include_archived).await {
        Ok(boards) => Ok(Json(boards)),
        Err(e) => {
            error!("Failed to list boards: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a single board with all columns and cards
pub async fn get_board(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
) -> Result<Json<Board>, StatusCode> {
    match state.client.get_board(board_id).await {
        Ok(Some(board)) => Ok(Json(board)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get board {}: {}", board_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new board
pub async fn create_board(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateBoardRequest>,
) -> Result<Json<Board>, StatusCode> {
    match state.client.create_board(request).await {
        Ok(board) => {
            debug!("Created board: {} ({})", board.name, board.id);
            Ok(Json(board))
        }
        Err(e) => {
            error!("Failed to create board: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a board
pub async fn update_board(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
    Json(request): Json<UpdateBoardRequest>,
) -> Result<Json<Board>, StatusCode> {
    match state.client.update_board(board_id, request).await {
        Ok(Some(board)) => Ok(Json(board)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update board {}: {}", board_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a board
pub async fn delete_board(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.client.delete_board(board_id).await {
        Ok(true) => {
            debug!("Deleted board: {}", board_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete board {}: {}", board_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// =============================================================================
// Column Handlers
// =============================================================================

/// Add a column to a board
pub async fn create_column(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
    Json(request): Json<CreateColumnRequest>,
) -> Result<Json<Column>, StatusCode> {
    match state.client.create_column(board_id, request).await {
        Ok(column) => {
            debug!("Created column: {} in board {}", column.name, board_id);
            Ok(Json(column))
        }
        Err(e) => {
            error!("Failed to create column in board {}: {}", board_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a column
pub async fn update_column(
    State(state): State<Arc<AppState>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateColumnRequest>,
) -> Result<Json<Column>, StatusCode> {
    match state.client.update_column(column_id, request).await {
        Ok(Some(column)) => Ok(Json(column)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update column {}: {}", column_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Reorder a column
pub async fn reorder_column(
    State(state): State<Arc<AppState>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ReorderColumnRequest>,
) -> Result<Json<Column>, StatusCode> {
    match state.client.reorder_column(column_id, request).await {
        Ok(Some(column)) => Ok(Json(column)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to reorder column {}: {}", column_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a column
pub async fn delete_column(
    State(state): State<Arc<AppState>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.client.delete_column(column_id).await {
        Ok(true) => {
            debug!("Deleted column: {}", column_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete column {}: {}", column_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// =============================================================================
// Card Handlers
// =============================================================================

/// Add a card to a column
pub async fn create_card(
    State(state): State<Arc<AppState>>,
    Path((_board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateCardRequest>,
) -> Result<Json<Card>, StatusCode> {
    match state.client.create_card(column_id, request).await {
        Ok(card) => {
            debug!("Created card: {} in column {}", card.title, column_id);
            Ok(Json(card))
        }
        Err(e) => {
            error!("Failed to create card in column {}: {}", column_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a card
pub async fn update_card(
    State(state): State<Arc<AppState>>,
    Path((_board_id, _column_id, card_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<UpdateCardRequest>,
) -> Result<Json<Card>, StatusCode> {
    match state.client.update_card(card_id, request).await {
        Ok(Some(card)) => Ok(Json(card)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update card {}: {}", card_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Move a card to another column or position
pub async fn move_card(
    State(state): State<Arc<AppState>>,
    Path((_board_id, _column_id, card_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<MoveCardRequest>,
) -> Result<Json<Card>, StatusCode> {
    match state.client.move_card(card_id, request).await {
        Ok(Some(card)) => Ok(Json(card)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to move card {}: {}", card_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a card
pub async fn delete_card(
    State(state): State<Arc<AppState>>,
    Path((_board_id, _column_id, card_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.client.delete_card(card_id).await {
        Ok(true) => {
            debug!("Deleted card: {}", card_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete card {}: {}", card_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
