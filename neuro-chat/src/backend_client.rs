//! =============================================================================
//! Backend Client - LLM Gateway
//! =============================================================================
//! Client for communicating with tachikoma-backend's LLM endpoints.
//! All LLM operations go through the backend, which is the only gateway to Ollama.
//! =============================================================================

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::debug;

// =============================================================================
// Types for Backend LLM API
// =============================================================================

/// Chat message format for backend API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into() }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }
}

/// Response from non-streaming chat
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// Chunk from streaming chat
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    Start { model: String },
    Token { content: String },
    Done { 
        prompt_tokens: u64, 
        completion_tokens: u64,
        finish_reason: String,
    },
    Error { message: String },
}

/// Chunk from speculative streaming
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpeculativeChunk {
    Start { 
        draft_model: String, 
        target_model: String,
        lookahead: usize,
    },
    Tokens { content: String },
    Done { stats: SpeculativeStats },
    Error { message: String },
}

/// Statistics from speculative decoding
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SpeculativeStats {
    pub draft_tokens_generated: usize,
    pub tokens_accepted: usize,
    pub tokens_rejected: usize,
    pub acceptance_rate: f32,
    pub draft_model: String,
    pub target_model: String,
    pub iterations: usize,
}

/// LLM health status from backend
#[derive(Debug, Clone, Deserialize)]
pub struct LlmHealthStatus {
    pub healthy: bool,
    pub models_count: usize,
    pub models: Vec<String>,
    pub provider_url: String,
    pub error: Option<String>,
}

// =============================================================================
// Backend Client
// =============================================================================

/// Client for backend LLM endpoints
#[derive(Clone)]
pub struct BackendLlmClient {
    client: reqwest::Client,
    base_url: String,
}

impl BackendLlmClient {
    /// Create a new client for the backend LLM API
    pub fn new(backend_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: backend_url.trim_end_matches('/').to_string(),
        }
    }

    /// Check if the LLM provider (Ollama via backend) is healthy
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/llm/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(status) = response.json::<LlmHealthStatus>().await {
                        return status.healthy;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    /// Get detailed LLM health status
    pub async fn health_status(&self) -> Result<LlmHealthStatus, String> {
        let url = format!("{}/api/llm/health", self.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        
        if !response.status().is_success() {
            return Err(format!("Backend error: {}", response.status()));
        }
        
        response.json().await.map_err(|e| e.to_string())
    }

    /// List available models (from backend's Ollama)
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let status = self.health_status().await?;
        Ok(status.models)
    }

    /// Non-streaming chat completion
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<&str>,
    ) -> Result<ChatResponse, String> {
        let url = format!("{}/api/llm/chat", self.base_url);
        
        let body = json!({
            "messages": messages,
            "model": model
        });

        debug!("Sending chat request to backend: {}", url);

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(format!("Backend error: {} - {}", status, body));
        }

        response.json().await.map_err(|e| e.to_string())
    }

    /// Stream chat response via backend SSE
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<&str>,
        tx: mpsc::Sender<Result<StreamChunk, String>>,
    ) {
        let url = format!("{}/api/llm/chat/stream", self.base_url);
        
        let body = json!({
            "messages": messages,
            "model": model
        });

        debug!("Starting chat stream via backend: {}", url);

        let response = match self.client
            .post(&url)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(e.to_string())).await;
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            let _ = tx.send(Err(format!("Backend error: {} - {}", status, body))).await;
            return;
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process SSE lines
                    while let Some(pos) = buffer.find("\n\n") {
                        let event_block = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();
                        
                        // Parse SSE event
                        for line in event_block.lines() {
                            if let Some(data) = line.strip_prefix("data:") {
                                let data = data.trim();
                                if data.is_empty() {
                                    continue;
                                }
                                
                                match serde_json::from_str::<StreamChunk>(data) {
                                    Ok(chunk) => {
                                        let is_done = matches!(&chunk, StreamChunk::Done { .. });
                                        let is_error = matches!(&chunk, StreamChunk::Error { .. });
                                        
                                        if tx.send(Ok(chunk)).await.is_err() {
                                            return;
                                        }
                                        
                                        if is_done || is_error {
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        debug!("Failed to parse stream chunk: {} - {}", e, data);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string())).await;
                    return;
                }
            }
        }
    }

    /// Speculative decoding stream via backend SSE
    /// 
    /// Uses Light tier as draft model, Standard tier as target model by default.
    /// The backend handles all the speculative decoding logic.
    pub async fn speculative_stream(
        &self,
        messages: Vec<ChatMessage>,
        draft_model: Option<&str>,
        target_model: Option<&str>,
        lookahead: Option<usize>,
        tx: mpsc::Sender<SpeculativeChunk>,
    ) {
        let url = format!("{}/api/llm/chat/speculative/stream", self.base_url);
        
        let body = json!({
            "messages": messages,
            "draft_model": draft_model,
            "target_model": target_model,
            "lookahead": lookahead
        });

        debug!("Starting speculative stream via backend: {}", url);

        let response = match self.client
            .post(&url)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(SpeculativeChunk::Error { message: e.to_string() }).await;
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            let _ = tx.send(SpeculativeChunk::Error { 
                message: format!("Backend error: {} - {}", status, body) 
            }).await;
            return;
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // Process SSE lines
                    while let Some(pos) = buffer.find("\n\n") {
                        let event_block = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();
                        
                        // Parse SSE event
                        for line in event_block.lines() {
                            if let Some(data) = line.strip_prefix("data:") {
                                let data = data.trim();
                                if data.is_empty() {
                                    continue;
                                }
                                
                                match serde_json::from_str::<SpeculativeChunk>(data) {
                                    Ok(chunk) => {
                                        let is_done = matches!(&chunk, SpeculativeChunk::Done { .. });
                                        let is_error = matches!(&chunk, SpeculativeChunk::Error { .. });
                                        
                                        if tx.send(chunk).await.is_err() {
                                            return;
                                        }
                                        
                                        if is_done || is_error {
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        debug!("Failed to parse speculative chunk: {} - {}", e, data);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(SpeculativeChunk::Error { message: e.to_string() }).await;
                    return;
                }
            }
        }
    }

    /// Generate embedding for text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/api/llm/embed", self.base_url);
        
        let body = json!({ "text": text });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(format!("Backend error: {} - {}", status, body));
        }

        #[derive(Deserialize)]
        struct EmbedResponse {
            embedding: Vec<f32>,
        }

        let result: EmbedResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(result.embedding)
    }
}
