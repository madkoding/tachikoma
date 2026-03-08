use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Image Handlers
// =============================================================================

pub async fn list_images(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ImagesQuery>,
) -> Result<Json<ImagesResponse>, StatusCode> {
    debug!("Listing images with query: {:?}", query);
    
    let image_state = state.image_state.read().await;
    
    let mut images: Vec<Image> = image_state.images.values().cloned().collect();
    
    // Filter by album
    if let Some(album_id) = query.album_id {
        images.retain(|i| i.album_id == Some(album_id));
    }
    
    // Filter by source
    if let Some(source) = query.source {
        images.retain(|i| {
            let source_str = serde_json::to_string(&i.source)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            source_str == source
        });
    }
    
    // Filter by tag
    if let Some(tag) = query.tag {
        images.retain(|i| i.tags.contains(&tag));
    }
    
    // Filter by favorite
    if let Some(favorite) = query.favorite {
        images.retain(|i| i.favorite == favorite);
    }
    
    // Sort by created_at desc, favorites first
    images.sort_by(|a, b| {
        match (a.favorite, b.favorite) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.created_at.cmp(&a.created_at),
        }
    });
    
    let total = images.len();
    
    // Pagination
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let has_more = offset + limit < total;
    
    let images: Vec<Image> = images.into_iter().skip(offset).take(limit).collect();
    
    Ok(Json(ImagesResponse {
        images,
        total,
        has_more,
    }))
}

pub async fn get_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Image>, StatusCode> {
    debug!("Getting image: {}", id);
    
    let image_state = state.image_state.read().await;
    
    image_state
        .images
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_image(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateImageRequest>,
) -> Result<Json<Image>, StatusCode> {
    debug!("Creating image: {}", request.name);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let source = if request.base64_data.is_some() {
        ImageSource::Uploaded
    } else if request.url.is_some() {
        ImageSource::External
    } else {
        ImageSource::Uploaded
    };
    
    let image = Image {
        id,
        name: request.name,
        prompt: None,
        negative_prompt: None,
        source,
        url: request.url,
        thumbnail_url: None,
        base64_data: request.base64_data,
        width: None,
        height: None,
        format: None,
        size_bytes: None,
        album_id: request.album_id,
        tags: request.tags.unwrap_or_default(),
        favorite: false,
        model: None,
        seed: None,
        steps: None,
        cfg_scale: None,
        created_at: now,
    };
    
    let mut image_state = state.image_state.write().await;
    image_state.images.insert(id, image.clone());
    
    Ok(Json(image))
}

pub async fn update_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateImageRequest>,
) -> Result<Json<Image>, StatusCode> {
    debug!("Updating image: {}", id);
    
    let mut image_state = state.image_state.write().await;
    
    let image = image_state
        .images
        .get_mut(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = request.name {
        image.name = name;
    }
    if let Some(album_id) = request.album_id {
        image.album_id = Some(album_id);
    }
    if let Some(tags) = request.tags {
        image.tags = tags;
    }
    if let Some(favorite) = request.favorite {
        image.favorite = favorite;
    }
    
    Ok(Json(image.clone()))
}

pub async fn delete_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting image: {}", id);
    
    let mut image_state = state.image_state.write().await;
    
    if image_state.images.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Image Generation (Placeholder)
// =============================================================================

pub async fn generate_image(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateImageRequest>,
) -> Result<Json<GenerationStatus>, StatusCode> {
    info!("Generate image request: {}", request.prompt);
    
    // This is a placeholder - actual implementation would connect to
    // Stable Diffusion, DALL-E, or other image generation services
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    // Create placeholder image entry
    let image = Image {
        id,
        name: format!("Generated {}", now.format("%Y-%m-%d %H:%M")),
        prompt: Some(request.prompt.clone()),
        negative_prompt: request.negative_prompt,
        source: ImageSource::Generated,
        url: None,
        thumbnail_url: None,
        base64_data: None, // Would be filled by actual generation
        width: request.width.or(Some(512)),
        height: request.height.or(Some(512)),
        format: Some("png".to_string()),
        size_bytes: None,
        album_id: None,
        tags: vec!["generated".to_string()],
        favorite: false,
        model: request.model.clone(),
        seed: request.seed,
        steps: request.steps.or(Some(20)),
        cfg_scale: request.cfg_scale.or(Some(7.0)),
        created_at: now,
    };
    
    let mut image_state = state.image_state.write().await;
    image_state.images.insert(id, image.clone());
    
    // Return status - in real implementation this would be async
    Ok(Json(GenerationStatus {
        status: "pending".to_string(),
        progress: Some(0.0),
        image: Some(image),
        error: None,
    }))
}

// =============================================================================
// Album Handlers
// =============================================================================

pub async fn list_albums(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AlbumsResponse>, StatusCode> {
    debug!("Listing albums");
    
    let image_state = state.image_state.read().await;
    
    let mut albums: Vec<AlbumWithCount> = image_state
        .albums
        .values()
        .map(|album| {
            let image_count = image_state
                .images
                .values()
                .filter(|i| i.album_id == Some(album.id))
                .count();
            AlbumWithCount {
                album: album.clone(),
                image_count,
            }
        })
        .collect();
    
    albums.sort_by(|a, b| a.album.name.cmp(&b.album.name));
    
    Ok(Json(AlbumsResponse { albums }))
}

pub async fn get_album(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AlbumWithCount>, StatusCode> {
    debug!("Getting album: {}", id);
    
    let image_state = state.image_state.read().await;
    
    let album = image_state
        .albums
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let image_count = image_state
        .images
        .values()
        .filter(|i| i.album_id == Some(id))
        .count();
    
    Ok(Json(AlbumWithCount { album, image_count }))
}

pub async fn create_album(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateAlbumRequest>,
) -> Result<Json<Album>, StatusCode> {
    debug!("Creating album: {}", request.name);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let album = Album {
        id,
        name: request.name,
        description: request.description,
        cover_image_id: None,
        color: request.color,
        created_at: now,
        updated_at: now,
    };
    
    let mut image_state = state.image_state.write().await;
    image_state.albums.insert(id, album.clone());
    
    Ok(Json(album))
}

pub async fn update_album(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateAlbumRequest>,
) -> Result<Json<Album>, StatusCode> {
    debug!("Updating album: {}", id);
    
    let mut image_state = state.image_state.write().await;
    
    let album = image_state
        .albums
        .get_mut(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(name) = request.name {
        album.name = name;
    }
    if let Some(description) = request.description {
        album.description = Some(description);
    }
    if let Some(cover_image_id) = request.cover_image_id {
        album.cover_image_id = Some(cover_image_id);
    }
    if let Some(color) = request.color {
        album.color = Some(color);
    }
    
    album.updated_at = Utc::now();
    
    Ok(Json(album.clone()))
}

pub async fn delete_album(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting album: {}", id);
    
    let mut image_state = state.image_state.write().await;
    
    // Move images from this album to no album
    for image in image_state.images.values_mut() {
        if image.album_id == Some(id) {
            image.album_id = None;
        }
    }
    
    if image_state.albums.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Styles List
// =============================================================================

pub async fn list_styles() -> Json<Vec<serde_json::Value>> {
    Json(
        IMAGE_STYLES
            .iter()
            .map(|(name, prompt)| {
                serde_json::json!({
                    "name": name,
                    "prompt_addition": prompt
                })
            })
            .collect(),
    )
}

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "tachikoma-image",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
