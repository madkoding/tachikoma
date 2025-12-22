//! =============================================================================
//! Agent Handlers - Simplified
//! =============================================================================

use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::{debug, error, instrument, warn};

use crate::domain::ports::search_provider::SearchOptions;
use crate::infrastructure::api::dto::{
    CommandExecuteRequest, CommandResultDto, ErrorResponse,
    WebSearchRequest, WebSearchResultDto,
};
use crate::AppState;

/// POST /api/agent/search
#[instrument(skip(state, request))]
pub async fn web_search(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WebSearchRequest>,
) -> Result<Json<Vec<WebSearchResultDto>>, (StatusCode, Json<ErrorResponse>)> {
    debug!(query = %request.query, limit = request.limit, "Performing web search");

    let options = SearchOptions::with_limit(request.limit);

    match state.search_provider.search(&request.query, Some(options)).await {
        Ok(response) => {
            let results: Vec<WebSearchResultDto> = response.results
                .into_iter()
                .map(|r| WebSearchResultDto {
                    title: r.title,
                    url: r.url,
                    snippet: r.snippet,
                    source: r.engine.unwrap_or_else(|| "unknown".to_string()),
                    score: None,
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

/// POST /api/agent/execute
#[instrument(skip(state, request))]
pub async fn execute_command(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CommandExecuteRequest>,
) -> Result<Json<CommandResultDto>, (StatusCode, Json<ErrorResponse>)> {
    debug!(command = %request.command, "Executing command");

    // Validate command first
    match state.command_executor.validate(&request.command).await {
        Ok(false) => {
            warn!(command = %request.command, "Command not allowed");
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse::new("COMMAND_NOT_ALLOWED", "Command not in allowlist")),
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

    let options = request.working_directory.map(|dir| {
        crate::domain::ports::command_executor::ExecutionOptions::with_working_dir(&dir)
    });

    match state.command_executor.execute(&request.command, options).await {
        Ok(result) => {
            Ok(Json(CommandResultDto {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                duration_ms: result.execution_time_ms,
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

/// GET /api/agent/commands
#[instrument(skip(_state))]
pub async fn get_allowed_commands(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    // Return a static list for now
    Ok(Json(vec![
        "ls".to_string(), "cat".to_string(), "head".to_string(), "tail".to_string(),
        "grep".to_string(), "find".to_string(), "pwd".to_string(), "echo".to_string(),
        "git".to_string(), "cargo".to_string(), "npm".to_string(), "node".to_string(),
    ]))
}

/// GET /api/agent/search/categories  
#[instrument(skip(_state))]
pub async fn get_search_categories(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(vec![
        "general".to_string(), "images".to_string(), "videos".to_string(),
        "news".to_string(), "map".to_string(), "it".to_string(),
    ]))
}
