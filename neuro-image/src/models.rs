use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Image Source Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageSource {
    Generated,  // AI-generated
    Uploaded,   // User uploaded
    External,   // From URL
}

impl Default for ImageSource {
    fn default() -> Self {
        Self::Generated
    }
}

// =============================================================================
// Image Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    pub name: String,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub source: ImageSource,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub base64_data: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub size_bytes: Option<u64>,
    pub album_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub model: Option<String>,
    pub seed: Option<i64>,
    pub steps: Option<u32>,
    pub cfg_scale: Option<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateImageRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub steps: Option<u32>,
    pub cfg_scale: Option<f32>,
    pub seed: Option<i64>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateImageRequest {
    pub name: String,
    pub url: Option<String>,
    pub base64_data: Option<String>,
    pub album_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateImageRequest {
    pub name: Option<String>,
    pub album_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub favorite: Option<bool>,
}

// =============================================================================
// Album Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cover_image_id: Option<Uuid>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlbumRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlbumRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_image_id: Option<Uuid>,
    pub color: Option<String>,
}

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ImagesQuery {
    pub album_id: Option<Uuid>,
    pub source: Option<String>,
    pub tag: Option<String>,
    pub favorite: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// =============================================================================
// Image State (In-Memory Storage)
// =============================================================================

#[derive(Debug, Default)]
pub struct ImageState {
    pub images: HashMap<Uuid, Image>,
    pub albums: HashMap<Uuid, Album>,
}

impl ImageState {
    pub fn new() -> Self {
        let mut state = Self::default();
        
        // Create default album
        let default_album = Album {
            id: Uuid::new_v4(),
            name: "All Images".to_string(),
            description: Some("All generated and uploaded images".to_string()),
            cover_image_id: None,
            color: Some("#6366f1".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        state.albums.insert(default_album.id, default_album);
        
        state
    }
}

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ImagesResponse {
    pub images: Vec<Image>,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct AlbumsResponse {
    pub albums: Vec<AlbumWithCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlbumWithCount {
    #[serde(flatten)]
    pub album: Album,
    pub image_count: usize,
}

#[derive(Debug, Serialize)]
pub struct GenerationStatus {
    pub status: String,
    pub progress: Option<f32>,
    pub image: Option<Image>,
    pub error: Option<String>,
}

// =============================================================================
// Predefined Styles
// =============================================================================

pub const IMAGE_STYLES: &[(&str, &str)] = &[
    ("realistic", "photorealistic, 8k, detailed"),
    ("anime", "anime style, vibrant colors, detailed"),
    ("oil_painting", "oil painting, classical, artistic"),
    ("watercolor", "watercolor painting, soft, artistic"),
    ("digital_art", "digital art, concept art, detailed"),
    ("sketch", "pencil sketch, black and white, detailed"),
    ("cyberpunk", "cyberpunk, neon, futuristic, detailed"),
    ("fantasy", "fantasy art, magical, detailed"),
];

pub const DEFAULT_NEGATIVE_PROMPT: &str = "blurry, low quality, distorted, deformed, ugly, bad anatomy";
