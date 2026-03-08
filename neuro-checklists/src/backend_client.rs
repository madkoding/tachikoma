//! Backend client - HTTP client to tachikoma-backend data layer

use reqwest::Client;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{
    Checklist, ChecklistItem, ChecklistWithItems, CreateChecklist, CreateChecklistItem,
    PaginatedChecklists, UpdateChecklist, UpdateChecklistItem,
};

pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/api/data", config.backend_url),
        }
    }

    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/health", self.base_url.replace("/api/data", "/api"));
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    // ==========================================================================
    // Checklist CRUD
    // ==========================================================================

    pub async fn get_all_checklists(
        &self,
        limit: usize,
        offset: usize,
        include_archived: bool,
    ) -> Result<Vec<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let page = (offset / limit) + 1;
        let url = format!(
            "{}/checklists?page={}&per_page={}&include_archived={}",
            self.base_url, page, limit, include_archived
        );
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let paginated: PaginatedChecklists = response.json().await?;
        Ok(paginated.data)
    }

    pub async fn count_checklists(
        &self,
        include_archived: bool,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/checklists?page=1&per_page=1&include_archived={}",
            self.base_url, include_archived
        );
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Ok(0);
        }
        
        let paginated: PaginatedChecklists = response.json().await?;
        Ok(paginated.total)
    }

    pub async fn get_checklist(
        &self,
        id: Uuid,
    ) -> Result<Option<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let with_items: ChecklistWithItems = response.json().await?;
        Ok(Some(with_items.checklist))
    }

    pub async fn get_checklist_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<ChecklistWithItems>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let with_items: ChecklistWithItems = response.json().await?;
        Ok(Some(with_items))
    }

    pub async fn create_checklist(
        &self,
        data: CreateChecklist,
    ) -> Result<Checklist, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists", self.base_url);
        let response = self.client.post(&url).json(&data).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let checklist: Checklist = response.json().await?;
        Ok(checklist)
    }

    pub async fn update_checklist(
        &self,
        id: Uuid,
        data: UpdateChecklist,
    ) -> Result<Option<Checklist>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/{}", self.base_url, id);
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let checklist: Checklist = response.json().await?;
        Ok(Some(checklist))
    }

    pub async fn delete_checklist(
        &self,
        id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/{}", self.base_url, id);
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        Ok(response.status().is_success())
    }

    // ==========================================================================
    // Checklist Items CRUD
    // ==========================================================================

    pub async fn get_items(
        &self,
        checklist_id: Uuid,
    ) -> Result<Vec<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/{}/items", self.base_url, checklist_id);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let items: Vec<ChecklistItem> = response.json().await?;
        Ok(items)
    }

    pub async fn add_item(
        &self,
        checklist_id: Uuid,
        data: CreateChecklistItem,
    ) -> Result<ChecklistItem, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/{}/items", self.base_url, checklist_id);
        let response = self.client.post(&url).json(&data).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let item: ChecklistItem = response.json().await?;
        Ok(item)
    }

    pub async fn update_item(
        &self,
        item_id: Uuid,
        data: UpdateChecklistItem,
    ) -> Result<Option<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/items/{}", self.base_url, item_id);
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let item: ChecklistItem = response.json().await?;
        Ok(Some(item))
    }

    pub async fn toggle_item(
        &self,
        item_id: Uuid,
    ) -> Result<Option<ChecklistItem>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/items/{}/toggle", self.base_url, item_id);
        let response = self.client.post(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let item: ChecklistItem = response.json().await?;
        Ok(Some(item))
    }

    pub async fn delete_item(
        &self,
        item_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/checklists/items/{}", self.base_url, item_id);
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        Ok(response.status().is_success())
    }
}
