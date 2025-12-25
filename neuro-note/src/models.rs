use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Note Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub pinned: bool,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub color: Option<String>,
    pub pinned: Option<bool>,
    pub archived: Option<bool>,
}

// =============================================================================
// Folder Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
}

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct NotesQuery {
    pub folder_id: Option<Uuid>,
    pub tag: Option<String>,
    pub archived: Option<bool>,
    pub pinned: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

// =============================================================================
// Notes State (In-Memory Storage)
// =============================================================================

#[derive(Debug, Default)]
pub struct NotesState {
    pub notes: HashMap<Uuid, Note>,
    pub folders: HashMap<Uuid, Folder>,
}

impl NotesState {
    pub fn new() -> Self {
        let mut state = Self::default();
        
        // Create default folder
        let default_folder = Folder {
            id: Uuid::new_v4(),
            name: "My Notes".to_string(),
            parent_id: None,
            color: Some("#6366f1".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        state.folders.insert(default_folder.id, default_folder);
        
        state
    }
}

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Serialize)]
pub struct NotesResponse {
    pub notes: Vec<Note>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct FolderWithNotes {
    pub folder: Folder,
    pub notes: Vec<Note>,
    pub note_count: usize,
}

// =============================================================================
// Predefined Colors
// =============================================================================

pub const NOTE_COLORS: &[(&str, &str)] = &[
    ("default", "#ffffff"),
    ("yellow", "#fef08a"),
    ("green", "#bbf7d0"),
    ("blue", "#bfdbfe"),
    ("purple", "#ddd6fe"),
    ("pink", "#fbcfe8"),
    ("orange", "#fed7aa"),
    ("red", "#fecaca"),
];
