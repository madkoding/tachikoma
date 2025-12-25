//! HTTP handlers for music API

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Command;
use tokio_util::io::ReaderStream;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::cover_art::{CoverArtService, ThumbnailQuality};
use crate::models::*;
use crate::youtube::YouTubeService;
use crate::AppState;

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "neuro-music",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// =============================================================================
// Playlist Handlers
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPlaylistsQuery {
    pub include_songs: Option<bool>,
}

pub async fn list_playlists(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListPlaylistsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let playlists = state.db.get_all_playlists().await
        .map_err(|e| {
            error!(error = %e, "Failed to get playlists");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if query.include_songs.unwrap_or(false) {
        let mut result = Vec::new();
        for playlist in playlists {
            let songs = state.db.get_songs_by_playlist(playlist.id).await
                .unwrap_or_default();
            result.push(serde_json::json!({
                "playlist": playlist,
                "songs": songs
            }));
        }
        Ok(Json(result))
    } else {
        let result: Vec<serde_json::Value> = playlists
            .into_iter()
            .map(|p| serde_json::to_value(p).unwrap())
            .collect();
        Ok(Json(result))
    }
}

pub async fn get_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlaylistWithSongs>, StatusCode> {
    let playlist = state.db.get_playlist_with_songs(id).await
        .map_err(|e| {
            error!(error = %e, "Failed to get playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(playlist))
}

pub async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreatePlaylist>,
) -> Result<(StatusCode, Json<Playlist>), StatusCode> {
    let playlist = state.db.create_playlist(data).await
        .map_err(|e| {
            error!(error = %e, "Failed to create playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(playlist)))
}

pub async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePlaylist>,
) -> Result<Json<Playlist>, StatusCode> {
    let playlist = state.db.update_playlist(id, data).await
        .map_err(|e| {
            error!(error = %e, "Failed to update playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(playlist))
}

pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state.db.delete_playlist(id).await
        .map_err(|e| {
            error!(error = %e, "Failed to delete playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Song Handlers
// =============================================================================

pub async fn add_song(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(data): Json<CreateSong>,
) -> Result<(StatusCode, Json<Song>), StatusCode> {
    // Verify playlist exists
    state.db.get_playlist(playlist_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get YouTube metadata
    let metadata = state.youtube.get_metadata(&data.youtube_url).await
        .map_err(|e| {
            error!(error = %e, "Failed to get YouTube metadata");
            StatusCode::BAD_REQUEST
        })?;

    // Check if song already exists in playlist
    if let Ok(Some(_)) = state.db.get_song_by_youtube_id(&metadata.id, playlist_id).await {
        return Err(StatusCode::CONFLICT);
    }

    // Create song with enhanced data
    let mut create_data = data.clone();
    
    // Try to get cover art from MusicBrainz if not provided
    if create_data.cover_url.is_none() {
        let cover = state.cover_art.search_cover(
            &metadata.title,
            metadata.uploader.as_deref()
        ).await;
        
        if let Some(cover) = cover {
            create_data.cover_url = Some(cover.url);
        }
    }

    let song = state.db.create_song(playlist_id, create_data, metadata).await
        .map_err(|e| {
            error!(error = %e, "Failed to create song");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(song)))
}

pub async fn update_song(
    State(state): State<Arc<AppState>>,
    Path((playlist_id, song_id)): Path<(Uuid, Uuid)>,
    Json(data): Json<UpdateSong>,
) -> Result<Json<Song>, StatusCode> {
    // Verify song belongs to playlist
    let song = state.db.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if song.playlist_id != playlist_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let updated = state.db.update_song(song_id, data).await
        .map_err(|e| {
            error!(error = %e, "Failed to update song");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(updated))
}

pub async fn delete_song(
    State(state): State<Arc<AppState>>,
    Path((playlist_id, song_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Verify song belongs to playlist
    let song = state.db.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if song.playlist_id != playlist_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let deleted = state.db.delete_song(song_id).await
        .map_err(|e| {
            error!(error = %e, "Failed to delete song");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Debug, Deserialize)]
pub struct ReorderSongsRequest {
    pub song_ids: Vec<Uuid>,
}

pub async fn reorder_songs(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(data): Json<ReorderSongsRequest>,
) -> Result<StatusCode, StatusCode> {
    state.db.reorder_songs(playlist_id, data.song_ids).await
        .map_err(|e| {
            error!(error = %e, "Failed to reorder songs");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

// =============================================================================
// Streaming Handler
// =============================================================================

pub async fn stream_song(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let song = state.db.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    info!(song_id = %song_id, title = %song.title, "Starting stream");

    // Get stream URL from YouTube
    let stream_info = state.youtube.get_audio_stream_url(&song.youtube_id).await
        .map_err(|e| {
            error!(error = %e, "Failed to get stream URL");
            StatusCode::BAD_GATEWAY
        })?;

    // Use ffmpeg to transcode audio
    // Apply minimal processing to keep sound neutral for frontend equalizer:
    // - highpass: Remove inaudible sub-bass (<25Hz) that causes analyzer spikes
    // - loudnorm: Normalize volume between songs
    // - alimiter: Soft limiter to control peaks without compressing dynamics
    let child = Command::new(&state.config.ffmpeg_path)
        .args([
            "-i", &stream_info.url,
            "-vn",                          // No video
            "-acodec", "libopus",           // Opus codec for good quality
            "-b:a", "128k",                 // 128kbps bitrate
            "-ar", "48000",                 // 48kHz sample rate
            "-ac", "2",                     // Stereo
            "-af", "highpass=f=25,loudnorm=I=-14:TP=-1:LRA=11,alimiter=limit=0.95:level=false",
            "-f", "ogg",                    // OGG container
            "-",                            // Output to stdout
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| {
            error!(error = %e, "Failed to spawn ffmpeg");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let stdout = child.stdout.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let stream = ReaderStream::new(tokio::io::BufReader::new(stdout));
    let body = Body::from_stream(stream);

    // Update play count
    let _ = state.db.increment_play_count(song_id).await;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/ogg")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("X-Song-Title", song.title)
        .header("X-Song-Duration", song.duration.to_string())
        .body(body)
        .unwrap())
}

/// Get stream info without starting the stream
pub async fn get_stream_info(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<Uuid>,
) -> Result<Json<StreamInfo>, StatusCode> {
    let song = state.db.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let stream_url = state.youtube.get_audio_stream_url(&song.youtube_id).await
        .map_err(|e| {
            error!(error = %e, "Failed to get stream URL");
            StatusCode::BAD_GATEWAY
        })?;

    Ok(Json(StreamInfo {
        song_id,
        stream_url: format!("/api/music/stream/{}", song_id),
        format: "ogg".to_string(),
        bitrate: 128,
        sample_rate: 48000,
    }))
}

// =============================================================================
// Search & Discovery
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

pub async fn search_youtube(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<crate::youtube::SearchResult>>, StatusCode> {
    let results = state.youtube.search(&query.q, query.limit.unwrap_or(10)).await
        .map_err(|e| {
            error!(error = %e, "Search failed");
            StatusCode::BAD_GATEWAY
        })?;

    Ok(Json(results))
}

pub async fn get_youtube_metadata(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UrlQuery>,
) -> Result<Json<YouTubeMetadata>, StatusCode> {
    let metadata = state.youtube.get_metadata(&query.url).await
        .map_err(|e| {
            error!(error = %e, "Failed to get metadata");
            StatusCode::BAD_REQUEST
        })?;

    Ok(Json(metadata))
}

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    pub url: String,
}

// =============================================================================
// Cover Art
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CoverSearchQuery {
    pub title: String,
    pub artist: Option<String>,
}

pub async fn search_cover(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CoverSearchQuery>,
) -> Result<Json<Option<crate::cover_art::CoverArtResult>>, StatusCode> {
    let result = state.cover_art.search_cover(
        &query.title,
        query.artist.as_deref()
    ).await;

    Ok(Json(result))
}

// =============================================================================
// Equalizer
// =============================================================================

pub async fn get_equalizer(
    State(state): State<Arc<AppState>>,
) -> Json<EqualizerSettings> {
    // Return default settings if there's any error (including deserialization)
    let settings = state.db.get_equalizer_settings().await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to get equalizer settings, using defaults");
            EqualizerSettings::default()
        });

    Json(settings)
}

pub async fn update_equalizer(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<EqualizerSettings>,
) -> Result<Json<EqualizerSettings>, StatusCode> {
    state.db.save_equalizer_settings(settings.clone()).await
        .map_err(|e| {
            error!(error = %e, "Failed to save equalizer settings");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(settings))
}

#[derive(Debug, Deserialize)]
pub struct PresetQuery {
    pub name: String,
}

pub async fn get_equalizer_preset(
    Query(query): Query<PresetQuery>,
) -> Result<Json<EqualizerSettings>, StatusCode> {
    let preset = match query.name.as_str() {
        "flat" => EqualizerSettings::preset_flat(),
        "bass_boost" => EqualizerSettings::preset_bass_boost(),
        "treble_boost" => EqualizerSettings::preset_treble_boost(),
        "vocal" => EqualizerSettings::preset_vocal(),
        "rock" => EqualizerSettings::preset_rock(),
        "electronic" => EqualizerSettings::preset_electronic(),
        "acoustic" => EqualizerSettings::preset_acoustic(),
        _ => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Json(preset))
}

// =============================================================================
// Listening History & Stats
// =============================================================================

pub async fn get_listening_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<ListeningEntry>>, StatusCode> {
    let history = state.db.get_listening_history(query.limit.unwrap_or(50)).await
        .map_err(|e| {
            error!(error = %e, "Failed to get history");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(history))
}

pub async fn get_most_played(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Song>>, StatusCode> {
    let songs = state.db.get_most_played_songs(query.limit.unwrap_or(20)).await
        .map_err(|e| {
            error!(error = %e, "Failed to get most played");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(songs))
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}
