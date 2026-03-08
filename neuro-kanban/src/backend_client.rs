//! Backend client - HTTP client to tachikoma-backend data layer

use reqwest::Client;
use uuid::Uuid;
use tracing::{debug, error};

use crate::config::Config;
use crate::models::*;

pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/api/data/kanban", config.backend_url),
        }
    }

    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/health", self.base_url.replace("/api/data/kanban", "/api"));
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    // ==========================================================================
    // Board CRUD
    // ==========================================================================

    pub async fn get_all_boards(
        &self,
        include_archived: bool,
    ) -> Result<Vec<BoardSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/boards?include_archived={}", self.base_url, include_archived);
        debug!("GET {}", url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let boards: Vec<BoardSummary> = response.json().await?;
        Ok(boards)
    }

    pub async fn get_board(
        &self,
        id: Uuid,
    ) -> Result<Option<Board>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/boards/{}", self.base_url, id);
        debug!("GET {}", url);
        
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let board: Board = response.json().await?;
        Ok(Some(board))
    }

    pub async fn create_board(
        &self,
        data: CreateBoardRequest,
    ) -> Result<Board, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/boards", self.base_url);
        debug!("POST {}", url);
        
        let response = self.client.post(&url).json(&data).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let board: Board = response.json().await?;
        Ok(board)
    }

    pub async fn update_board(
        &self,
        id: Uuid,
        data: UpdateBoardRequest,
    ) -> Result<Option<Board>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/boards/{}", self.base_url, id);
        debug!("PATCH {}", url);
        
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let board: Board = response.json().await?;
        Ok(Some(board))
    }

    pub async fn delete_board(
        &self,
        id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/boards/{}", self.base_url, id);
        debug!("DELETE {}", url);
        
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        Ok(response.status().is_success())
    }

    // ==========================================================================
    // Column CRUD
    // ==========================================================================

    pub async fn create_column(
        &self,
        board_id: Uuid,
        data: CreateColumnRequest,
    ) -> Result<Column, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/boards/{}/columns", self.base_url, board_id);
        debug!("POST {}", url);
        
        let response = self.client.post(&url).json(&data).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let column: Column = response.json().await?;
        Ok(column)
    }

    pub async fn update_column(
        &self,
        column_id: Uuid,
        data: UpdateColumnRequest,
    ) -> Result<Option<Column>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/columns/{}", self.base_url, column_id);
        debug!("PATCH {}", url);
        
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let column: Column = response.json().await?;
        Ok(Some(column))
    }

    pub async fn reorder_column(
        &self,
        column_id: Uuid,
        data: ReorderColumnRequest,
    ) -> Result<Option<Column>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/columns/{}/reorder", self.base_url, column_id);
        debug!("POST {}", url);
        
        let response = self.client.post(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let column: Column = response.json().await?;
        Ok(Some(column))
    }

    pub async fn delete_column(
        &self,
        column_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/columns/{}", self.base_url, column_id);
        debug!("DELETE {}", url);
        
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        Ok(response.status().is_success())
    }

    // ==========================================================================
    // Card CRUD
    // ==========================================================================

    pub async fn create_card(
        &self,
        column_id: Uuid,
        data: CreateCardRequest,
    ) -> Result<Card, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/columns/{}/cards", self.base_url, column_id);
        debug!("POST {}", url);
        
        let response = self.client.post(&url).json(&data).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let card: Card = response.json().await?;
        Ok(card)
    }

    pub async fn update_card(
        &self,
        card_id: Uuid,
        data: UpdateCardRequest,
    ) -> Result<Option<Card>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/cards/{}", self.base_url, card_id);
        debug!("PATCH {}", url);
        
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let card: Card = response.json().await?;
        Ok(Some(card))
    }

    pub async fn move_card(
        &self,
        card_id: Uuid,
        data: MoveCardRequest,
    ) -> Result<Option<Card>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/cards/{}/move", self.base_url, card_id);
        debug!("POST {}", url);
        
        let response = self.client.post(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let card: Card = response.json().await?;
        Ok(Some(card))
    }

    pub async fn delete_card(
        &self,
        card_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/cards/{}", self.base_url, card_id);
        debug!("DELETE {}", url);
        
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        Ok(response.status().is_success())
    }
}
