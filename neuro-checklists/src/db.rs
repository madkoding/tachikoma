//! Database connection and repository

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{Checklist, ChecklistItem, ChecklistRecord, ChecklistItemRecord, CreateChecklist, CreateChecklistItem, UpdateChecklist, UpdateChecklistItem};

pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    pub async fn connect(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // SurrealDB ws driver expects just host:port
        let db_url = config.database_url
            .replace("ws://", "")
            .replace("wss://", "");
        
        tracing::info!("Connecting to database at: {}", db_url);
        
        let client = Surreal::new::<Ws>(&db_url).await?;
        
        tracing::info!("WebSocket connected, signing in...");
        
        client.signin(Root {
            username: &config.database_user,
            password: &config.database_pass,
        }).await?;
        
        tracing::info!("Signed in, selecting namespace/database...");

        client.use_ns(&config.database_namespace).use_db(&config.database_name).await?;

        // Initialize schema
        Self::init_schema(&client).await?;

        Ok(Self { client })
    }

    async fn init_schema(client: &Surreal<Client>) -> Result<(), Box<dyn std::error::Error>> {
        // Create tables (OVERWRITE ensures idempotency in SurrealDB 1.5.x)
        client.query(r#"
            DEFINE TABLE checklist SCHEMAFULL;
            DEFINE FIELD title ON TABLE checklist TYPE string;
            DEFINE FIELD description ON TABLE checklist TYPE option<string>;
            DEFINE FIELD priority ON TABLE checklist TYPE int DEFAULT 3;
            DEFINE FIELD due_date ON TABLE checklist TYPE option<datetime>;
            DEFINE FIELD notification_interval ON TABLE checklist TYPE option<int>;
            DEFINE FIELD last_reminded ON TABLE checklist TYPE option<datetime>;
            DEFINE FIELD is_archived ON TABLE checklist TYPE bool DEFAULT false;
            DEFINE FIELD created_at ON TABLE checklist TYPE datetime DEFAULT time::now();
            DEFINE FIELD updated_at ON TABLE checklist TYPE datetime DEFAULT time::now();

            DEFINE TABLE checklist_item SCHEMAFULL;
            DEFINE FIELD checklist_id ON TABLE checklist_item TYPE string;
            DEFINE FIELD content ON TABLE checklist_item TYPE string;
            DEFINE FIELD is_completed ON TABLE checklist_item TYPE bool DEFAULT false;
            DEFINE FIELD completed_at ON TABLE checklist_item TYPE option<datetime>;
            DEFINE FIELD item_order ON TABLE checklist_item TYPE int DEFAULT 0;
            DEFINE FIELD created_at ON TABLE checklist_item TYPE datetime DEFAULT time::now();

            DEFINE INDEX idx_item_checklist ON TABLE checklist_item COLUMNS checklist_id;
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
        let records: Vec<ChecklistRecord> = result.take(0)?;
        Ok(records.into_iter().map(Checklist::from).collect())
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
        let query = format!("SELECT * FROM checklist:`{}`", id);
        let mut result = self.client.query(&query).await?;
        let record: Option<ChecklistRecord> = result.take(0)?;
        Ok(record.map(Checklist::from))
    }

    pub async fn create_checklist(&self, data: CreateChecklist) -> Result<Checklist, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4();

        // Use SQL query to create - this handles datetime properly with time::now()
        // Use NONE instead of null for option types in SurrealDB
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

        let mut result = self.client
            .query(&query)
            .bind(("title", data.title.clone()))
            .bind(("description", data.description.clone()))
            .bind(("priority", data.priority.unwrap_or(3)))
            .bind(("due_date", data.due_date))
            .await?;

        let record: Option<ChecklistRecord> = result.take(0)?;
        record.map(Checklist::from).ok_or_else(|| "Failed to create checklist".into())
    }

    pub async fn update_checklist(&self, id: Uuid, data: UpdateChecklist) -> Result<Option<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let existing = self.get_checklist(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        // Use SQL UPDATE to avoid datetime serialization issues
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

        let mut result = self.client
            .query(&query)
            .bind(("title", data.title.unwrap_or(existing.title)))
            .bind(("description", data.description.or(existing.description)))
            .bind(("priority", data.priority.unwrap_or(existing.priority)))
            .bind(("due_date", data.due_date.or(existing.due_date)))
            .bind(("notification_interval", data.notification_interval.or(existing.notification_interval)))
            .bind(("is_archived", data.is_archived.unwrap_or(existing.is_archived)))
            .await?;

        let record: Option<ChecklistRecord> = result.take(0)?;
        Ok(record.map(Checklist::from))
    }

    pub async fn delete_checklist(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // First check if checklist exists
        let exists = self.get_checklist(id).await?;
        if exists.is_none() {
            return Ok(false);
        }

        // Delete all items first
        self.client
            .query("DELETE FROM checklist_item WHERE checklist_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete checklist
        let query = format!("DELETE checklist:`{}`", id);
        self.client.query(&query).await?;

        Ok(true)
    }

    // ==========================================================================
    // Checklist Items CRUD
    // ==========================================================================

    pub async fn get_items(&self, checklist_id: Uuid) -> Result<Vec<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM checklist_item WHERE checklist_id = $id ORDER BY item_order ASC")
            .bind(("id", checklist_id.to_string()))
            .await?;

        let records: Vec<ChecklistItemRecord> = result.take(0)?;
        Ok(records.into_iter().map(ChecklistItem::from).collect())
    }

    pub async fn add_item(&self, checklist_id: Uuid, data: CreateChecklistItem) -> Result<ChecklistItem, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4();

        // Get max order
        let mut result = self.client
            .query("SELECT math::max(item_order) as max_order FROM checklist_item WHERE checklist_id = $id GROUP ALL")
            .bind(("id", checklist_id.to_string()))
            .await?;
        
        let max_order: Option<MaxOrderResult> = result.take(0)?;
        let order = data.order.unwrap_or_else(|| max_order.map(|m| m.max_order + 1).unwrap_or(0));

        // Use SQL query to create - this handles datetime properly with time::now()
        // Note: completed_at needs to use time::now() conditionally in SQL, use NONE instead of null
        let query = if data.is_completed {
            format!(
                r#"CREATE checklist_item:`{}` SET
                    checklist_id = $checklist_id,
                    content = $content,
                    is_completed = true,
                    completed_at = time::now(),
                    item_order = $order,
                    created_at = time::now()
                "#,
                id
            )
        } else {
            format!(
                r#"CREATE checklist_item:`{}` SET
                    checklist_id = $checklist_id,
                    content = $content,
                    is_completed = false,
                    completed_at = NONE,
                    item_order = $order,
                    created_at = time::now()
                "#,
                id
            )
        };

        let mut result = self.client
            .query(&query)
            .bind(("checklist_id", checklist_id.to_string()))
            .bind(("content", data.content.clone()))
            .bind(("order", order))
            .await?;

        let record: Option<ChecklistItemRecord> = result.take(0)?;

        // Update checklist's updated_at
        self.client
            .query(format!("UPDATE checklist:`{}` SET updated_at = time::now()", checklist_id))
            .await?;

        record.map(ChecklistItem::from).ok_or_else(|| "Failed to create item".into())
    }

    pub async fn update_item(&self, item_id: Uuid, data: UpdateChecklistItem) -> Result<Option<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("SELECT * FROM checklist_item:`{}`", item_id);
        let mut result = self.client.query(&query).await?;
        let existing: Option<ChecklistItemRecord> = result.take(0)?;

        if existing.is_none() {
            return Ok(None);
        }
        let existing_record = existing.unwrap();
        let existing = ChecklistItem::from(existing_record.clone());

        let is_completed = data.is_completed.unwrap_or(existing.is_completed);
        
        // Use SQL UPDATE to handle datetime properly, use NONE instead of null
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

        let mut result = self.client
            .query(&query)
            .bind(("content", data.content.unwrap_or(existing.content)))
            .bind(("is_completed", is_completed))
            .bind(("order", data.order.unwrap_or(existing.order)))
            .await?;

        let record: Option<ChecklistItemRecord> = result.take(0)?;
        Ok(record.map(ChecklistItem::from))
    }

    pub async fn toggle_item(&self, item_id: Uuid) -> Result<Option<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("SELECT * FROM checklist_item:`{}`", item_id);
        let mut result = self.client.query(&query).await?;
        let existing: Option<ChecklistItemRecord> = result.take(0)?;

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

    pub async fn delete_item(&self, item_id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // First check if item exists
        let query = format!("SELECT * FROM checklist_item:`{}`", item_id);
        let mut result = self.client.query(&query).await?;
        let existing: Option<ChecklistItemRecord> = result.take(0)?;
        
        if existing.is_none() {
            return Ok(false);
        }

        let query = format!("DELETE checklist_item:`{}`", item_id);
        self.client.query(&query).await?;

        Ok(true)
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
