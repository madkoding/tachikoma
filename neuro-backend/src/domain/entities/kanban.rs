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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_board() -> Board {
        Board {
            id: Uuid::new_v4(),
            name: "Test Board".to_string(),
            description: None,
            color: None,
            is_archived: false,
            columns: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_board_to_summary_empty() {
        let board = make_board();
        let summary = board.to_summary();
        assert_eq!(summary.column_count, 0);
        assert_eq!(summary.card_count, 0);
        assert_eq!(summary.name, "Test Board");
    }

    #[test]
    fn test_board_to_summary_with_cards() {
        let mut board = make_board();
        let col_id = Uuid::new_v4();
        let card = Card {
            id: Uuid::new_v4(),
            column_id: col_id,
            title: "Task".to_string(),
            description: None,
            color: None,
            labels: vec![],
            due_date: None,
            order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        board.columns.push(Column {
            id: col_id,
            board_id: board.id,
            name: "To Do".to_string(),
            color: None,
            wip_limit: None,
            order: 0,
            cards: vec![card],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        let summary = board.to_summary();
        assert_eq!(summary.column_count, 1);
        assert_eq!(summary.card_count, 1);
    }

    #[test]
    fn test_move_card_fields() {
        let mv = MoveCard {
            target_column_id: Uuid::new_v4(),
            target_order: 2,
        };
        assert_eq!(mv.target_order, 2);
    }

    #[test]
    fn test_create_board_with_defaults() {
        let req = CreateBoard {
            name: "Sprint".to_string(),
            description: None,
            color: Some("#ff0000".to_string()),
            with_default_columns: Some(true),
        };
        assert_eq!(req.name, "Sprint");
        assert_eq!(req.with_default_columns, Some(true));
    }

    #[test]
    fn test_update_board_partial() {
        let req = UpdateBoard {
            name: None,
            description: Some("Updated".to_string()),
            color: None,
            is_archived: Some(true),
        };
        assert_eq!(req.is_archived, Some(true));
        assert!(req.name.is_none());
    }
}
