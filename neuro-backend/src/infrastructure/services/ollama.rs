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
    ports::llm_provider::{
        ChatRequest, ChatResponse, EmbeddingRequest, EmbeddingResponse,
        LlmProvider, ModelInfo, StreamCallback,
    },
};
use crate::infrastructure::config::OllamaConfig;

/// =============================================================================
/// OllamaClient - HTTP client for Ollama API
/// =============================================================================
/// Provides access to local LLM inference through Ollama's REST API.
/// Supports both chat completions and embeddings generation.
/// =============================================================================
#[derive(Clone)]
pub struct OllamaClient {
    /// HTTP client for API requests
    client: Client,
    /// Configuration settings
    config: OllamaConfig,
}

/// Ollama API chat request format
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Chat message in Ollama format
#[derive(Debug, Serialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

/// Generation options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
}

/// Ollama API chat response format
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaResponseMessage,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    prompt_eval_count: u64,
    #[serde(default)]
    eval_count: u64,
}

/// Response message format
#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

/// Ollama API embedding request format
#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

/// Ollama API embedding response format
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    model: String,
    embeddings: Vec<Vec<f32>>,
}

/// Ollama model list response
#[derive(Debug, Deserialize)]
struct OllamaModelListResponse {
    models: Vec<OllamaModelInfo>,
}

/// Model info from Ollama
#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    modified_at: String,
    #[serde(default)]
    digest: String,
}

/// Model details response
#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    #[serde(default)]
    parameters: String,
    #[serde(default)]
    template: String,
    #[serde(default)]
    system: String,
}

impl OllamaClient {
    /// =========================================================================
    /// Create a new OllamaClient
    /// =========================================================================
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Build the API URL for a given endpoint
    fn api_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.config.host, endpoint)
    }

    /// Estimate token count from text (rough approximation)
    fn estimate_tokens(text: &str) -> u64 {
        // Rough estimate: ~4 characters per token for English text
        (text.len() as f64 / 4.0).ceil() as u64
    }

    /// Extract parameter count from model name (heuristic)
    fn extract_parameters(model_name: &str) -> Option<u64> {
        // Common patterns: qwen2.5:7b, llama3:13b, ministral-3b
        let lower = model_name.to_lowercase();
        
        if lower.contains("70b") {
            return Some(70_000_000_000);
        }
        if lower.contains("34b") || lower.contains("35b") {
            return Some(34_000_000_000);
        }
        if lower.contains("14b") || lower.contains("13b") {
            return Some(14_000_000_000);
        }
        if lower.contains("7b") || lower.contains("8b") {
            return Some(7_000_000_000);
        }
        if lower.contains("3b") || lower.contains("4b") {
            return Some(3_000_000_000);
        }
        if lower.contains("1b") || lower.contains("1.5b") {
            return Some(1_000_000_000);
        }
        
        None
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    /// =========================================================================
    /// Send a chat completion request
    /// =========================================================================
    #[instrument(skip(self, request), fields(model = %request.model))]
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, DomainError> {
        let messages: Vec<OllamaChatMessage> = request.messages
            .iter()
            .map(|m| OllamaChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                top_p: request.top_p,
                top_k: None,
            }),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %request.model, "Sending chat request");

        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
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

        Ok(ChatResponse {
            content: ollama_response.message.content,
            model: ollama_response.model,
            tokens_prompt: ollama_response.prompt_eval_count,
            tokens_completion: ollama_response.eval_count,
            finish_reason: if ollama_response.done {
                "stop".to_string()
            } else {
                "length".to_string()
            },
        })
    }

    /// =========================================================================
    /// Stream a chat completion with callback
    /// =========================================================================
    #[instrument(skip(self, request, callback), fields(model = %request.model))]
    async fn chat_stream(
        &self,
        request: ChatRequest,
        callback: StreamCallback,
    ) -> Result<ChatResponse, DomainError> {
        let messages: Vec<OllamaChatMessage> = request.messages
            .iter()
            .map(|m| OllamaChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens.map(|t| t as i32),
                top_p: request.top_p,
                top_k: None,
            }),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %request.model, "Starting streaming chat request");

        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama API error: {} - {}",
                status, body
            )));
        }

        let mut full_content = String::new();
        let mut final_response: Option<OllamaChatResponse> = None;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| DomainError::llm_error(format!("Stream error: {}", e)))?;
            
            let text = String::from_utf8_lossy(&chunk);
            
            // Parse each line as a JSON response
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<OllamaChatResponse>(line) {
                    Ok(resp) => {
                        let token = resp.message.content.clone();
                        full_content.push_str(&token);
                        
                        // Call the callback with the token
                        callback(token);
                        
                        if resp.done {
                            final_response = Some(resp);
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, line = %line, "Failed to parse streaming response");
                    }
                }
            }
        }

        let resp = final_response.ok_or_else(|| {
            DomainError::llm_error("Stream ended without completion")
        })?;

        Ok(ChatResponse {
            content: full_content,
            model: resp.model,
            tokens_prompt: resp.prompt_eval_count,
            tokens_completion: resp.eval_count,
            finish_reason: "stop".to_string(),
        })
    }

    /// =========================================================================
    /// Generate embeddings for text
    /// =========================================================================
    #[instrument(skip(self, request))]
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, DomainError> {
        let model = request.model.as_deref()
            .unwrap_or(&self.config.embedding_model);

        let ollama_request = OllamaEmbeddingRequest {
            model: model.to_string(),
            input: request.texts.clone(),
        };

        let url = self.api_url("api/embed");
        debug!(url = %url, model = %model, count = request.texts.len(), "Generating embeddings");

        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama embedding error: {} - {}",
                status, body
            )));
        }

        let ollama_response: OllamaEmbeddingResponse = response.json().await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;

        let dimensions = ollama_response.embeddings
            .first()
            .map(|e| e.len())
            .unwrap_or(0);

        debug!(
            model = %ollama_response.model,
            count = ollama_response.embeddings.len(),
            dimensions = dimensions,
            "Embeddings generated"
        );

        Ok(EmbeddingResponse {
            embeddings: ollama_response.embeddings,
            model: ollama_response.model,
            dimensions,
            tokens_used: request.texts.iter().map(|t| Self::estimate_tokens(t)).sum(),
        })
    }

    /// =========================================================================
    /// List available models
    /// =========================================================================
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
            let body = response.text().await.unwrap_or_default();
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
                id: m.name.clone(),
                name: m.name.clone(),
                size_bytes: Some(m.size),
                parameters: Self::extract_parameters(&m.name),
                context_length: Some(4096), // Default, would need model details API
                is_embedding_model: m.name.contains("embed") || m.name.contains("nomic"),
            })
            .collect();

        debug!(count = models.len(), "Models listed");

        Ok(models)
    }

    /// =========================================================================
    /// Get specific model info
    /// =========================================================================
    #[instrument(skip(self))]
    async fn get_model_info(&self, model_id: &str) -> Result<Option<ModelInfo>, DomainError> {
        let models = self.list_models().await?;
        Ok(models.into_iter().find(|m| m.id == model_id))
    }

    /// =========================================================================
    /// Check model availability
    /// =========================================================================
    #[instrument(skip(self))]
    async fn is_model_available(&self, model_id: &str) -> Result<bool, DomainError> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m.id == model_id))
    }

    /// =========================================================================
    /// Pull (download) a model
    /// =========================================================================
    #[instrument(skip(self))]
    async fn pull_model(&self, model_id: &str) -> Result<(), DomainError> {
        let url = self.api_url("api/pull");
        
        #[derive(Serialize)]
        struct PullRequest {
            name: String,
            stream: bool,
        }

        debug!(model = %model_id, "Pulling model");

        let response = self.client
            .post(&url)
            .json(&PullRequest {
                name: model_id.to_string(),
                stream: false,
            })
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Failed to pull model: {} - {}",
                status, body
            )));
        }

        debug!(model = %model_id, "Model pulled successfully");

        Ok(())
    }

    /// =========================================================================
    /// Check provider health
    /// =========================================================================
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
