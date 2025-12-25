//! Domain models and DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;

// =============================================================================
// Helper to convert SurrealDB Thing to UUID
// =============================================================================

fn thing_to_uuid(thing: &Thing) -> Option<Uuid> {
    match &thing.id {
        surrealdb::sql::Id::String(s) => Uuid::parse_str(s).ok(),
        _ => None,
    }
}

// =============================================================================
// Internal DB Record types (with Thing id)
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ChecklistRecord {
    pub id: Thing,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
    pub notification_interval: Option<i64>,
    pub last_reminded: Option<DateTime<Utc>>,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChecklistItemRecord {
    pub id: Thing,
    pub checklist_id: String,
    pub content: String,
    pub is_completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(rename = "item_order")]
    pub order: i32,
    pub created_at: DateTime<Utc>,
}

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

impl From<ChecklistRecord> for Checklist {
    fn from(record: ChecklistRecord) -> Self {
        Checklist {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            title: record.title,
            description: record.description,
            priority: record.priority,
            due_date: record.due_date,
            notification_interval: record.notification_interval,
            last_reminded: record.last_reminded,
            is_archived: record.is_archived,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
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

impl From<ChecklistItemRecord> for ChecklistItem {
    fn from(record: ChecklistItemRecord) -> Self {
        ChecklistItem {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            checklist_id: Uuid::parse_str(&record.checklist_id).unwrap_or_default(),
            content: record.content,
            is_completed: record.is_completed,
            completed_at: record.completed_at,
            order: record.order,
            created_at: record.created_at,
        }
    }
}

// =============================================================================
// Request DTOs
// =============================================================================

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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateChecklistItem {
    pub content: String,
    #[serde(default)]
    pub is_completed: bool,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_interval: Option<i64>,
    pub is_archived: bool,
    pub total_items: usize,
    pub completed_items: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistWithItemsResponse {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_interval: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reminded: Option<String>,
    pub is_archived: bool,
    pub items: Vec<ChecklistItemResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistItemResponse {
    pub id: Uuid,
    pub content: String,
    pub is_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

// =============================================================================
// Conversion helpers
// =============================================================================

impl Checklist {
    pub fn to_response(&self, items: &[ChecklistItem]) -> ChecklistResponse {
        let completed_items = items.iter().filter(|i| i.is_completed).count();
        
        ChecklistResponse {
            id: self.id,
            title: self.title.clone(),
            description: self.description.clone(),
            priority: self.priority,
            due_date: self.due_date.map(|d| d.to_rfc3339()),
            notification_interval: self.notification_interval,
            is_archived: self.is_archived,
            total_items: items.len(),
            completed_items,
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }

    pub fn to_response_with_items(&self, items: Vec<ChecklistItem>) -> ChecklistWithItemsResponse {
        ChecklistWithItemsResponse {
            id: self.id,
            title: self.title.clone(),
            description: self.description.clone(),
            priority: self.priority,
            due_date: self.due_date.map(|d| d.to_rfc3339()),
            notification_interval: self.notification_interval,
            last_reminded: self.last_reminded.map(|d| d.to_rfc3339()),
            is_archived: self.is_archived,
            items: items.into_iter().map(|i| i.to_response()).collect(),
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }
}

impl ChecklistItem {
    pub fn to_response(&self) -> ChecklistItemResponse {
        ChecklistItemResponse {
            id: self.id,
            content: self.content.clone(),
            is_completed: self.is_completed,
            completed_at: self.completed_at.map(|d| d.to_rfc3339()),
            order: self.order,
            created_at: self.created_at.to_rfc3339(),
        }
    }
}

// =============================================================================
// Markdown Parser
// =============================================================================

pub fn parse_markdown_checklist(markdown: &str) -> (String, Vec<CreateChecklistItem>) {
    let mut title = String::from("Imported Checklist");
    let mut items = Vec::new();
    let checkbox_regex = regex::Regex::new(r"^[-*]\s*\[([ xX])\]\s*(.+)$").unwrap();

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Check for title
        if trimmed.starts_with('#') {
            title = trimmed.trim_start_matches('#').trim().to_string();
            continue;
        }

        // Parse checkbox items
        if let Some(captures) = checkbox_regex.captures(trimmed) {
            let is_completed = captures.get(1).map(|m| m.as_str().to_lowercase() == "x").unwrap_or(false);
            let content = captures.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            
            if !content.is_empty() {
                items.push(CreateChecklistItem {
                    content,
                    is_completed,
                    order: Some(items.len() as i32),
                });
            }
        }
    }

    (title, items)
}
