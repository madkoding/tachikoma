//! =============================================================================
//! HTTP Handlers for Agent Tools
//! =============================================================================

use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::AppState;
use crate::searxng::{SearchRequest, SearchResponse, SearchResult};
use crate::executor::{ExecuteRequest, ExecuteResult};

// =============================================================================
// Health Check
// =============================================================================

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub searxng_url: String,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "neuro-agent".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        searxng_url: state.config.searxng_url.clone(),
    })
}

// =============================================================================
// Web Search
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct WebSearchRequest {
    pub query: String,
    #[serde(default)]
    pub categories: Option<Vec<String>>,
    #[serde(default)]
    pub engines: Option<Vec<String>>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub time_range: Option<String>,
    #[serde(default)]
    pub max_results: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct WebSearchResponse {
    pub success: bool,
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub suggestions: Vec<String>,
    pub answers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn web_search(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WebSearchRequest>,
) -> Result<Json<WebSearchResponse>, (StatusCode, Json<WebSearchResponse>)> {
    info!("Web search request: {}", payload.query);

    let search_request = SearchRequest {
        query: payload.query,
        categories: payload.categories,
        engines: payload.engines,
        language: payload.language,
        pageno: payload.page,
        time_range: payload.time_range,
    };

    match state.searxng.search(&search_request).await {
        Ok(response) => {
            let mut results = response.results;
            
            // Limit results if requested
            if let Some(max) = payload.max_results {
                results.truncate(max);
            }

            Ok(Json(WebSearchResponse {
                success: true,
                results,
                total: response.total,
                suggestions: response.suggestions,
                answers: response.answers,
                error: None,
            }))
        }
        Err(e) => {
            error!("Search failed: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(WebSearchResponse {
                    success: false,
                    results: vec![],
                    total: 0,
                    suggestions: vec![],
                    answers: vec![],
                    error: Some(e),
                }),
            ))
        }
    }
}

// =============================================================================
// Command Execution
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn execute_command(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CommandRequest>,
) -> Result<Json<CommandResponse>, (StatusCode, Json<CommandResponse>)> {
    info!("Command execution request: {} {:?}", payload.command, payload.args);

    let exec_request = ExecuteRequest {
        command: payload.command,
        args: payload.args,
        working_dir: payload.working_dir,
        timeout_secs: payload.timeout_secs,
    };

    let result = state.executor.execute(&exec_request, &state.config.allowed_commands).await;

    let response = CommandResponse {
        success: result.success,
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        truncated: result.truncated,
        error: result.error,
    };

    if response.success {
        Ok(Json(response))
    } else {
        Err((StatusCode::BAD_REQUEST, Json(response)))
    }
}

// =============================================================================
// List Allowed Commands
// =============================================================================

#[derive(Debug, Serialize)]
pub struct AllowedCommandsResponse {
    pub commands: Vec<String>,
}

pub async fn list_allowed_commands(
    State(state): State<Arc<AppState>>,
) -> Json<AllowedCommandsResponse> {
    Json(AllowedCommandsResponse {
        commands: state.config.allowed_commands.clone(),
    })
}
