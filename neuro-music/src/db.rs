//! Database connection and repository for music service

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use uuid::Uuid;

use crate::config::Config;
use crate::models::*;

pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    pub async fn connect(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // SurrealDB ws driver expects just host:port
        let db_url = config.database_url
            .replace("ws://", "")
            .replace("wss://", "");
        
        tracing::info!("Connecting to database at: {}", db_url);
        
        let client = Surreal::new::<Ws>(&db_url).await?;
        
        tracing::info!("WebSocket connected, signing in...");
        
        client.signin(Root {
            username: &config.database_user,
            password: &config.database_pass,
        }).await?;
        
        tracing::info!("Signed in, selecting namespace/database...");

        client.use_ns(&config.database_namespace).use_db(&config.database_name).await?;

        // Initialize schema
        Self::init_schema(&client).await?;

        Ok(Self { client })
    }

    async fn init_schema(client: &Surreal<Client>) -> Result<(), Box<dyn std::error::Error>> {
        // SurrealDB 1.5.x does not support IF NOT EXISTS syntax
        client.query(r#"
            DEFINE TABLE playlist SCHEMAFULL;
            DEFINE FIELD name ON TABLE playlist TYPE string;
            DEFINE FIELD description ON TABLE playlist TYPE option<string>;
            DEFINE FIELD cover_url ON TABLE playlist TYPE option<string>;
            DEFINE FIELD is_suggestions ON TABLE playlist TYPE bool DEFAULT false;
            DEFINE FIELD shuffle ON TABLE playlist TYPE bool DEFAULT false;
            DEFINE FIELD repeat_mode ON TABLE playlist TYPE string DEFAULT 'off';
            DEFINE FIELD song_count ON TABLE playlist TYPE int DEFAULT 0;
            DEFINE FIELD total_duration ON TABLE playlist TYPE int DEFAULT 0;
            DEFINE FIELD created_at ON TABLE playlist TYPE datetime DEFAULT time::now();
            DEFINE FIELD updated_at ON TABLE playlist TYPE datetime DEFAULT time::now();

            DEFINE TABLE song SCHEMAFULL;
            DEFINE FIELD playlist_id ON TABLE song TYPE string;
            DEFINE FIELD youtube_id ON TABLE song TYPE string;
            DEFINE FIELD youtube_url ON TABLE song TYPE string;
            DEFINE FIELD title ON TABLE song TYPE string;
            DEFINE FIELD artist ON TABLE song TYPE option<string>;
            DEFINE FIELD album ON TABLE song TYPE option<string>;
            DEFINE FIELD duration ON TABLE song TYPE int DEFAULT 0;
            DEFINE FIELD cover_url ON TABLE song TYPE option<string>;
            DEFINE FIELD thumbnail_url ON TABLE song TYPE option<string>;
            DEFINE FIELD song_order ON TABLE song TYPE int DEFAULT 0;
            DEFINE FIELD play_count ON TABLE song TYPE int DEFAULT 0;
            DEFINE FIELD last_played ON TABLE song TYPE option<datetime>;
            DEFINE FIELD created_at ON TABLE song TYPE datetime DEFAULT time::now();
            DEFINE INDEX idx_song_playlist ON TABLE song COLUMNS playlist_id;

            DEFINE TABLE listening_history SCHEMAFULL;
            DEFINE FIELD song_id ON TABLE listening_history TYPE string;
            DEFINE FIELD youtube_id ON TABLE listening_history TYPE string;
            DEFINE FIELD title ON TABLE listening_history TYPE string;
            DEFINE FIELD artist ON TABLE listening_history TYPE option<string>;
            DEFINE FIELD listened_at ON TABLE listening_history TYPE datetime DEFAULT time::now();

            DEFINE TABLE equalizer_settings SCHEMAFULL;
            DEFINE FIELD enabled ON TABLE equalizer_settings TYPE bool DEFAULT true;
            DEFINE FIELD preset ON TABLE equalizer_settings TYPE option<string>;
            DEFINE FIELD bands ON TABLE equalizer_settings TYPE array DEFAULT [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        "#).await?;

        Ok(())
    }

    // ==========================================================================
    // Playlist CRUD
    // ==========================================================================

    pub async fn get_all_playlists(&self) -> Result<Vec<Playlist>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM playlist ORDER BY created_at DESC")
            .await?;
        let records: Vec<PlaylistRecord> = result.take(0)?;
        Ok(records.into_iter().map(Playlist::from).collect())
    }

    pub async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("SELECT * FROM playlist:`{}`", id);
        let mut result = self.client
            .query(&query)
            .await?;
        let records: Vec<PlaylistRecord> = result.take(0)?;
        Ok(records.into_iter().next().map(Playlist::from))
    }

    pub async fn get_playlist_with_songs(&self, id: Uuid) -> Result<Option<PlaylistWithSongs>, Box<dyn std::error::Error + Send + Sync>> {
        let playlist = match self.get_playlist(id).await? {
            Some(p) => p,
            None => return Ok(None),
        };
        let songs = self.get_songs_by_playlist(id).await?;
        Ok(Some(PlaylistWithSongs { playlist, songs }))
    }

    pub async fn create_playlist(&self, data: CreatePlaylist) -> Result<Playlist, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S%.9fZ").to_string();
        
        // Use direct record ID syntax and time::datetime() for proper datetime conversion
        let query = format!(
            "CREATE playlist:`{}` SET name = $name, description = $description, cover_url = $cover_url, is_suggestions = false, shuffle = false, repeat_mode = 'off', song_count = 0, total_duration = 0, created_at = <datetime>$now, updated_at = <datetime>$now RETURN AFTER",
            id
        );

        let mut result = self.client
            .query(&query)
            .bind(("name", data.name.clone()))
            .bind(("description", data.description.clone()))
            .bind(("cover_url", data.cover_url.clone()))
            .bind(("now", now_str))
            .await?;

        // Verify the record was created
        let created: Option<PlaylistRecord> = result.take(0)?;
        if created.is_none() {
            return Err("Failed to create playlist - no record returned".into());
        }

        Ok(Playlist {
            id,
            name: data.name,
            description: data.description,
            cover_url: data.cover_url,
            is_suggestions: false,
            shuffle: false,
            repeat_mode: RepeatMode::Off,
            song_count: 0,
            total_duration: 0,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn update_playlist(&self, id: Uuid, data: UpdatePlaylist) -> Result<Option<Playlist>, Box<dyn std::error::Error + Send + Sync>> {
        let existing = match self.get_playlist(id).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        let now = chrono::Utc::now();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S%.9fZ").to_string();
        let name = data.name.unwrap_or(existing.name.clone());
        let description = data.description.or(existing.description.clone());
        let cover_url = data.cover_url.or(existing.cover_url.clone());
        let shuffle = data.shuffle.unwrap_or(existing.shuffle);
        let repeat_mode = data.repeat_mode.unwrap_or(existing.repeat_mode.clone());

        let query = format!(
            "UPDATE playlist:`{}` SET name = $name, description = $description, cover_url = $cover_url, shuffle = $shuffle, repeat_mode = $repeat_mode, updated_at = <datetime>$now",
            id
        );
        self.client
            .query(&query)
            .bind(("name", name.clone()))
            .bind(("description", description.clone()))
            .bind(("cover_url", cover_url.clone()))
            .bind(("shuffle", shuffle))
            .bind(("repeat_mode", repeat_mode.to_string()))
            .bind(("now", now_str))
            .await?;

        Ok(Some(Playlist {
            id: existing.id,
            name,
            description,
            cover_url,
            is_suggestions: existing.is_suggestions,
            shuffle,
            repeat_mode,
            song_count: existing.song_count,
            total_duration: existing.total_duration,
            created_at: existing.created_at,
            updated_at: now,
        }))
    }

    pub async fn delete_playlist(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Delete all songs in playlist first
        self.client
            .query("DELETE song WHERE playlist_id = $playlist_id")
            .bind(("playlist_id", id.to_string()))
            .await?;

        let query = format!("DELETE playlist:`{}` RETURN BEFORE", id);
        let mut result = self.client
            .query(&query)
            .await?;
        let deleted: Vec<PlaylistRecord> = result.take(0)?;
        Ok(!deleted.is_empty())
    }

    // ==========================================================================
    // Song CRUD
    // ==========================================================================

    pub async fn get_songs_by_playlist(&self, playlist_id: Uuid) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM song WHERE playlist_id = $playlist_id ORDER BY song_order ASC")
            .bind(("playlist_id", playlist_id.to_string()))
            .await?;
        let records: Vec<SongRecord> = result.take(0)?;
        Ok(records.into_iter().map(Song::from).collect())
    }

    pub async fn get_song(&self, id: Uuid) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("SELECT * FROM song:`{}`", id);
        let mut result = self.client
            .query(&query)
            .await?;
        let records: Vec<SongRecord> = result.take(0)?;
        Ok(records.into_iter().next().map(Song::from))
    }

    pub async fn get_song_by_youtube_id(&self, youtube_id: &str, playlist_id: Uuid) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM song WHERE youtube_id = $youtube_id AND playlist_id = $playlist_id LIMIT 1")
            .bind(("youtube_id", youtube_id.to_string()))
            .bind(("playlist_id", playlist_id.to_string()))
            .await?;
        let records: Vec<SongRecord> = result.take(0)?;
        Ok(records.into_iter().next().map(Song::from))
    }

    pub async fn create_song(&self, playlist_id: Uuid, data: CreateSong, metadata: YouTubeMetadata) -> Result<Song, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Get current max order
        let mut result = self.client
            .query("SELECT song_order FROM song WHERE playlist_id = $playlist_id ORDER BY song_order DESC LIMIT 1")
            .bind(("playlist_id", playlist_id.to_string()))
            .await?;
        let max_order: Vec<serde_json::Value> = result.take(0)?;
        let next_order = max_order.first()
            .and_then(|v| v.get("song_order"))
            .and_then(|v| v.as_i64())
            .map(|n| n as i32 + 1)
            .unwrap_or(0);

        let query = format!(
            "CREATE song:`{}` SET playlist_id = $playlist_id, youtube_id = $youtube_id, youtube_url = $youtube_url, title = $title, artist = $artist, album = $album, duration = $duration, cover_url = $cover_url, thumbnail_url = $thumbnail_url, song_order = $song_order, play_count = 0, created_at = <datetime>$now",
            id
        );
        let now_str = now.format("%Y-%m-%dT%H:%M:%S%.9fZ").to_string();
        self.client
            .query(&query)
            .bind(("playlist_id", playlist_id.to_string()))
            .bind(("youtube_id", metadata.id.clone()))
            .bind(("youtube_url", data.youtube_url.clone()))
            .bind(("title", metadata.title.clone()))
            .bind(("artist", metadata.uploader.clone()))
            .bind(("album", None::<String>))
            .bind(("duration", metadata.duration as i32))
            .bind(("cover_url", data.cover_url.clone()))
            .bind(("thumbnail_url", metadata.thumbnail.clone()))
            .bind(("song_order", next_order))
            .bind(("now", now_str))
            .await?;

        // Update playlist counts
        self.update_playlist_stats(playlist_id).await?;

        Ok(Song {
            id,
            playlist_id,
            youtube_id: metadata.id,
            youtube_url: data.youtube_url,
            title: metadata.title,
            artist: metadata.uploader,
            album: None,
            duration: metadata.duration,
            cover_url: data.cover_url,
            thumbnail_url: metadata.thumbnail,
            song_order: next_order,
            play_count: 0,
            last_played: None,
            created_at: now,
        })
    }

    pub async fn update_song(&self, id: Uuid, data: UpdateSong) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let existing = match self.get_song(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let title = data.title.unwrap_or(existing.title.clone());
        let artist = data.artist.or(existing.artist.clone());
        let album = data.album.or(existing.album.clone());
        let cover_url = data.cover_url.or(existing.cover_url.clone());

        let query = format!(
            "UPDATE song:`{}` SET title = $title, artist = $artist, album = $album, cover_url = $cover_url",
            id
        );
        self.client
            .query(&query)
            .bind(("title", title.clone()))
            .bind(("artist", artist.clone()))
            .bind(("album", album.clone()))
            .bind(("cover_url", cover_url.clone()))
            .await?;

        Ok(Some(Song {
            id: existing.id,
            playlist_id: existing.playlist_id,
            youtube_id: existing.youtube_id,
            youtube_url: existing.youtube_url,
            title,
            artist,
            album,
            duration: existing.duration,
            cover_url,
            thumbnail_url: existing.thumbnail_url,
            song_order: existing.song_order,
            play_count: existing.play_count,
            last_played: existing.last_played,
            created_at: existing.created_at,
        }))
    }

    pub async fn delete_song(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let song = self.get_song(id).await?;
        
        let query = format!("DELETE song:`{}` RETURN BEFORE", id);
        let mut result = self.client
            .query(&query)
            .await?;
        let deleted: Vec<SongRecord> = result.take(0)?;

        if let Some(s) = song {
            self.update_playlist_stats(s.playlist_id).await?;
        }

        Ok(!deleted.is_empty())
    }

    pub async fn increment_play_count(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("UPDATE song:`{}` SET play_count += 1, last_played = time::now()", id);
        self.client
            .query(&query)
            .await?;
        Ok(())
    }

    pub async fn reorder_songs(&self, playlist_id: Uuid, song_ids: Vec<Uuid>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (index, song_id) in song_ids.iter().enumerate() {
            let query = format!("UPDATE song:`{}` SET song_order = $order", song_id);
            self.client
                .query(&query)
                .bind(("order", index as i32))
                .await?;
        }
        Ok(())
    }

    async fn update_playlist_stats(&self, playlist_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT count() as song_count, math::sum(duration) as total_duration FROM song WHERE playlist_id = $playlist_id GROUP ALL")
            .bind(("playlist_id", playlist_id.to_string()))
            .await?;
        
        let stats: Vec<serde_json::Value> = result.take(0)?;
        let (song_count, total_duration) = stats.first()
            .map(|v| {
                let count = v.get("song_count").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                let duration = v.get("total_duration").and_then(|d| d.as_i64()).unwrap_or(0) as i32;
                (count, duration)
            })
            .unwrap_or((0, 0));

        let query = format!(
            "UPDATE playlist:`{}` SET song_count = $count, total_duration = $duration",
            playlist_id
        );
        self.client
            .query(&query)
            .bind(("count", song_count))
            .bind(("duration", total_duration))
            .await?;

        Ok(())
    }

    // ==========================================================================
    // Listening History & Equalizer
    // ==========================================================================

    pub async fn add_listening_entry(&self, entry: ListeningEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .query("CREATE listening_history SET song_id = $song_id, youtube_id = $youtube_id, title = $title, artist = $artist, listened_at = time::now()")
            .bind(("song_id", entry.song_id.to_string()))
            .bind(("youtube_id", entry.youtube_id))
            .bind(("title", entry.title))
            .bind(("artist", entry.artist))
            .await?;
        Ok(())
    }

    pub async fn get_listening_history(&self, limit: usize) -> Result<Vec<ListeningEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM listening_history ORDER BY listened_at DESC LIMIT $limit")
            .bind(("limit", limit as i64))
            .await?;
        let history: Vec<ListeningEntry> = result.take(0)?;
        Ok(history)
    }

    pub async fn get_most_played_songs(&self, limit: usize) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM song ORDER BY play_count DESC LIMIT $limit")
            .bind(("limit", limit as i64))
            .await?;
        let records: Vec<SongRecord> = result.take(0)?;
        Ok(records.into_iter().map(Song::from).collect())
    }

    pub async fn get_equalizer_settings(&self) -> Result<EqualizerSettings, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = self.client
            .query("SELECT * FROM equalizer_settings LIMIT 1")
            .await?;
        let settings: Vec<EqualizerSettings> = result.take(0)?;
        Ok(settings.into_iter().next().unwrap_or_default())
    }

    pub async fn save_equalizer_settings(&self, settings: EqualizerSettings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.query("DELETE equalizer_settings").await?;
        
        self.client
            .query("CREATE equalizer_settings SET enabled = $enabled, preset = $preset, bands = $bands")
            .bind(("enabled", settings.enabled))
            .bind(("preset", settings.preset))
            .bind(("bands", settings.bands))
            .await?;
        
        Ok(())
    }
}
