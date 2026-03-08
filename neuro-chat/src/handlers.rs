//! =============================================================================
//! API Handlers
//! =============================================================================
//! 
//! All LLM operations go through tachikoma-backend's /api/llm/* endpoints.
//! This service no longer connects directly to Ollama.
//! =============================================================================

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::{Event, Sse}, IntoResponse},
    Json,
};
use chrono::Utc;
use futures::stream::Stream;
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};
use surrealdb::sql::Datetime;
use tokio::sync::mpsc;
use tracing::error;
use uuid::Uuid;

use crate::{
    models::*,
    backend_client::{ChatMessage as LlmMessage, SpeculativeChunk, StreamChunk},
    AppState,
};

// ============================================================================
// Health Check
// ============================================================================

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = state.db.health_check().await.unwrap_or(false);
    let llm_healthy = state.llm_client.health_check().await;
    let memory_healthy = state.memory_client.health_check().await;

    let status = if db_healthy && llm_healthy { "healthy" } else { "degraded" };

    Json(json!({
        "status": status,
        "service": "tachikoma-chat",
        "version": env!("CARGO_PKG_VERSION"),
        "services": {
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "backend_llm": if llm_healthy { "healthy" } else { "unhealthy" },
            "memory": if memory_healthy { "healthy" } else { "unavailable" },
        }
    }))
}

/// List available models (via backend)
pub async fn list_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.llm_client.list_models().await {
        Ok(models) => Json(json!({ "models": models })).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": e }))).into_response(),
    }
}

// ============================================================================
// Chat Operations
// ============================================================================

/// Send a message and get a complete response (via backend LLM gateway)
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    // Model is optional - backend will use defaults if not specified
    let model = request.model.clone();
    
    // Get or create conversation
    let conversation_id = match request.conversation_id {
        Some(id) => id,
        None => create_conversation(&state).await.unwrap_or_else(|_| Uuid::new_v4()),
    };

    // Build messages with context
    let mut messages = vec![];
    
    // Add system prompt
    messages.push(LlmMessage {
        role: "system".to_string(),
        content: get_system_prompt(),
    });

    // Add memory context if enabled
    if request.include_memories {
        if let Ok(memories) = state.memory_client.search(&request.message, 5, 0.5).await {
            if !memories.is_empty() {
                let memory_context = memories
                    .iter()
                    .map(|m| format!("- {}", m.memory.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                messages.push(LlmMessage {
                    role: "system".to_string(),
                    content: format!("Relevant memories:\n{}", memory_context),
                });
            }
        }
    }

    // Add conversation history
    if let Ok(history) = get_conversation_messages(&state, conversation_id).await {
        for msg in history.iter().rev().take(10).rev() {
            messages.push(LlmMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            });
        }
    }

    // Add user message
    messages.push(LlmMessage {
        role: "user".to_string(),
        content: request.message.clone(),
    });

    // Call backend LLM gateway
    match state.llm_client.chat(messages, model.as_deref()).await {
        Ok(response) => {
            let message_id = Uuid::new_v4();
            
            // Save messages to database
            let _ = save_message(&state, conversation_id, MessageRole::User, &request.message).await;
            let _ = save_message(&state, conversation_id, MessageRole::Assistant, &response.content).await;

            let resp = SendMessageResponse {
                content: response.content,
                conversation_id,
                message_id,
                model: response.model,
                tokens_prompt: response.prompt_tokens as i32,
                tokens_completion: response.completion_tokens as i32,
                processing_time_ms: start.elapsed().as_millis() as u64,
            };
            Json(resp).into_response()
        }
        Err(e) => {
            error!("Backend LLM error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))).into_response()
        }
    }
}

/// Stream a message response via SSE (via backend LLM gateway)
pub async fn stream_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Model is optional - backend will use defaults if not specified
    let model = request.model.clone();
    
    // Get or create conversation
    let conversation_id = match request.conversation_id {
        Some(id) => id,
        None => create_conversation(&state).await.unwrap_or_else(|_| Uuid::new_v4()),
    };

    // Build messages
    let mut messages = vec![];
    
    messages.push(LlmMessage {
        role: "system".to_string(),
        content: get_system_prompt(),
    });

    // Add memory context
    if request.include_memories {
        if let Ok(memories) = state.memory_client.search(&request.message, 5, 0.5).await {
            if !memories.is_empty() {
                let memory_context = memories
                    .iter()
                    .map(|m| format!("- {}", m.memory.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                messages.push(LlmMessage {
                    role: "system".to_string(),
                    content: format!("Relevant memories:\n{}", memory_context),
                });
            }
        }
    }

    // Add history
    if let Ok(history) = get_conversation_messages(&state, conversation_id).await {
        for msg in history.iter().rev().take(10).rev() {
            messages.push(LlmMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            });
        }
    }

    messages.push(LlmMessage {
        role: "user".to_string(),
        content: request.message.clone(),
    });

    // Create channel for streaming
    let (tx, mut rx) = mpsc::channel::<Result<StreamChunk, String>>(100);
    
    // Spawn backend streaming task
    let llm_client = state.llm_client.clone();
    let model_clone = model.clone();
    tokio::spawn(async move {
        llm_client.chat_stream(messages, model_clone.as_deref(), tx).await;
    });

    // Create SSE stream
    let user_message = request.message.clone();
    let state_clone = state.clone();
    let model_display = model.unwrap_or_else(|| "default".to_string());
    
    let stream = async_stream::stream! {
        // Send start event
        yield Ok(Event::default()
            .event("message")
            .data(json!({
                "type": "start",
                "conversation_id": conversation_id,
                "model": model_display
            }).to_string()));

        let mut full_content = String::new();
        let mut prompt_tokens: u64 = 0;
        let mut completion_tokens: u64 = 0;

        while let Some(result) = rx.recv().await {
            match result {
                Ok(chunk) => match chunk {
                    StreamChunk::Start { model: _ } => {
                        // Already sent start event
                    }
                    StreamChunk::Token { content } => {
                        full_content.push_str(&content);
                        yield Ok(Event::default()
                            .event("message")
                            .data(json!({
                                "type": "chunk",
                                "content": content
                            }).to_string()));
                    }
                    StreamChunk::Done { prompt_tokens: pt, completion_tokens: ct, .. } => {
                        prompt_tokens = pt;
                        completion_tokens = ct;
                        break;
                    }
                    StreamChunk::Error { message } => {
                        yield Ok(Event::default()
                            .event("message")
                            .data(json!({
                                "type": "error",
                                "error": message
                            }).to_string()));
                        break;
                    }
                }
                Err(e) => {
                    yield Ok(Event::default()
                        .event("message")
                        .data(json!({
                            "type": "error",
                            "error": e
                        }).to_string()));
                    break;
                }
            }
        }

        // Save messages
        let _ = save_message(&state_clone, conversation_id, MessageRole::User, &user_message).await;
        let _ = save_message(&state_clone, conversation_id, MessageRole::Assistant, &full_content).await;

        // Send done event
        yield Ok(Event::default()
            .event("message")
            .data(json!({
                "type": "done",
                "conversation_id": conversation_id,
                "tokens_prompt": prompt_tokens,
                "tokens_completion": completion_tokens
            }).to_string()));
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// ============================================================================
// Conversation Operations
// ============================================================================

/// List all conversations
pub async fn list_conversations(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sql = "SELECT * FROM conversation ORDER BY updated_at DESC LIMIT 50";
    
    match state.db.client().query(sql).await {
        Ok(mut response) => {
            let records: Vec<ConversationRecord> = response.take(0).unwrap_or_default();
            let conversations: Vec<Conversation> = records.into_iter().map(|r| r.to_conversation()).collect();
            Json(json!({ "conversations": conversations })).into_response()
        }
        Err(e) => {
            error!("Failed to list conversations: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Get a conversation with its messages
pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Get conversation
    let conv_sql = "SELECT * FROM type::thing('conversation', $id)";
    let conv_result = state.db.client()
        .query(conv_sql)
        .bind(("id", id.to_string()))
        .await;

    let conversation = match conv_result {
        Ok(mut response) => {
            let records: Vec<ConversationRecord> = response.take(0).unwrap_or_default();
            match records.into_iter().next() {
                Some(r) => r.to_conversation(),
                None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Conversation not found" }))).into_response(),
            }
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    // Get messages
    let messages = get_conversation_messages(&state, id).await.unwrap_or_default();

    Json(ConversationWithMessages {
        conversation,
        messages,
    }).into_response()
}

/// Delete a conversation
pub async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Delete messages first
    let delete_msgs = "DELETE chat_message WHERE conversation_id = $id";
    let _ = state.db.client()
        .query(delete_msgs)
        .bind(("id", id.to_string()))
        .await;

    // Delete conversation
    let sql = "DELETE type::thing('conversation', $id)";
    match state.db.client().query(sql).bind(("id", id.to_string())).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_system_prompt() -> String {
    r#"Eres un asistente de IA amigable y útil llamado TACHIKOMA. 
Respondes en español de forma concisa y clara.
Tienes acceso a memorias del usuario que te ayudan a personalizar las respuestas.
Siempre intentas ser útil y proporcionar información precisa."#.to_string()
}

async fn create_conversation(state: &Arc<AppState>) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let sql = r#"
        CREATE type::thing('conversation', $id) SET
            title = NONE,
            created_at = $now,
            updated_at = $now,
            archived = false,
            message_count = 0
    "#;

    state.db.client()
        .query(sql)
        .bind(("id", id.to_string()))
        .bind(("now", Datetime::from(now)))
        .await
        .map_err(|e| e.to_string())?;

    Ok(id)
}

async fn get_conversation_messages(state: &Arc<AppState>, conversation_id: Uuid) -> Result<Vec<ChatMessage>, String> {
    let sql = "SELECT * FROM chat_message WHERE conversation_id = $id ORDER BY created_at ASC";
    
    let mut response = state.db.client()
        .query(sql)
        .bind(("id", conversation_id.to_string()))
        .await
        .map_err(|e| e.to_string())?;

    let records: Vec<ChatMessageRecord> = response.take(0).unwrap_or_default();
    Ok(records.into_iter().map(|r| r.to_message()).collect())
}

async fn save_message(
    state: &Arc<AppState>,
    conversation_id: Uuid,
    role: MessageRole,
    content: &str,
) -> Result<(), String> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let sql = r#"
        CREATE type::thing('chat_message', $id) SET
            conversation_id = $conversation_id,
            role = $role,
            content = $content,
            metadata = {},
            created_at = $now
    "#;

    state.db.client()
        .query(sql)
        .bind(("id", id.to_string()))
        .bind(("conversation_id", conversation_id.to_string()))
        .bind(("role", role.to_string()))
        .bind(("content", content))
        .bind(("now", Datetime::from(now)))
        .await
        .map_err(|e| e.to_string())?;

    // Update conversation
    let update_sql = r#"
        UPDATE type::thing('conversation', $id) SET
            updated_at = $now,
            message_count = message_count + 1
    "#;

    state.db.client()
        .query(update_sql)
        .bind(("id", conversation_id.to_string()))
        .bind(("now", Datetime::from(now)))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
// ============================================================================
// Speculative Decoding Handler
// ============================================================================

/// Stream a message response using speculative decoding via SSE
/// 
/// Calls backend's speculative_stream endpoint which uses a fast draft model 
/// to generate tokens speculatively, then verifies with a larger target model.
pub async fn speculative_stream(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SpeculativeMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Models are optional - backend will use tier defaults (Light=draft, Standard=target)
    let draft_model = request.draft_model.clone();
    let target_model = request.target_model.clone();
    let lookahead = request.lookahead
        .unwrap_or(state.config.speculative_lookahead);

    // Get or create conversation
    let conversation_id = match request.conversation_id {
        Some(id) => id,
        None => create_conversation(&state).await.unwrap_or_else(|_| Uuid::new_v4()),
    };

    // Build messages
    let mut messages = vec![];
    
    messages.push(LlmMessage {
        role: "system".to_string(),
        content: get_system_prompt(),
    });

    // Add memory context
    if request.include_memories {
        if let Ok(memories) = state.memory_client.search(&request.message, 5, 0.5).await {
            if !memories.is_empty() {
                let memory_context = memories
                    .iter()
                    .map(|m| format!("- {}", m.memory.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                messages.push(LlmMessage {
                    role: "system".to_string(),
                    content: format!("Relevant memories:\n{}", memory_context),
                });
            }
        }
    }

    // Add history
    if let Ok(history) = get_conversation_messages(&state, conversation_id).await {
        for msg in history.iter().rev().take(10).rev() {
            messages.push(LlmMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            });
        }
    }

    messages.push(LlmMessage {
        role: "user".to_string(),
        content: request.message.clone(),
    });

    // Create channel for speculative streaming
    let (tx, mut rx) = mpsc::channel::<SpeculativeChunk>(100);

    // Spawn speculative generation task via backend
    let llm_client = state.llm_client.clone();
    let draft_clone = draft_model.clone();
    let target_clone = target_model.clone();
    tokio::spawn(async move {
        llm_client.speculative_stream(
            messages, 
            draft_clone.as_deref(), 
            target_clone.as_deref(), 
            Some(lookahead), 
            tx
        ).await;
    });

    // Create SSE stream
    let user_message = request.message.clone();
    let state_clone = state.clone();
    let draft_display = draft_model.unwrap_or_else(|| "light".to_string());
    let target_display = target_model.unwrap_or_else(|| "standard".to_string());

    let stream = async_stream::stream! {
        // Send start event
        yield Ok(Event::default()
            .event("message")
            .data(json!({
                "type": "start",
                "conversation_id": conversation_id,
                "draft_model": draft_display,
                "target_model": target_display,
                "mode": "speculative"
            }).to_string()));

        let mut full_content = String::new();

        while let Some(chunk) = rx.recv().await {
            match chunk {
                SpeculativeChunk::Start { .. } => {
                    // Already sent start event above
                }
                SpeculativeChunk::Tokens { content } => {
                    full_content.push_str(&content);
                    yield Ok(Event::default()
                        .event("message")
                        .data(json!({
                            "type": "chunk",
                            "content": content
                        }).to_string()));
                }
                SpeculativeChunk::Done { stats } => {
                    // Save messages
                    let _ = save_message(&state_clone, conversation_id, MessageRole::User, &user_message).await;
                    let _ = save_message(&state_clone, conversation_id, MessageRole::Assistant, &full_content).await;

                    // Send done event with stats
                    yield Ok(Event::default()
                        .event("message")
                        .data(json!({
                            "type": "done",
                            "conversation_id": conversation_id,
                            "speculative_stats": {
                                "draft_tokens_generated": stats.draft_tokens_generated,
                                "tokens_accepted": stats.tokens_accepted,
                                "tokens_rejected": stats.tokens_rejected,
                                "acceptance_rate": stats.acceptance_rate,
                                "iterations": stats.iterations,
                                "draft_model": stats.draft_model,
                                "target_model": stats.target_model
                            }
                        }).to_string()));
                    break;
                }
                SpeculativeChunk::Error { message } => {
                    yield Ok(Event::default()
                        .event("message")
                        .data(json!({
                            "type": "error",
                            "error": message
                        }).to_string()));
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}