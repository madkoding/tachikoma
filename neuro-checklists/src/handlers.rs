//! HTTP Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    #[serde(default)]
    pub include_archived: bool,
}

fn default_page() -> usize { 1 }
fn default_per_page() -> usize { 50 }

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "neuro-checklists",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// =============================================================================
// Checklist Handlers
// =============================================================================

/// GET /api/checklists
pub async fn list_checklists(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<ChecklistResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let offset = (params.page.saturating_sub(1)) * params.per_page;

    match state.db.get_all_checklists(params.per_page, offset, params.include_archived).await {
        Ok(checklists) => {
            let total = state.db.count_checklists(params.include_archived).await.unwrap_or(0);
            let total_pages = (total + params.per_page - 1) / params.per_page;

            // Get items for each checklist to calculate counts
            let mut responses = Vec::new();
            for checklist in checklists {
                let items = state.db.get_items(checklist.id).await.unwrap_or_default();
                responses.push(checklist.to_response(&items));
            }

            Ok(Json(PaginatedResponse {
                data: responses,
                total,
                page: params.page,
                per_page: params.per_page,
                total_pages,
            }))
        }
        Err(e) => {
            error!(error = %e, "Failed to list checklists");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/checklists/:id
pub async fn get_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChecklistWithItemsResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_checklist(id).await {
        Ok(Some(checklist)) => {
            let items = state.db.get_items(id).await.unwrap_or_default();
            Ok(Json(checklist.to_response_with_items(items)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Checklist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to get checklist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/checklists
pub async fn create_checklist(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateChecklist>,
) -> Result<(StatusCode, Json<ChecklistWithItemsResponse>), (StatusCode, Json<ErrorResponse>)> {
    debug!(title = %request.title, "Creating new checklist");

    let items_to_create = request.items.clone();

    match state.db.create_checklist(request).await {
        Ok(checklist) => {
            // Add initial items
            let mut created_items = Vec::new();
            for item_data in items_to_create {
                if let Ok(item) = state.db.add_item(checklist.id, item_data).await {
                    created_items.push(item);
                }
            }

            Ok((StatusCode::CREATED, Json(checklist.to_response_with_items(created_items))))
        }
        Err(e) => {
            error!(error = %e, "Failed to create checklist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/checklists/:id
pub async fn update_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateChecklist>,
) -> Result<Json<ChecklistWithItemsResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.update_checklist(id, request).await {
        Ok(Some(checklist)) => {
            let items = state.db.get_items(id).await.unwrap_or_default();
            Ok(Json(checklist.to_response_with_items(items)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Checklist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update checklist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/checklists/:id
pub async fn delete_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.db.delete_checklist(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Checklist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete checklist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// Checklist Item Handlers
// =============================================================================

/// POST /api/checklists/:id/items
pub async fn add_item(
    State(state): State<Arc<AppState>>,
    Path(checklist_id): Path<Uuid>,
    Json(request): Json<CreateChecklistItem>,
) -> Result<(StatusCode, Json<ChecklistItemResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Verify checklist exists
    match state.db.get_checklist(checklist_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("NOT_FOUND", "Checklist not found")),
            ));
        }
        Err(e) => {
            error!(error = %e, "Failed to get checklist");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ));
        }
    }

    match state.db.add_item(checklist_id, request).await {
        Ok(item) => Ok((StatusCode::CREATED, Json(item.to_response()))),
        Err(e) => {
            error!(error = %e, "Failed to add item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/checklists/:checklist_id/items/:item_id
pub async fn update_item(
    State(state): State<Arc<AppState>>,
    Path((_, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateChecklistItem>,
) -> Result<Json<ChecklistItemResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.update_item(item_id, request).await {
        Ok(Some(item)) => Ok(Json(item.to_response())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Item not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/checklists/:checklist_id/items/:item_id/toggle
pub async fn toggle_item(
    State(state): State<Arc<AppState>>,
    Path((_, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ChecklistItemResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.toggle_item(item_id).await {
        Ok(Some(item)) => Ok(Json(item.to_response())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Item not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to toggle item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/checklists/:checklist_id/items/:item_id
pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path((_, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.db.delete_item(item_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Item not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// Import Handler
// =============================================================================

/// POST /api/checklists/import
pub async fn import_from_markdown(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ImportMarkdown>,
) -> Result<(StatusCode, Json<ChecklistWithItemsResponse>), (StatusCode, Json<ErrorResponse>)> {
    debug!("Importing checklist from markdown");

    let (parsed_title, items) = parse_markdown_checklist(&request.markdown);
    
    if items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("INVALID_MARKDOWN", "No checkbox items found in markdown")),
        ));
    }

    let title = request.title.unwrap_or(parsed_title);

    let create_request = CreateChecklist {
        title,
        description: None,
        priority: Some(3),
        due_date: None,
        items,
    };

    create_checklist(State(state), Json(create_request)).await
}
