//! =============================================================================
//! Music Repository Port
//! =============================================================================

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::music::{
    CreatePlaylist, CreateSong, EqualizerSettings, ListeningEntry, Playlist,
    PlaylistWithSongs, Song, UpdatePlaylist, UpdateSong, YouTubeMetadata,
};
use crate::domain::errors::DomainError;

/// Abstract repository interface for music
#[async_trait]
pub trait MusicRepository: Send + Sync {
    // =========================================================================
    // Playlist CRUD
    // =========================================================================

    /// Get all playlists
    async fn get_all_playlists(&self) -> Result<Vec<Playlist>, DomainError>;

    /// Get a playlist by ID
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>, DomainError>;

    /// Get a playlist with all its songs
    async fn get_playlist_with_songs(&self, id: Uuid) -> Result<Option<PlaylistWithSongs>, DomainError>;

    /// Create a new playlist
    async fn create_playlist(&self, data: CreatePlaylist) -> Result<Playlist, DomainError>;

    /// Update a playlist
    async fn update_playlist(
        &self,
        id: Uuid,
        data: UpdatePlaylist,
    ) -> Result<Option<Playlist>, DomainError>;

    /// Delete a playlist
    async fn delete_playlist(&self, id: Uuid) -> Result<bool, DomainError>;

    // =========================================================================
    // Song CRUD
    // =========================================================================

    /// Get all songs for a playlist
    async fn get_songs_by_playlist(&self, playlist_id: Uuid) -> Result<Vec<Song>, DomainError>;

    /// Get a song by ID
    async fn get_song(&self, id: Uuid) -> Result<Option<Song>, DomainError>;

    /// Get a song by YouTube ID within a playlist
    async fn get_song_by_youtube_id(
        &self,
        youtube_id: &str,
        playlist_id: Uuid,
    ) -> Result<Option<Song>, DomainError>;

    /// Create a new song
    async fn create_song(
        &self,
        playlist_id: Uuid,
        data: CreateSong,
        metadata: YouTubeMetadata,
    ) -> Result<Song, DomainError>;

    /// Update a song
    async fn update_song(&self, id: Uuid, data: UpdateSong) -> Result<Option<Song>, DomainError>;

    /// Delete a song
    async fn delete_song(&self, id: Uuid) -> Result<bool, DomainError>;

    /// Increment play count for a song
    async fn increment_play_count(&self, id: Uuid) -> Result<(), DomainError>;

    /// Reorder songs in a playlist
    async fn reorder_songs(&self, playlist_id: Uuid, song_ids: Vec<Uuid>) -> Result<(), DomainError>;

    // =========================================================================
    // History & Stats
    // =========================================================================

    /// Add listening history entry
    async fn add_listening_entry(&self, entry: ListeningEntry) -> Result<(), DomainError>;

    /// Get listening history
    async fn get_listening_history(&self, limit: usize) -> Result<Vec<ListeningEntry>, DomainError>;

    /// Get most played songs
    async fn get_most_played_songs(&self, limit: usize) -> Result<Vec<Song>, DomainError>;

    // =========================================================================
    // Equalizer
    // =========================================================================

    /// Get equalizer settings
    async fn get_equalizer_settings(&self) -> Result<EqualizerSettings, DomainError>;

    /// Save equalizer settings
    async fn save_equalizer_settings(&self, settings: EqualizerSettings) -> Result<(), DomainError>;

    // =========================================================================
    // Likes & Special Playlists
    // =========================================================================

    /// Get all liked songs across all playlists
    async fn get_liked_songs(&self) -> Result<Vec<Song>, DomainError>;

    /// Update the last_suggestions_update timestamp for a playlist
    async fn update_suggestions_timestamp(&self, id: Uuid) -> Result<(), DomainError>;

    /// Get total play count for a song across all playlists (by youtube_id)
    async fn get_total_play_count_by_youtube_id(&self, youtube_id: &str) -> Result<i32, DomainError>;
}
