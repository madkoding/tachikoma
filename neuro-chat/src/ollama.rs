//! =============================================================================
//! Ollama Client
//! =============================================================================

use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::models::*;

#[derive(Clone)]
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Send a chat message and get a complete response
    pub async fn chat(
        &self,
        messages: Vec<OllamaMessage>,
        model: &str,
    ) -> Result<OllamaChatResponse, String> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = OllamaChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_ctx: Some(8192),
            }),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        response.json().await.map_err(|e| e.to_string())
    }

    /// Stream chat response chunks
    pub async fn chat_stream(
        &self,
        messages: Vec<OllamaMessage>,
        model: &str,
        tx: mpsc::Sender<Result<OllamaStreamChunk, String>>,
    ) {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "options": {
                "temperature": 0.7,
                "num_ctx": 8192
            }
        });

        debug!("Starting Ollama stream to {}", url);

        let response = match self.client
            .post(&url)
            .json(&request)
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
            let _ = tx.send(Err(format!("Ollama error: {}", response.status()))).await;
            return;
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

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
                                let is_done = chunk.done;
                                if tx.send(Ok(chunk)).await.is_err() {
                                    return;
                                }
                                if is_done {
                                    return;
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse chunk: {} - {}", e, line);
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

    /// Generate embedding for text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/api/embeddings", self.base_url);

        let response = self.client
            .post(&url)
            .json(&json!({
                "model": "nomic-embed-text",
                "prompt": text
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        #[derive(serde::Deserialize)]
        struct EmbeddingResponse {
            embedding: Vec<f32>,
        }

        let result: EmbeddingResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(result.embedding)
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        #[derive(serde::Deserialize)]
        struct ModelInfo {
            name: String,
        }

        #[derive(serde::Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>,
        }

        let result: ModelsResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(result.models.into_iter().map(|m| m.name).collect())
    }

    /// Check if Ollama is healthy
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }
}
