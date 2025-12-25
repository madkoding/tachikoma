//! =============================================================================
//! SurrealDB Checklist Repository
//! =============================================================================

use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::sql::{Datetime, Thing};
use tracing::{debug, error};
use uuid::Uuid;

use crate::domain::entities::checklist::{
    Checklist, ChecklistItem, CreateChecklist, CreateChecklistItem,
    UpdateChecklist, UpdateChecklistItem,
};
use crate::domain::errors::DomainError;
use crate::domain::ports::checklist_repository::ChecklistRepository;
use crate::infrastructure::database::DatabasePool;

/// SurrealDB implementation of ChecklistRepository
#[derive(Clone)]
pub struct SurrealChecklistRepository {
    pool: DatabasePool,
}

// =============================================================================
// Internal Record Types (with SurrealDB Thing IDs)
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
struct ChecklistRecord {
    id: Thing,
    title: String,
    description: Option<String>,
    priority: i32,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    notification_interval: Option<i64>,
    last_reminded: Option<chrono::DateTime<chrono::Utc>>,
    is_archived: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChecklistItemRecord {
    id: Thing,
    checklist_id: String,
    content: String,
    is_completed: bool,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "item_order")]
    order: i32,
    created_at: chrono::DateTime<chrono::Utc>,
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
// Implementation
// =============================================================================

impl SurrealChecklistRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChecklistRepository for SurrealChecklistRepository {
    async fn get_all_checklists(
        &self,
        limit: usize,
        offset: usize,
        include_archived: bool,
    ) -> Result<Vec<Checklist>, DomainError> {
        let query = if include_archived {
            format!(
                "SELECT * FROM checklist ORDER BY created_at DESC LIMIT {} START {}",
                limit, offset
            )
        } else {
            format!(
                "SELECT * FROM checklist WHERE is_archived = false ORDER BY created_at DESC LIMIT {} START {}",
                limit, offset
            )
        };

        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<ChecklistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().map(Checklist::from).collect())
    }

    async fn count_checklists(&self, include_archived: bool) -> Result<usize, DomainError> {
        let query = if include_archived {
            "SELECT count() FROM checklist GROUP ALL"
        } else {
            "SELECT count() FROM checklist WHERE is_archived = false GROUP ALL"
        };

        let mut result = self.pool.client().query(query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let count: Option<CountResult> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(count.map(|c| c.count).unwrap_or(0))
    }

    async fn get_checklist(&self, id: Uuid) -> Result<Option<Checklist>, DomainError> {
        let query = format!("SELECT * FROM checklist:`{}`", id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let record: Option<ChecklistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(Checklist::from))
    }

    async fn create_checklist(&self, data: CreateChecklist) -> Result<Checklist, DomainError> {
        let id = Uuid::new_v4();

        let query = format!(
            r#"CREATE checklist:`{}` SET
                title = $title,
                description = $description,
                priority = $priority,
                due_date = $due_date,
                notification_interval = NONE,
                last_reminded = NONE,
                is_archived = false,
                created_at = time::now(),
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("title", data.title.clone()))
            .bind(("description", data.description.clone()))
            .bind(("priority", data.priority.unwrap_or(3)))
            .bind(("due_date", data.due_date))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ChecklistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        record
            .map(Checklist::from)
            .ok_or_else(|| DomainError::database("Failed to create checklist"))
    }

    async fn update_checklist(
        &self,
        id: Uuid,
        data: UpdateChecklist,
    ) -> Result<Option<Checklist>, DomainError> {
        let existing = self.get_checklist(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let query = format!(
            r#"UPDATE checklist:`{}` SET
                title = $title,
                description = $description,
                priority = $priority,
                due_date = $due_date,
                notification_interval = $notification_interval,
                is_archived = $is_archived,
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("title", data.title.unwrap_or(existing.title)))
            .bind(("description", data.description.or(existing.description)))
            .bind(("priority", data.priority.unwrap_or(existing.priority)))
            .bind(("due_date", data.due_date.or(existing.due_date)))
            .bind(("notification_interval", data.notification_interval.or(existing.notification_interval)))
            .bind(("is_archived", data.is_archived.unwrap_or(existing.is_archived)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ChecklistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(Checklist::from))
    }

    async fn delete_checklist(&self, id: Uuid) -> Result<bool, DomainError> {
        let exists = self.get_checklist(id).await?;
        if exists.is_none() {
            return Ok(false);
        }

        // Delete all items first
        self.pool.client()
            .query("DELETE FROM checklist_item WHERE checklist_id = $id")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Delete checklist
        let query = format!("DELETE checklist:`{}`", id);
        self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(true)
    }

    async fn get_items(&self, checklist_id: Uuid) -> Result<Vec<ChecklistItem>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM checklist_item WHERE checklist_id = $id ORDER BY item_order ASC")
            .bind(("id", checklist_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<ChecklistItemRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().map(ChecklistItem::from).collect())
    }

    async fn add_item(
        &self,
        checklist_id: Uuid,
        data: CreateChecklistItem,
    ) -> Result<ChecklistItem, DomainError> {
        let id = Uuid::new_v4();

        // Get max order
        let mut result = self.pool.client()
            .query("SELECT math::max(item_order) as max_order FROM checklist_item WHERE checklist_id = $id GROUP ALL")
            .bind(("id", checklist_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let max_order: Option<MaxOrderResult> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        let order = data.order.unwrap_or_else(|| max_order.map(|m| m.max_order + 1).unwrap_or(0));

        let query = format!(
            r#"CREATE checklist_item:`{}` SET
                checklist_id = $checklist_id,
                content = $content,
                is_completed = false,
                completed_at = NONE,
                item_order = $order,
                created_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("checklist_id", checklist_id.to_string()))
            .bind(("content", data.content.clone()))
            .bind(("order", order))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ChecklistItemRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Update checklist's updated_at
        self.pool.client()
            .query(format!("UPDATE checklist:`{}` SET updated_at = time::now()", checklist_id))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        record
            .map(ChecklistItem::from)
            .ok_or_else(|| DomainError::database("Failed to create item"))
    }

    async fn update_item(
        &self,
        item_id: Uuid,
        data: UpdateChecklistItem,
    ) -> Result<Option<ChecklistItem>, DomainError> {
        let query = format!("SELECT * FROM checklist_item:`{}`", item_id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let existing: Option<ChecklistItemRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        if existing.is_none() {
            return Ok(None);
        }
        let existing = ChecklistItem::from(existing.unwrap());

        let is_completed = data.is_completed.unwrap_or(existing.is_completed);
        
        let query = if is_completed && existing.completed_at.is_none() {
            format!(
                r#"UPDATE checklist_item:`{}` SET
                    content = $content,
                    is_completed = true,
                    completed_at = time::now(),
                    item_order = $order
                "#,
                item_id
            )
        } else if !is_completed {
            format!(
                r#"UPDATE checklist_item:`{}` SET
                    content = $content,
                    is_completed = false,
                    completed_at = NONE,
                    item_order = $order
                "#,
                item_id
            )
        } else {
            format!(
                r#"UPDATE checklist_item:`{}` SET
                    content = $content,
                    is_completed = $is_completed,
                    item_order = $order
                "#,
                item_id
            )
        };

        let mut result = self.pool.client()
            .query(&query)
            .bind(("content", data.content.unwrap_or(existing.content)))
            .bind(("is_completed", is_completed))
            .bind(("order", data.order.unwrap_or(existing.order)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<ChecklistItemRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(ChecklistItem::from))
    }

    async fn toggle_item(&self, item_id: Uuid) -> Result<Option<ChecklistItem>, DomainError> {
        let query = format!("SELECT * FROM checklist_item:`{}`", item_id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let existing: Option<ChecklistItemRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        if let Some(record) = existing {
            let item = ChecklistItem::from(record);
            self.update_item(item_id, UpdateChecklistItem {
                content: None,
                is_completed: Some(!item.is_completed),
                order: None,
            }).await
        } else {
            Ok(None)
        }
    }

    async fn delete_item(&self, item_id: Uuid) -> Result<bool, DomainError> {
        let query = format!("SELECT * FROM checklist_item:`{}`", item_id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let existing: Option<ChecklistItemRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        if existing.is_none() {
            return Ok(false);
        }

        let query = format!("DELETE checklist_item:`{}`", item_id);
        self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(true)
    }
}
