//! =============================================================================
//! Checklist Handlers
//! =============================================================================

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::entities::checklist::{
    Checklist, ChecklistItem, ChecklistWithItems, CreateChecklist, CreateChecklistItem,
    PaginatedChecklists, UpdateChecklist, UpdateChecklistItem,
};
use crate::infrastructure::api::dto::ErrorResponse;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListChecklistsParams {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    #[serde(default)]
    pub include_archived: bool,
}

fn default_page() -> usize { 1 }
fn default_per_page() -> usize { 20 }

/// GET /api/data/checklists
#[instrument(skip(state))]
pub async fn list_checklists(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListChecklistsParams>,
) -> Result<Json<PaginatedChecklists>, (StatusCode, Json<ErrorResponse>)> {
    let offset = (params.page.saturating_sub(1)) * params.per_page;

    match state.checklist_repository.get_all_checklists(
        params.per_page,
        offset,
        params.include_archived,
    ).await {
        Ok(checklists) => {
            let total = state.checklist_repository
                .count_checklists(params.include_archived)
                .await
                .unwrap_or(0);
            let total_pages = (total + params.per_page - 1) / params.per_page;

            Ok(Json(PaginatedChecklists {
                data: checklists,
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

/// GET /api/data/checklists/:id
#[instrument(skip(state))]
pub async fn get_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ChecklistWithItems>, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.get_checklist(id).await {
        Ok(Some(checklist)) => {
            let items = state.checklist_repository.get_items(id).await.unwrap_or_default();
            Ok(Json(ChecklistWithItems { checklist, items }))
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

/// POST /api/data/checklists
#[instrument(skip(state, data))]
pub async fn create_checklist(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreateChecklist>,
) -> Result<(StatusCode, Json<Checklist>), (StatusCode, Json<ErrorResponse>)> {
    // Clone items before moving data
    let items = data.items.clone();
    
    match state.checklist_repository.create_checklist(data).await {
        Ok(checklist) => {
            // Create items if provided
            for item_data in items {
                let _ = state.checklist_repository.add_item(checklist.id, item_data).await;
            }
            Ok((StatusCode::CREATED, Json(checklist)))
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

/// PATCH /api/data/checklists/:id
#[instrument(skip(state, data))]
pub async fn update_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdateChecklist>,
) -> Result<Json<Checklist>, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.update_checklist(id, data).await {
        Ok(Some(checklist)) => Ok(Json(checklist)),
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

/// DELETE /api/data/checklists/:id
#[instrument(skip(state))]
pub async fn delete_checklist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.delete_checklist(id).await {
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
// Checklist Items
// =============================================================================

/// GET /api/data/checklists/:id/items
#[instrument(skip(state))]
pub async fn list_checklist_items(
    State(state): State<Arc<AppState>>,
    Path(checklist_id): Path<Uuid>,
) -> Result<Json<Vec<ChecklistItem>>, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.get_items(checklist_id).await {
        Ok(items) => Ok(Json(items)),
        Err(e) => {
            error!(error = %e, "Failed to get checklist items");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/checklists/:id/items
#[instrument(skip(state, data))]
pub async fn create_checklist_item(
    State(state): State<Arc<AppState>>,
    Path(checklist_id): Path<Uuid>,
    Json(data): Json<CreateChecklistItem>,
) -> Result<(StatusCode, Json<ChecklistItem>), (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.add_item(checklist_id, data).await {
        Ok(item) => Ok((StatusCode::CREATED, Json(item))),
        Err(e) => {
            error!(error = %e, "Failed to create checklist item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/data/checklists/items/:id
#[instrument(skip(state, data))]
pub async fn update_checklist_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<Uuid>,
    Json(data): Json<UpdateChecklistItem>,
) -> Result<Json<ChecklistItem>, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.update_item(item_id, data).await {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Item not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update checklist item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/checklists/items/:id/toggle
#[instrument(skip(state))]
pub async fn toggle_checklist_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<ChecklistItem>, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.toggle_item(item_id).await {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Item not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to toggle checklist item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("TOGGLE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/data/checklists/items/:id
#[instrument(skip(state))]
pub async fn delete_checklist_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.checklist_repository.delete_item(item_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Item not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete checklist item");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}
