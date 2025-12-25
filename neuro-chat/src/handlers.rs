//! =============================================================================
//! API Handlers
//! =============================================================================

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::{Event, Sse}, IntoResponse},
    Json,
};
use chrono::Utc;
use futures::stream::{self, Stream};
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};
use surrealdb::sql::Datetime;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    models::*,
    AppState,
};

// ============================================================================
// Health Check
// ============================================================================

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = state.db.health_check().await.unwrap_or(false);
    let ollama_healthy = state.ollama.health_check().await;
    let memory_healthy = state.memory_client.health_check().await;

    let status = if db_healthy && ollama_healthy { "healthy" } else { "degraded" };

    Json(json!({
        "status": status,
        "service": "neuro-chat",
        "version": env!("CARGO_PKG_VERSION"),
        "services": {
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "ollama": if ollama_healthy { "healthy" } else { "unhealthy" },
            "memory": if memory_healthy { "healthy" } else { "unavailable" },
        }
    }))
}

/// List available models
pub async fn list_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.ollama.list_models().await {
        Ok(models) => Json(json!({ "models": models })).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": e }))).into_response(),
    }
}

// ============================================================================
// Chat Operations
// ============================================================================

/// Send a message and get a complete response
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let model = request.model.unwrap_or_else(|| state.config.default_model.clone());
    
    // Get or create conversation
    let conversation_id = match request.conversation_id {
        Some(id) => id,
        None => create_conversation(&state).await.unwrap_or_else(|_| Uuid::new_v4()),
    };

    // Build messages with context
    let mut messages = vec![];
    
    // Add system prompt
    messages.push(OllamaMessage {
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
                messages.push(OllamaMessage {
                    role: "system".to_string(),
                    content: format!("Relevant memories:\n{}", memory_context),
                });
            }
        }
    }

    // Add conversation history
    if let Ok(history) = get_conversation_messages(&state, conversation_id).await {
        for msg in history.iter().rev().take(10).rev() {
            messages.push(OllamaMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            });
        }
    }

    // Add user message
    messages.push(OllamaMessage {
        role: "user".to_string(),
        content: request.message.clone(),
    });

    // Call Ollama
    match state.ollama.chat(messages, &model).await {
        Ok(response) => {
            let message_id = Uuid::new_v4();
            
            // Save messages to database
            let _ = save_message(&state, conversation_id, MessageRole::User, &request.message).await;
            let _ = save_message(&state, conversation_id, MessageRole::Assistant, &response.message.content).await;

            let resp = SendMessageResponse {
                content: response.message.content,
                conversation_id,
                message_id,
                model,
                tokens_prompt: response.prompt_eval_count,
                tokens_completion: response.eval_count,
                processing_time_ms: start.elapsed().as_millis() as u64,
            };
            Json(resp).into_response()
        }
        Err(e) => {
            error!("Ollama error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))).into_response()
        }
    }
}

/// Stream a message response via SSE
pub async fn stream_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let model = request.model.unwrap_or_else(|| state.config.default_model.clone());
    
    // Get or create conversation
    let conversation_id = match request.conversation_id {
        Some(id) => id,
        None => create_conversation(&state).await.unwrap_or_else(|_| Uuid::new_v4()),
    };

    // Build messages
    let mut messages = vec![];
    
    messages.push(OllamaMessage {
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
                messages.push(OllamaMessage {
                    role: "system".to_string(),
                    content: format!("Relevant memories:\n{}", memory_context),
                });
            }
        }
    }

    // Add history
    if let Ok(history) = get_conversation_messages(&state, conversation_id).await {
        for msg in history.iter().rev().take(10).rev() {
            messages.push(OllamaMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            });
        }
    }

    messages.push(OllamaMessage {
        role: "user".to_string(),
        content: request.message.clone(),
    });

    // Create channel for streaming
    let (tx, mut rx) = mpsc::channel::<Result<OllamaStreamChunk, String>>(100);
    
    // Spawn Ollama streaming task
    let ollama = state.ollama.clone();
    let model_clone = model.clone();
    tokio::spawn(async move {
        ollama.chat_stream(messages, &model_clone, tx).await;
    });

    // Create SSE stream
    let user_message = request.message.clone();
    let state_clone = state.clone();
    
    let stream = async_stream::stream! {
        // Send start event
        yield Ok(Event::default()
            .event("message")
            .data(json!({
                "type": "start",
                "conversation_id": conversation_id,
                "model": model
            }).to_string()));

        let mut full_content = String::new();
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;

        while let Some(result) = rx.recv().await {
            match result {
                Ok(chunk) => {
                    if let Some(msg) = &chunk.message {
                        full_content.push_str(&msg.content);
                        yield Ok(Event::default()
                            .event("message")
                            .data(json!({
                                "type": "chunk",
                                "content": msg.content
                            }).to_string()));
                    }
                    
                    if chunk.done {
                        prompt_tokens = chunk.prompt_eval_count;
                        completion_tokens = chunk.eval_count;
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
    r#"Eres un asistente de IA amigable y útil llamado NEURO. 
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
