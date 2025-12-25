use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// =============================================================================
// Card (Tarjeta Kanban)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub labels: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardRequest {
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub labels: Option<Vec<String>>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCardRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub labels: Option<Vec<String>>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCardRequest {
    pub target_column_id: Uuid,
    pub target_order: i32,
}

// =============================================================================
// Column (Columna Kanban)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub wip_limit: Option<i32>, // Work In Progress limit
    pub order: i32,
    pub cards: Vec<Card>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateColumnRequest {
    pub name: String,
    pub color: Option<String>,
    pub wip_limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateColumnRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub wip_limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderColumnRequest {
    pub target_order: i32,
}

// =============================================================================
// Board (Tablero Kanban)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_archived: bool,
    pub columns: Vec<Column>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_archived: bool,
    pub column_count: usize,
    pub card_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    /// If true, create default columns (To Do, In Progress, Done)
    pub with_default_columns: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBoardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_archived: Option<bool>,
}

// =============================================================================
// In-Memory State
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct KanbanState {
    pub boards: HashMap<Uuid, Board>,
}

impl KanbanState {
    pub fn new() -> Self {
        Self {
            boards: HashMap::new(),
        }
    }
}

impl Board {
    pub fn to_summary(&self) -> BoardSummary {
        let card_count: usize = self.columns.iter().map(|c| c.cards.len()).sum();
        BoardSummary {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            color: self.color.clone(),
            is_archived: self.is_archived,
            column_count: self.columns.len(),
            card_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// =============================================================================
// Default Columns
// =============================================================================

pub fn create_default_columns(board_id: Uuid) -> Vec<Column> {
    let now = Utc::now();
    vec![
        Column {
            id: Uuid::new_v4(),
            board_id,
            name: "To Do".to_string(),
            color: Some("#6366f1".to_string()), // Indigo
            wip_limit: None,
            order: 0,
            cards: vec![],
            created_at: now,
            updated_at: now,
        },
        Column {
            id: Uuid::new_v4(),
            board_id,
            name: "In Progress".to_string(),
            color: Some("#f59e0b".to_string()), // Amber
            wip_limit: Some(5),
            order: 1,
            cards: vec![],
            created_at: now,
            updated_at: now,
        },
        Column {
            id: Uuid::new_v4(),
            board_id,
            name: "Done".to_string(),
            color: Some("#22c55e".to_string()), // Green
            wip_limit: None,
            order: 2,
            cards: vec![],
            created_at: now,
            updated_at: now,
        },
    ]
}
