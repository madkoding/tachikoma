use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Document Handlers
// =============================================================================

pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DocsQuery>,
) -> Result<Json<DocumentsResponse>, StatusCode> {
    debug!("Listing documents with query: {:?}", query);
    
    let docs_state = state.docs_state.read().await;
    
    let mut documents: Vec<Document> = docs_state.documents.values().cloned().collect();
    
    // Filter by folder
    if let Some(folder_id) = query.folder_id {
        documents.retain(|d| d.folder_id == Some(folder_id));
    }
    
    // Filter by doc_type
    if let Some(doc_type) = query.doc_type {
        documents.retain(|d| {
            let type_str = serde_json::to_string(&d.doc_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            type_str == doc_type
        });
    }
    
    // Filter by tag
    if let Some(tag) = query.tag {
        documents.retain(|d| d.tags.contains(&tag));
    }
    
    // Filter by starred
    if let Some(starred) = query.starred {
        documents.retain(|d| d.starred == starred);
    }
    
    // Sort by last_accessed_at, starred first
    documents.sort_by(|a, b| {
        match (a.starred, b.starred) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.last_accessed_at.cmp(&a.last_accessed_at),
        }
    });
    
    let total = documents.len();
    
    Ok(Json(DocumentsResponse { documents, total }))
}

pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Document>, StatusCode> {
    debug!("Getting document: {}", id);
    
    let mut docs_state = state.docs_state.write().await;
    
    if let Some(doc) = docs_state.documents.get_mut(&id) {
        doc.last_accessed_at = Utc::now();
        Ok(Json(doc.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn create_document(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, StatusCode> {
    debug!("Creating document: {}", request.name);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let content = request.content.clone().unwrap_or_default();
    let doc_type = request.doc_type.unwrap_or_else(|| {
        detect_doc_type(&request.name, request.mime_type.as_deref())
    });
    
    let document = Document {
        id,
        name: request.name,
        doc_type,
        content: content.clone(),
        folder_id: request.folder_id,
        tags: request.tags.unwrap_or_default(),
        size_bytes: content.len() as u64,
        mime_type: request.mime_type,
        starred: false,
        shared: false,
        created_at: now,
        updated_at: now,
        last_accessed_at: now,
    };
    
    let mut docs_state = state.docs_state.write().await;
    docs_state.documents.insert(id, document.clone());
    
    Ok(Json(document))
}

pub async fn update_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateDocumentRequest>,
) -> Result<Json<Document>, StatusCode> {
    debug!("Updating document: {}", id);
    
    let mut docs_state = state.docs_state.write().await;
    
    let doc = docs_state
        .documents
        .get_mut(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = request.name {
        doc.name = name;
    }
    if let Some(content) = request.content {
        doc.size_bytes = content.len() as u64;
        doc.content = content;
    }
    if let Some(folder_id) = request.folder_id {
        doc.folder_id = Some(folder_id);
    }
    if let Some(tags) = request.tags {
        doc.tags = tags;
    }
    if let Some(starred) = request.starred {
        doc.starred = starred;
    }
    if let Some(shared) = request.shared {
        doc.shared = shared;
    }
    
    doc.updated_at = Utc::now();
    
    Ok(Json(doc.clone()))
}

pub async fn delete_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting document: {}", id);
    
    let mut docs_state = state.docs_state.write().await;
    
    if docs_state.documents.remove(&id).is_some() {
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
    
    let docs_state = state.docs_state.read().await;
    
    let mut folders: Vec<DocFolder> = docs_state.folders.values().cloned().collect();
    folders.sort_by(|a, b| a.name.cmp(&b.name));
    
    let total = folders.len();
    
    Ok(Json(FoldersResponse { folders, total }))
}

pub async fn get_folder_contents(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<FolderContents>, StatusCode> {
    debug!("Getting folder contents: {}", id);
    
    let docs_state = state.docs_state.read().await;
    
    let folder = docs_state
        .folders
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let subfolders: Vec<DocFolder> = docs_state
        .folders
        .values()
        .filter(|f| f.parent_id == Some(id))
        .cloned()
        .collect();
    
    let documents: Vec<Document> = docs_state
        .documents
        .values()
        .filter(|d| d.folder_id == Some(id))
        .cloned()
        .collect();
    
    Ok(Json(FolderContents {
        folder,
        subfolders,
        documents,
    }))
}

pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateFolderRequest>,
) -> Result<Json<DocFolder>, StatusCode> {
    debug!("Creating folder: {}", request.name);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let folder = DocFolder {
        id,
        name: request.name,
        parent_id: request.parent_id,
        color: request.color,
        created_at: now,
        updated_at: now,
    };
    
    let mut docs_state = state.docs_state.write().await;
    docs_state.folders.insert(id, folder.clone());
    
    Ok(Json(folder))
}

pub async fn update_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateFolderRequest>,
) -> Result<Json<DocFolder>, StatusCode> {
    debug!("Updating folder: {}", id);
    
    let mut docs_state = state.docs_state.write().await;
    
    let folder = docs_state
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
    
    let mut docs_state = state.docs_state.write().await;
    
    // Move documents and subfolders to parent or root
    for doc in docs_state.documents.values_mut() {
        if doc.folder_id == Some(id) {
            doc.folder_id = None;
        }
    }
    
    for folder in docs_state.folders.values_mut() {
        if folder.parent_id == Some(id) {
            folder.parent_id = None;
        }
    }
    
    if docs_state.folders.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Search Handler
// =============================================================================

pub async fn search_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<DocumentsResponse>, StatusCode> {
    debug!("Searching documents: {}", query.q);
    
    let docs_state = state.docs_state.read().await;
    let search_lower = query.q.to_lowercase();
    
    let mut documents: Vec<Document> = docs_state
        .documents
        .values()
        .filter(|d| {
            d.name.to_lowercase().contains(&search_lower) ||
            d.content.to_lowercase().contains(&search_lower) ||
            d.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
        })
        .cloned()
        .collect();
    
    documents.sort_by(|a, b| b.last_accessed_at.cmp(&a.last_accessed_at));
    
    let total = documents.len();
    
    Ok(Json(DocumentsResponse { documents, total }))
}

// =============================================================================
// Storage Stats
// =============================================================================

pub async fn get_storage_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StorageStats>, StatusCode> {
    debug!("Getting storage stats");
    
    let docs_state = state.docs_state.read().await;
    
    let mut by_type: HashMap<String, usize> = HashMap::new();
    let mut total_size_bytes: u64 = 0;
    
    for doc in docs_state.documents.values() {
        let type_str = serde_json::to_string(&doc.doc_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        *by_type.entry(type_str).or_insert(0) += 1;
        total_size_bytes += doc.size_bytes;
    }
    
    Ok(Json(StorageStats {
        total_documents: docs_state.documents.len(),
        total_folders: docs_state.folders.len(),
        total_size_bytes,
        by_type,
    }))
}

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "neuro-docs",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
