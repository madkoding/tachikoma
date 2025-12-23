//! =============================================================================
//! Memory Handlers - Simplified
//! =============================================================================

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::domain::entities::memory::{MemoryNode, MemoryType};
use crate::infrastructure::api::dto::{
    CreateMemoryRequest, ErrorResponse, MemoryDto, PaginatedResponse,
    SearchResultDto, SemanticSearchRequest, UpdateMemoryRequest,
};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListMemoriesParams {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    #[serde(default)]
    #[allow(dead_code)]
    pub memory_type: Option<String>,
}

fn default_page() -> usize { 1 }
fn default_per_page() -> usize { 20 }

/// GET /api/memories
#[instrument(skip(state))]
pub async fn list_memories(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListMemoriesParams>,
) -> Result<Json<PaginatedResponse<MemoryDto>>, (StatusCode, Json<ErrorResponse>)> {
    let offset = (params.page.saturating_sub(1)) * params.per_page;

    match state.memory_service.get_all_memories(params.per_page, offset).await {
        Ok(memories) => {
            let total = state.memory_service.count_memories().await.unwrap_or(0);
            let total_pages = (total + params.per_page - 1) / params.per_page;

            let data: Vec<MemoryDto> = memories.into_iter().map(memory_to_dto).collect();

            Ok(Json(PaginatedResponse {
                data,
                total,
                page: params.page,
                per_page: params.per_page,
                total_pages,
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to list memories");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/memories/:id
#[instrument(skip(state))]
pub async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<Uuid>,
) -> Result<Json<MemoryDto>, (StatusCode, Json<ErrorResponse>)> {
    match state.memory_service.get_memory(memory_id).await {
        Ok(memory) => Ok(Json(memory_to_dto(memory))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::new("NOT_FOUND", "Memory not found")),
                ))
            } else {
                error!(error = %e, "Failed to get memory");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
                ))
            }
        }
    }
}

/// POST /api/memories
#[instrument(skip(state, request))]
pub async fn create_memory(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateMemoryRequest>,
) -> Result<(StatusCode, Json<MemoryDto>), (StatusCode, Json<ErrorResponse>)> {
    debug!(content_len = request.content.len(), "Creating new memory");

    let memory_type = parse_memory_type(&request.memory_type);

    match state.memory_service.create_memory(request.content, memory_type, None).await {
        Ok(memory) => Ok((StatusCode::CREATED, Json(memory_to_dto(memory)))),
        Err(e) => {
            error!(error = %e, "Failed to create memory");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/memories/:id
#[instrument(skip(state, request))]
pub async fn update_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<Uuid>,
    Json(request): Json<UpdateMemoryRequest>,
) -> Result<Json<MemoryDto>, (StatusCode, Json<ErrorResponse>)> {
    match state.memory_service.update_memory(memory_id, request.content, None, None).await {
        Ok(memory) => Ok(Json(memory_to_dto(memory))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::new("NOT_FOUND", "Memory not found")),
                ))
            } else {
                error!(error = %e, "Failed to update memory");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
                ))
            }
        }
    }
}

/// DELETE /api/memories/:id
#[instrument(skip(state))]
pub async fn delete_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.memory_service.delete_memory(memory_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Memory not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete memory");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/memories/search
#[instrument(skip(state, request))]
pub async fn search_memories(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SemanticSearchRequest>,
) -> Result<Json<Vec<SearchResultDto>>, (StatusCode, Json<ErrorResponse>)> {
    debug!(query = %request.query, limit = request.limit, "Performing semantic search");

    match state.memory_service.search(&request.query, request.limit).await {
        Ok(results) => {
            let dtos: Vec<SearchResultDto> = results
                .into_iter()
                .filter(|(_, similarity)| *similarity >= request.min_similarity)
                .map(|(memory, similarity)| SearchResultDto {
                    memory: memory_to_dto(memory),
                    similarity,
                })
                .collect();

            Ok(Json(dtos))
        }
        Err(e) => {
            error!(error = %e, "Semantic search failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("SEARCH_ERROR", e.to_string())),
            ))
        }
    }
}

fn memory_to_dto(memory: MemoryNode) -> MemoryDto {
    MemoryDto {
        id: memory.id,
        content: memory.content,
        memory_type: format!("{:?}", memory.memory_type).to_lowercase(),
        importance_score: memory.importance_score,
        created_at: memory.created_at.to_rfc3339(),
        updated_at: memory.updated_at.to_rfc3339(),
        access_count: memory.access_count,
        metadata: serde_json::to_value(&memory.metadata).unwrap_or_default(),
    }
}

fn parse_memory_type(type_str: &str) -> MemoryType {
    match type_str.to_lowercase().as_str() {
        "fact" => MemoryType::Fact,
        "preference" => MemoryType::Preference,
        "procedure" => MemoryType::Procedure,
        "conversation" => MemoryType::Conversation,
        "context" => MemoryType::Context,
        "semantictag" | "semantic_tag" => MemoryType::SemanticTag,
        "issue" => MemoryType::Issue,
        "insight" => MemoryType::Insight,
        "externalknowledge" | "external_knowledge" => MemoryType::ExternalKnowledge,
        "codesnippet" | "code_snippet" => MemoryType::CodeSnippet,
        "task" => MemoryType::Task,
        "entity" => MemoryType::Entity,
        "goal" => MemoryType::Goal,
        "skill" => MemoryType::Skill,
        "event" => MemoryType::Event,
        "opinion" => MemoryType::Opinion,
        "experience" => MemoryType::Experience,
        _ => MemoryType::General,
    }
}
