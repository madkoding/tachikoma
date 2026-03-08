//! HTTP Handlers for Checklists Microservice
//! Uses BackendClient for data persistence

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Response DTOs
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

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
        "service": "tachikoma-checklists",
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
    debug!(page = params.page, per_page = params.per_page, "Listing checklists");

    let offset = (params.page.saturating_sub(1)) * params.per_page;
    
    match state.client.get_all_checklists(params.per_page, offset, params.include_archived).await {
        Ok(checklists) => {
            // Get total count for pagination
            let total = state.client.count_checklists(params.include_archived).await.unwrap_or(checklists.len());
            let total_pages = (total + params.per_page - 1) / params.per_page.max(1);

            // Get items for each checklist
            let mut responses = Vec::new();
            for checklist in checklists {
                let items = state.client.get_items(checklist.id).await.unwrap_or_default();
                responses.push(ChecklistResponse::from_checklist_with_items(checklist, items));
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
                Json(ErrorResponse::new("BACKEND_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/checklists/:id
pub async fn get_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChecklistResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(id = %id, "Getting checklist");

    match state.client.get_checklist(id).await {
        Ok(Some(checklist)) => {
            let items = state.client.get_items(id).await.unwrap_or_default();
            Ok(Json(ChecklistResponse::from_checklist_with_items(checklist, items)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Checklist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to get checklist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("BACKEND_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/checklists
pub async fn create_checklist(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateChecklist>,
) -> Result<(StatusCode, Json<ChecklistResponse>), (StatusCode, Json<ErrorResponse>)> {
    debug!(title = %request.title, "Creating new checklist");

    let items_to_create = request.items.clone();

    match state.client.create_checklist(request).await {
        Ok(checklist) => {
            // Add initial items
            let mut created_items = Vec::new();
            for item_data in items_to_create {
                if let Ok(item) = state.client.add_item(checklist.id, item_data).await {
                    created_items.push(item);
                }
            }

            Ok((StatusCode::CREATED, Json(ChecklistResponse::from_checklist_with_items(checklist, created_items))))
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
) -> Result<Json<ChecklistResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(id = %id, "Updating checklist");

    match state.client.update_checklist(id, request).await {
        Ok(Some(checklist)) => {
            let items = state.client.get_items(id).await.unwrap_or_default();
            Ok(Json(ChecklistResponse::from_checklist_with_items(checklist, items)))
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
    debug!(id = %id, "Deleting checklist");

    match state.client.delete_checklist(id).await {
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
) -> Result<(StatusCode, Json<ChecklistItem>), (StatusCode, Json<ErrorResponse>)> {
    debug!(checklist_id = %checklist_id, "Adding item to checklist");

    // Verify checklist exists
    match state.client.get_checklist(checklist_id).await {
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
                Json(ErrorResponse::new("BACKEND_ERROR", e.to_string())),
            ));
        }
    }

    match state.client.add_item(checklist_id, request).await {
        Ok(item) => Ok((StatusCode::CREATED, Json(item))),
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
) -> Result<Json<ChecklistItem>, (StatusCode, Json<ErrorResponse>)> {
    debug!(item_id = %item_id, "Updating item");

    match state.client.update_item(item_id, request).await {
        Ok(Some(item)) => Ok(Json(item)),
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
) -> Result<Json<ChecklistItem>, (StatusCode, Json<ErrorResponse>)> {
    debug!(item_id = %item_id, "Toggling item");

    match state.client.toggle_item(item_id).await {
        Ok(Some(item)) => Ok(Json(item)),
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
    debug!(item_id = %item_id, "Deleting item");

    match state.client.delete_item(item_id).await {
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
) -> Result<(StatusCode, Json<ChecklistResponse>), (StatusCode, Json<ErrorResponse>)> {
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

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse a markdown checklist into title and items
fn parse_markdown_checklist(markdown: &str) -> (String, Vec<CreateChecklistItem>) {
    let mut title = String::from("Imported Checklist");
    let mut items = Vec::new();
    let mut order = 0;

    for line in markdown.lines() {
        let trimmed = line.trim();
        
        // Check for title (# header)
        if trimmed.starts_with("# ") {
            title = trimmed[2..].trim().to_string();
            continue;
        }

        // Check for checkbox items: - [ ] or - [x] or * [ ] or * [x]
        let checkbox_patterns = [
            ("- [ ] ", false),
            ("- [x] ", true),
            ("- [X] ", true),
            ("* [ ] ", false),
            ("* [x] ", true),
            ("* [X] ", true),
        ];

        for (pattern, _is_completed) in checkbox_patterns {
            if trimmed.starts_with(pattern) {
                let content = trimmed[pattern.len()..].trim().to_string();
                if !content.is_empty() {
                    items.push(CreateChecklistItem {
                        content,
                        order: Some(order),
                    });
                    order += 1;
                }
                break;
            }
        }
    }

    (title, items)
}
