//! =============================================================================
//! OpenAI-compatible Client - LLM Provider Implementation
//! =============================================================================
//! Implements the LlmProvider port using the OpenAI Chat Completions API.
//! Works with any OpenAI-compatible provider: OpenAI, Groq, Together,
//! OpenRouter, LM Studio, vLLM, Ollama's /v1 endpoint, etc.
//! =============================================================================

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, instrument, warn};

use crate::domain::{
    errors::DomainError,
    ports::llm_provider::{
        ChatMessage, GenerationResult, LlmHealthStatus, LlmProvider, ModelInfo,
        SpeculativeChunk, SpeculativeStats, StreamChunk,
    },
};
use crate::infrastructure::config::OpenAiConfig;

/// =============================================================================
/// OpenAiClient - HTTP client for OpenAI-compatible APIs
/// =============================================================================
#[derive(Clone)]
pub struct OpenAiClient {
    client: Client,
    config: OpenAiConfig,
}

// ============================================================================
// OpenAI API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    model: String,
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct StreamChunkResponse {
    #[serde(default)]
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct ModelListResponse {
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

// ============================================================================
// Implementation
// ============================================================================

impl OpenAiClient {
    pub fn new(config: OpenAiConfig) -> Self {
        let mut builder = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs));
        if !config.api_key.is_empty() {
            builder = builder.default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                        .expect("valid api key"),
                );
                h
            });
        }
        let client = builder.build().expect("Failed to create HTTP client");
        Self { client, config }
    }

    fn api_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint)
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }

    fn embedding_model(&self) -> &str {
        &self.config.embedding_model
    }

    /// Convert chat messages to a prompt string (for speculative decoding).
    /// OpenAI-compatible APIs don't expose a raw generate endpoint, so we
    /// approximate with a single user message.
    fn messages_to_prompt(messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();
        for msg in messages {
            match msg.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n\n", msg.content)),
                "user" => prompt.push_str(&format!("User: {}\n\n", msg.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n\n", msg.content)),
                _ => {}
            }
        }
        prompt.push_str("Assistant: ");
        prompt
    }

    async fn chat_inner(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
        stream: bool,
    ) -> Result<reqwest::Response, DomainError> {
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream,
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };
        let url = self.api_url("chat/completions");
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "OpenAI API error: {} - {}",
                status, body
            )));
        }
        Ok(response)
    }
}

#[async_trait]
impl LlmProvider for OpenAiClient {
    #[instrument(skip(self, prompt))]
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<GenerationResult, DomainError> {
        let model_name = model.unwrap_or_else(|| self.default_model());
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];
        let response = self.chat_inner(messages, model_name, false).await?;
        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;
        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| DomainError::llm_error("No choices returned"))?;
        Ok(GenerationResult {
            content: choice.message.content.unwrap_or_default(),
            model: parsed.model,
            prompt_tokens: parsed.usage.prompt_tokens,
            completion_tokens: parsed.usage.completion_tokens,
            finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".to_string()),
        })
    }

    #[instrument(skip(self, messages))]
    async fn chat(&self, messages: Vec<ChatMessage>, model: Option<&str>) -> Result<GenerationResult, DomainError> {
        let model_name = model.unwrap_or_else(|| self.default_model());
        let response = self.chat_inner(messages, model_name, false).await?;
        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;
        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| DomainError::llm_error("No choices returned"))?;
        Ok(GenerationResult {
            content: choice.message.content.unwrap_or_default(),
            model: parsed.model,
            prompt_tokens: parsed.usage.prompt_tokens,
            completion_tokens: parsed.usage.completion_tokens,
            finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".to_string()),
        })
    }

    #[instrument(skip(self, messages, tx))]
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<&str>,
        tx: mpsc::Sender<StreamChunk>,
    ) {
        let model_name = model.unwrap_or_else(|| self.default_model()).to_string();
        let _ = tx.send(StreamChunk::Start { model: model_name.clone() }).await;

        let response = match self.chat_inner(messages, &model_name, true).await {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(StreamChunk::Error { message: e.to_string() }).await;
                return;
            }
        };

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let prompt_tokens = 0u64;
        let mut completion_tokens = 0u64;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        // Skip SSE "data:" prefix; ignore [DONE]
                        let data = line.strip_prefix("data:").unwrap_or(line).trim();
                        if data == "[DONE]" {
                            let _ = tx.send(StreamChunk::Done {
                                prompt_tokens,
                                completion_tokens,
                                finish_reason: "stop".to_string(),
                            }).await;
                            return;
                        }
                        match serde_json::from_str::<StreamChunkResponse>(data) {
                            Ok(chunk) => {
                                for choice in chunk.choices {
                                    if let Some(content) = choice.delta.content {
                                        if !content.is_empty() {
                                            completion_tokens += 1;
                                            let _ = tx.send(StreamChunk::Token { content }).await;
                                        }
                                    }
                                    if let Some(reason) = choice.finish_reason {
                                        if !reason.is_empty() {
                                            let _ = tx.send(StreamChunk::Done {
                                                prompt_tokens,
                                                completion_tokens,
                                                finish_reason: reason,
                                            }).await;
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse stream chunk: {} - {}", e, data);
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(StreamChunk::Error { message: e.to_string() }).await;
                    return;
                }
            }
        }
    }

    #[instrument(skip(self, messages, tx))]
    async fn speculative_stream(
        &self,
        messages: Vec<ChatMessage>,
        draft_model: Option<&str>,
        target_model: Option<&str>,
        lookahead: Option<usize>,
        tx: mpsc::Sender<SpeculativeChunk>,
    ) {
        // OpenAI-compatible APIs don't expose a raw generate endpoint, so
        // speculative decoding degrades to a plain chat completion.
        let target = target_model.unwrap_or_else(|| self.default_model());
        let lookahead_tokens = lookahead.unwrap_or(5);
        let _ = tx.send(SpeculativeChunk::Start {
            draft_model: draft_model.unwrap_or(target).to_string(),
            target_model: target.to_string(),
            lookahead: lookahead_tokens,
        }).await;

        let prompt = Self::messages_to_prompt(&messages);
        let result = self.generate(&prompt, Some(target)).await;
        match result {
            Ok(r) => {
                let _ = tx.send(SpeculativeChunk::Tokens { content: r.content }).await;
                let _ = tx.send(SpeculativeChunk::Done {
                    stats: SpeculativeStats {
                        draft_tokens_generated: 0,
                        tokens_accepted: 0,
                        tokens_rejected: 0,
                        acceptance_rate: 0.0,
                        draft_model: target.to_string(),
                        target_model: target.to_string(),
                        iterations: 0,
                    },
                }).await;
            }
            Err(e) => {
                let _ = tx.send(SpeculativeChunk::Error { message: e.to_string() }).await;
            }
        }
    }

    #[instrument(skip(self, text))]
    async fn embed(&self, text: &str) -> Result<Vec<f32>, DomainError> {
        let mut batch = self.embed_batch(&[text.to_string()]).await?;
        batch.pop().ok_or_else(|| DomainError::llm_error("No embedding returned"))
    }

    #[instrument(skip(self, texts))]
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, DomainError> {
        if texts.len() > 100 {
            return Err(DomainError::llm_error(format!(
                "Batch too large: {} texts (max 100)", texts.len()
            )));
        }
        let request = EmbeddingRequest {
            model: self.embedding_model().to_string(),
            input: texts.to_vec(),
        };
        let url = self.api_url("embeddings");
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "OpenAI embedding error: {} - {}",
                status, body
            )));
        }
        let parsed: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;
        Ok(parsed.data.into_iter().map(|d| d.embedding).collect())
    }

    #[instrument(skip(self))]
    async fn list_models(&self) -> Result<Vec<ModelInfo>, DomainError> {
        let url = self.api_url("models");
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Request failed: {}", e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "OpenAI API error: {} - {}",
                status, body
            )));
        }
        let parsed: ModelListResponse = response
            .json()
            .await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse response: {}", e)))?;
        Ok(parsed
            .data
            .into_iter()
            .map(|m| ModelInfo {
                name: m.id,
                size: 0,
                modified_at: String::new(),
                parameters: None,
                is_embedding: false,
            })
            .collect())
    }

    #[instrument(skip(self))]
    async fn pull_model(&self, _model_name: &str) -> Result<(), DomainError> {
        // OpenAI-compatible APIs manage models server-side; nothing to pull.
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<bool, DomainError> {
        let url = self.api_url("models");
        let result = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
        match result {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!(error = %e, "OpenAI health check failed");
                Ok(false)
            }
        }
    }

    #[instrument(skip(self))]
    async fn health_status(&self) -> Result<LlmHealthStatus, DomainError> {
        let url = self.api_url("models");
        let result = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let models = self.list_models().await.unwrap_or_default();
                    let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                    Ok(LlmHealthStatus {
                        healthy: true,
                        models_count: models.len(),
                        models: model_names,
                        provider_url: self.config.base_url.clone(),
                        provider: self.config.provider.clone(),
                        error: None,
                    })
                } else {
                    Ok(LlmHealthStatus {
                        healthy: false,
                        models_count: 0,
                        models: vec![],
                        provider_url: self.config.base_url.clone(),
                        provider: self.config.provider.clone(),
                        error: Some(format!("HTTP {}", response.status())),
                    })
                }
            }
            Err(e) => Ok(LlmHealthStatus {
                healthy: false,
                models_count: 0,
                models: vec![],
                provider_url: self.config.base_url.clone(),
                provider: "openai".to_string(),
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(url: String) -> OpenAiConfig {
        OpenAiConfig {
            base_url: url,
            api_key: "test-key".to_string(),
            default_model: "test-model".to_string(),
            embedding_model: "test-embed".to_string(),
            timeout_secs: 10,
            provider: "openai".to_string(),
        }
    }

    #[tokio::test]
    async fn test_generate_success() {
        let server = MockServer::start().await;
        let client = OpenAiClient::new(test_config(server.uri()));

        Mock::given(method("POST"))
            .and(path("chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "test-model",
                "choices": [{"message": {"content": "Hello!"}, "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 5, "completion_tokens": 3}
            })))
            .mount(&server)
            .await;

        let result = client.generate("Hi", None).await.unwrap();
        assert_eq!(result.content, "Hello!");
        assert_eq!(result.model, "test-model");
        assert_eq!(result.prompt_tokens, 5);
        assert_eq!(result.completion_tokens, 3);
    }

    #[tokio::test]
    async fn test_generate_http_error() {
        let server = MockServer::start().await;
        let client = OpenAiClient::new(test_config(server.uri()));

        Mock::given(method("POST"))
            .and(path("chat/completions"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let result = client.generate("Hi", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_embed_success() {
        let server = MockServer::start().await;
        let client = OpenAiClient::new(test_config(server.uri()));

        Mock::given(method("POST"))
            .and(path("embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"embedding": [0.1, 0.2, 0.3]}]
            })))
            .mount(&server)
            .await;

        let result = client.embed("test").await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 0.1);
    }

    #[tokio::test]
    async fn test_health_check_ok() {
        let server = MockServer::start().await;
        let client = OpenAiClient::new(test_config(server.uri()));

        Mock::given(method("GET"))
            .and(path("models"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        assert!(client.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_health_check_down() {
        let client = OpenAiClient::new(test_config("http://127.0.0.1:1".to_string()));
        assert!(!client.health_check().await.unwrap());
    }
}
