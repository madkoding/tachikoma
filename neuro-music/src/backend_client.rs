//! Backend client - HTTP client to tachikoma-backend data layer

use reqwest::Client;
use serde::Serialize;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{
    CreatePlaylist, CreateSong, EqualizerSettings, ListeningEntry, Playlist,
    PlaylistWithSongs, Song, UpdatePlaylist, UpdateSong, YouTubeMetadata,
};

pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/api/data", config.backend_url),
        }
    }

    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/health", self.base_url.replace("/api/data", "/api"));
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    // ==========================================================================
    // Playlist CRUD
    // ==========================================================================

    pub async fn get_all_playlists(&self) -> Result<Vec<Playlist>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let playlists: Vec<Playlist> = response.json().await?;
        Ok(playlists)
    }

    pub async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let with_songs: PlaylistWithSongs = response.json().await?;
        Ok(Some(with_songs.playlist))
    }

    pub async fn get_playlist_with_songs(&self, id: Uuid) -> Result<Option<PlaylistWithSongs>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let with_songs: PlaylistWithSongs = response.json().await?;
        Ok(Some(with_songs))
    }

    pub async fn create_playlist(&self, data: CreatePlaylist) -> Result<Playlist, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists", self.base_url);
        let response = self.client.post(&url).json(&data).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let playlist: Playlist = response.json().await?;
        Ok(playlist)
    }

    pub async fn update_playlist(&self, id: Uuid, data: UpdatePlaylist) -> Result<Option<Playlist>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}", self.base_url, id);
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let playlist: Playlist = response.json().await?;
        Ok(Some(playlist))
    }

    pub async fn delete_playlist(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}", self.base_url, id);
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(true)
    }

    // ==========================================================================
    // Song CRUD
    // ==========================================================================

    pub async fn get_songs_by_playlist(&self, playlist_id: Uuid) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}/songs", self.base_url, playlist_id);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let songs: Vec<Song> = response.json().await?;
        Ok(songs)
    }

    pub async fn get_song(&self, id: Uuid) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/songs/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let song: Song = response.json().await?;
        Ok(Some(song))
    }

    pub async fn get_song_by_youtube_id(&self, youtube_id: &str, playlist_id: Uuid) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/songs/by-youtube-id?youtube_id={}&playlist_id={}",
            self.base_url, youtube_id, playlist_id
        );
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let song: Option<Song> = response.json().await?;
        Ok(song)
    }

    pub async fn create_song(&self, playlist_id: Uuid, data: CreateSong, metadata: YouTubeMetadata) -> Result<Song, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}/songs", self.base_url, playlist_id);
        
        // Send exactly what the backend expects:
        // { "youtube_url": "...", "cover_url": "...", "metadata": { "youtube_id": "...", ... } }
        #[derive(Serialize)]
        struct Request {
            youtube_url: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            cover_url: Option<String>,
            metadata: YouTubeMetadata,
        }
        
        let request = Request { 
            youtube_url: data.youtube_url,
            cover_url: data.cover_url,
            metadata,
        };
        
        // Log the JSON being sent for debugging - use error! level to ensure visibility
        let json_str = serde_json::to_string_pretty(&request).unwrap_or_default();
        tracing::error!(
            url = %url,
            json_length = %json_str.len(),
            youtube_id = %request.metadata.youtube_id,
            "🔍 DEBUG: Create song request"
        );
        tracing::error!("🔍 DEBUG: Full Request JSON:\n{}", json_str);
        
        let response = self.client.post(&url).json(&request).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let song: Song = response.json().await?;
        Ok(song)
    }

    pub async fn update_song(&self, id: Uuid, data: UpdateSong) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/songs/{}", self.base_url, id);
        let response = self.client.patch(&url).json(&data).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let song: Song = response.json().await?;
        Ok(Some(song))
    }

    pub async fn delete_song(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/songs/{}", self.base_url, id);
        let response = self.client.delete(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(true)
    }

    pub async fn increment_play_count(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/songs/{}/play", self.base_url, id);
        let response = self.client.post(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(())
    }

    pub async fn reorder_songs(&self, playlist_id: Uuid, song_ids: Vec<Uuid>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}/reorder", self.base_url, playlist_id);
        
        #[derive(Serialize)]
        struct Request {
            song_ids: Vec<Uuid>,
        }
        
        let response = self.client.post(&url).json(&Request { song_ids }).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(())
    }

    // ==========================================================================
    // Listening History & Stats
    // ==========================================================================

    pub async fn add_listening_entry(&self, entry: ListeningEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/music/history", self.base_url);
        let response = self.client.post(&url).json(&entry).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(())
    }

    pub async fn get_listening_history(&self, limit: usize) -> Result<Vec<ListeningEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/music/history?limit={}", self.base_url, limit);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let history: Vec<ListeningEntry> = response.json().await?;
        Ok(history)
    }

    pub async fn get_most_played_songs(&self, limit: usize) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/music/top-songs?limit={}", self.base_url, limit);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let songs: Vec<Song> = response.json().await?;
        Ok(songs)
    }

    // ==========================================================================
    // Equalizer Settings
    // ==========================================================================

    pub async fn get_equalizer_settings(&self) -> Result<EqualizerSettings, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/music/equalizer", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            // Return default on error
            return Ok(EqualizerSettings::default());
        }
        
        let settings: EqualizerSettings = response.json().await?;
        Ok(settings)
    }

    pub async fn save_equalizer_settings(&self, settings: EqualizerSettings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/music/equalizer", self.base_url);
        let response = self.client.put(&url).json(&settings).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(())
    }

    // ==========================================================================
    // Special Playlists (Favorites & Suggestions)
    // ==========================================================================

    /// Get the favorites playlist, creating it if it doesn't exist
    pub async fn get_or_create_favorites_playlist(&self) -> Result<Playlist, Box<dyn std::error::Error + Send + Sync>> {
        // First, try to find existing favorites playlist
        let playlists = self.get_all_playlists().await?;
        if let Some(favorites) = playlists.into_iter().find(|p| p.is_favorites) {
            return Ok(favorites);
        }

        // Create favorites playlist
        let url = format!("{}/playlists", self.base_url);
        
        #[derive(Serialize)]
        struct CreateSpecialPlaylist {
            name: String,
            description: Option<String>,
            is_favorites: bool,
            is_suggestions: bool,
        }
        
        let response = self.client.post(&url).json(&CreateSpecialPlaylist {
            name: "Me gusta ❤️".to_string(),
            description: Some("Tus canciones favoritas".to_string()),
            is_favorites: true,
            is_suggestions: false,
        }).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let playlist: Playlist = response.json().await?;
        Ok(playlist)
    }

    /// Get the suggestions playlist, creating it if it doesn't exist
    pub async fn get_or_create_suggestions_playlist(&self) -> Result<Playlist, Box<dyn std::error::Error + Send + Sync>> {
        // First, try to find existing suggestions playlist
        let playlists = self.get_all_playlists().await?;
        if let Some(suggestions) = playlists.into_iter().find(|p| p.is_suggestions) {
            return Ok(suggestions);
        }

        // Create suggestions playlist
        let url = format!("{}/playlists", self.base_url);
        
        #[derive(Serialize)]
        struct CreateSpecialPlaylist {
            name: String,
            description: Option<String>,
            is_favorites: bool,
            is_suggestions: bool,
        }
        
        let response = self.client.post(&url).json(&CreateSpecialPlaylist {
            name: "Sugerencias ✨".to_string(),
            description: Some("Canciones recomendadas basadas en tus gustos".to_string()),
            is_favorites: false,
            is_suggestions: true,
        }).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        let playlist: Playlist = response.json().await?;
        Ok(playlist)
    }

    /// Get all liked songs across all playlists
    pub async fn get_liked_songs(&self) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/songs/liked", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            // Return empty on error (endpoint might not exist yet)
            return Ok(Vec::new());
        }
        
        let songs: Vec<Song> = response.json().await?;
        Ok(songs)
    }

    /// Toggle like status on a song
    pub async fn toggle_song_like(&self, id: Uuid) -> Result<Song, Box<dyn std::error::Error + Send + Sync>> {
        // Get current song
        let song = self.get_song(id).await?
            .ok_or("Song not found")?;
        
        // Toggle is_liked
        let update = UpdateSong {
            is_liked: Some(!song.is_liked),
            ..Default::default()
        };
        
        let updated = self.update_song(id, update).await?
            .ok_or("Failed to update song")?;
        
        Ok(updated)
    }

    /// Update suggestions playlist timestamp
    pub async fn update_suggestions_timestamp(&self, playlist_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/playlists/{}/suggestions-timestamp", self.base_url, playlist_id);
        let response = self.client.post(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Backend error {}: {}", status, text).into());
        }
        
        Ok(())
    }

    /// Delete all songs from a playlist
    pub async fn clear_playlist_songs(&self, playlist_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let songs = self.get_songs_by_playlist(playlist_id).await?;
        for song in songs {
            self.delete_song(song.id).await?;
        }
        Ok(())
    }
}
