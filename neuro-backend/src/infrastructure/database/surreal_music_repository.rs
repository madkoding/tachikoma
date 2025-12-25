//! =============================================================================
//! SurrealDB Music Repository
//! =============================================================================

use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::sql::Thing;
use tracing::debug;
use uuid::Uuid;

use crate::domain::entities::music::{
    CreatePlaylist, CreateSong, EqualizerSettings, ListeningEntry, Playlist,
    PlaylistWithSongs, RepeatMode, Song, UpdatePlaylist, UpdateSong, YouTubeMetadata,
};
use crate::domain::errors::DomainError;
use crate::domain::ports::music_repository::MusicRepository;
use crate::infrastructure::database::DatabasePool;

/// SurrealDB implementation of MusicRepository
#[derive(Clone)]
pub struct SurrealMusicRepository {
    pool: DatabasePool,
}

// =============================================================================
// Internal Record Types (with SurrealDB Thing IDs)
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
struct PlaylistRecord {
    id: Thing,
    name: String,
    description: Option<String>,
    cover_url: Option<String>,
    is_suggestions: bool,
    shuffle: bool,
    repeat_mode: String,
    song_count: i32,
    total_duration: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct SongRecord {
    id: Thing,
    playlist_id: String,
    youtube_id: String,
    youtube_url: String,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    duration: i64,
    cover_url: Option<String>,
    thumbnail_url: Option<String>,
    song_order: i32,
    play_count: i32,
    last_played: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn thing_to_uuid(thing: &Thing) -> Uuid {
    Uuid::parse_str(&thing.id.to_raw()).unwrap_or_else(|_| Uuid::nil())
}

impl From<PlaylistRecord> for Playlist {
    fn from(record: PlaylistRecord) -> Self {
        Playlist {
            id: thing_to_uuid(&record.id),
            name: record.name,
            description: record.description,
            cover_url: record.cover_url,
            is_suggestions: record.is_suggestions,
            shuffle: record.shuffle,
            repeat_mode: record.repeat_mode.parse().unwrap_or_default(),
            song_count: record.song_count,
            total_duration: record.total_duration,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl From<SongRecord> for Song {
    fn from(record: SongRecord) -> Self {
        Song {
            id: thing_to_uuid(&record.id),
            playlist_id: Uuid::parse_str(&record.playlist_id).unwrap_or_else(|_| Uuid::nil()),
            youtube_id: record.youtube_id,
            youtube_url: record.youtube_url,
            title: record.title,
            artist: record.artist,
            album: record.album,
            duration: record.duration,
            cover_url: record.cover_url,
            thumbnail_url: record.thumbnail_url,
            song_order: record.song_order,
            play_count: record.play_count,
            last_played: record.last_played,
            created_at: record.created_at,
        }
    }
}

// =============================================================================
// Implementation
// =============================================================================

impl SurrealMusicRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    async fn update_playlist_stats(&self, playlist_id: Uuid) -> Result<(), DomainError> {
        let mut result = self.pool.client()
            .query("SELECT count() as song_count, math::sum(duration) as total_duration FROM song WHERE playlist_id = $playlist_id GROUP ALL")
            .bind(("playlist_id", playlist_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let stats: Vec<serde_json::Value> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let (song_count, total_duration) = stats.first()
            .map(|v| {
                let count = v.get("song_count").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                let duration = v.get("total_duration").and_then(|d| d.as_i64()).unwrap_or(0);
                (count, duration)
            })
            .unwrap_or((0, 0));

        let query = format!(
            "UPDATE playlist:`{}` SET song_count = $count, total_duration = $duration, updated_at = time::now()",
            playlist_id
        );
        self.pool.client()
            .query(&query)
            .bind(("count", song_count))
            .bind(("duration", total_duration))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl MusicRepository for SurrealMusicRepository {
    async fn get_all_playlists(&self) -> Result<Vec<Playlist>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM playlist ORDER BY created_at DESC")
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<PlaylistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().map(Playlist::from).collect())
    }

    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>, DomainError> {
        let query = format!("SELECT * FROM playlist:`{}`", id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<PlaylistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().next().map(Playlist::from))
    }

    async fn get_playlist_with_songs(&self, id: Uuid) -> Result<Option<PlaylistWithSongs>, DomainError> {
        let playlist = match self.get_playlist(id).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        let songs = self.get_songs_by_playlist(id).await?;

        Ok(Some(PlaylistWithSongs { playlist, songs }))
    }

    async fn create_playlist(&self, data: CreatePlaylist) -> Result<Playlist, DomainError> {
        let id = Uuid::new_v4();

        let query = format!(
            r#"CREATE playlist:`{}` SET
                name = $name,
                description = $description,
                cover_url = $cover_url,
                is_suggestions = false,
                shuffle = false,
                repeat_mode = 'off',
                song_count = 0,
                total_duration = 0,
                created_at = time::now(),
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("name", data.name.clone()))
            .bind(("description", data.description.clone()))
            .bind(("cover_url", data.cover_url.clone()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<PlaylistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        record
            .map(Playlist::from)
            .ok_or_else(|| DomainError::database("Failed to create playlist"))
    }

    async fn update_playlist(
        &self,
        id: Uuid,
        data: UpdatePlaylist,
    ) -> Result<Option<Playlist>, DomainError> {
        let existing = match self.get_playlist(id).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        let query = format!(
            r#"UPDATE playlist:`{}` SET
                name = $name,
                description = $description,
                cover_url = $cover_url,
                shuffle = $shuffle,
                repeat_mode = $repeat_mode,
                updated_at = time::now()
            "#,
            id
        );

        let mut result = self.pool.client()
            .query(&query)
            .bind(("name", data.name.unwrap_or(existing.name)))
            .bind(("description", data.description.or(existing.description)))
            .bind(("cover_url", data.cover_url.or(existing.cover_url)))
            .bind(("shuffle", data.shuffle.unwrap_or(existing.shuffle)))
            .bind(("repeat_mode", data.repeat_mode.unwrap_or(existing.repeat_mode).to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record: Option<PlaylistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(record.map(Playlist::from))
    }

    async fn delete_playlist(&self, id: Uuid) -> Result<bool, DomainError> {
        // Delete all songs first
        self.pool.client()
            .query("DELETE song WHERE playlist_id = $playlist_id")
            .bind(("playlist_id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let query = format!("DELETE playlist:`{}` RETURN BEFORE", id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let deleted: Vec<PlaylistRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(!deleted.is_empty())
    }

    async fn get_songs_by_playlist(&self, playlist_id: Uuid) -> Result<Vec<Song>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM song WHERE playlist_id = $playlist_id ORDER BY song_order ASC")
            .bind(("playlist_id", playlist_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<SongRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().map(Song::from).collect())
    }

    async fn get_song(&self, id: Uuid) -> Result<Option<Song>, DomainError> {
        let query = format!("SELECT * FROM song:`{}`", id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<SongRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().next().map(Song::from))
    }

    async fn get_song_by_youtube_id(
        &self,
        youtube_id: &str,
        playlist_id: Uuid,
    ) -> Result<Option<Song>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM song WHERE youtube_id = $youtube_id AND playlist_id = $playlist_id LIMIT 1")
            .bind(("youtube_id", youtube_id.to_string()))
            .bind(("playlist_id", playlist_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<SongRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().next().map(Song::from))
    }

    async fn create_song(
        &self,
        playlist_id: Uuid,
        data: CreateSong,
        metadata: YouTubeMetadata,
    ) -> Result<Song, DomainError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Get current max order
        let mut result = self.pool.client()
            .query("SELECT song_order FROM song WHERE playlist_id = $playlist_id ORDER BY song_order DESC LIMIT 1")
            .bind(("playlist_id", playlist_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let max_order: Vec<serde_json::Value> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let next_order = max_order.first()
            .and_then(|v| v.get("song_order"))
            .and_then(|v| v.as_i64())
            .map(|n| n as i32 + 1)
            .unwrap_or(0);

        let query = format!(
            r#"CREATE song:`{}` SET
                playlist_id = $playlist_id,
                youtube_id = $youtube_id,
                youtube_url = $youtube_url,
                title = $title,
                artist = $artist,
                album = $album,
                duration = $duration,
                cover_url = NONE,
                thumbnail_url = $thumbnail_url,
                song_order = $song_order,
                play_count = 0,
                last_played = NONE,
                created_at = time::now()
            "#,
            id
        );

        self.pool.client()
            .query(&query)
            .bind(("playlist_id", playlist_id.to_string()))
            .bind(("youtube_id", metadata.youtube_id.clone()))
            .bind(("youtube_url", data.youtube_url.clone()))
            .bind(("title", metadata.title.clone()))
            .bind(("artist", metadata.artist.clone()))
            .bind(("album", metadata.album.clone()))
            .bind(("duration", metadata.duration))
            .bind(("thumbnail_url", metadata.thumbnail_url.clone()))
            .bind(("song_order", next_order))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Update playlist counts
        self.update_playlist_stats(playlist_id).await?;

        Ok(Song {
            id,
            playlist_id,
            youtube_id: metadata.youtube_id,
            youtube_url: data.youtube_url,
            title: metadata.title,
            artist: metadata.artist,
            album: metadata.album,
            duration: metadata.duration,
            cover_url: None,
            thumbnail_url: metadata.thumbnail_url,
            song_order: next_order,
            play_count: 0,
            last_played: None,
            created_at: now,
        })
    }

    async fn update_song(&self, id: Uuid, data: UpdateSong) -> Result<Option<Song>, DomainError> {
        let existing = match self.get_song(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let query = format!(
            "UPDATE song:`{}` SET title = $title, artist = $artist, album = $album, cover_url = $cover_url, song_order = $song_order",
            id
        );

        self.pool.client()
            .query(&query)
            .bind(("title", data.title.clone().unwrap_or(existing.title.clone())))
            .bind(("artist", data.artist.clone().or(existing.artist.clone())))
            .bind(("album", data.album.clone().or(existing.album.clone())))
            .bind(("cover_url", data.cover_url.clone().or(existing.cover_url.clone())))
            .bind(("song_order", data.song_order.unwrap_or(existing.song_order)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        self.get_song(id).await
    }

    async fn delete_song(&self, id: Uuid) -> Result<bool, DomainError> {
        let song = self.get_song(id).await?;
        
        let query = format!("DELETE song:`{}` RETURN BEFORE", id);
        let mut result = self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let deleted: Vec<SongRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        if let Some(s) = song {
            self.update_playlist_stats(s.playlist_id).await?;
        }

        Ok(!deleted.is_empty())
    }

    async fn increment_play_count(&self, id: Uuid) -> Result<(), DomainError> {
        let query = format!("UPDATE song:`{}` SET play_count += 1, last_played = time::now()", id);
        self.pool.client().query(&query).await
            .map_err(|e| DomainError::database(e.to_string()))?;
        Ok(())
    }

    async fn reorder_songs(&self, playlist_id: Uuid, song_ids: Vec<Uuid>) -> Result<(), DomainError> {
        for (index, song_id) in song_ids.iter().enumerate() {
            let query = format!("UPDATE song:`{}` SET song_order = $order", song_id);
            self.pool.client()
                .query(&query)
                .bind(("order", index as i32))
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;
        }
        Ok(())
    }

    async fn add_listening_entry(&self, entry: ListeningEntry) -> Result<(), DomainError> {
        self.pool.client()
            .query("CREATE listening_history SET song_id = $song_id, youtube_id = $youtube_id, title = $title, artist = $artist, listened_at = time::now()")
            .bind(("song_id", entry.song_id.to_string()))
            .bind(("youtube_id", entry.youtube_id))
            .bind(("title", entry.title))
            .bind(("artist", entry.artist))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        Ok(())
    }

    async fn get_listening_history(&self, limit: usize) -> Result<Vec<ListeningEntry>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM listening_history ORDER BY listened_at DESC LIMIT $limit")
            .bind(("limit", limit as i64))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let history: Vec<ListeningEntry> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(history)
    }

    async fn get_most_played_songs(&self, limit: usize) -> Result<Vec<Song>, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM song ORDER BY play_count DESC LIMIT $limit")
            .bind(("limit", limit as i64))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let records: Vec<SongRecord> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(records.into_iter().map(Song::from).collect())
    }

    async fn get_equalizer_settings(&self) -> Result<EqualizerSettings, DomainError> {
        let mut result = self.pool.client()
            .query("SELECT * FROM equalizer_settings LIMIT 1")
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let settings: Vec<EqualizerSettings> = result.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(settings.into_iter().next().unwrap_or_default())
    }

    async fn save_equalizer_settings(&self, settings: EqualizerSettings) -> Result<(), DomainError> {
        self.pool.client()
            .query("DELETE equalizer_settings")
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        self.pool.client()
            .query("CREATE equalizer_settings SET enabled = $enabled, preset = $preset, bands = $bands")
            .bind(("enabled", settings.enabled))
            .bind(("preset", settings.preset))
            .bind(("bands", settings.bands))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        Ok(())
    }
}
