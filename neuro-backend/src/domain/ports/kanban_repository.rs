//! =============================================================================
//! Kanban Repository Port
//! =============================================================================

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::kanban::{
    Board, BoardSummary, Card, Column,
    CreateBoard, CreateCard, CreateColumn,
    MoveCard, ReorderColumn,
    UpdateBoard, UpdateCard, UpdateColumn,
};
use crate::domain::errors::DomainError;

/// Abstract repository interface for Kanban boards
#[async_trait]
pub trait KanbanRepository: Send + Sync {
    // =========================================================================
    // Board CRUD
    // =========================================================================
    
    /// Get all boards (summaries only)
    async fn get_all_boards(&self, include_archived: bool) -> Result<Vec<BoardSummary>, DomainError>;

    /// Get a single board with all columns and cards
    async fn get_board(&self, id: Uuid) -> Result<Option<Board>, DomainError>;

    /// Create a new board
    async fn create_board(&self, data: CreateBoard) -> Result<Board, DomainError>;

    /// Update an existing board
    async fn update_board(&self, id: Uuid, data: UpdateBoard) -> Result<Option<Board>, DomainError>;

    /// Delete a board (cascades to columns and cards)
    async fn delete_board(&self, id: Uuid) -> Result<bool, DomainError>;

    // =========================================================================
    // Column CRUD
    // =========================================================================

    /// Add a column to a board
    async fn create_column(&self, board_id: Uuid, data: CreateColumn) -> Result<Column, DomainError>;

    /// Update a column
    async fn update_column(&self, column_id: Uuid, data: UpdateColumn) -> Result<Option<Column>, DomainError>;

    /// Reorder a column within a board
    async fn reorder_column(&self, column_id: Uuid, data: ReorderColumn) -> Result<Option<Column>, DomainError>;

    /// Delete a column (cascades to cards)
    async fn delete_column(&self, column_id: Uuid) -> Result<bool, DomainError>;

    // =========================================================================
    // Card CRUD
    // =========================================================================

    /// Add a card to a column
    async fn create_card(&self, column_id: Uuid, data: CreateCard) -> Result<Card, DomainError>;

    /// Update a card
    async fn update_card(&self, card_id: Uuid, data: UpdateCard) -> Result<Option<Card>, DomainError>;

    /// Move a card to another column/position
    async fn move_card(&self, card_id: Uuid, data: MoveCard) -> Result<Option<Card>, DomainError>;

    /// Delete a card
    async fn delete_card(&self, card_id: Uuid) -> Result<bool, DomainError>;
}
