//! =============================================================================
//! System Handlers
//! =============================================================================
//! HTTP handlers for health checks and system information.
//! =============================================================================

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{instrument, warn};

use crate::infrastructure::api::dto::{ErrorResponse, HealthResponse, ModelInfoDto, ServiceStatusDto};
use crate::AppState;

/// Application start time for uptime calculation
static mut START_TIME: Option<Instant> = None;

/// Initialize start time (call once at application startup)
pub fn init_start_time() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}

/// Get uptime in seconds
fn get_uptime_seconds() -> u64 {
    unsafe {
        START_TIME
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0)
    }
}

/// =============================================================================
/// Health check
/// =============================================================================
/// GET /api/health
/// =============================================================================
#[instrument(skip(state))]
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check database
    let db_status = match state.database_pool.client()
        .query("INFO FOR DB")
        .await {
            Ok(_) => "healthy".to_string(),
            Err(e) => {
                warn!(error = %e, "Database health check failed");
                format!("unhealthy: {}", e)
            }
        };

    // Check LLM provider
    let llm_status = match state.llm_provider.health_check().await {
        Ok(true) => "healthy".to_string(),
        Ok(false) => "unhealthy".to_string(),
        Err(e) => {
            warn!(error = %e, "LLM health check failed");
            format!("unhealthy: {}", e)
        }
    };

    // Check search provider
    let search_status = if state.search_provider.is_healthy().await {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    // Overall status
    let overall_status = if db_status == "healthy" && llm_status == "healthy" {
        "healthy"
    } else if db_status == "healthy" {
        "degraded"
    } else {
        "unhealthy"
    };

    Ok(Json(HealthResponse {
        status: overall_status.to_string(),
        services: ServiceStatusDto {
            database: db_status,
            llm: llm_status,
            search: search_status,
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
    }))
}

/// =============================================================================
/// Readiness check (for Kubernetes)
/// =============================================================================
/// GET /api/ready
/// =============================================================================
#[instrument(skip(state))]
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    // Only check database for readiness
    match state.database_pool.client()
        .query("INFO FOR DB")
        .await {
            Ok(_) => Ok(StatusCode::OK),
            Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
        }
}

/// =============================================================================
/// Liveness check (for Kubernetes)
/// =============================================================================
/// GET /api/live
/// =============================================================================
pub async fn liveness_check() -> StatusCode {
    // Simple liveness - if we can respond, we're alive
    StatusCode::OK
}

/// =============================================================================
/// GET /api/models
/// =============================================================================
#[instrument(skip(state))]
pub async fn list_models(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ModelInfoDto>>, (StatusCode, Json<ErrorResponse>)> {
    match state.llm_provider.list_models().await {
        Ok(models) => {
            let dtos: Vec<ModelInfoDto> = models
                .into_iter()
                .map(|m| ModelInfoDto {
                    id: m.name.clone(),
                    name: m.name,
                    size_bytes: Some(m.size),
                    parameters: m.parameters,
                    context_length: None,
                    is_embedding_model: m.is_embedding,
                })
                .collect();

            Ok(Json(dtos))
        }
        Err(e) => {
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("LLM_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Get system info
/// =============================================================================
/// GET /api/system/info
/// =============================================================================
#[instrument(skip(state))]
pub async fn system_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SystemInfoDto>, (StatusCode, Json<ErrorResponse>)> {
    // Get memory stats
    let memory_count = state.memory_service.count_memories().await.unwrap_or(0);
    
    // Get graph stats
    let graph_stats = state.memory_service.get_graph_stats().await.ok();

    Ok(Json(SystemInfoDto {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
        memory_count,
        total_edges: graph_stats.as_ref().map(|s| s.total_edges).unwrap_or(0),
        rust_version: rustc_version(),
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct SystemInfoDto {
    pub version: String,
    pub uptime_seconds: u64,
    pub memory_count: usize,
    pub total_edges: usize,
    pub rust_version: String,
}

fn rustc_version() -> String {
    // This would be set at compile time in a real implementation
    "1.75.0".to_string()
}
