//! =============================================================================
//! Kanban Domain Entities
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Card (Tarjeta Kanban)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(rename = "card_order")]
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCard {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCard {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoveCard {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wip_limit: Option<i32>,
    #[serde(rename = "column_order")]
    pub order: i32,
    pub cards: Vec<Card>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateColumn {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub wip_limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateColumn {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub wip_limit: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReorderColumn {
    pub target_order: i32,
}

// =============================================================================
// Board (Tablero Kanban)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub is_archived: bool,
    pub column_count: usize,
    pub card_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBoard {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    /// If true, create default columns (To Do, In Progress, Done)
    #[serde(default)]
    pub with_default_columns: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBoard {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub is_archived: Option<bool>,
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
