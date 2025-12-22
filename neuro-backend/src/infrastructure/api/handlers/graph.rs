//! =============================================================================
//! Graph Handlers
//! =============================================================================
//! HTTP handlers for graph visualization and admin endpoints.
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

use crate::domain::ports::memory_repository::RelationDirection;
use crate::domain::value_objects::relation::{GraphEdge, Relation};
use crate::infrastructure::api::dto::{
    CreateRelationRequest, ErrorResponse, GraphEdgeDto, GraphExportDto,
    GraphStatsDto, MemoryDto,
};
use crate::AppState;

/// =============================================================================
/// Query parameters for getting relations
/// =============================================================================
#[derive(Debug, Deserialize)]
pub struct GetRelationsParams {
    /// Relation type filter
    #[serde(default)]
    pub relation_type: Option<String>,
    /// Direction (incoming, outgoing, both)
    #[serde(default = "default_direction")]
    pub direction: String,
}

fn default_direction() -> String {
    "both".to_string()
}

/// =============================================================================
/// Get graph statistics
/// =============================================================================
/// GET /api/admin/graph/stats
/// =============================================================================
#[instrument(skip(state))]
pub async fn get_graph_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GraphStatsDto>, (StatusCode, Json<ErrorResponse>)> {
    match state.memory_service.get_graph_stats().await {
        Ok(stats) => {
            Ok(Json(GraphStatsDto {
                total_nodes: stats.total_nodes,
                total_edges: stats.total_edges,
                nodes_by_type: stats.nodes_by_type,
                edges_by_type: stats.edges_by_type,
                avg_connections: stats.avg_connections,
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to get graph stats");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Export full graph (for visualization)
/// =============================================================================
/// GET /api/admin/graph/export
/// =============================================================================
#[instrument(skip(state))]
pub async fn export_graph(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GraphExportDto>, (StatusCode, Json<ErrorResponse>)> {
    match state.memory_service.export_graph().await {
        Ok(export) => {
            let nodes: Vec<MemoryDto> = export.nodes
                .into_iter()
                .map(|m| MemoryDto {
                    id: m.id,
                    content: m.content,
                    memory_type: format!("{:?}", m.memory_type).to_lowercase(),
                    importance_score: m.importance_score,
                    created_at: m.created_at.to_rfc3339(),
                    updated_at: m.updated_at.to_rfc3339(),
                    access_count: m.access_count,
                    metadata: serde_json::to_value(&m.metadata).unwrap_or_default(),
                })
                .collect();

            let edges: Vec<GraphEdgeDto> = export.edges
                .into_iter()
                .map(edge_to_dto)
                .collect();

            Ok(Json(GraphExportDto {
                nodes,
                edges,
                exported_at: export.exported_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to export graph");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("EXPORT_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Get relations for a memory
/// =============================================================================
/// GET /api/memories/:id/relations
/// =============================================================================
#[instrument(skip(state))]
pub async fn get_memory_relations(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<Uuid>,
    Query(params): Query<GetRelationsParams>,
) -> Result<Json<Vec<GraphEdgeDto>>, (StatusCode, Json<ErrorResponse>)> {
    let relation_type = params.relation_type.as_ref().and_then(|r| parse_relation(r));
    let direction = parse_direction(&params.direction);

    match state.memory_service.get_relations(memory_id, relation_type, direction).await {
        Ok(edges) => {
            let dtos: Vec<GraphEdgeDto> = edges.into_iter().map(edge_to_dto).collect();
            Ok(Json(dtos))
        }
        Err(e) => {
            error!(error = %e, "Failed to get relations");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Create relation between memories
/// =============================================================================
/// POST /api/memories/relations
/// =============================================================================
#[instrument(skip(state, request))]
pub async fn create_relation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateRelationRequest>,
) -> Result<(StatusCode, Json<GraphEdgeDto>), (StatusCode, Json<ErrorResponse>)> {
    let relation = match parse_relation(&request.relation) {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_RELATION",
                    format!("Unknown relation type: {}", request.relation),
                )),
            ));
        }
    };

    match state.memory_service.create_relation(
        request.from_id,
        request.to_id,
        relation,
        request.confidence,
    ).await {
        Ok(created_edge) => {
            Ok((StatusCode::CREATED, Json(edge_to_dto(created_edge))))
        }
        Err(e) => {
            error!(error = %e, "Failed to create relation");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Delete relation
/// =============================================================================
/// DELETE /api/memories/:from_id/relations/:to_id
/// =============================================================================
#[instrument(skip(state))]
pub async fn delete_relation(
    State(state): State<Arc<AppState>>,
    Path((from_id, to_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<DeleteRelationParams>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let relation = match parse_relation(&params.relation_type) {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("INVALID_RELATION", "Unknown relation type")),
            ));
        }
    };

    match state.memory_service.delete_relation(from_id, to_id, relation).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Relation not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete relation");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteRelationParams {
    pub relation_type: String,
}

/// =============================================================================
/// Get related memories (graph traversal)
/// =============================================================================
/// GET /api/memories/:id/related
/// =============================================================================
#[instrument(skip(state))]
pub async fn get_related_memories(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<Uuid>,
    Query(params): Query<RelatedMemoriesParams>,
) -> Result<Json<Vec<RelatedMemoryDto>>, (StatusCode, Json<ErrorResponse>)> {
    match state.memory_service.get_related_memories(
        memory_id,
        params.max_depth.unwrap_or(2),
        None,
    ).await {
        Ok(results) => {
            let dtos: Vec<RelatedMemoryDto> = results
                .into_iter()
                .map(|(memory, edge)| RelatedMemoryDto {
                    memory: MemoryDto {
                        id: memory.id,
                        content: memory.content,
                        memory_type: format!("{:?}", memory.memory_type).to_lowercase(),
                        importance_score: memory.importance_score,
                        created_at: memory.created_at.to_rfc3339(),
                        updated_at: memory.updated_at.to_rfc3339(),
                        access_count: memory.access_count,
                        metadata: serde_json::to_value(&memory.metadata).unwrap_or_default(),
                    },
                    edge: edge_to_dto(edge),
                })
                .collect();

            Ok(Json(dtos))
        }
        Err(e) => {
            error!(error = %e, "Failed to get related memories");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RelatedMemoriesParams {
    pub max_depth: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
pub struct RelatedMemoryDto {
    pub memory: MemoryDto,
    pub edge: GraphEdgeDto,
}

/// =============================================================================
/// Helper functions
/// =============================================================================

/// Convert GraphEdge to DTO
fn edge_to_dto(edge: GraphEdge) -> GraphEdgeDto {
    GraphEdgeDto {
        from_id: edge.from_id,
        to_id: edge.to_id,
        relation: format!("{}", edge.relation),
        confidence: edge.confidence,
        created_at: edge.created_at.to_rfc3339(),
    }
}

/// Parse relation string to enum
fn parse_relation(relation_str: &str) -> Option<Relation> {
    match relation_str.to_lowercase().as_str() {
        "related_to" | "relatedto" => Some(Relation::RelatedTo),
        "causes" => Some(Relation::Causes),
        "part_of" | "partof" => Some(Relation::PartOf),
        "follows" => Some(Relation::Follows),
        "contradicts" => Some(Relation::Contradicts),
        "supports" => Some(Relation::Supports),
        "derived_from" | "derivedfrom" => Some(Relation::DerivedFrom),
        "same_as" | "sameas" => Some(Relation::SameAs),
        "context_of" | "contextof" => Some(Relation::ContextOf),
        "references" => Some(Relation::References),
        "supersedes" => Some(Relation::Supersedes),
        _ => None,
    }
}

/// Parse direction string to enum
fn parse_direction(direction_str: &str) -> RelationDirection {
    match direction_str.to_lowercase().as_str() {
        "incoming" | "in" => RelationDirection::Incoming,
        "outgoing" | "out" => RelationDirection::Outgoing,
        _ => RelationDirection::Both,
    }
}
