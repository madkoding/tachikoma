use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Note Handlers
// =============================================================================

pub async fn list_notes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NotesQuery>,
) -> Result<Json<NotesResponse>, StatusCode> {
    debug!("Listing notes with query: {:?}", query);
    
    let notes_state = state.notes_state.read().await;
    
    let mut notes: Vec<Note> = notes_state.notes.values().cloned().collect();
    
    // Filter by folder
    if let Some(folder_id) = query.folder_id {
        notes.retain(|n| n.folder_id == Some(folder_id));
    }
    
    // Filter by tag
    if let Some(tag) = query.tag {
        notes.retain(|n| n.tags.contains(&tag));
    }
    
    // Filter by archived
    if let Some(archived) = query.archived {
        notes.retain(|n| n.archived == archived);
    } else {
        // By default, don't show archived
        notes.retain(|n| !n.archived);
    }
    
    // Filter by pinned
    if let Some(pinned) = query.pinned {
        notes.retain(|n| n.pinned == pinned);
    }
    
    // Sort: pinned first, then by updated_at
    notes.sort_by(|a, b| {
        match (a.pinned, b.pinned) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.updated_at.cmp(&a.updated_at),
        }
    });
    
    let total = notes.len();
    
    Ok(Json(NotesResponse { notes, total }))
}

pub async fn get_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Note>, StatusCode> {
    debug!("Getting note: {}", id);
    
    let notes_state = state.notes_state.read().await;
    
    notes_state
        .notes
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<Note>, StatusCode> {
    debug!("Creating note: {}", request.title);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let note = Note {
        id,
        title: request.title,
        content: request.content.unwrap_or_default(),
        folder_id: request.folder_id,
        tags: request.tags.unwrap_or_default(),
        color: request.color,
        pinned: false,
        archived: false,
        created_at: now,
        updated_at: now,
    };
    
    let mut notes_state = state.notes_state.write().await;
    notes_state.notes.insert(id, note.clone());
    
    Ok(Json(note))
}

pub async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateNoteRequest>,
) -> Result<Json<Note>, StatusCode> {
    debug!("Updating note: {}", id);
    
    let mut notes_state = state.notes_state.write().await;
    
    let note = notes_state
        .notes
        .get_mut(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(title) = request.title {
        note.title = title;
    }
    if let Some(content) = request.content {
        note.content = content;
    }
    if let Some(folder_id) = request.folder_id {
        note.folder_id = Some(folder_id);
    }
    if let Some(tags) = request.tags {
        note.tags = tags;
    }
    if let Some(color) = request.color {
        note.color = Some(color);
    }
    if let Some(pinned) = request.pinned {
        note.pinned = pinned;
    }
    if let Some(archived) = request.archived {
        note.archived = archived;
    }
    
    note.updated_at = Utc::now();
    
    Ok(Json(note.clone()))
}

pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting note: {}", id);
    
    let mut notes_state = state.notes_state.write().await;
    
    if notes_state.notes.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Folder Handlers
// =============================================================================

pub async fn list_folders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<FoldersResponse>, StatusCode> {
    debug!("Listing folders");
    
    let notes_state = state.notes_state.read().await;
    
    let mut folders: Vec<Folder> = notes_state.folders.values().cloned().collect();
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    
    let total = folders.len();
    
    Ok(Json(FoldersResponse { folders, total }))
}

pub async fn get_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<FolderWithNotes>, StatusCode> {
    debug!("Getting folder: {}", id);
    
    let notes_state = state.notes_state.read().await;
    
    let folder = notes_state
        .folders
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let notes: Vec<Note> = notes_state
        .notes
        .values()
        .filter(|n| n.folder_id == Some(id) && !n.archived)
        .cloned()
        .collect();
    
    let note_count = notes.len();
    
    Ok(Json(FolderWithNotes {
        folder,
        notes,
        note_count,
    }))
}

pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, StatusCode> {
    debug!("Creating folder: {}", request.name);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let folder = Folder {
        id,
        name: request.name,
        parent_id: request.parent_id,
        color: request.color,
        created_at: now,
        updated_at: now,
    };
    
    let mut notes_state = state.notes_state.write().await;
    notes_state.folders.insert(id, folder.clone());
    
    Ok(Json(folder))
}

pub async fn update_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateFolderRequest>,
) -> Result<Json<Folder>, StatusCode> {
    debug!("Updating folder: {}", id);
    
    let mut notes_state = state.notes_state.write().await;
    
    let folder = notes_state
        .folders
        .get_mut(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = request.name {
        folder.name = name;
    }
    if let Some(parent_id) = request.parent_id {
        folder.parent_id = Some(parent_id);
    }
    if let Some(color) = request.color {
        folder.color = Some(color);
    }
    
    folder.updated_at = Utc::now();
    
    Ok(Json(folder.clone()))
}

pub async fn delete_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting folder: {}", id);
    
    let mut notes_state = state.notes_state.write().await;
    
    // Move notes in this folder to no folder
    for note in notes_state.notes.values_mut() {
        if note.folder_id == Some(id) {
            note.folder_id = None;
        }
    }
    
    if notes_state.folders.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Search Handler
// =============================================================================

pub async fn search_notes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<NotesResponse>, StatusCode> {
    debug!("Searching notes: {}", query.q);
    
    let notes_state = state.notes_state.read().await;
    let search_lower = query.q.to_lowercase();
    
    let mut notes: Vec<Note> = notes_state
        .notes
        .values()
        .filter(|n| {
            !n.archived &&
            (n.title.to_lowercase().contains(&search_lower) ||
             n.content.to_lowercase().contains(&search_lower) ||
             n.tags.iter().any(|t| t.to_lowercase().contains(&search_lower)))
        })
        .cloned()
        .collect();
    
    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    
    let total = notes.len();
    
    Ok(Json(NotesResponse { notes, total }))
}

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "tachikoma-note",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
