//! =============================================================================
//! Ollama Client - LLM Provider Implementation
//! =============================================================================
//! Implements the LlmProvider port using the Ollama API.
//! Handles chat completions, embeddings, speculative decoding, and model management.
//! 
//! This is the ONLY component that talks directly to Ollama.
//! All other services must use this through the backend API.
//! =============================================================================

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

use crate::domain::{
    errors::DomainError,
    ports::llm_provider::{
        ChatMessage, GenerationResult, LlmHealthStatus, LlmProvider, ModelInfo,
        SpeculativeChunk, SpeculativeStats, StreamChunk,
    },
    value_objects::model_tier::ModelTier,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

impl From<ChatMessage> for OllamaChatMessage {
    fn from(msg: ChatMessage) -> Self {
        Self {
            role: msg.role,
            content: msg.content,
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<i32>,
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

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    message: Option<OllamaResponseMessage>,
    done: bool,
    #[serde(default)]
    prompt_eval_count: u64,
    #[serde(default)]
    eval_count: u64,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaGenerateOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<bool>,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    #[serde(default)]
    done: bool,
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
    #[allow(dead_code)]
    pub fn default_model(&self) -> &str {
        &self.config.default_model
    }

    /// Determine keep_alive value based on model tier
    /// Light models (3b) and embedding models stay loaded forever, others unload after 5 minutes
    fn get_keep_alive_for_model(model_name: &str) -> serde_json::Value {
        let lower = model_name.to_lowercase();
        if lower.contains("3b") || lower.contains("ministral") || lower.contains("embed") || lower.contains("nomic") {
            serde_json::json!(-1)
        } else {
            serde_json::json!("5m")
        }
    }

    /// Get the embedding model name
    #[allow(dead_code)]
    pub fn embedding_model(&self) -> &str {
        &self.config.embedding_model
    }

    /// Convert chat messages to a prompt string (for speculative decoding)
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

    /// Generate streaming response - returns byte stream for SSE
    #[allow(dead_code)]
    pub async fn generate_stream_response(
        &self,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<reqwest::Response, DomainError> {
        let model_name = model.unwrap_or(&self.config.default_model);

        let keep_alive = Self::get_keep_alive_for_model(model_name);
        let request = OllamaChatRequest {
            model: model_name.to_string(),
            messages: vec![OllamaChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: true,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_predict: Some(2048),
                num_ctx: Some(8192),
            }),
            keep_alive: Some(keep_alive.clone()),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %model_name, keep_alive = %keep_alive, "Sending streaming chat request");

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama API error: {} - {}",
                status, body
            )));
        }

        Ok(response)
    }

    /// Internal: Generate with /api/generate endpoint (for speculative decoding)
    async fn generate_raw(
        &self,
        prompt: &str,
        model: &str,
        num_tokens: Option<i32>,
    ) -> Result<String, DomainError> {
        let url = self.api_url("api/generate");
        
        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            raw: Some(true),
            options: Some(OllamaGenerateOptions {
                temperature: Some(0.7),
                num_ctx: Some(8192),
                num_predict: num_tokens,
            }),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::llm_error(format!("Generate request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::llm_error(format!(
                "Ollama generate error: {} - {}",
                status, body
            )));
        }

        let result: OllamaGenerateResponse = response.json().await
            .map_err(|e| DomainError::llm_error(format!("Failed to parse generate response: {}", e)))?;
        
        Ok(result.response)
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    // =========================================================================
    // Basic Generation
    // =========================================================================

    /// Generate text completion from a simple prompt
    #[instrument(skip(self, prompt))]
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<GenerationResult, DomainError> {
        let model_name = model.unwrap_or(&self.config.default_model);
        let keep_alive = Self::get_keep_alive_for_model(model_name);

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
                num_ctx: Some(8192),
            }),
            keep_alive: Some(keep_alive.clone()),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %model_name, keep_alive = %keep_alive, "Sending chat request");

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

    // =========================================================================
    // Chat (with message history)
    // =========================================================================

    /// Chat completion with message history
    #[instrument(skip(self, messages))]
    async fn chat(&self, messages: Vec<ChatMessage>, model: Option<&str>) -> Result<GenerationResult, DomainError> {
        let model_name = model.unwrap_or(&self.config.default_model);
        let keep_alive = Self::get_keep_alive_for_model(model_name);

        let ollama_messages: Vec<OllamaChatMessage> = messages.into_iter().map(Into::into).collect();

        let request = OllamaChatRequest {
            model: model_name.to_string(),
            messages: ollama_messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_predict: Some(2048),
                num_ctx: Some(8192),
            }),
            keep_alive: Some(keep_alive.clone()),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %model_name, "Sending chat request with messages");

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

        Ok(GenerationResult {
            content: ollama_response.message.content,
            model: ollama_response.model,
            prompt_tokens: ollama_response.prompt_eval_count,
            completion_tokens: ollama_response.eval_count,
            finish_reason: if ollama_response.done { "stop" } else { "length" }.to_string(),
        })
    }

    /// Stream chat response chunks via channel
    #[instrument(skip(self, messages, tx))]
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<&str>,
        tx: mpsc::Sender<StreamChunk>,
    ) {
        let model_name = model.unwrap_or(&self.config.default_model).to_string();
        let keep_alive = Self::get_keep_alive_for_model(&model_name);

        let ollama_messages: Vec<OllamaChatMessage> = messages.into_iter().map(Into::into).collect();

        let request = OllamaChatRequest {
            model: model_name.clone(),
            messages: ollama_messages,
            stream: true,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_predict: Some(2048),
                num_ctx: Some(8192),
            }),
            keep_alive: Some(keep_alive),
        };

        let url = self.api_url("api/chat");
        debug!(url = %url, model = %model_name, "Starting chat stream");

        // Send start event
        let _ = tx.send(StreamChunk::Start { model: model_name.clone() }).await;

        let response = match self.client.post(&url).json(&request).send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(StreamChunk::Error { message: e.to_string() }).await;
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            let _ = tx.send(StreamChunk::Error { 
                message: format!("Ollama error: {} - {}", status, body) 
            }).await;
            return;
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut prompt_tokens = 0u64;
        let mut completion_tokens = 0u64;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process complete JSON lines
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();
                        
                        if line.trim().is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<OllamaStreamChunk>(&line) {
                            Ok(chunk) => {
                                if let Some(msg) = &chunk.message {
                                    if !msg.content.is_empty() {
                                        let _ = tx.send(StreamChunk::Token { 
                                            content: msg.content.clone() 
                                        }).await;
                                    }
                                }

                                if chunk.prompt_eval_count > 0 {
                                    prompt_tokens = chunk.prompt_eval_count;
                                }
                                if chunk.eval_count > 0 {
                                    completion_tokens = chunk.eval_count;
                                }

                                if chunk.done {
                                    let _ = tx.send(StreamChunk::Done {
                                        prompt_tokens,
                                        completion_tokens,
                                        finish_reason: "stop".to_string(),
                                    }).await;
                                    return;
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse stream chunk: {} - {}", e, line);
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

    // =========================================================================
    // Speculative Decoding
    // =========================================================================

    /// Speculative decoding with draft and target models
    #[instrument(skip(self, messages, tx))]
    async fn speculative_stream(
        &self,
        messages: Vec<ChatMessage>,
        draft_model: Option<&str>,
        target_model: Option<&str>,
        lookahead: Option<usize>,
        tx: mpsc::Sender<SpeculativeChunk>,
    ) {
        // Use ModelTier defaults if not specified
        let draft = draft_model.unwrap_or(ModelTier::Light.default_model());
        let target = target_model.unwrap_or(ModelTier::Standard.default_model());
        let lookahead_tokens = lookahead.unwrap_or(5);

        // Send start event
        let _ = tx.send(SpeculativeChunk::Start {
            draft_model: draft.to_string(),
            target_model: target.to_string(),
            lookahead: lookahead_tokens,
        }).await;

        // Build initial prompt from messages
        let mut prompt = Self::messages_to_prompt(&messages);
        let mut stats = SpeculativeStats {
            draft_model: draft.to_string(),
            target_model: target.to_string(),
            ..Default::default()
        };

        let max_tokens = 2048;
        let mut total_tokens = 0;

        info!(
            "Starting speculative decoding: draft={}, target={}, lookahead={}",
            draft, target, lookahead_tokens
        );

        loop {
            if total_tokens >= max_tokens {
                debug!("Reached max tokens limit");
                break;
            }

            stats.iterations += 1;

            // Step 1: Draft model generates K tokens
            let draft_result = self.generate_raw(&prompt, draft, Some(lookahead_tokens as i32)).await;
            
            let draft_tokens = match draft_result {
                Ok(tokens) => tokens,
                Err(e) => {
                    error!("Draft model error: {}", e);
                    let _ = tx.send(SpeculativeChunk::Error { message: e.to_string() }).await;
                    break;
                }
            };

            if draft_tokens.is_empty() {
                debug!("Draft model returned empty response, finishing");
                break;
            }

            stats.draft_tokens_generated += draft_tokens.chars().count();
            debug!("Draft generated: {:?}", draft_tokens);

            // Step 2: Verify each character/token with target model
            let mut accepted_text = String::new();
            let mut current_prompt = prompt.clone();
            let mut chars = draft_tokens.chars().peekable();
            let mut rejected = false;

            while let Some(draft_char) = chars.next() {
                // Target model generates its own prediction
                let target_result = self.generate_raw(&current_prompt, target, Some(1)).await;
                
                let target_token = match target_result {
                    Ok(t) => t,
                    Err(e) => {
                        warn!("Target model error during verification: {}", e);
                        // On error, accept the draft token and continue
                        accepted_text.push(draft_char);
                        current_prompt.push(draft_char);
                        stats.tokens_accepted += 1;
                        continue;
                    }
                };

                // Check for end of sequence
                if target_token.is_empty() || target_token.contains("<|endoftext|>") || target_token.contains("</s>") {
                    debug!("Target model signaled end of sequence");
                    if !accepted_text.is_empty() {
                        let _ = tx.send(SpeculativeChunk::Tokens { content: accepted_text.clone() }).await;
                    }
                    // Calculate final stats
                    if stats.draft_tokens_generated > 0 {
                        stats.acceptance_rate = stats.tokens_accepted as f32 / stats.draft_tokens_generated as f32;
                    }
                    let _ = tx.send(SpeculativeChunk::Done { stats }).await;
                    return;
                }

                let target_char = target_token.chars().next().unwrap_or('\0');

                // Compare draft vs target
                if draft_char == target_char {
                    // Match! Accept the token
                    accepted_text.push(draft_char);
                    current_prompt.push(draft_char);
                    stats.tokens_accepted += 1;
                    debug!("Token accepted: {:?}", draft_char);
                } else {
                    // Mismatch! Use target's token instead
                    debug!(
                        "Token rejected: draft={:?}, target={:?}",
                        draft_char, target_char
                    );
                    stats.tokens_rejected += 1;
                    
                    // Add target's token instead
                    accepted_text.push(target_char);
                    current_prompt.push(target_char);
                    rejected = true;
                    
                    // Count remaining draft tokens as rejected
                    let remaining: usize = chars.count();
                    stats.tokens_rejected += remaining;
                    break;
                }
            }

            // Send accepted tokens to stream
            if !accepted_text.is_empty() {
                prompt.push_str(&accepted_text);
                total_tokens += accepted_text.len();
                
                let _ = tx.send(SpeculativeChunk::Tokens { content: accepted_text }).await;
            }

            // Check for natural end (draft produced fewer tokens than requested)
            if draft_tokens.len() < lookahead_tokens && !rejected {
                debug!("Draft model finished early, ending generation");
                break;
            }

            // Small delay to prevent overwhelming the models
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Calculate final acceptance rate
        if stats.draft_tokens_generated > 0 {
            stats.acceptance_rate = stats.tokens_accepted as f32 / stats.draft_tokens_generated as f32;
        }

        info!(
            "Speculative decoding complete: {} iterations, {}/{} tokens accepted ({:.1}%)",
            stats.iterations,
            stats.tokens_accepted,
            stats.draft_tokens_generated,
            stats.acceptance_rate * 100.0
        );

        let _ = tx.send(SpeculativeChunk::Done { stats }).await;
    }

    /// Generate specific number of tokens
    #[instrument(skip(self, prompt))]
    async fn generate_tokens(
        &self,
        prompt: &str,
        model: &str,
        num_tokens: i32,
    ) -> Result<String, DomainError> {
        self.generate_raw(prompt, model, Some(num_tokens)).await
    }

    // =========================================================================
    // Embeddings
    // =========================================================================

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

    // =========================================================================
    // Model Management
    // =========================================================================

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

    // =========================================================================
    // Health
    // =========================================================================

    /// Check provider health (simple)
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

    /// Get detailed health status
    #[instrument(skip(self))]
    async fn health_status(&self) -> Result<LlmHealthStatus, DomainError> {
        let url = self.api_url("api/tags");

        let result = self.client
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
                        provider_url: self.config.url.clone(),
                        error: None,
                    })
                } else {
                    Ok(LlmHealthStatus {
                        healthy: false,
                        models_count: 0,
                        models: vec![],
                        provider_url: self.config.url.clone(),
                        error: Some(format!("HTTP {}", response.status())),
                    })
                }
            }
            Err(e) => {
                Ok(LlmHealthStatus {
                    healthy: false,
                    models_count: 0,
                    models: vec![],
                    provider_url: self.config.url.clone(),
                    error: Some(e.to_string()),
                })
            }
        }
    }
}
