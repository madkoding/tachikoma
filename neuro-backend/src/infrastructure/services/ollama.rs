//! =============================================================================
//! Ollama Client - LLM Provider Implementation
//! =============================================================================
//! Implements the LlmProvider port using the Ollama API.
//! Handles chat completions, embeddings, and model management.
//! =============================================================================

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, warn};

use crate::domain::{
    errors::DomainError,
    ports::llm_provider::{LlmProvider, GenerationResult, ModelInfo},
};
use crate::infrastructure::config::OllamaConfig;

/// =============================================================================
/// OllamaClient - HTTP client for Ollama API
/// =============================================================================
#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

// ============================================================================
// Ollama API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaResponseMessage,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    prompt_eval_count: u64,
    #[serde(default)]
    eval_count: u64,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelListResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    modified_at: String,
}

// ============================================================================
// Implementation
// ============================================================================

impl OllamaClient {
    /// Create a new OllamaClient
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Build the API URL for a given endpoint
    fn api_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.config.url, endpoint)
    }

    /// Extract parameter count from model name (heuristic)
    fn extract_parameters(model_name: &str) -> Option<u64> {
        let lower = model_name.to_lowercase();
        
        if lower.contains("70b") { return Some(70_000_000_000); }
        if lower.contains("34b") || lower.contains("35b") { return Some(34_000_000_000); }
        if lower.contains("14b") || lower.contains("13b") { return Some(14_000_000_000); }
        if lower.contains("7b") || lower.contains("8b") { return Some(7_000_000_000); }
        if lower.contains("3b") || lower.contains("4b") { return Some(3_000_000_000); }
        if lower.contains("1b") || lower.contains("1.5b") { return Some(1_000_000_000); }
        
        None
    }

    /// Get the default model name
    pub fn default_model(&self) -> &str {
        &self.config.default_model
    }

    /// Get the embedding model name
    pub fn embedding_model(&self) -> &str {
        &self.config.embedding_model
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    /// Generate text completion
    #[instrument(skip(self, prompt))]
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<GenerationResult, DomainError> {
        let model_name = model.unwrap_or(&self.config.default_model);

        let request = OllamaChatRequest {
            model: model_name.to_string(),
            messages: vec![OllamaChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_predict: Some(2048),
            }),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %model_name, "Sending chat request");

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama API error: {} - {}",
                status, body
            )));
        }

        let ollama_response: OllamaChatResponse = response.json().await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;

        debug!(
            model = %ollama_response.model,
            tokens_prompt = ollama_response.prompt_eval_count,
            tokens_completion = ollama_response.eval_count,
            "Chat completion received"
        );

        Ok(GenerationResult {
            content: ollama_response.message.content,
            model: ollama_response.model,
            prompt_tokens: ollama_response.prompt_eval_count,
            completion_tokens: ollama_response.eval_count,
            finish_reason: if ollama_response.done { "stop" } else { "length" }.to_string(),
        })
    }

    /// Generate text embedding
    #[instrument(skip(self, text))]
    async fn embed(&self, text: &str) -> Result<Vec<f32>, DomainError> {
        let request = OllamaEmbeddingRequest {
            model: self.config.embedding_model.clone(),
            input: vec![text.to_string()],
        };

        let url = self.api_url("api/embed");
        debug!(url = %url, "Generating embedding");

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama embedding error: {} - {}",
                status, body
            )));
        }

        let ollama_response: OllamaEmbeddingResponse = response.json().await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;

        ollama_response.embeddings
            .into_iter()
            .next()
            .ok_or_else(|| DomainError::llm_error("No embedding returned"))
    }

    /// Generate embeddings for multiple texts
    #[instrument(skip(self, texts))]
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, DomainError> {
        let request = OllamaEmbeddingRequest {
            model: self.config.embedding_model.clone(),
            input: texts.to_vec(),
        };

        let url = self.api_url("api/embed");
        debug!(url = %url, count = texts.len(), "Generating batch embeddings");

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama embedding error: {} - {}",
                status, body
            )));
        }

        let ollama_response: OllamaEmbeddingResponse = response.json().await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;

        Ok(ollama_response.embeddings)
    }

    /// Check if a model is available
    #[instrument(skip(self))]
    async fn is_model_available(&self, model_name: &str) -> bool {
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.name == model_name || m.name.starts_with(model_name)),
            Err(_) => false,
        }
    }

    /// List available models
    #[instrument(skip(self))]
    async fn list_models(&self) -> Result<Vec<ModelInfo>, DomainError> {
        let url = self.api_url("api/tags");
        debug!(url = %url, "Listing available models");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama API error: {} - {}",
                status, body
            )));
        }

        let ollama_response: OllamaModelListResponse = response.json().await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;

        let models: Vec<ModelInfo> = ollama_response.models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name.clone(),
                size: m.size,
                modified_at: m.modified_at,
                parameters: Self::extract_parameters(&m.name),
                is_embedding: m.name.contains("embed") || m.name.contains("nomic"),
            })
            .collect();

        debug!(count = models.len(), "Models listed");
        Ok(models)
    }

    /// Pull/download a model
    #[instrument(skip(self))]
    async fn pull_model(&self, model_name: &str) -> Result<(), DomainError> {
        let url = self.api_url("api/pull");

        #[derive(Serialize)]
        struct PullRequest {
            name: String,
            stream: bool,
        }

        debug!(model = %model_name, "Pulling model");

        let response = self.client
            .post(&url)
            .json(&PullRequest {
                name: model_name.to_string(),
                stream: false,
            })
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Failed to pull model: {} - {}",
                status, body
            )));
        }

        debug!(model = %model_name, "Model pulled successfully");
        Ok(())
    }

    /// Check provider health
    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<bool, DomainError> {
        let url = self.api_url("api/tags");

        let result = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match result {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!(error = %e, "Ollama health check failed");
                Ok(false)
            }
        }
    }
}
