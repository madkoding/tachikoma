//! Database connection and repository

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{Checklist, ChecklistItem, CreateChecklist, CreateChecklistItem, UpdateChecklist, UpdateChecklistItem};

pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    pub async fn connect(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Surreal::new::<Ws>(&config.database_url).await?;
        
        client.signin(Root {
            username: &config.database_user,
            password: &config.database_pass,
        }).await?;

        client.use_ns(&config.database_namespace).use_db(&config.database_name).await?;

        // Initialize schema
        Self::init_schema(&client).await?;

        Ok(Self { client })
    }

    async fn init_schema(client: &Surreal<Client>) -> Result<(), Box<dyn std::error::Error>> {
        // Create tables if they don't exist
        client.query(r#"
            DEFINE TABLE IF NOT EXISTS checklist SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS title ON TABLE checklist TYPE string;
            DEFINE FIELD IF NOT EXISTS description ON TABLE checklist TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS priority ON TABLE checklist TYPE int DEFAULT 3;
            DEFINE FIELD IF NOT EXISTS due_date ON TABLE checklist TYPE option<datetime>;
            DEFINE FIELD IF NOT EXISTS notification_interval ON TABLE checklist TYPE option<int>;
            DEFINE FIELD IF NOT EXISTS last_reminded ON TABLE checklist TYPE option<datetime>;
            DEFINE FIELD IF NOT EXISTS is_archived ON TABLE checklist TYPE bool DEFAULT false;
            DEFINE FIELD IF NOT EXISTS created_at ON TABLE checklist TYPE datetime DEFAULT time::now();
            DEFINE FIELD IF NOT EXISTS updated_at ON TABLE checklist TYPE datetime DEFAULT time::now();

            DEFINE TABLE IF NOT EXISTS checklist_item SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS checklist_id ON TABLE checklist_item TYPE string;
            DEFINE FIELD IF NOT EXISTS content ON TABLE checklist_item TYPE string;
            DEFINE FIELD IF NOT EXISTS is_completed ON TABLE checklist_item TYPE bool DEFAULT false;
            DEFINE FIELD IF NOT EXISTS completed_at ON TABLE checklist_item TYPE option<datetime>;
            DEFINE FIELD IF NOT EXISTS item_order ON TABLE checklist_item TYPE int DEFAULT 0;
            DEFINE FIELD IF NOT EXISTS created_at ON TABLE checklist_item TYPE datetime DEFAULT time::now();

            DEFINE INDEX IF NOT EXISTS idx_item_checklist ON TABLE checklist_item COLUMNS checklist_id;
        "#).await?;

        Ok(())
    }

    // ==========================================================================
    // Checklist CRUD
    // ==========================================================================

    pub async fn get_all_checklists(&self, limit: usize, offset: usize, include_archived: bool) -> Result<Vec<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let query = if include_archived {
            format!("SELECT * FROM checklist ORDER BY created_at DESC LIMIT {} START {}", limit, offset)
        } else {
            format!("SELECT * FROM checklist WHERE is_archived = false ORDER BY created_at DESC LIMIT {} START {}", limit, offset)
        };

        let mut result = self.client.query(&query).await?;
        let checklists: Vec<Checklist> = result.take(0)?;
        Ok(checklists)
    }

    pub async fn count_checklists(&self, include_archived: bool) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let query = if include_archived {
            "SELECT count() FROM checklist GROUP ALL"
        } else {
            "SELECT count() FROM checklist WHERE is_archived = false GROUP ALL"
        };

        let mut result = self.client.query(query).await?;
        let count: Option<CountResult> = result.take(0)?;
        Ok(count.map(|c| c.count).unwrap_or(0))
    }

    pub async fn get_checklist(&self, id: Uuid) -> Result<Option<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let checklist: Option<Checklist> = self.client
            .select(("checklist", id.to_string()))
            .await?;
        Ok(checklist)
    }

    pub async fn create_checklist(&self, data: CreateChecklist) -> Result<Checklist, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let checklist = Checklist {
            id,
            title: data.title,
            description: data.description,
            priority: data.priority.unwrap_or(3),
            due_date: data.due_date,
            notification_interval: None,
            last_reminded: None,
            is_archived: false,
            created_at: now,
            updated_at: now,
        };

        let created: Option<Checklist> = self.client
            .create(("checklist", id.to_string()))
            .content(&checklist)
            .await?;

        created.ok_or_else(|| "Failed to create checklist".into())
    }

    pub async fn update_checklist(&self, id: Uuid, data: UpdateChecklist) -> Result<Option<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let existing = self.get_checklist(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let updated = Checklist {
            id,
            title: data.title.unwrap_or(existing.title),
            description: data.description.or(existing.description),
            priority: data.priority.unwrap_or(existing.priority),
            due_date: data.due_date.or(existing.due_date),
            notification_interval: data.notification_interval.or(existing.notification_interval),
            last_reminded: existing.last_reminded,
            is_archived: data.is_archived.unwrap_or(existing.is_archived),
            created_at: existing.created_at,
            updated_at: chrono::Utc::now(),
        };

        let result: Option<Checklist> = self.client
            .update(("checklist", id.to_string()))
            .content(&updated)
            .await?;

        Ok(result)
    }

    pub async fn delete_checklist(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Delete all items first
        self.client
            .query("DELETE FROM checklist_item WHERE checklist_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete checklist
        let deleted: Option<Checklist> = self.client
            .delete(("checklist", id.to_string()))
            .await?;

        Ok(deleted.is_some())
    }

    // ==========================================================================
    // Checklist Items CRUD
    // ==========================================================================

    pub async fn get_items(&self, checklist_id: Uuid) -> Result<Vec<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM checklist_item WHERE checklist_id = $id ORDER BY item_order ASC")
            .bind(("id", checklist_id.to_string()))
            .await?;

        let items: Vec<ChecklistItem> = result.take(0)?;
        Ok(items)
    }

    pub async fn add_item(&self, checklist_id: Uuid, data: CreateChecklistItem) -> Result<ChecklistItem, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Get max order
        let mut result = self.client
            .query("SELECT math::max(item_order) as max_order FROM checklist_item WHERE checklist_id = $id GROUP ALL")
            .bind(("id", checklist_id.to_string()))
            .await?;
        
        let max_order: Option<MaxOrderResult> = result.take(0)?;
        let order = data.order.unwrap_or_else(|| max_order.map(|m| m.max_order + 1).unwrap_or(0));

        let item = ChecklistItem {
            id,
            checklist_id,
            content: data.content,
            is_completed: data.is_completed,
            completed_at: if data.is_completed { Some(now) } else { None },
            order,
            created_at: now,
        };

        let created: Option<ChecklistItem> = self.client
            .create(("checklist_item", id.to_string()))
            .content(&item)
            .await?;

        // Update checklist's updated_at
        self.client
            .query("UPDATE checklist SET updated_at = time::now() WHERE id = $id")
            .bind(("id", format!("checklist:{}", checklist_id)))
            .await?;

        created.ok_or_else(|| "Failed to create item".into())
    }

    pub async fn update_item(&self, item_id: Uuid, data: UpdateChecklistItem) -> Result<Option<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let existing: Option<ChecklistItem> = self.client
            .select(("checklist_item", item_id.to_string()))
            .await?;

        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let now = chrono::Utc::now();
        let is_completed = data.is_completed.unwrap_or(existing.is_completed);
        
        let updated = ChecklistItem {
            id: item_id,
            checklist_id: existing.checklist_id,
            content: data.content.unwrap_or(existing.content),
            is_completed,
            completed_at: if is_completed && existing.completed_at.is_none() {
                Some(now)
            } else if !is_completed {
                None
            } else {
                existing.completed_at
            },
            order: data.order.unwrap_or(existing.order),
            created_at: existing.created_at,
        };

        let result: Option<ChecklistItem> = self.client
            .update(("checklist_item", item_id.to_string()))
            .content(&updated)
            .await?;

        Ok(result)
    }

    pub async fn toggle_item(&self, item_id: Uuid) -> Result<Option<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let existing: Option<ChecklistItem> = self.client
            .select(("checklist_item", item_id.to_string()))
            .await?;

        if let Some(item) = existing {
            self.update_item(item_id, UpdateChecklistItem {
                content: None,
                is_completed: Some(!item.is_completed),
                order: None,
            }).await
        } else {
            Ok(None)
        }
    }

    pub async fn delete_item(&self, item_id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let deleted: Option<ChecklistItem> = self.client
            .delete(("checklist_item", item_id.to_string()))
            .await?;

        Ok(deleted.is_some())
    }
}

#[derive(serde::Deserialize)]
struct CountResult {
    count: usize,
}

#[derive(serde::Deserialize)]
struct MaxOrderResult {
    max_order: i32,
}
