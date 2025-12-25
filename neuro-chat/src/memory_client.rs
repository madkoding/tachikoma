//! =============================================================================
//! Memory Service Client
//! =============================================================================

use tracing::{debug, error};

use crate::models::*;

#[derive(Clone)]
pub struct MemoryClient {
    client: reqwest::Client,
    base_url: String,
}

impl MemoryClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Search for relevant memories
    pub async fn search(&self, query: &str, limit: usize, threshold: f64) -> Result<Vec<MemorySearchResult>, String> {
        let url = format!("{}/api/memories/search", self.base_url);
        
        let request = MemorySearchRequest {
            query: query.to_string(),
            limit: Some(limit),
            threshold: Some(threshold),
        };

        debug!("Searching memories: {}", query);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Memory service error: {}", response.status()));
        }

        #[derive(serde::Deserialize)]
        struct SearchResponse {
            results: Vec<MemorySearchResult>,
        }

        let result: SearchResponse = response.json().await.map_err(|e| e.to_string())?;
        debug!("Found {} memories", result.results.len());
        Ok(result.results)
    }

    /// Create a new memory
    pub async fn create(&self, content: &str, memory_type: &str) -> Result<Memory, String> {
        let url = format!("{}/api/memories", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "content": content,
                "memory_type": memory_type
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Memory service error: {}", response.status()));
        }

        response.json().await.map_err(|e| e.to_string())
    }

    /// Check if memory service is healthy
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }
}
