//! Domain models and DTOs
//! No longer depends on SurrealDB - uses backend data layer

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Domain Models
// =============================================================================

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistWithItems {
    #[serde(flatten)]
    pub checklist: Checklist,
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedChecklists {
    pub data: Vec<Checklist>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

// =============================================================================
// Request DTOs
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChecklistItem {
    pub content: String,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChecklistItem {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub is_completed: Option<bool>,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportMarkdown {
    pub markdown: String,
    #[serde(default)]
    pub title: Option<String>,
}

// =============================================================================
// Response DTOs
// =============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistResponse {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_interval: Option<i64>,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<ChecklistItem>,
    pub progress: Progress,
}

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub completed: usize,
    pub total: usize,
    pub percentage: f32,
}

impl ChecklistResponse {
    pub fn from_checklist_with_items(checklist: Checklist, items: Vec<ChecklistItem>) -> Self {
        let completed = items.iter().filter(|i| i.is_completed).count();
        let total = items.len();
        let percentage = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        ChecklistResponse {
            id: checklist.id,
            title: checklist.title,
            description: checklist.description,
            priority: checklist.priority,
            due_date: checklist.due_date,
            notification_interval: checklist.notification_interval,
            is_archived: checklist.is_archived,
            created_at: checklist.created_at,
            updated_at: checklist.updated_at,
            items,
            progress: Progress {
                completed,
                total,
                percentage,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListChecklistsResponse {
    pub checklists: Vec<ChecklistSummary>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistSummary {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: Progress,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationInfo {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
}

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ListChecklistsQuery {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    #[serde(default)]
    pub include_archived: bool,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReorderItemsRequest {
    pub item_ids: Vec<Uuid>,
}
