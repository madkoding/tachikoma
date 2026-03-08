//! =============================================================================
//! Kanban Handlers
//! =============================================================================

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::entities::kanban::{
    Board, BoardSummary, Card, Column,
    CreateBoard, CreateCard, CreateColumn,
    MoveCard, ReorderColumn,
    UpdateBoard, UpdateCard, UpdateColumn,
};
use crate::infrastructure::api::dto::ErrorResponse;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListBoardsParams {
    #[serde(default)]
    pub include_archived: bool,
}

// =============================================================================
// Board Handlers
// =============================================================================

/// GET /api/data/kanban/boards
#[instrument(skip(state))]
pub async fn list_boards(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListBoardsParams>,
) -> Result<Json<Vec<BoardSummary>>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.get_all_boards(params.include_archived).await {
        Ok(boards) => Ok(Json(boards)),
        Err(e) => {
            error!(error = %e, "Failed to list kanban boards");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/data/kanban/boards/:id
#[instrument(skip(state))]
pub async fn get_board(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Board>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.get_board(id).await {
        Ok(Some(board)) => Ok(Json(board)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Board not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to get kanban board");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/kanban/boards
#[instrument(skip(state, data))]
pub async fn create_board(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreateBoard>,
) -> Result<Json<Board>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.create_board(data).await {
        Ok(board) => Ok(Json(board)),
        Err(e) => {
            error!(error = %e, "Failed to create kanban board");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/data/kanban/boards/:id
#[instrument(skip(state, data))]
pub async fn update_board(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdateBoard>,
) -> Result<Json<Board>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.update_board(id, data).await {
        Ok(Some(board)) => Ok(Json(board)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Board not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update kanban board");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/data/kanban/boards/:id
#[instrument(skip(state))]
pub async fn delete_board(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.delete_board(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Board not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete kanban board");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// Column Handlers
// =============================================================================

/// POST /api/data/kanban/boards/:board_id/columns
#[instrument(skip(state, data))]
pub async fn create_column(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
    Json(data): Json<CreateColumn>,
) -> Result<Json<Column>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.create_column(board_id, data).await {
        Ok(column) => Ok(Json(column)),
        Err(e) => {
            error!(error = %e, "Failed to create kanban column");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/data/kanban/columns/:column_id
#[instrument(skip(state, data))]
pub async fn update_column(
    State(state): State<Arc<AppState>>,
    Path(column_id): Path<Uuid>,
    Json(data): Json<UpdateColumn>,
) -> Result<Json<Column>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.update_column(column_id, data).await {
        Ok(Some(column)) => Ok(Json(column)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Column not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update kanban column");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/kanban/columns/:column_id/reorder
#[instrument(skip(state, data))]
pub async fn reorder_column(
    State(state): State<Arc<AppState>>,
    Path(column_id): Path<Uuid>,
    Json(data): Json<ReorderColumn>,
) -> Result<Json<Column>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.reorder_column(column_id, data).await {
        Ok(Some(column)) => Ok(Json(column)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Column not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to reorder kanban column");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/data/kanban/columns/:column_id
#[instrument(skip(state))]
pub async fn delete_column(
    State(state): State<Arc<AppState>>,
    Path(column_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.delete_column(column_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Column not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete kanban column");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// Card Handlers
// =============================================================================

/// POST /api/data/kanban/columns/:column_id/cards
#[instrument(skip(state, data))]
pub async fn create_card(
    State(state): State<Arc<AppState>>,
    Path(column_id): Path<Uuid>,
    Json(data): Json<CreateCard>,
) -> Result<Json<Card>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.create_card(column_id, data).await {
        Ok(card) => Ok(Json(card)),
        Err(e) => {
            error!(error = %e, "Failed to create kanban card");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/data/kanban/cards/:card_id
#[instrument(skip(state, data))]
pub async fn update_card(
    State(state): State<Arc<AppState>>,
    Path(card_id): Path<Uuid>,
    Json(data): Json<UpdateCard>,
) -> Result<Json<Card>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.update_card(card_id, data).await {
        Ok(Some(card)) => Ok(Json(card)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Card not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update kanban card");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/kanban/cards/:card_id/move
#[instrument(skip(state, data))]
pub async fn move_card(
    State(state): State<Arc<AppState>>,
    Path(card_id): Path<Uuid>,
    Json(data): Json<MoveCard>,
) -> Result<Json<Card>, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.move_card(card_id, data).await {
        Ok(Some(card)) => Ok(Json(card)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Card not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to move kanban card");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/data/kanban/cards/:card_id
#[instrument(skip(state))]
pub async fn delete_card(
    State(state): State<Arc<AppState>>,
    Path(card_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.kanban_repository.delete_card(card_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Card not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete kanban card");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}
