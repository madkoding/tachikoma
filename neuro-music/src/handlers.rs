//! HTTP handlers for music API
//! Uses BackendClient for data persistence

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response, Sse},
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::process::Command;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tokio_util::io::ReaderStream;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::events::{MusicEvent, PlaylistEventData, SongEventData};
use crate::models::*;
use crate::AppState;

/// Error response with message
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl ErrorResponse {
    pub fn new(code: &str, error: impl Into<String>) -> Self {
        Self { code: code.to_string(), error: error.into() }
    }
}

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
    let playlists = state.client.get_all_playlists().await
        .map_err(|e| {
            error!(error = %e, "Failed to get playlists");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if query.include_songs.unwrap_or(false) {
        let mut result = Vec::new();
        for playlist in playlists {
            let songs = state.client.get_songs_by_playlist(playlist.id).await
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
    let playlist = state.client.get_playlist_with_songs(id).await
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
    let playlist = state.client.create_playlist(data).await
        .map_err(|e| {
            error!(error = %e, "Failed to create playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Broadcast event
    state.event_broadcaster.broadcast(MusicEvent::PlaylistCreated(
        PlaylistEventData::from(&playlist)
    ));
    info!(playlist_id = %playlist.id, name = %playlist.name, "Playlist created, event broadcasted");

    Ok((StatusCode::CREATED, Json(playlist)))
}

pub async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePlaylist>,
) -> Result<Json<Playlist>, StatusCode> {
    let playlist = state.client.update_playlist(id, data).await
        .map_err(|e| {
            error!(error = %e, "Failed to update playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Broadcast event
    state.event_broadcaster.broadcast(MusicEvent::PlaylistUpdated(
        PlaylistEventData::from(&playlist)
    ));

    Ok(Json(playlist))
}

pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Get playlist to check if it's special
    let playlist = state.client.get_playlist(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Playlist no encontrada"))))?;
    
    // Prevent deletion of special playlists
    if playlist.is_favorites {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("FORBIDDEN", "No puedes eliminar la playlist 'Me gusta'"))));
    }
    if playlist.is_suggestions {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("FORBIDDEN", "No puedes eliminar la playlist 'Sugerencias'"))));
    }
    
    let deleted = state.client.delete_playlist(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("DELETE_ERROR", e.to_string()))))?;

    if deleted {
        // Broadcast event
        state.event_broadcaster.broadcast(MusicEvent::PlaylistDeleted {
            id: id.to_string()
        });
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Playlist no encontrada"))))
    }
}

// =============================================================================
// Song Handlers
// =============================================================================

pub async fn add_song(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(data): Json<CreateSong>,
) -> Result<(StatusCode, Json<Song>), (StatusCode, Json<ErrorResponse>)> {
    // Verify playlist exists
    state.client.get_playlist(playlist_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Playlist no encontrada"))))?;

    // Get YouTube metadata
    let mut metadata = state.youtube.get_metadata(&data.youtube_url).await
        .map_err(|e| {
            error!(error = %e, "Failed to get YouTube metadata");
            (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("YOUTUBE_ERROR", e)))
        })?;

    // Check if song already exists in playlist
    if let Ok(Some(_)) = state.client.get_song_by_youtube_id(&metadata.youtube_id, playlist_id).await {
        return Err((StatusCode::CONFLICT, Json(ErrorResponse::new("DUPLICATE", "Esta canción ya está en la playlist"))));
    }

    // Enrich metadata using MusicBrainz or LLM
    let enriched = state.metadata_enricher.enrich(
        &metadata.title,
        metadata.artist.as_deref()
    ).await;
    
    // Update metadata with enriched data
    metadata.title = enriched.title;
    metadata.artist = enriched.artist;
    metadata.album = enriched.album;

    // Create song with enhanced data
    let mut create_data = data.clone();
    
    // Override with enriched data if not provided by user
    if create_data.title.is_none() {
        create_data.title = Some(metadata.title.clone());
    }
    if create_data.artist.is_none() {
        create_data.artist = metadata.artist.clone();
    }
    if create_data.album.is_none() {
        create_data.album = metadata.album.clone();
    }
    
    // Try to get cover art from MusicBrainz if not provided
    if create_data.cover_url.is_none() {
        let cover = state.cover_art.search_cover(
            &metadata.title,
            metadata.artist.as_deref()
        ).await;
        
        if let Some(cover) = cover {
            create_data.cover_url = Some(cover.url);
        } else {
            // Fallback to YouTube thumbnail (high quality)
            create_data.cover_url = Some(crate::cover_art::CoverArtService::get_youtube_thumbnail(
                &metadata.youtube_id,
                crate::cover_art::ThumbnailQuality::High
            ));
        }
    }

    let song = state.client.create_song(playlist_id, create_data, metadata).await
        .map_err(|e| {
            error!(error = %e, "Failed to create song");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("CREATE_ERROR", e.to_string())))
        })?;

    // Broadcast event
    state.event_broadcaster.broadcast(MusicEvent::SongAdded {
        playlist_id: playlist_id.to_string(),
        song: SongEventData::from(&song),
    });
    info!(song_id = %song.id, title = %song.title, "Song added, event broadcasted");

    Ok((StatusCode::CREATED, Json(song)))
}

pub async fn update_song(
    State(state): State<Arc<AppState>>,
    Path((playlist_id, song_id)): Path<(Uuid, Uuid)>,
    Json(data): Json<UpdateSong>,
) -> Result<Json<Song>, StatusCode> {
    // Verify song belongs to playlist
    let song = state.client.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if song.playlist_id != playlist_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let updated = state.client.update_song(song_id, data).await
        .map_err(|e| {
            error!(error = %e, "Failed to update song");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Broadcast event
    state.event_broadcaster.broadcast(MusicEvent::SongUpdated(
        SongEventData::from(&updated)
    ));

    Ok(Json(updated))
}

pub async fn delete_song(
    State(state): State<Arc<AppState>>,
    Path((playlist_id, song_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Verify song belongs to playlist
    let song = state.client.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if song.playlist_id != playlist_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let deleted = state.client.delete_song(song_id).await
        .map_err(|e| {
            error!(error = %e, "Failed to delete song");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        // Broadcast event
        state.event_broadcaster.broadcast(MusicEvent::SongRemoved {
            playlist_id: playlist_id.to_string(),
            song_id: song_id.to_string(),
        });
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
    state.client.reorder_songs(playlist_id, data.song_ids).await
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
    let song = state.client.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    info!(song_id = %song_id, title = %song.title, "Starting stream");

    // Check if song is downloaded locally
    if state.downloader.is_downloaded(&song.youtube_id).await {
        let file_path = state.downloader.get_song_path(&song.youtube_id);
        info!(song_id = %song_id, path = %file_path.display(), "Streaming from local file");

        // Open file and stream it
        let file = tokio::fs::File::open(&file_path).await
            .map_err(|e| {
                error!(error = %e, "Failed to open local file");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let stream = ReaderStream::new(tokio::io::BufReader::new(file));
        let body = Body::from_stream(stream);

        // Update play count
        let _ = state.client.increment_play_count(song_id).await;

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "audio/ogg")
            .header(header::CACHE_CONTROL, "no-cache")
            .header("X-Song-Title", song.title)
            .header("X-Song-Duration", song.duration.to_string())
            .header("X-Stream-Source", "local")
            .body(body)
            .unwrap());
    }

    // Not downloaded, stream from YouTube
    info!(song_id = %song_id, "Streaming from YouTube");

    // Get stream URL from YouTube
    let stream_info = state.youtube.get_audio_stream_url(&song.youtube_id).await
        .map_err(|e| {
            error!(error = %e, "Failed to get stream URL");
            StatusCode::BAD_GATEWAY
        })?;

    // Use ffmpeg to transcode audio
    let child = Command::new(&state.config.ffmpeg_path)
        .args([
            "-i", &stream_info.url,
            "-vn",
            "-acodec", "libopus",
            "-b:a", "128k",
            "-ar", "48000",
            "-ac", "2",
            "-af", "highpass=f=25,loudnorm=I=-14:TP=-1:LRA=11,alimiter=limit=0.95:level=false",
            "-f", "ogg",
            "-",
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
    let _ = state.client.increment_play_count(song_id).await;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/ogg")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("X-Song-Title", song.title)
        .header("X-Song-Duration", song.duration.to_string())
        .header("X-Stream-Source", "youtube")
        .body(body)
        .unwrap())
}

/// Download a song as OGG for client-side caching
/// This downloads/converts the entire song and returns it as a blob
pub async fn download_song(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let song = state.client.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    info!(song_id = %song_id, title = %song.title, "Starting download for cache");

    // Check if song is already downloaded locally on server
    if state.downloader.is_downloaded(&song.youtube_id).await {
        let file_path = state.downloader.get_song_path(&song.youtube_id);
        info!(song_id = %song_id, path = %file_path.display(), "Serving from local file");

        let file_content = tokio::fs::read(&file_path).await
            .map_err(|e| {
                error!(error = %e, "Failed to read local file");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "audio/ogg")
            .header(header::CONTENT_LENGTH, file_content.len().to_string())
            .header(header::CACHE_CONTROL, "public, max-age=31536000")
            .header("X-Song-Id", song_id.to_string())
            .body(Body::from(file_content))
            .unwrap());
    }

    // Download from YouTube to server cache first
    info!(song_id = %song_id, "Downloading from YouTube");
    
    let file_path = state.downloader.download_song(&song.youtube_id).await
        .map_err(|e| {
            error!(error = %e, "Failed to download song");
            StatusCode::BAD_GATEWAY
        })?;

    // Read the downloaded file
    let file_content = tokio::fs::read(&file_path).await
        .map_err(|e| {
            error!(error = %e, "Failed to read downloaded file");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(song_id = %song_id, size = file_content.len(), "Download complete");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/ogg")
        .header(header::CONTENT_LENGTH, file_content.len().to_string())
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .header("X-Song-Id", song_id.to_string())
        .body(Body::from(file_content))
        .unwrap())
}

/// Get stream info without starting the stream
pub async fn get_stream_info(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<Uuid>,
) -> Result<Json<StreamInfo>, StatusCode> {
    let song = state.client.get_song(song_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let _stream_url = state.youtube.get_audio_stream_url(&song.youtube_id).await
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

/// Search YouTube with enriched metadata (title, artist, album)
pub async fn search_youtube_enriched(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<crate::metadata_enricher::EnrichedSearchResult>>, StatusCode> {
    let results = state.youtube.search(&query.q, query.limit.unwrap_or(10)).await
        .map_err(|e| {
            error!(error = %e, "Search failed");
            StatusCode::BAD_GATEWAY
        })?;

    // Convert to enrichment format
    let to_enrich: Vec<crate::metadata_enricher::SearchResultToEnrich> = results
        .into_iter()
        .map(|r| crate::metadata_enricher::SearchResultToEnrich {
            video_id: r.video_id,
            title: r.title,
            artist: r.artist,
            channel: Some(r.channel),
            duration: r.duration,
            thumbnail: r.thumbnail,
            view_count: r.view_count,
        })
        .collect();

    // Enrich all results
    let enriched = state.metadata_enricher.enrich_search_results(to_enrich).await;

    Ok(Json(enriched))
}

/// Enrich metadata for a single video
#[derive(Debug, Deserialize)]
pub struct EnrichRequest {
    pub title: String,
    pub channel: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnrichResponse {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub source: crate::metadata_enricher::MetadataSource,
}

pub async fn enrich_metadata(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EnrichRequest>,
) -> Result<Json<EnrichResponse>, StatusCode> {
    let enriched = state.metadata_enricher.enrich(
        &request.title,
        request.channel.as_deref()
    ).await;

    Ok(Json(EnrichResponse {
        title: enriched.title,
        artist: enriched.artist,
        album: enriched.album,
        source: enriched.source,
    }))
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
    let settings = state.client.get_equalizer_settings().await
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
    state.client.save_equalizer_settings(settings.clone()).await
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
    let history = state.client.get_listening_history(query.limit.unwrap_or(50)).await
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
    let songs = state.client.get_most_played_songs(query.limit.unwrap_or(20)).await
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

// =============================================================================
// Special Playlists (Favorites & Suggestions)
// =============================================================================

/// Initialize special playlists (Favorites and Suggestions)
pub async fn init_special_playlists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Playlist>>, StatusCode> {
    let mut playlists = Vec::new();
    
    // Get or create favorites playlist
    let favorites = state.client.get_or_create_favorites_playlist().await
        .map_err(|e| {
            error!(error = %e, "Failed to init favorites playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    playlists.push(favorites);
    
    // Get or create suggestions playlist
    let suggestions = state.client.get_or_create_suggestions_playlist().await
        .map_err(|e| {
            error!(error = %e, "Failed to init suggestions playlist");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    playlists.push(suggestions);
    
    info!("Special playlists initialized");
    Ok(Json(playlists))
}

/// Toggle like on a song
pub async fn toggle_song_like(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<Uuid>,
) -> Result<Json<Song>, (StatusCode, Json<ErrorResponse>)> {
    // Get original song
    let song = state.client.get_song(song_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Canción no encontrada"))))?;
    
    let new_liked_status = !song.is_liked;
    
    // Update original song's like status
    let update = UpdateSong {
        is_liked: Some(new_liked_status),
        ..Default::default()
    };
    
    let updated_song = state.client.update_song(song_id, update).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("UPDATE_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Canción no encontrada"))))?;
    
    // Get or create favorites playlist
    let favorites = state.client.get_or_create_favorites_playlist().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("FAVORITES_ERROR", e.to_string()))))?;
    
    if new_liked_status {
        // Add to favorites if not already there
        let favorites_songs = state.client.get_songs_by_playlist(favorites.id).await
            .unwrap_or_default();
        
        let already_in_favorites = favorites_songs.iter().any(|s| s.youtube_id == updated_song.youtube_id);
        
        if !already_in_favorites {
            // Add song to favorites
            let create_data = CreateSong {
                youtube_url: updated_song.youtube_url.clone(),
                title: Some(updated_song.title.clone()),
                artist: updated_song.artist.clone(),
                album: updated_song.album.clone(),
                cover_url: updated_song.cover_url.clone(),
            };
            
            let metadata = YouTubeMetadata {
                youtube_id: updated_song.youtube_id.clone(),
                title: updated_song.title.clone(),
                artist: updated_song.artist.clone(),
                album: updated_song.album.clone(),
                duration: updated_song.duration,
                thumbnail_url: updated_song.thumbnail_url.clone(),
            };
            
            let _ = state.client.create_song(favorites.id, create_data, metadata).await;
            info!(song_id = %song_id, "Added song to favorites");
        }

        // Start background download for liked songs
        let youtube_id = updated_song.youtube_id.clone();
        let downloader = state.downloader.clone();
        if !downloader.is_downloaded(&youtube_id).await {
            info!(youtube_id = %youtube_id, "Starting background download for liked song");
            downloader.download_in_background(youtube_id);
        }
    } else {
        // Remove from favorites
        let favorites_songs = state.client.get_songs_by_playlist(favorites.id).await
            .unwrap_or_default();
        
        if let Some(fav_song) = favorites_songs.iter().find(|s| s.youtube_id == updated_song.youtube_id) {
            let _ = state.client.delete_song(fav_song.id).await;
            info!(song_id = %song_id, "Removed song from favorites");
        }

        // Optionally delete the downloaded file when unliking
        // (commented out - we keep files for now since song might still be in other playlists)
        // let _ = state.downloader.delete_song(&updated_song.youtube_id).await;
    }
    
    Ok(Json(updated_song))
}

/// Fetch and update cover art for a song
pub async fn fetch_song_cover(
    State(state): State<Arc<AppState>>,
    Path(song_id): Path<Uuid>,
) -> Result<Json<Song>, (StatusCode, Json<ErrorResponse>)> {
    // Get the song
    let song = state.client.get_song(song_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Canción no encontrada"))))?;
    
    // Try to get cover art from MusicBrainz
    let cover = state.cover_art.search_cover(
        &song.title,
        song.artist.as_deref()
    ).await;
    
    let cover_url = if let Some(cover) = cover {
        cover.url
    } else {
        // Fallback to YouTube thumbnail (high quality)
        crate::cover_art::CoverArtService::get_youtube_thumbnail(
            &song.youtube_id,
            crate::cover_art::ThumbnailQuality::High
        )
    };
    
    // Update the song with the new cover
    let update = UpdateSong {
        cover_url: Some(cover_url),
        ..Default::default()
    };
    
    let updated_song = state.client.update_song(song_id, update).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("UPDATE_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Canción no encontrada"))))?;
    
    info!(song_id = %song_id, "Updated song cover");
    Ok(Json(updated_song))
}

/// Get all liked songs
pub async fn get_liked_songs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Song>>, StatusCode> {
    let songs = state.client.get_liked_songs().await
        .map_err(|e| {
            error!(error = %e, "Failed to get liked songs");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(songs))
}

/// Refresh suggestions playlist
pub async fn refresh_suggestions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PlaylistWithSongs>, (StatusCode, Json<ErrorResponse>)> {
    info!("Refreshing suggestions playlist");
    
    // Get suggestions playlist
    let suggestions = state.client.get_or_create_suggestions_playlist().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("SUGGESTIONS_ERROR", e.to_string()))))?;
    
    // Get top played songs to base suggestions on
    let top_songs = state.client.get_most_played_songs(10).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("TOP_SONGS_ERROR", e.to_string()))))?;
    
    if top_songs.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("NO_HISTORY", "No hay suficiente historial para generar sugerencias. Escucha más música primero."))));
    }
    
    // Clear current suggestions
    let _ = state.client.clear_playlist_songs(suggestions.id).await;
    
    // Build search queries based on top songs
    let mut added_youtube_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut songs_added = 0;
    let target_songs = 30;
    
    // Collect existing youtube IDs from all playlists to avoid duplicates
    let all_playlists = state.client.get_all_playlists().await.unwrap_or_default();
    for playlist in &all_playlists {
        if let Ok(songs) = state.client.get_songs_by_playlist(playlist.id).await {
            for song in songs {
                added_youtube_ids.insert(song.youtube_id.clone());
            }
        }
    }
    
    // Search for related songs
    for seed_song in &top_songs {
        if songs_added >= target_songs {
            break;
        }
        
        // Build search query
        let search_query = if let Some(ref artist) = seed_song.artist {
            format!("{} {}", artist, seed_song.title.split(" - ").next().unwrap_or(&seed_song.title))
        } else {
            format!("{} music similar", seed_song.title.split(" - ").next().unwrap_or(&seed_song.title))
        };
        
        // Search YouTube
        if let Ok(results) = state.youtube.search(&search_query, 5).await {
            for result in results {
                if songs_added >= target_songs {
                    break;
                }
                
                // Skip if already in library
                if added_youtube_ids.contains(&result.video_id) {
                    continue;
                }
                
                // Get full metadata
                let url = format!("https://www.youtube.com/watch?v={}", result.video_id);
                if let Ok(metadata) = state.youtube.get_metadata(&url).await {
                    let create_data = CreateSong {
                        youtube_url: url.clone(),
                        title: Some(metadata.title.clone()),
                        artist: metadata.artist.clone(),
                        album: metadata.album.clone(),
                        cover_url: None,
                    };
                    
                    if let Ok(_) = state.client.create_song(suggestions.id, create_data, metadata.clone()).await {
                        added_youtube_ids.insert(result.video_id.clone());
                        songs_added += 1;
                        info!(youtube_id = %result.video_id, title = %metadata.title, "Added suggestion");
                    }
                }
            }
        }
    }
    
    // Update suggestions timestamp
    let _ = state.client.update_suggestions_timestamp(suggestions.id).await;
    
    // Return updated playlist
    let updated_playlist = state.client.get_playlist_with_songs(suggestions.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("GET_ERROR", e.to_string()))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("NOT_FOUND", "Playlist de sugerencias no encontrada"))))?;
    
    info!(songs_added = songs_added, "Suggestions refreshed");
    Ok(Json(updated_playlist))
}

// =============================================================================
// Server-Sent Events (SSE) Handler
// =============================================================================

/// SSE endpoint for real-time music events
/// Clients connect here to receive playlist/song updates in real-time
pub async fn sse_events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let rx = state.event_broadcaster.subscribe();
    
    // Convert broadcast receiver to SSE stream
    let stream = BroadcastStream::new(rx)
        .map(|result| {
            match result {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    debug!(event = %json, "Sending SSE event");
                    Ok(axum::response::sse::Event::default().data(json))
                }
                Err(e) => {
                    error!(error = %e, "SSE broadcast error");
                    Ok(axum::response::sse::Event::default().data("{}"))
                }
            }
        });
    
    // Add heartbeat every 30 seconds to keep connection alive
    let heartbeat = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(Duration::from_secs(30))
    ).map(|_| {
        let event = MusicEvent::Heartbeat;
        let json = serde_json::to_string(&event).unwrap_or_default();
        Ok(axum::response::sse::Event::default().data(json))
    });
    
    let combined = stream.merge(heartbeat);
    
    Sse::new(combined)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive")
        )
}
