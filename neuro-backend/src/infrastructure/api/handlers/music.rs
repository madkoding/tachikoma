//! =============================================================================
//! Music Handlers
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
    pub metadata: YouTubeMetadata,
}

/// POST /api/data/playlists/:id/songs
#[instrument(skip(state, data))]
pub async fn create_song(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(data): Json<CreateSongRequest>,
) -> Result<(StatusCode, Json<Song>), (StatusCode, Json<ErrorResponse>)> {
    match state.music_repository.create_song(playlist_id, data.song, data.metadata).await {
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
