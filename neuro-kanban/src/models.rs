//! =============================================================================
//! Kanban Models
//! =============================================================================
//! No longer depends on SurrealDB - uses backend data layer
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
    pub description: Option<String>,
    pub color: Option<String>,
    pub labels: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    #[serde(alias = "card_order")]
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
    pub wip_limit: Option<i32>,
    #[serde(alias = "column_order")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_card(column_id: Uuid) -> Card {
        Card {
            id: Uuid::new_v4(),
            column_id,
            title: "Task".to_string(),
            description: None,
            color: None,
            labels: vec!["bug".to_string()],
            due_date: None,
            order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_column(board_id: Uuid) -> Column {
        Column {
            id: Uuid::new_v4(),
            board_id,
            name: "To Do".to_string(),
            color: None,
            wip_limit: None,
            order: 0,
            cards: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_board() -> Board {
        Board {
            id: Uuid::new_v4(),
            name: "Project".to_string(),
            description: None,
            color: None,
            is_archived: false,
            columns: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn board_serde_roundtrip() {
        let b = sample_board();
        let json = serde_json::to_string(&b).unwrap();
        let back: Board = serde_json::from_str(&json).unwrap();
        assert_eq!(b.id, back.id);
        assert_eq!(b.name, back.name);
        assert!(back.columns.is_empty());
    }

    #[test]
    fn card_order_alias() {
        let card = sample_card(Uuid::new_v4());
        let json = serde_json::to_string(&card).unwrap();
        let back: Card = serde_json::from_str(&json).unwrap();
        assert_eq!(card.order, back.order);
        let alt_json = json.replace("\"order\"", "\"card_order\"");
        let back2: Card = serde_json::from_str(&alt_json).unwrap();
        assert_eq!(card.order, back2.order);
    }

    #[test]
    fn column_with_cards_serde_roundtrip() {
        let board_id = Uuid::new_v4();
        let mut col = sample_column(board_id);
        col.cards = vec![sample_card(col.id), sample_card(col.id)];
        let json = serde_json::to_string(&col).unwrap();
        let back: Column = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cards.len(), 2);
    }

    #[test]
    fn create_board_request_serde() {
        let json = r#"{"name":"New Board","with_default_columns":true}"#;
        let req: CreateBoardRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "New Board");
        assert_eq!(req.with_default_columns, Some(true));
    }

    #[test]
    fn move_card_request_serde() {
        let target_id = Uuid::new_v4();
        let json = format!(r#"{{"target_column_id":"{}","target_order":3}}"#, target_id);
        let req: MoveCardRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.target_column_id, target_id);
        assert_eq!(req.target_order, 3);
    }
}
