//! =============================================================================
//! Chat Handlers - Simplified
//! =============================================================================

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, instrument};
use uuid::Uuid;
use tokio_stream::wrappers::ReceiverStream;

use crate::domain::entities::chat::{ChatRequest, ChatMessage, MessageMetadata};
use crate::infrastructure::api::dto::{
    ChatMessageRequest, ChatMessageResponse, ConversationDto, ConversationWithMessagesDto, ChatMessageDto, ErrorResponse,
};
use crate::infrastructure::request_logger::{REQUEST_LOGGER, spawn_spinner_task};
use crate::AppState;

/// POST /api/chat
#[instrument(skip(state, request))]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatMessageRequest>,
) -> Result<Json<ChatMessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();
    let msg_len = request.message.len();
    let conv_id = request.conversation_id.map(|id| id.to_string()).unwrap_or_else(|| "new".to_string());
    
    // Start request logging
    REQUEST_LOGGER.start_request(
        "CHAT",
        &format!("conv={} len={}", &conv_id[..8.min(conv_id.len())], msg_len)
    ).await;
    
    let spinner = spawn_spinner_task("Generating response...".to_string());

    let chat_request = ChatRequest {
        message: request.message.clone(),
        conversation_id: request.conversation_id,
        language: "en".to_string(),
        stream: request.stream,
    };

    match state.chat_service.chat(chat_request).await {
        Ok(response) => {
            spinner.abort();
            let processing_time = start.elapsed();
            
            REQUEST_LOGGER.complete_success(
                response.message.metadata.prompt_tokens.unwrap_or(0) as u32,
                response.message.metadata.completion_tokens.unwrap_or(0) as u32,
                &response.message.metadata.model.clone().unwrap_or_default()
            ).await;

            Ok(Json(ChatMessageResponse {
                content: response.message.content.clone(),
                conversation_id: response.conversation_id,
                message_id: response.message.id,
                model: response.message.metadata.model.unwrap_or_default(),
                tokens_prompt: response.message.metadata.prompt_tokens.unwrap_or(0),
                tokens_completion: response.message.metadata.completion_tokens.unwrap_or(0),
                processing_time_ms: processing_time.as_millis() as u64,
                extracted_memories: Vec::new(),
                tools_used: response.tools_used,
            }))
        }
        Err(e) => {
            spinner.abort();
            REQUEST_LOGGER.complete_error(&e.to_string()).await;
            error!(error = %e, "Failed to process chat message");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CHAT_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/chat/stream
#[instrument(skip(state, request))]
pub async fn stream_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let conversation_id = request.conversation_id.unwrap_or_else(Uuid::new_v4);
    let message = request.message.clone();
    let message_for_memory = request.message.clone(); // Clone for memory extraction
    let msg_len = message.len();
    let conv_id_str = conversation_id.to_string();
    
    // Start request logging
    REQUEST_LOGGER.start_request(
        "STREAM",
        &format!("conv={} len={}", &conv_id_str[..8], msg_len)
    ).await;
    
    // Get services
    let memory_service = state.memory_service.clone();
    let chat_service = state.chat_service.clone();
    
    // =========================================================================
    // TOOL DETECTION - Run before streaming to check if we need to execute tools
    // =========================================================================
    let tools_used = chat_service.detect_and_execute_tools(&message).await;
    let tools_context = if !tools_used.is_empty() {
        info!("🔧 Tools executed before streaming: {:?}", tools_used.iter().map(|(n, _)| n).collect::<Vec<_>>());
        Some(tools_used)
    } else {
        None
    };
    
    // Create a channel for streaming
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(100);
    
    // Spawn task to handle streaming
    let _spinner = spawn_spinner_task("Streaming response...".to_string());
    
    tokio::spawn(async move {
        let start = Instant::now();
        
        // Get conversation history
        let conversation = chat_service.get_conversation(conversation_id).await;
        
        // Get relevant memory context
        let context_memories = memory_service.search(&message, 5).await.unwrap_or_default();
        let memory_ids: Vec<Uuid> = context_memories.iter().map(|(m, _)| m.id).collect();
        
        // Build system prompt (using the same prompt from ChatService)
        let mut system_prompt = chat_service.system_prompt.clone();
        
        // Add memory context to system prompt if available
        if !context_memories.is_empty() {
            system_prompt.push_str("\n\nTienes acceso a estos recuerdos relevantes del usuario:\n");
            for (memory, score) in context_memories.iter().take(3) {
                if *score > 0.3 {
                    system_prompt.push_str(&format!("- {}\n", memory.content));
                }
            }
        }
        
        // Add tool results to system prompt if tools were executed
        if let Some(ref tools) = tools_context {
            system_prompt.push_str("\n\n=== RESULTADOS DE HERRAMIENTAS EJECUTADAS ===\n");
            system_prompt.push_str("Las siguientes herramientas se ejecutaron automáticamente. Usa esta información para responder:\n\n");
            for (tool_name, tool_result) in tools {
                system_prompt.push_str(&format!("📌 {} :\n{}\n\n", tool_name, tool_result));
            }
            system_prompt.push_str("Responde al usuario basándote en los resultados anteriores.\n");
        }
        
        // Build messages array for Ollama
        let mut ollama_messages: Vec<serde_json::Value> = vec![
            serde_json::json!({
                "role": "system",
                "content": system_prompt
            })
        ];
        
        // Add conversation history (last 10 messages to avoid context overflow)
        if let Some(conv) = conversation {
            let history_messages: Vec<_> = conv.messages.iter()
                .rev()
                .take(10)
                .rev()
                .collect();
            
            for msg in history_messages {
                let role = match msg.role {
                    crate::domain::entities::chat::MessageRole::User => "user",
                    crate::domain::entities::chat::MessageRole::Assistant => "assistant",
                    crate::domain::entities::chat::MessageRole::System => "system",
                    crate::domain::entities::chat::MessageRole::Tool => "assistant",
                };
                ollama_messages.push(serde_json::json!({
                    "role": role,
                    "content": msg.content
                }));
            }
        }
        
        // Add current user message
        ollama_messages.push(serde_json::json!({
            "role": "user",
            "content": message
        }));
        
        // Select model
        let model = chat_service.select_model_for_task(&message);
        
        // Send initial event
        let init_data = serde_json::json!({
            "type": "start",
            "conversation_id": conversation_id,
            "model": model,
        });
        let _ = tx.send(Ok(Event::default().event("message").data(init_data.to_string()))).await;
        
        // Make streaming request to Ollama
        let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let client = reqwest::Client::new();
        
        let ollama_request = serde_json::json!({
            "model": model,
            "messages": ollama_messages,
            "stream": true,
            "options": {
                "temperature": 0.8,
                "num_predict": 2048
            }
        });
        
        match client.post(format!("{}/api/chat", ollama_url))
            .json(&ollama_request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let mut stream = response.bytes_stream();
                    let mut full_content = String::new();
                    let mut prompt_tokens: u64 = 0;
                    let mut completion_tokens: u64 = 0;
                    
                    while let Some(chunk_result) = futures_util::StreamExt::next(&mut stream).await {
                        match chunk_result {
                            Ok(chunk) => {
                                if let Ok(text) = std::str::from_utf8(&chunk) {
                                    for line in text.lines() {
                                        if line.is_empty() { continue; }
                                        
                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                            if let Some(msg) = json.get("message") {
                                                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                                                    full_content.push_str(content);
                                                    
                                                    let chunk_data = serde_json::json!({
                                                        "type": "chunk",
                                                        "content": content,
                                                    });
                                                    let _ = tx.send(Ok(Event::default().event("message").data(chunk_data.to_string()))).await;
                                                }
                                            }
                                            
                                            if json.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                                                prompt_tokens = json.get("prompt_eval_count").and_then(|c| c.as_u64()).unwrap_or(0);
                                                completion_tokens = json.get("eval_count").and_then(|c| c.as_u64()).unwrap_or(0);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Stream chunk error");
                                break;
                            }
                        }
                    }
                    
                    // Save to database
                    let user_message = ChatMessage::user(conversation_id, message);
                    let mut assistant_message = ChatMessage::assistant(conversation_id, full_content.clone());
                    assistant_message.metadata = MessageMetadata {
                        model: Some(model.clone()),
                        context_memory_ids: memory_ids,
                        generation_time_ms: Some(start.elapsed().as_millis() as u64),
                        prompt_tokens: Some(prompt_tokens),
                        completion_tokens: Some(completion_tokens),
                        token_count: Some((prompt_tokens + completion_tokens) as u32),
                        ..Default::default()
                    };
                    
                    // Send final event FIRST (before saving to DB)
                    let final_data = serde_json::json!({
                        "type": "done",
                        "conversation_id": conversation_id,
                        "message_id": assistant_message.id,
                        "model": model,
                        "tokens_prompt": prompt_tokens,
                        "tokens_completion": completion_tokens,
                        "processing_time_ms": start.elapsed().as_millis() as u64,
                    });
                    let _ = tx.send(Ok(Event::default().event("message").data(final_data.to_string()))).await;
                    
                    // Log stream completion
                    let chunks_count = full_content.matches("").count() as u32 / 10; // Approximate
                    REQUEST_LOGGER.complete_stream(
                        chunks_count,
                        (prompt_tokens + completion_tokens) as u32,
                        &model
                    ).await;
                    
                    // Drop the sender to signal stream completion
                    drop(tx);
                    
                    // Extract and store memories from user message
                    chat_service.extract_and_store_memories(&message_for_memory).await;
                    
                    // NOW save to database (after stream is closed)
                    tracing::info!(conversation_id = %conversation_id, "Saving conversation after stream completed");
                    chat_service.update_conversation_direct(conversation_id, user_message, assistant_message.clone()).await;
                    tracing::info!(conversation_id = %conversation_id, "Conversation save completed");
                } else {
                    let error_msg = format!("Ollama error: {}", response.status());
                    REQUEST_LOGGER.complete_error(&error_msg).await;
                    let error_data = serde_json::json!({
                        "type": "error",
                        "error": error_msg,
                    });
                    let _ = tx.send(Ok(Event::default().event("message").data(error_data.to_string()))).await;
                }
            }
            Err(e) => {
                let error_msg = format!("Request failed: {}", e);
                REQUEST_LOGGER.complete_error(&error_msg).await;
                let error_data = serde_json::json!({
                    "type": "error",
                    "error": error_msg,
                });
                let _ = tx.send(Ok(Event::default().event("message").data(error_data.to_string()))).await;
            }
        }
        
        tracing::info!("Stream handler task completed");
    });

    Sse::new(ReceiverStream::new(rx))
}

/// GET /api/chat/conversations/:id
#[instrument(skip(state))]
pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ConversationWithMessagesDto>, (StatusCode, Json<ErrorResponse>)> {
    match state.chat_service.get_conversation(conversation_id).await {
        Some(conversation) => {
            let messages: Vec<ChatMessageDto> = conversation.messages.iter().map(|m| {
                ChatMessageDto {
                    id: m.id,
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.clone(),
                    model: m.metadata.model.clone(),
                    tokens_prompt: m.metadata.prompt_tokens,
                    tokens_completion: m.metadata.completion_tokens,
                    created_at: m.created_at.to_rfc3339(),
                }
            }).collect();

            Ok(Json(ConversationWithMessagesDto {
                id: conversation.id,
                title: conversation.title.unwrap_or_else(|| "Untitled".to_string()),
                messages,
                created_at: conversation.created_at.to_rfc3339(),
                updated_at: conversation.updated_at.to_rfc3339(),
            }))
        }
        None => {
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("NOT_FOUND", "Conversation not found")),
            ))
        }
    }
}

/// GET /api/chat/conversations
#[instrument(skip(state))]
pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ConversationDto>>, (StatusCode, Json<ErrorResponse>)> {
    let conversations = state.chat_service.list_conversations().await;
    let dtos: Vec<ConversationDto> = conversations
        .into_iter()
        .map(|(id, title, updated_at)| ConversationDto {
            id,
            title: title.unwrap_or_else(|| "Untitled".to_string()),
            message_count: 0,
            created_at: updated_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(dtos))
}

/// DELETE /api/chat/conversations/:id
#[instrument(skip(state))]
pub async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if state.chat_service.delete_conversation(conversation_id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Conversation not found")),
        ))
    }
}
