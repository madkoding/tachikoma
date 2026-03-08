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
use serde::Deserialize;
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};
use surrealdb::sql::Datetime;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    models::*,
    AppState,
};

// ============================================================================
// Health Check
// ============================================================================

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.db.health_check().await {
        Ok(true) => Json(json!({
            "status": "healthy",
            "service": "tachikoma-memory",
            "version": env!("CARGO_PKG_VERSION"),
        })),
        _ => Json(json!({
            "status": "unhealthy",
            "service": "tachikoma-memory",
        })),
    }
}

// ============================================================================
// Memory CRUD Operations
// ============================================================================

/// List all memories
pub async fn list_memories(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sql = "SELECT * FROM memory ORDER BY created_at DESC LIMIT 100";
    
    match state.db.client().query(sql).await {
        Ok(mut response) => {
            let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
            let memories: Vec<Memory> = records.into_iter().map(|r| r.to_memory()).collect();
            Json(json!({ "memories": memories, "count": memories.len() })).into_response()
        }
        Err(e) => {
            error!("Failed to list memories: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Get a single memory by ID
pub async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let sql = "SELECT * FROM type::thing('memory', $id)";
    
    match state.db.client().query(sql).bind(("id", id.to_string())).await {
        Ok(mut response) => {
            let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
            match records.into_iter().next() {
                Some(record) => Json(record.to_memory()).into_response(),
                None => (StatusCode::NOT_FOUND, Json(json!({ "error": "Memory not found" }))).into_response(),
            }
        }
        Err(e) => {
            error!("Failed to get memory: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Create a new memory
pub async fn create_memory(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateMemoryRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let memory_type = request.memory_type.unwrap_or_else(|| "fact".to_string());
    let importance = request.importance_score.unwrap_or(0.5);
    
    // Generate embedding vector
    let vector = match generate_embedding(&state.backend_url, &request.content).await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to generate embedding: {}", e);
            vec![]
        }
    };

    let request_metadata = request.metadata.clone().unwrap_or_default();
    let metadata = serde_json::to_value(&request_metadata).unwrap_or_default();

    let sql = r#"
        CREATE type::thing('memory', $id) SET
            content = $content,
            vector = $vector,
            memory_type = $memory_type,
            metadata = $metadata,
            created_at = $created_at,
            updated_at = $updated_at,
            access_count = 0,
            importance_score = $importance_score
    "#;

    match state.db.client()
        .query(sql)
        .bind(("id", id.to_string()))
        .bind(("content", request.content.clone()))
        .bind(("vector", vector.clone()))
        .bind(("memory_type", memory_type.clone()))
        .bind(("metadata", metadata))
        .bind(("created_at", Datetime::from(now)))
        .bind(("updated_at", Datetime::from(now)))
        .bind(("importance_score", importance))
        .await
    {
        Ok(_) => {
            info!("Created memory {}", id);
            let memory = Memory {
                id,
                content: request.content,
                memory_type: memory_type.parse().unwrap_or(MemoryType::Fact),
                vector,
                metadata: request_metadata,
                importance_score: importance,
                access_count: 0,
                created_at: now,
                updated_at: now,
            };
            (StatusCode::CREATED, Json(memory)).into_response()
        }
        Err(e) => {
            error!("Failed to create memory: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Update a memory
pub async fn update_memory(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateMemoryRequest>,
) -> impl IntoResponse {
    let now = Utc::now();
    let mut updates = vec!["updated_at = $updated_at".to_string()];
    
    if request.content.is_some() {
        updates.push("content = $content".to_string());
    }
    if request.memory_type.is_some() {
        updates.push("memory_type = $memory_type".to_string());
    }
    if request.importance_score.is_some() {
        updates.push("importance_score = $importance_score".to_string());
    }
    if request.metadata.is_some() {
        updates.push("metadata = $metadata".to_string());
    }

    let sql = format!(
        "UPDATE type::thing('memory', $id) SET {}",
        updates.join(", ")
    );

    let mut query = state.db.client().query(&sql)
        .bind(("id", id.to_string()))
        .bind(("updated_at", Datetime::from(now)));

    if let Some(content) = &request.content {
        query = query.bind(("content", content.clone()));
    }
    if let Some(memory_type) = &request.memory_type {
        query = query.bind(("memory_type", memory_type.clone()));
    }
    if let Some(importance) = request.importance_score {
        query = query.bind(("importance_score", importance));
    }
    if let Some(metadata) = &request.metadata {
        query = query.bind(("metadata", serde_json::to_value(metadata).unwrap_or_default()));
    }

    match query.await {
        Ok(mut response) => {
            let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
            match records.into_iter().next() {
                Some(record) => Json(record.to_memory()).into_response(),
                None => (StatusCode::NOT_FOUND, Json(json!({ "error": "Memory not found" }))).into_response(),
            }
        }
        Err(e) => {
            error!("Failed to update memory: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Delete a memory
pub async fn delete_memory(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // First check if it exists
    let check_sql = "SELECT * FROM type::thing('memory', $id)";
    match state.db.client().query(check_sql).bind(("id", id.to_string())).await {
        Ok(mut response) => {
            let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
            if records.is_empty() {
                return (StatusCode::NOT_FOUND, Json(json!({ "error": "Memory not found" }))).into_response();
            }
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
        }
    }

    // Delete the memory
    let sql = "DELETE type::thing('memory', $id)";
    match state.db.client().query(sql).bind(("id", id.to_string())).await {
        Ok(_) => {
            info!("Deleted memory {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            error!("Failed to delete memory: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

// ============================================================================
// Search Operations
// ============================================================================

/// Semantic search for memories
pub async fn search_memories(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchMemoriesRequest>,
) -> impl IntoResponse {
    let limit = request.limit.unwrap_or(10);
    let threshold = request.threshold.unwrap_or(0.5);

    // Generate embedding for query
    let query_vector = match generate_embedding(&state.backend_url, &request.query).await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to generate query embedding: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
        }
    };

    // Get all memories with vectors
    let sql = if let Some(ref memory_type) = request.memory_type {
        format!(
            "SELECT * FROM memory WHERE memory_type = '{}' AND array::len(vector) > 0",
            memory_type
        )
    } else {
        "SELECT * FROM memory WHERE array::len(vector) > 0".to_string()
    };

    match state.db.client().query(&sql).await {
        Ok(mut response) => {
            let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
            
            // Calculate similarities and filter
            let mut results: Vec<SearchResult> = records
                .into_iter()
                .filter_map(|record| {
                    let similarity = cosine_similarity(&query_vector, &record.vector);
                    if similarity >= threshold {
                        Some(SearchResult {
                            memory: record.to_memory(),
                            similarity,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by similarity and limit
            results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
            results.truncate(limit);

            Json(json!({ "results": results, "count": results.len() })).into_response()
        }
        Err(e) => {
            error!("Failed to search memories: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

// ============================================================================
// Relation Operations
// ============================================================================

/// Get relations for a memory
pub async fn get_memory_relations(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let sql = r#"
        SELECT * FROM related_to 
        WHERE in = type::thing('memory', $id) OR out = type::thing('memory', $id)
    "#;

    match state.db.client().query(sql).bind(("id", id.to_string())).await {
        Ok(mut response) => {
            let records: Vec<RelationRecord> = response.take(0).unwrap_or_default();
            let relations: Vec<Relation> = records.into_iter().map(|r| r.to_relation()).collect();
            Json(json!({ "relations": relations, "count": relations.len() })).into_response()
        }
        Err(e) => {
            error!("Failed to get relations: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Get related memories
pub async fn get_related_memories(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let sql = r#"
        SELECT out.* as memory FROM related_to 
        WHERE in = type::thing('memory', $id)
    "#;

    match state.db.client().query(sql).bind(("id", id.to_string())).await {
        Ok(mut response) => {
            let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
            let memories: Vec<Memory> = records.into_iter().map(|r| r.to_memory()).collect();
            Json(json!({ "memories": memories, "count": memories.len() })).into_response()
        }
        Err(e) => {
            error!("Failed to get related memories: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Create a relation between memories
pub async fn create_relation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateRelationRequest>,
) -> impl IntoResponse {
    let now = Utc::now();
    let confidence = request.confidence.unwrap_or(1.0);
    let metadata = request.metadata.unwrap_or(json!({}));

    let sql = format!(
        r#"
        RELATE memory:`{}`->related_to->memory:`{}` SET
            relation_type = $relation_type,
            confidence = $confidence,
            metadata = $metadata,
            created_at = $created_at
        "#,
        request.from_id, request.to_id
    );

    match state.db.client()
        .query(&sql)
        .bind(("relation_type", request.relation_type.clone()))
        .bind(("confidence", confidence))
        .bind(("metadata", metadata.clone()))
        .bind(("created_at", Datetime::from(now)))
        .await
    {
        Ok(_) => {
            info!("Created relation {} -> {}", request.from_id, request.to_id);
            let relation = Relation {
                from_id: request.from_id,
                to_id: request.to_id,
                relation_type: request.relation_type,
                confidence,
                metadata,
                created_at: now,
            };
            (StatusCode::CREATED, Json(relation)).into_response()
        }
        Err(e) => {
            error!("Failed to create relation: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// Delete a relation
pub async fn delete_relation(
    State(state): State<Arc<AppState>>,
    Path((from_id, to_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let sql = format!(
        "DELETE related_to WHERE in = memory:`{}` AND out = memory:`{}`",
        from_id, to_id
    );

    match state.db.client().query(&sql).await {
        Ok(_) => {
            info!("Deleted relation {} -> {}", from_id, to_id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            error!("Failed to delete relation: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

// ============================================================================
// Graph Admin Operations
// ============================================================================

/// Get graph statistics
pub async fn get_graph_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let memory_sql = "SELECT memory_type, count() as count FROM memory GROUP BY memory_type";
    let relation_sql = "SELECT count() as count FROM related_to";

    let mut memories_by_type = std::collections::HashMap::new();
    let mut total_memories = 0;
    let mut total_relations = 0;

    if let Ok(mut response) = state.db.client().query(memory_sql).await {
        #[derive(serde::Deserialize)]
        struct TypeCount {
            memory_type: String,
            count: i64,
        }
        let counts: Vec<TypeCount> = response.take(0).unwrap_or_default();
        for tc in counts {
            memories_by_type.insert(tc.memory_type, tc.count as usize);
            total_memories += tc.count as usize;
        }
    }

    if let Ok(mut response) = state.db.client().query(relation_sql).await {
        #[derive(serde::Deserialize)]
        struct Count {
            count: i64,
        }
        let counts: Vec<Count> = response.take(0).unwrap_or_default();
        total_relations = counts.first().map(|c| c.count as usize).unwrap_or(0);
    }

    Json(GraphStats {
        total_memories,
        total_relations,
        memories_by_type,
    })
}

/// Export full graph data
pub async fn export_graph(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let memory_sql = "SELECT * FROM memory";
    let relation_sql = "SELECT * FROM related_to";

    let mut memories = vec![];
    let mut relations = vec![];

    if let Ok(mut response) = state.db.client().query(memory_sql).await {
        let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();
        memories = records.into_iter().map(|r| r.to_memory()).collect();
    }

    if let Ok(mut response) = state.db.client().query(relation_sql).await {
        let records: Vec<RelationRecord> = response.take(0).unwrap_or_default();
        relations = records.into_iter().map(|r| r.to_relation()).collect();
    }

    Json(json!({
        "memories": memories,
        "relations": relations,
        "exported_at": Utc::now(),
    }))
}

/// SSE endpoint for graph events
pub async fn subscribe_graph_events(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Simple heartbeat stream for now
    let stream = stream::unfold(0u64, |count| async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let event = Event::default()
            .event("heartbeat")
            .data(json!({ "count": count }).to_string());
        Some((Ok(event), count + 1))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate embedding using tachikoma-backend LLM gateway
/// The backend routes to Ollama and handles model selection
async fn generate_embedding(backend_url: &str, text: &str) -> Result<Vec<f32>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/llm/embed", backend_url);

    let response = client
        .post(&url)
        .json(&json!({
            "text": text
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Backend LLM error: {}", response.status()));
    }

    #[derive(serde::Deserialize)]
    struct EmbedResponse {
        embedding: Vec<f32>,
    }

    let result: EmbedResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(result.embedding)
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    (dot / (mag_a * mag_b)) as f64
}
