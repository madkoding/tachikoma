//! =============================================================================
//! LLM Handlers
//! =============================================================================
//! HTTP handlers for LLM operations - the ONLY gateway to Ollama.
//! All microservices must use these endpoints instead of connecting directly.
//! =============================================================================

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, info};

use crate::domain::ports::llm_provider::{
    ChatMessage, LlmHealthStatus, SpeculativeChunk, StreamChunk,
};
use crate::AppState;

// =============================================================================
// Request/Response DTOs
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
    pub dimensions: usize,
}

#[derive(Debug, Deserialize)]
pub struct EmbedBatchRequest {
    pub texts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbedBatchResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub count: usize,
    pub dimensions: usize,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpeculativeStreamRequest {
    pub messages: Vec<ChatMessage>,
    /// Override draft model (default: Light tier - qwen2.5:3b)
    pub draft_model: Option<String>,
    /// Override target model (default: Standard tier - qwen2.5-coder:7b)
    pub target_model: Option<String>,
    /// Number of tokens to generate speculatively (default: 5)
    pub lookahead: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub num_tokens: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub content: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

// =============================================================================
// Health Endpoint
// =============================================================================

/// GET /api/llm/health
/// Returns detailed health status of the LLM provider (Ollama)
pub async fn llm_health(
    State(state): State<Arc<AppState>>,
) -> Json<LlmHealthStatus> {
    let status = state.llm_provider.health_status().await.unwrap_or_else(|e| {
        LlmHealthStatus {
            healthy: false,
            models_count: 0,
            models: vec![],
            provider_url: "unknown".to_string(),
            error: Some(e.to_string()),
        }
    });
    
    Json(status)
}

// =============================================================================
// Embedding Endpoints
// =============================================================================

/// POST /api/llm/embed
/// Generate embedding for a single text
pub async fn llm_embed(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmbedRequest>,
) -> Result<Json<EmbedResponse>, (axum::http::StatusCode, String)> {
    debug!("Generating embedding for text of length {}", request.text.len());
    
    let embedding = state.llm_provider.embed(&request.text).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let dimensions = embedding.len();
    
    Ok(Json(EmbedResponse {
        embedding,
        dimensions,
    }))
}

/// POST /api/llm/embed/batch
/// Generate embeddings for multiple texts
pub async fn llm_embed_batch(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmbedBatchRequest>,
) -> Result<Json<EmbedBatchResponse>, (axum::http::StatusCode, String)> {
    debug!("Generating batch embeddings for {} texts", request.texts.len());
    
    let embeddings = state.llm_provider.embed_batch(&request.texts).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let count = embeddings.len();
    let dimensions = embeddings.first().map(|e| e.len()).unwrap_or(0);
    
    Ok(Json(EmbedBatchResponse {
        embeddings,
        count,
        dimensions,
    }))
}

// =============================================================================
// Chat Endpoints
// =============================================================================

/// POST /api/llm/chat
/// Non-streaming chat completion with message history
pub async fn llm_chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<GenerateResponse>, (axum::http::StatusCode, String)> {
    debug!("Chat request with {} messages", request.messages.len());
    
    let result = state.llm_provider.chat(request.messages, request.model.as_deref()).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(GenerateResponse {
        content: result.content,
        model: result.model,
        prompt_tokens: result.prompt_tokens,
        completion_tokens: result.completion_tokens,
    }))
}

/// POST /api/llm/chat/stream
/// Streaming chat completion with SSE
pub async fn llm_chat_stream(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatStreamRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("Starting chat stream with {} messages", request.messages.len());
    
    let (tx, rx) = tokio::sync::mpsc::channel::<StreamChunk>(100);
    
    let llm_provider = state.llm_provider.clone();
    let messages = request.messages;
    let model = request.model;
    
    tokio::spawn(async move {
        llm_provider.chat_stream(messages, model.as_deref(), tx).await;
    });
    
    let stream = ReceiverStream::new(rx).map(|chunk| {
        let data = serde_json::to_string(&chunk).unwrap_or_default();
        Ok(Event::default().event("message").data(data))
    });
    
    Sse::new(stream)
}

/// POST /api/llm/chat/speculative/stream
/// Speculative decoding streaming with SSE
/// Uses Light tier as draft model, Standard tier as target model by default
pub async fn llm_speculative_stream(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SpeculativeStreamRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!(
        "Starting speculative stream with {} messages, draft={:?}, target={:?}, lookahead={:?}",
        request.messages.len(),
        request.draft_model,
        request.target_model,
        request.lookahead
    );
    
    let (tx, rx) = tokio::sync::mpsc::channel::<SpeculativeChunk>(100);
    
    let llm_provider = state.llm_provider.clone();
    let messages = request.messages;
    let draft_model = request.draft_model;
    let target_model = request.target_model;
    let lookahead = request.lookahead;
    
    tokio::spawn(async move {
        llm_provider.speculative_stream(
            messages,
            draft_model.as_deref(),
            target_model.as_deref(),
            lookahead,
            tx,
        ).await;
    });
    
    let stream = ReceiverStream::new(rx).map(|chunk| {
        let data = serde_json::to_string(&chunk).unwrap_or_default();
        Ok(Event::default().event("message").data(data))
    });
    
    Sse::new(stream)
}

// =============================================================================
// Generate Endpoint
// =============================================================================

/// POST /api/llm/generate
/// Raw text generation from a prompt
pub async fn llm_generate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (axum::http::StatusCode, String)> {
    debug!("Generate request with prompt length {}", request.prompt.len());
    
    let result = state.llm_provider.generate(&request.prompt, request.model.as_deref()).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(GenerateResponse {
        content: result.content,
        model: result.model,
        prompt_tokens: result.prompt_tokens,
        completion_tokens: result.completion_tokens,
    }))
}
