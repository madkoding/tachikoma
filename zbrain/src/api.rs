//! NEURO-OS API client for Z-Brain CLI.
//!
//! Provides HTTP client functionality to communicate with the NEURO-OS backend,
//! including chat endpoints, memory queries, and system health checks.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// API client for NEURO-OS backend
#[derive(Debug, Clone)]
pub struct NeuroClient {
    client: Client,
    base_url: String,
}

/// Chat request payload
#[derive(Debug, Serialize)]
struct ChatRequest {
    conversation_id: String,
    message: String,
}

/// Chat response from API
#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
    #[allow(dead_code)]
    model_used: Option<String>,
    #[allow(dead_code)]
    tokens_used: Option<u32>,
}

/// Health check response
#[derive(Debug, Deserialize)]
struct HealthResponse {
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    ollama: bool,
    #[allow(dead_code)]
    surrealdb: bool,
    #[allow(dead_code)]
    searxng: bool,
}

/// Memory item from API
#[derive(Debug, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: String,
    pub importance: f32,
    pub created_at: String,
}

/// System models response
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<String>,
}

impl NeuroClient {
    /// Create a new NEURO-OS API client
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Check if the API is healthy and reachable
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/system/health", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to NEURO-OS")?;

        if response.status().is_success() {
            let health: HealthResponse = response.json().await?;
            Ok(health.ollama)
        } else {
            Ok(false)
        }
    }

    /// Send a chat message and get a response
    pub async fn chat(&self, conversation_id: &str, message: &str) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            conversation_id: conversation_id.to_string(),
            message: message.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response.response)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Chat request failed ({}): {}",
                status,
                error_text
            ))
        }
    }

    /// Search memories by query
    pub async fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let url = format!(
            "{}/api/memories/search?query={}&limit={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to search memories")?;

        if response.status().is_success() {
            let memories: Vec<Memory> = response.json().await?;
            Ok(memories)
        } else {
            Ok(vec![])
        }
    }

    /// Get available models
    pub async fn get_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/system/models", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get models")?;

        if response.status().is_success() {
            let models: ModelsResponse = response.json().await?;
            Ok(models.models)
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = NeuroClient::new("http://localhost:3000");
        assert_eq!(client.base_url, "http://localhost:3000");
    }

    #[test]
    fn test_client_url_normalization() {
        let client = NeuroClient::new("http://localhost:3000/");
        assert_eq!(client.base_url, "http://localhost:3000");
    }
}
