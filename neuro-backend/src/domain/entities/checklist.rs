//! =============================================================================
//! Checklist Domain Entities
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Checklist entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_interval: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reminded: Option<DateTime<Utc>>,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Checklist item entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub content: String,
    pub is_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(rename = "item_order")]
    pub order: i32,
    pub created_at: DateTime<Utc>,
}

/// Request to create a checklist
#[derive(Debug, Clone, Deserialize)]
pub struct CreateChecklist {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub items: Vec<CreateChecklistItem>,
}

/// Request to update a checklist
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChecklist {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notification_interval: Option<i64>,
    #[serde(default)]
    pub is_archived: Option<bool>,
}

/// Request to create a checklist item
#[derive(Debug, Clone, Deserialize)]
pub struct CreateChecklistItem {
    pub content: String,
    #[serde(default)]
    pub order: Option<i32>,
}

/// Request to update a checklist item
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChecklistItem {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub is_completed: Option<bool>,
    #[serde(default)]
    pub order: Option<i32>,
}

/// Checklist with items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistWithItems {
    #[serde(flatten)]
    pub checklist: Checklist,
    pub items: Vec<ChecklistItem>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedChecklists {
    pub data: Vec<Checklist>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}
