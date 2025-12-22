//! =============================================================================
//! Chat Handlers - Simplified
//! =============================================================================

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream::{self, Stream};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::domain::entities::chat::ChatRequest;
use crate::infrastructure::api::dto::{
    ChatMessageRequest, ChatMessageResponse, ConversationDto, ErrorResponse, MemoryDto,
};
use crate::AppState;

/// POST /api/chat
#[instrument(skip(state, request))]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatMessageRequest>,
) -> Result<Json<ChatMessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();
    
    debug!(
        conversation_id = ?request.conversation_id,
        stream = request.stream,
        "Processing chat message"
    );

    let chat_request = ChatRequest {
        message: request.message.clone(),
        conversation_id: request.conversation_id,
        language: "en".to_string(),
        stream: request.stream,
    };

    match state.chat_service.chat(chat_request).await {
        Ok(response) => {
            let processing_time = start.elapsed();

            Ok(Json(ChatMessageResponse {
                content: response.message.content.clone(),
                conversation_id: response.conversation_id,
                message_id: response.message.id,
                model: response.message.metadata.model.unwrap_or_default(),
                tokens_prompt: response.message.metadata.token_count.unwrap_or(0) as u64,
                tokens_completion: response.message.metadata.token_count.unwrap_or(0) as u64,
                processing_time_ms: processing_time.as_millis() as u64,
                extracted_memories: Vec::new(),
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to process chat message");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CHAT_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/chat/stream
#[instrument(skip(_state, request))]
pub async fn stream_message(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ChatMessageRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let conversation_id = request.conversation_id.unwrap_or_else(Uuid::new_v4);
    
    let stream = stream::once(async move {
        Ok(Event::default()
            .event("message")
            .data(format!(r#"{{"conversation_id":"{}","status":"streaming_not_implemented"}}"#, conversation_id)))
    });

    Sse::new(stream)
}

/// GET /api/chat/conversations/:id
#[instrument(skip(state))]
pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ConversationDto>, (StatusCode, Json<ErrorResponse>)> {
    match state.chat_service.get_conversation(conversation_id).await {
        Some(conversation) => {
            Ok(Json(ConversationDto {
                id: conversation.id,
                title: conversation.title.unwrap_or_else(|| "Untitled".to_string()),
                message_count: conversation.messages.len(),
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
