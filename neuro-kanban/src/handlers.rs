use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
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
// Board Handlers
// =============================================================================

/// List all boards (summaries)
pub async fn list_boards(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<BoardSummary>> {
    let kanban = state.kanban.read().await;
    let mut boards: Vec<BoardSummary> = kanban
        .boards
        .values()
        .map(|b| b.to_summary())
        .collect();
    
    // Sort by created_at descending
    boards.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    Json(boards)
}

/// Get a single board with all columns and cards
pub async fn get_board(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
) -> Result<Json<Board>, StatusCode> {
    let kanban = state.kanban.read().await;
    
    kanban
        .boards
        .get(&board_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create a new board
pub async fn create_board(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateBoardRequest>,
) -> Result<Json<Board>, StatusCode> {
    let now = Utc::now();
    let board_id = Uuid::new_v4();
    
    let columns = if request.with_default_columns.unwrap_or(true) {
        create_default_columns(board_id)
    } else {
        vec![]
    };
    
    let board = Board {
        id: board_id,
        name: request.name,
        description: request.description,
        color: request.color,
        is_archived: false,
        columns,
        created_at: now,
        updated_at: now,
    };
    
    let mut kanban = state.kanban.write().await;
    kanban.boards.insert(board_id, board.clone());
    
    debug!("Created board: {} ({})", board.name, board_id);
    Ok(Json(board))
}

/// Update a board
pub async fn update_board(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
    Json(request): Json<UpdateBoardRequest>,
) -> Result<Json<Board>, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = request.name {
        board.name = name;
    }
    if let Some(description) = request.description {
        board.description = Some(description);
    }
    if let Some(color) = request.color {
        board.color = Some(color);
    }
    if let Some(is_archived) = request.is_archived {
        board.is_archived = is_archived;
    }
    board.updated_at = Utc::now();
    
    Ok(Json(board.clone()))
}

/// Delete a board
pub async fn delete_board(
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    if kanban.boards.remove(&board_id).is_some() {
        debug!("Deleted board: {}", board_id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
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
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let now = Utc::now();
    let order = board.columns.len() as i32;
    
    let column = Column {
        id: Uuid::new_v4(),
        board_id,
        name: request.name,
        color: request.color,
        wip_limit: request.wip_limit,
        order,
        cards: vec![],
        created_at: now,
        updated_at: now,
    };
    
    board.columns.push(column.clone());
    board.updated_at = now;
    
    debug!("Created column: {} in board {}", column.name, board_id);
    Ok(Json(column))
}

/// Update a column
pub async fn update_column(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateColumnRequest>,
) -> Result<Json<Column>, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let column = board
        .columns
        .iter_mut()
        .find(|c| c.id == column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = request.name {
        column.name = name;
    }
    if let Some(color) = request.color {
        column.color = Some(color);
    }
    if let Some(wip_limit) = request.wip_limit {
        column.wip_limit = Some(wip_limit);
    }
    column.updated_at = Utc::now();
    
    Ok(Json(column.clone()))
}

/// Reorder a column
pub async fn reorder_column(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ReorderColumnRequest>,
) -> Result<Json<Board>, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Find current position
    let current_pos = board
        .columns
        .iter()
        .position(|c| c.id == column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Remove and reinsert at new position
    let column = board.columns.remove(current_pos);
    let new_pos = (request.target_order as usize).min(board.columns.len());
    board.columns.insert(new_pos, column);
    
    // Update order values
    for (i, col) in board.columns.iter_mut().enumerate() {
        col.order = i as i32;
    }
    board.updated_at = Utc::now();
    
    Ok(Json(board.clone()))
}

/// Delete a column
pub async fn delete_column(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let initial_len = board.columns.len();
    board.columns.retain(|c| c.id != column_id);
    
    if board.columns.len() < initial_len {
        // Re-order remaining columns
        for (i, col) in board.columns.iter_mut().enumerate() {
            col.order = i as i32;
        }
        board.updated_at = Utc::now();
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Card Handlers
// =============================================================================

/// Add a card to a column
pub async fn create_card(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateCardRequest>,
) -> Result<Json<Card>, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let column = board
        .columns
        .iter_mut()
        .find(|c| c.id == column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Check WIP limit
    if let Some(limit) = column.wip_limit {
        if column.cards.len() >= limit as usize {
            error!("WIP limit reached for column {}", column_id);
            return Err(StatusCode::CONFLICT);
        }
    }
    
    let now = Utc::now();
    let order = column.cards.len() as i32;
    
    let card = Card {
        id: Uuid::new_v4(),
        column_id,
        title: request.title,
        description: request.description,
        color: request.color,
        labels: request.labels.unwrap_or_default(),
        due_date: request.due_date,
        order,
        created_at: now,
        updated_at: now,
    };
    
    column.cards.push(card.clone());
    column.updated_at = now;
    board.updated_at = now;
    
    debug!("Created card: {} in column {}", card.title, column_id);
    Ok(Json(card))
}

/// Update a card
pub async fn update_card(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id, card_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<UpdateCardRequest>,
) -> Result<Json<Card>, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let column = board
        .columns
        .iter_mut()
        .find(|c| c.id == column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let card = column
        .cards
        .iter_mut()
        .find(|c| c.id == card_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(title) = request.title {
        card.title = title;
    }
    if let Some(description) = request.description {
        card.description = Some(description);
    }
    if let Some(color) = request.color {
        card.color = Some(color);
    }
    if let Some(labels) = request.labels {
        card.labels = labels;
    }
    if let Some(due_date) = request.due_date {
        card.due_date = Some(due_date);
    }
    card.updated_at = Utc::now();
    
    Ok(Json(card.clone()))
}

/// Move a card to another column or position
pub async fn move_card(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id, card_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<MoveCardRequest>,
) -> Result<Json<Board>, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Find and remove card from source column
    let source_column = board
        .columns
        .iter_mut()
        .find(|c| c.id == column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let card_pos = source_column
        .cards
        .iter()
        .position(|c| c.id == card_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let mut card = source_column.cards.remove(card_pos);
    
    // Re-order source column
    for (i, c) in source_column.cards.iter_mut().enumerate() {
        c.order = i as i32;
    }
    source_column.updated_at = Utc::now();
    
    // Find target column
    let target_column = board
        .columns
        .iter_mut()
        .find(|c| c.id == request.target_column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Check WIP limit on target
    if let Some(limit) = target_column.wip_limit {
        if target_column.cards.len() >= limit as usize {
            // Revert: put card back
            let source = board
                .columns
                .iter_mut()
                .find(|c| c.id == column_id)
                .unwrap();
            source.cards.insert(card_pos, card);
            return Err(StatusCode::CONFLICT);
        }
    }
    
    // Insert card at target position
    card.column_id = request.target_column_id;
    card.updated_at = Utc::now();
    
    let insert_pos = (request.target_order as usize).min(target_column.cards.len());
    target_column.cards.insert(insert_pos, card);
    
    // Re-order target column
    for (i, c) in target_column.cards.iter_mut().enumerate() {
        c.order = i as i32;
    }
    target_column.updated_at = Utc::now();
    board.updated_at = Utc::now();
    
    Ok(Json(board.clone()))
}

/// Delete a card
pub async fn delete_card(
    State(state): State<Arc<AppState>>,
    Path((board_id, column_id, card_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let mut kanban = state.kanban.write().await;
    
    let board = kanban
        .boards
        .get_mut(&board_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let column = board
        .columns
        .iter_mut()
        .find(|c| c.id == column_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let initial_len = column.cards.len();
    column.cards.retain(|c| c.id != card_id);
    
    if column.cards.len() < initial_len {
        // Re-order remaining cards
        for (i, c) in column.cards.iter_mut().enumerate() {
            c.order = i as i32;
        }
        column.updated_at = Utc::now();
        board.updated_at = Utc::now();
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
