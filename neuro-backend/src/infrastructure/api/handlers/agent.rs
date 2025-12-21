//! =============================================================================
//! Agent Handlers
//! =============================================================================
//! HTTP handlers for AI agent capabilities (search, command execution).
//! =============================================================================

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::{debug, error, instrument, warn};

use crate::domain::ports::search_provider::SearchQuery;
use crate::infrastructure::api::dto::{
    CommandExecuteRequest, CommandResultDto, ErrorResponse,
    WebSearchRequest, WebSearchResultDto,
};
use crate::AppState;

/// =============================================================================
/// Web search
/// =============================================================================
/// POST /api/agent/search
/// =============================================================================
#[instrument(skip(state, request))]
pub async fn web_search(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WebSearchRequest>,
) -> Result<Json<Vec<WebSearchResultDto>>, (StatusCode, Json<ErrorResponse>)> {
    debug!(query = %request.query, limit = request.limit, "Performing web search");

    let search_query = SearchQuery {
        query: request.query,
        categories: request.category.map(|c| vec![c]).unwrap_or_default(),
        language: None,
        limit: Some(request.limit as u32),
        page: Some(1),
        time_range: None,
        safe_search: Some(true),
    };

    match state.search_provider.search(search_query).await {
        Ok(response) => {
            let results: Vec<WebSearchResultDto> = response.results
                .into_iter()
                .map(|r| WebSearchResultDto {
                    title: r.title,
                    url: r.url,
                    snippet: r.snippet,
                    source: r.source,
                    score: r.score,
                })
                .collect();

            Ok(Json(results))
        }
        Err(e) => {
            error!(error = %e, "Web search failed");
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("SEARCH_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Execute command
/// =============================================================================
/// POST /api/agent/execute
/// =============================================================================
#[instrument(skip(state, request))]
pub async fn execute_command(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CommandExecuteRequest>,
) -> Result<Json<CommandResultDto>, (StatusCode, Json<ErrorResponse>)> {
    debug!(command = %request.command, "Executing command");

    // First check if command is allowed
    match state.command_executor.is_command_allowed(&request.command).await {
        Ok(false) => {
            warn!(command = %request.command, "Command not allowed");
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse::new(
                    "COMMAND_NOT_ALLOWED",
                    "This command is not in the allowed whitelist",
                )),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CHECK_ERROR", e.to_string())),
            ));
        }
        Ok(true) => {}
    }

    let command_request = crate::domain::ports::command_executor::CommandRequest {
        command: request.command,
        working_directory: request.working_directory,
        environment: None,
        timeout_seconds: request.timeout_seconds,
    };

    match state.command_executor.execute(command_request).await {
        Ok(result) => {
            Ok(Json(CommandResultDto {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                duration_ms: result.duration_ms,
                timed_out: result.timed_out,
            }))
        }
        Err(e) => {
            error!(error = %e, "Command execution failed");
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("EXECUTION_ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Get allowed commands
/// =============================================================================
/// GET /api/agent/commands
/// =============================================================================
#[instrument(skip(state))]
pub async fn get_allowed_commands(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    match state.command_executor.get_allowed_commands().await {
        Ok(commands) => Ok(Json(commands)),
        Err(e) => {
            error!(error = %e, "Failed to get allowed commands");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("ERROR", e.to_string())),
            ))
        }
    }
}

/// =============================================================================
/// Get search categories
/// =============================================================================
/// GET /api/agent/search/categories
/// =============================================================================
#[instrument(skip(state))]
pub async fn get_search_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    match state.search_provider.get_categories().await {
        Ok(categories) => Ok(Json(categories)),
        Err(e) => {
            error!(error = %e, "Failed to get search categories");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("ERROR", e.to_string())),
            ))
        }
    }
}
