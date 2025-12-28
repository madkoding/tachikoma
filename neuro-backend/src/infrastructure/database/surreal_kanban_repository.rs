//! =============================================================================
//! SurrealDB Kanban Repository
//! =============================================================================

use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::sql::Thing;
use tracing::{debug, error};
use uuid::Uuid;

use crate::domain::entities::kanban::{
    Board, BoardSummary, Card, Column,
    CreateBoard, CreateCard, CreateColumn,
    MoveCard, ReorderColumn,
    UpdateBoard, UpdateCard, UpdateColumn,
};
use crate::domain::errors::DomainError;
use crate::domain::ports::kanban_repository::KanbanRepository;
use crate::infrastructure::database::DatabasePool;

/// SurrealDB implementation of KanbanRepository
#[derive(Clone)]
pub struct SurrealKanbanRepository {
    pool: DatabasePool,
}

// =============================================================================
// Internal Record Types (with SurrealDB Thing IDs)
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
struct BoardRecord {
    id: Thing,
    name: String,
    description: Option<String>,
    color: Option<String>,
    is_archived: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnRecord {
    id: Thing,
    board_id: String,
    name: String,
    color: Option<String>,
    wip_limit: Option<i32>,
    #[serde(rename = "column_order")]
    order: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct CardRecord {
    id: Thing,
    column_id: String,
    title: String,
    description: Option<String>,
    color: Option<String>,
    labels: Vec<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "card_order")]
    order: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct CountResult {
    count: usize,
}

#[derive(Deserialize)]
struct MaxOrderResult {
    max_order: i32,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn thing_to_uuid(thing: &Thing) -> Option<Uuid> {
    match &thing.id {
        surrealdb::sql::Id::String(s) => Uuid::parse_str(s).ok(),
        _ => None,
    }
}

impl From<BoardRecord> for Board {
    fn from(record: BoardRecord) -> Self {
        Board {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            name: record.name,
            description: record.description,
            color: record.color,
            is_archived: record.is_archived,
            columns: vec![], // Will be populated separately
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl From<ColumnRecord> for Column {
    fn from(record: ColumnRecord) -> Self {
        Column {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            board_id: Uuid::parse_str(&record.board_id).unwrap_or_default(),
            name: record.name,
            color: record.color,
            wip_limit: record.wip_limit,
            order: record.order,
            cards: vec![], // Will be populated separately
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl From<CardRecord> for Card {
    fn from(record: CardRecord) -> Self {
        Card {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            column_id: Uuid::parse_str(&record.column_id).unwrap_or_default(),
            title: record.title,
            description: record.description,
            color: record.color,
            labels: record.labels,
            due_date: record.due_date,
            order: record.order,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

// =============================================================================
// Default Columns
// =============================================================================

fn create_default_columns() -> Vec<(&'static str, Option<&'static str>, Option<i32>)> {
    vec![
        ("To Do", Some("#6366f1"), None),        // Indigo
        ("In Progress", Some("#f59e0b"), Some(5)), // Amber
        ("Done", Some("#22c55e"), None),          // Green
    ]
}

// =============================================================================
// Implementation
// =============================================================================

impl SurrealKanbanRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Get all columns for a board
    async fn get_columns_for_board(&self, board_id: Uuid) -> Result<Vec<Column>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM kanban_column WHERE board_id = $id ORDER BY column_order ASC")
            .bind(("id", board_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<ColumnRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut columns: Vec<Column> = Vec::new();
        for record in records {
            let column_id = thing_to_uuid(&record.id).unwrap_or_default();
            let mut column = Column::from(record);
            column.cards = self.get_cards_for_column(column_id).await?;
            columns.push(column);
        }
        
        Ok(columns)
    }

    /// Get all cards for a column
    async fn get_cards_for_column(&self, column_id: Uuid) -> Result<Vec<Card>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM kanban_card WHERE column_id = $id ORDER BY card_order ASC")
            .bind(("id", column_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<CardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().map(Card::from).collect())
    }

    /// Get a column by ID
    async fn get_column(&self, column_id: Uuid) -> Result<Option<Column>, DomainError> {
        let query = format!("SELECT * FROM kanban_column:`{}`", column_id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let record: Option<ColumnRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        match record {
            Some(r) => {
                let mut column = Column::from(r);
                column.cards = self.get_cards_for_column(column.id).await?;
                Ok(Some(column))
            }
            None => Ok(None),
        }
    }

    /// Get a card by ID
    async fn get_card(&self, card_id: Uuid) -> Result<Option<Card>, DomainError> {
        let query = format!("SELECT * FROM kanban_card:`{}`", card_id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let record: Option<CardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(Card::from))
    }

    /// Count columns for a board
    async fn count_columns(&self, board_id: Uuid) -> Result<usize, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT count() FROM kanban_column WHERE board_id = $id GROUP ALL")
            .bind(("id", board_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let count: Option<CountResult> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(count.map(|c| c.count).unwrap_or(0))
    }

    /// Count cards for a board (across all columns)
    async fn count_cards(&self, board_id: Uuid) -> Result<usize, DomainError> {
        let mut result = self.pool.client()
            .query(r#"
                SELECT count() FROM kanban_card 
                WHERE column_id IN (SELECT VALUE string::concat('kanban_column:', id) FROM kanban_column WHERE board_id = $id)
                GROUP ALL
            "#)
            .bind(("id", board_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let count: Option<CountResult> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(count.map(|c| c.count).unwrap_or(0))
    }
}

#[async_trait]
impl KanbanRepository for SurrealKanbanRepository {
    // =========================================================================
    // Board CRUD
    // =========================================================================

    async fn get_all_boards(&self, include_archived: bool) -> Result<Vec<BoardSummary>, DomainError> {
        let query = if include_archived {
            "SELECT * FROM kanban_board ORDER BY created_at DESC"
        } else {
            "SELECT * FROM kanban_board WHERE is_archived = false ORDER BY created_at DESC"
        };

        let mut result = self.pool.client().query(query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<BoardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let mut summaries = Vec::new();
        for record in records {
            let board_id = thing_to_uuid(&record.id).unwrap_or_default();
            let column_count = self.count_columns(board_id).await.unwrap_or(0);
            let card_count = self.count_cards(board_id).await.unwrap_or(0);
            
            summaries.push(BoardSummary {
                id: board_id,
                name: record.name,
                description: record.description,
                color: record.color,
                is_archived: record.is_archived,
                column_count,
                card_count,
                created_at: record.created_at,
                updated_at: record.updated_at,
            });
        }
        
        Ok(summaries)
    }

    async fn get_board(&self, id: Uuid) -> Result<Option<Board>, DomainError> {
        let query = format!("SELECT * FROM kanban_board:`{}`", id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let record: Option<BoardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        match record {
            Some(r) => {
                let mut board = Board::from(r);
                board.columns = self.get_columns_for_board(board.id).await?;
                Ok(Some(board))
            }
            None => Ok(None),
        }
    }

    async fn create_board(&self, data: CreateBoard) -> Result<Board, DomainError> {
        let id = Uuid::new_v4();

        let query = format!(
            r#"CREATE kanban_board:`{}` SET
                name = $name,
                description = $description,
                color = $color,
                is_archived = false,
                created_at = time::now(),
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("name", data.name.clone()))
            .bind(("description", data.description.clone()))
            .bind(("color", data.color.clone()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<BoardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let mut board = record
            .map(Board::from)
            .ok_or_else(|| DomainError::database("Failed to create board"))?;

        // Create default columns if requested
        if data.with_default_columns.unwrap_or(true) {
            let defaults = create_default_columns();
            for (i, (name, color, wip_limit)) in defaults.into_iter().enumerate() {
                let col = self.create_column_internal(id, CreateColumn {
                    name: name.to_string(),
                    color: color.map(|s| s.to_string()),
                    wip_limit,
                }, i as i32).await?;
                board.columns.push(col);
            }
        }

        debug!("Created kanban board: {} ({})", board.name, id);
        Ok(board)
    }

    async fn update_board(&self, id: Uuid, data: UpdateBoard) -> Result<Option<Board>, DomainError> {
        let existing = self.get_board(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let query = format!(
            r#"UPDATE kanban_board:`{}` SET
                name = $name,
                description = $description,
                color = $color,
                is_archived = $is_archived,
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("name", data.name.unwrap_or(existing.name)))
            .bind(("description", data.description.or(existing.description)))
            .bind(("color", data.color.or(existing.color)))
            .bind(("is_archived", data.is_archived.unwrap_or(existing.is_archived)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<BoardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        match record {
            Some(r) => {
                let mut board = Board::from(r);
                board.columns = self.get_columns_for_board(board.id).await?;
                Ok(Some(board))
            }
            None => Ok(None),
        }
    }

    async fn delete_board(&self, id: Uuid) -> Result<bool, DomainError> {
        let exists = self.get_board(id).await?;
        if exists.is_none() {
            return Ok(false);
        }

        // Delete all cards first
        self.pool.client()
            .query(r#"
                DELETE FROM kanban_card WHERE column_id IN 
                (SELECT VALUE string::concat('', <string>id) FROM kanban_column WHERE board_id = $id)
            "#)
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Delete all columns
        self.pool.client()
            .query("DELETE FROM kanban_column WHERE board_id = $id")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Delete board
        let query = format!("DELETE kanban_board:`{}`", id);
        self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!("Deleted kanban board: {}", id);
        Ok(true)
    }

    // =========================================================================
    // Column CRUD
    // =========================================================================

    async fn create_column(&self, board_id: Uuid, data: CreateColumn) -> Result<Column, DomainError> {
        // Get max order
        let mut result = self.pool.client()
            .query("SELECT math::max(column_order) as max_order FROM kanban_column WHERE board_id = $id GROUP ALL")
            .bind(("id", board_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let max_order: Option<MaxOrderResult> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        let order = max_order.map(|m| m.max_order + 1).unwrap_or(0);

        self.create_column_internal(board_id, data, order).await
    }

    async fn update_column(&self, column_id: Uuid, data: UpdateColumn) -> Result<Option<Column>, DomainError> {
        let existing = self.get_column(column_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let query = format!(
            r#"UPDATE kanban_column:`{}` SET
                name = $name,
                color = $color,
                wip_limit = $wip_limit,
                updated_at = time::now()
            "#,
            column_id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("name", data.name.unwrap_or(existing.name)))
            .bind(("color", data.color.or(existing.color)))
            .bind(("wip_limit", data.wip_limit.or(existing.wip_limit)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ColumnRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        match record {
            Some(r) => {
                let mut column = Column::from(r);
                column.cards = self.get_cards_for_column(column.id).await?;
                Ok(Some(column))
            }
            None => Ok(None),
        }
    }

    async fn reorder_column(&self, column_id: Uuid, data: ReorderColumn) -> Result<Option<Column>, DomainError> {
        let existing = self.get_column(column_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();
        let old_order = existing.order;
        let new_order = data.target_order;

        if old_order == new_order {
            return Ok(Some(existing));
        }

        // Shift other columns
        if new_order > old_order {
            // Moving down: shift columns in range (old, new] up by -1
            self.pool.client()
                .query(r#"
                    UPDATE kanban_column SET column_order = column_order - 1 
                    WHERE board_id = $board_id AND column_order > $old AND column_order <= $new
                "#)
                .bind(("board_id", existing.board_id.to_string()))
                .bind(("old", old_order))
                .bind(("new", new_order))
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;
        } else {
            // Moving up: shift columns in range [new, old) down by +1
            self.pool.client()
                .query(r#"
                    UPDATE kanban_column SET column_order = column_order + 1 
                    WHERE board_id = $board_id AND column_order >= $new AND column_order < $old
                "#)
                .bind(("board_id", existing.board_id.to_string()))
                .bind(("old", old_order))
                .bind(("new", new_order))
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;
        }

        // Update target column
        let query = format!(
            "UPDATE kanban_column:`{}` SET column_order = $order, updated_at = time::now()",
            column_id
        );
        let mut result = self.pool.client()
            .query(&query)
            .bind(("order", new_order))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ColumnRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        match record {
            Some(r) => {
                let mut column = Column::from(r);
                column.cards = self.get_cards_for_column(column.id).await?;
                Ok(Some(column))
            }
            None => Ok(None),
        }
    }

    async fn delete_column(&self, column_id: Uuid) -> Result<bool, DomainError> {
        let exists = self.get_column(column_id).await?;
        if exists.is_none() {
            return Ok(false);
        }

        // Delete all cards
        self.pool.client()
            .query("DELETE FROM kanban_card WHERE column_id = $id")
            .bind(("id", column_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Delete column
        let query = format!("DELETE kanban_column:`{}`", column_id);
        self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(true)
    }

    // =========================================================================
    // Card CRUD
    // =========================================================================

    async fn create_card(&self, column_id: Uuid, data: CreateCard) -> Result<Card, DomainError> {
        let id = Uuid::new_v4();

        // Get max order
        let mut result = self.pool.client()
            .query("SELECT math::max(card_order) as max_order FROM kanban_card WHERE column_id = $id GROUP ALL")
            .bind(("id", column_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let max_order: Option<MaxOrderResult> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        let order = max_order.map(|m| m.max_order + 1).unwrap_or(0);

        let labels = data.labels.unwrap_or_default();

        let query = format!(
            r#"CREATE kanban_card:`{}` SET
                column_id = $column_id,
                title = $title,
                description = $description,
                color = $color,
                labels = $labels,
                due_date = $due_date,
                card_order = $order,
                created_at = time::now(),
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("column_id", column_id.to_string()))
            .bind(("title", data.title.clone()))
            .bind(("description", data.description.clone()))
            .bind(("color", data.color.clone()))
            .bind(("labels", labels))
            .bind(("due_date", data.due_date))
            .bind(("order", order))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<CardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        record
            .map(Card::from)
            .ok_or_else(|| DomainError::database("Failed to create card"))
    }

    async fn update_card(&self, card_id: Uuid, data: UpdateCard) -> Result<Option<Card>, DomainError> {
        let existing = self.get_card(card_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let query = format!(
            r#"UPDATE kanban_card:`{}` SET
                title = $title,
                description = $description,
                color = $color,
                labels = $labels,
                due_date = $due_date,
                updated_at = time::now()
            "#,
            card_id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("title", data.title.unwrap_or(existing.title)))
            .bind(("description", data.description.or(existing.description)))
            .bind(("color", data.color.or(existing.color)))
            .bind(("labels", data.labels.unwrap_or(existing.labels)))
            .bind(("due_date", data.due_date.or(existing.due_date)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<CardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(Card::from))
    }

    async fn move_card(&self, card_id: Uuid, data: MoveCard) -> Result<Option<Card>, DomainError> {
        let existing = self.get_card(card_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();
        let old_column_id = existing.column_id;
        let old_order = existing.order;
        let new_column_id = data.target_column_id;
        let new_order = data.target_order;

        // If same column, just reorder
        if old_column_id == new_column_id {
            if old_order == new_order {
                return Ok(Some(existing));
            }
            
            if new_order > old_order {
                self.pool.client()
                    .query(r#"
                        UPDATE kanban_card SET card_order = card_order - 1 
                        WHERE column_id = $col AND card_order > $old AND card_order <= $new
                    "#)
                    .bind(("col", old_column_id.to_string()))
                    .bind(("old", old_order))
                    .bind(("new", new_order))
                    .await
                    .map_err(|e| DomainError::database(e.to_string()))?;
            } else {
                self.pool.client()
                    .query(r#"
                        UPDATE kanban_card SET card_order = card_order + 1 
                        WHERE column_id = $col AND card_order >= $new AND card_order < $old
                    "#)
                    .bind(("col", old_column_id.to_string()))
                    .bind(("old", old_order))
                    .bind(("new", new_order))
                    .await
                    .map_err(|e| DomainError::database(e.to_string()))?;
            }
        } else {
            // Moving to different column
            // Shift down cards in old column
            self.pool.client()
                .query(r#"
                    UPDATE kanban_card SET card_order = card_order - 1 
                    WHERE column_id = $col AND card_order > $old
                "#)
                .bind(("col", old_column_id.to_string()))
                .bind(("old", old_order))
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            // Shift up cards in new column
            self.pool.client()
                .query(r#"
                    UPDATE kanban_card SET card_order = card_order + 1 
                    WHERE column_id = $col AND card_order >= $new
                "#)
                .bind(("col", new_column_id.to_string()))
                .bind(("new", new_order))
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;
        }

        // Update the card
        let query = format!(
            r#"UPDATE kanban_card:`{}` SET
                column_id = $column_id,
                card_order = $order,
                updated_at = time::now()
            "#,
            card_id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("column_id", new_column_id.to_string()))
            .bind(("order", new_order))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<CardRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(Card::from))
    }

    async fn delete_card(&self, card_id: Uuid) -> Result<bool, DomainError> {
        let exists = self.get_card(card_id).await?;
        if exists.is_none() {
            return Ok(false);
        }

        let query = format!("DELETE kanban_card:`{}`", card_id);
        self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(true)
    }
}

impl SurrealKanbanRepository {
    /// Internal helper to create column with specified order
    async fn create_column_internal(&self, board_id: Uuid, data: CreateColumn, order: i32) -> Result<Column, DomainError> {
        let id = Uuid::new_v4();

        let query = format!(
            r#"CREATE kanban_column:`{}` SET
                board_id = $board_id,
                name = $name,
                color = $color,
                wip_limit = $wip_limit,
                column_order = $order,
                created_at = time::now(),
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("board_id", board_id.to_string()))
            .bind(("name", data.name.clone()))
            .bind(("color", data.color.clone()))
            .bind(("wip_limit", data.wip_limit))
            .bind(("order", order))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ColumnRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        record
            .map(|r| {
                let mut col = Column::from(r);
                col.cards = vec![];
                col
            })
            .ok_or_else(|| DomainError::database("Failed to create column"))
    }
}
