//! =============================================================================
//! Music Handlers
//! =============================================================================

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::domain::entities::music::{
    CreatePlaylist, CreateSong, EqualizerSettings, ListeningEntry, Playlist,
    PlaylistWithSongs, Song, UpdatePlaylist, UpdateSong, YouTubeMetadata,
};
use crate::infrastructure::api::dto::ErrorResponse;
use crate::AppState;

// =============================================================================
// Playlist Handlers
// =============================================================================

/// GET /api/data/playlists
#[instrument(skip(state))]
pub async fn list_playlists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Playlist>>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_all_playlists().await {
        Ok(playlists) => Ok(Json(playlists)),
        Err(e) => {
            error!(error = %e, "Failed to list playlists");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/data/playlists/:id
#[instrument(skip(state))]
pub async fn get_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlaylistWithSongs>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_playlist_with_songs(id).await {
        Ok(Some(playlist)) => Ok(Json(playlist)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Playlist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to get playlist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/playlists
#[instrument(skip(state, data))]
pub async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreatePlaylist>,
) -> Result<(StatusCode, Json<Playlist>), (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.create_playlist(data).await {
        Ok(playlist) => Ok((StatusCode::CREATED, Json(playlist))),
        Err(e) => {
            error!(error = %e, "Failed to create playlist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/data/playlists/:id
#[instrument(skip(state, data))]
pub async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePlaylist>,
) -> Result<Json<Playlist>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.update_playlist(id, data).await {
        Ok(Some(playlist)) => Ok(Json(playlist)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Playlist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update playlist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/data/playlists/:id
#[instrument(skip(state))]
pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.delete_playlist(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Playlist not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete playlist");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// Song Handlers
// =============================================================================

/// GET /api/data/playlists/:id/songs
#[instrument(skip(state))]
pub async fn list_playlist_songs(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
) -> Result<Json<Vec<Song>>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_songs_by_playlist(playlist_id).await {
        Ok(songs) => Ok(Json(songs)),
        Err(e) => {
            error!(error = %e, "Failed to list songs");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/data/songs/:id
#[instrument(skip(state))]
pub async fn get_song(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Song>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_song(id).await {
        Ok(Some(song)) => Ok(Json(song)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Song not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to get song");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// Query for finding song by YouTube ID
#[derive(Debug, Deserialize)]
pub struct FindSongByYoutubeQuery {
    pub youtube_id: String,
    pub playlist_id: Uuid,
}

/// GET /api/data/songs/by-youtube-id?youtube_id=...&playlist_id=...
#[instrument(skip(state))]
pub async fn get_song_by_youtube_id(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FindSongByYoutubeQuery>,
) -> Result<Json<Option<Song>>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_song_by_youtube_id(&query.youtube_id, query.playlist_id).await {
        Ok(song) => Ok(Json(song)),
        Err(e) => {
            error!(error = %e, "Failed to find song by YouTube ID");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// Request to create a song with metadata
#[derive(Debug, Deserialize)]
pub struct CreateSongRequest {
    #[serde(flatten)]
    pub song: CreateSong,
    // Accept flexible metadata shape (some clients send `id` or full `yt-dlp` output)
    pub metadata: serde_json::Value,
}

/// POST /api/data/playlists/:id/songs
#[instrument(skip(state, body))]
pub async fn create_song(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    body: Bytes,
) -> Result<(StatusCode, Json<Song>), (StatusCode, Json<ErrorResponse>)> {
    // Log raw body for debugging
    let body_str = String::from_utf8_lossy(&body);
    debug!(body = %body_str, "Received create song request");
    
    // Parse the JSON manually into a flexible structure
    let data: CreateSongRequest = serde_json::from_slice(&body)
        .map_err(|e| {
            error!(error = %e, body = %body_str, "Failed to parse create song request");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse::new("PARSE_ERROR", e.to_string())),
            )
        })?;

    // Normalize metadata: accept either { "youtube_id": "..." } or { "id": "..." },
    // also accept youtube_url and strip noisy fields like `description`.
    let mut meta_value = data.metadata.clone();

    // Use helper to normalize/sanitize metadata
    meta_value = normalize_metadata(meta_value);

    // helper: normalize & sanitize metadata (extracted below)


    // helper function: normalize & sanitize metadata
    fn normalize_metadata(mut meta: serde_json::Value) -> serde_json::Value {
        // 1) Map `id` -> `youtube_id` if missing
        if meta.get("youtube_id").is_none() {
            if let Some(id) = meta.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                if let Some(obj) = meta.as_object_mut() {
                    obj.insert("youtube_id".to_string(), serde_json::Value::String(id));
                }
            }
        }

        // 2) Extract from youtube_url if still missing
        if meta.get("youtube_id").is_none() {
            if let Some(url) = meta.get("youtube_url").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                if let Some(id) = extract_youtube_id(&url) {
                    if let Some(obj) = meta.as_object_mut() {
                        obj.insert("youtube_id".to_string(), serde_json::Value::String(id));
                    }
                }
            }
        }

        // 3) Remove noisy fields
        if let Some(obj) = meta.as_object_mut() {
            // Fields considered noisy / unnecessary
            for key in [
                "description",
                "uploader_url",
                "uploader",
                "uploader_id",
                "webpage_url",
                "extractor",
                "extractor_key",
                "upload_date",
                "categories",
                "tags",
            ] {
                obj.remove(key);
            }
        }

        meta
    }

    // Try to deserialize into the expected type
    let metadata: YouTubeMetadata = serde_json::from_value(meta_value.clone()).map_err(|e| {
        error!(error = %e, metadata = %meta_value, "Invalid metadata format after normalization");
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse::new("INVALID_METADATA", e.to_string())),
        )
    })?;

    // Ensure youtube_id is present
    if metadata.youtube_id.is_empty() {
        error!("Missing youtube_id in metadata after normalization");
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse::new("MISSING_FIELD", "metadata.youtube_id is required"))));
    }

    match state.music_repository.create_song(playlist_id, data.song, metadata).await {
        Ok(song) => Ok((StatusCode::CREATED, Json(song))),
        Err(e) => {
            error!(error = %e, "Failed to create song");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/data/songs/:id
#[instrument(skip(state, data))]
pub async fn update_song(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdateSong>,
) -> Result<Json<Song>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.update_song(id, data).await {
        Ok(Some(song)) => Ok(Json(song)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Song not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to update song");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/data/songs/:id
#[instrument(skip(state))]
pub async fn delete_song(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.delete_song(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", "Song not found")),
        )),
        Err(e) => {
            error!(error = %e, "Failed to delete song");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DELETE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/songs/:id/play
#[instrument(skip(state))]
pub async fn increment_song_play_count(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.increment_play_count(id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!(error = %e, "Failed to increment play count");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// Request to reorder songs
#[derive(Debug, Deserialize)]
pub struct ReorderSongsRequest {
    pub song_ids: Vec<Uuid>,
}

/// POST /api/data/playlists/:id/reorder
#[instrument(skip(state, data))]
pub async fn reorder_songs(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(data): Json<ReorderSongsRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.reorder_songs(playlist_id, data.song_ids).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!(error = %e, "Failed to reorder songs");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("REORDER_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// History & Stats
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize { 50 }

/// GET /api/data/music/history
#[instrument(skip(state))]
pub async fn get_listening_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<Vec<ListeningEntry>>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_listening_history(params.limit).await {
        Ok(history) => Ok(Json(history)),
        Err(e) => {
            error!(error = %e, "Failed to get listening history");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/music/history
#[instrument(skip(state, entry))]
pub async fn add_listening_entry(
    State(state): State<Arc<AppState>>,
    Json(entry): Json<ListeningEntry>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.add_listening_entry(entry).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            error!(error = %e, "Failed to add listening entry");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("CREATE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/data/music/top-songs
#[instrument(skip(state))]
pub async fn get_most_played_songs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<Vec<Song>>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_most_played_songs(params.limit).await {
        Ok(songs) => Ok(Json(songs)),
        Err(e) => {
            error!(error = %e, "Failed to get most played songs");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

// =============================================================================
// Equalizer
// =============================================================================

/// GET /api/data/music/equalizer
#[instrument(skip(state))]
pub async fn get_equalizer_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<EqualizerSettings>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_equalizer_settings().await {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => {
            error!(error = %e, "Failed to get equalizer settings");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// PUT /api/data/music/equalizer
#[instrument(skip(state, settings))]
pub async fn save_equalizer_settings(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<EqualizerSettings>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.save_equalizer_settings(settings).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!(error = %e, "Failed to save equalizer settings");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("SAVE_ERROR", e.to_string())),
            ))
        }
    }
}

/// GET /api/data/songs/liked
#[instrument(skip(state))]
pub async fn get_liked_songs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Song>>, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.get_liked_songs().await {
        Ok(songs) => Ok(Json(songs)),
        Err(e) => {
            error!(error = %e, "Failed to get liked songs");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("DATABASE_ERROR", e.to_string())),
            ))
        }
    }
}

/// POST /api/data/playlists/:id/suggestions-timestamp
#[instrument(skip(state))]
pub async fn update_suggestions_timestamp(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.update_suggestions_timestamp(id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            error!(error = %e, "Failed to update suggestions timestamp");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("UPDATE_ERROR", e.to_string())),
            ))
        }
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

/// Extract the YouTube video id from common URL formats:
/// - https://www.youtube.com/watch?v=ID
/// - https://youtu.be/ID
fn extract_youtube_id(url: &str) -> Option<String> {
    // Look for v= parameter
    if let Some(pos) = url.find("v=") {
        let tail = &url[pos + 2..];
        let id: String = tail
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !id.is_empty() {
            return Some(id);
        }
    }

    // Look for youtu.be short link
    if let Some(pos) = url.find("youtu.be/") {
        let tail = &url[pos + "youtu.be/".len()..];
        let id: String = tail
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !id.is_empty() {
            return Some(id);
        }
    }

    None
}

// ======================
// Unit tests
// ======================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_id_maps_to_youtube_id_and_removes_description() {
        let input = json!({
            "id": "XYZ123",
            "title": "Foo",
            "description": "some noisy description",
            "uploader_url": "http://example.com/uploader"
        });

        let out = normalize_metadata(input);
        assert_eq!(out.get("youtube_id").and_then(|v| v.as_str()), Some("XYZ123"));
        assert!(out.get("description").is_none());
        assert!(out.get("uploader_url").is_none());
    }

    #[test]
    fn test_extract_from_youtube_url() {
        let input = json!({
            "youtube_url": "https://www.youtube.com/watch?v=ABC987",
            "title": "Bar"
        });

        let out = normalize_metadata(input);
        assert_eq!(out.get("youtube_id").and_then(|v| v.as_str()), Some("ABC987"));
    }

    #[test]
    fn test_remove_multiple_noisy_fields() {
        let input = json!({
            "id": "ID1",
            "description": "x",
            "uploader": "user",
            "uploader_id": "u1",
            "webpage_url": "http://...",
            "extractor": "yt-dlp",
            "tags": ["a","b"],
            "categories": ["c"]
        });

        let out = normalize_metadata(input);
        assert_eq!(out.get("youtube_id").and_then(|v| v.as_str()), Some("ID1"));
        for f in &["description","uploader","uploader_id","webpage_url","extractor","tags","categories"] {
            assert!(out.get(*f).is_none(), "{} should be removed", f);
        }
    }
}
