use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Document Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentType {
    Text,
    Markdown,
    Code,
    Spreadsheet,
    Presentation,
    Pdf,
    Other,
}

impl Default for DocumentType {
    fn default() -> Self {
        Self::Text
    }
}

// =============================================================================
// Document Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub doc_type: DocumentType,
    pub content: String,
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub starred: bool,
    pub shared: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub doc_type: Option<DocumentType>,
    pub content: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub starred: Option<bool>,
    pub shared: Option<bool>,
}

// =============================================================================
// Folder Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFolder {
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
pub struct DocsQuery {
    pub folder_id: Option<Uuid>,
    pub doc_type: Option<String>,
    pub tag: Option<String>,
    pub starred: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

// =============================================================================
// Docs State (In-Memory Storage)
// =============================================================================

#[derive(Debug, Default)]
pub struct DocsState {
    pub documents: HashMap<Uuid, Document>,
    pub folders: HashMap<Uuid, DocFolder>,
}

impl DocsState {
    pub fn new() -> Self {
        let mut state = Self::default();
        
        // Create default folder
        let default_folder = DocFolder {
            id: Uuid::new_v4(),
            name: "My Documents".to_string(),
            parent_id: None,
            color: Some("#3b82f6".to_string()),
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
pub struct DocumentsResponse {
    pub documents: Vec<Document>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct FoldersResponse {
    pub folders: Vec<DocFolder>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct FolderContents {
    pub folder: DocFolder,
    pub subfolders: Vec<DocFolder>,
    pub documents: Vec<Document>,
}

#[derive(Debug, Serialize)]
pub struct StorageStats {
    pub total_documents: usize,
    pub total_folders: usize,
    pub total_size_bytes: u64,
    pub by_type: HashMap<String, usize>,
}

// =============================================================================
// File Type Detection
// =============================================================================

pub fn detect_doc_type(filename: &str, mime_type: Option<&str>) -> DocumentType {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    
    match ext.as_str() {
        "md" | "markdown" => DocumentType::Markdown,
        "txt" => DocumentType::Text,
        "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "hpp" 
        | "css" | "html" | "json" | "yaml" | "yml" | "toml" | "xml" | "sql" => DocumentType::Code,
        "xlsx" | "xls" | "csv" => DocumentType::Spreadsheet,
        "pptx" | "ppt" => DocumentType::Presentation,
        "pdf" => DocumentType::Pdf,
        _ => {
            if let Some(mime) = mime_type {
                if mime.starts_with("text/") {
                    return DocumentType::Text;
                }
            }
            DocumentType::Other
        }
    }
}
