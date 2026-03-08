//! Music events module for real-time updates via SSE
//! Provides event broadcasting for playlist and song changes

use serde::Serialize;
use tokio::sync::broadcast;

/// Capacity of the broadcast channel
const BROADCAST_CAPACITY: usize = 100;

/// Music events that can be broadcasted to clients
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum MusicEvent {
    // Playlist events
    PlaylistCreated(PlaylistEventData),
    PlaylistUpdated(PlaylistEventData),
    PlaylistDeleted { id: String },

    // Song events
    SongAdded {
        playlist_id: String,
        song: SongEventData,
    },
    SongRemoved {
        playlist_id: String,
        song_id: String,
    },
    SongUpdated(SongEventData),
    SongLiked {
        song_id: String,
        is_liked: bool,
    },

    // Download/streaming events
    DownloadStarted {
        youtube_id: String,
    },
    DownloadProgress {
        youtube_id: String,
        percent: u8,
    },
    DownloadComplete {
        youtube_id: String,
    },
    DownloadFailed {
        youtube_id: String,
        error: String,
    },

    // Heartbeat to keep connection alive
    Heartbeat,
}

/// Playlist data included in events
#[derive(Debug, Clone, Serialize)]
pub struct PlaylistEventData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub song_count: i32,
    pub total_duration: i64,
}

/// Song data included in events
#[derive(Debug, Clone, Serialize)]
pub struct SongEventData {
    pub id: String,
    pub playlist_id: String,
    pub youtube_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub cover_url: Option<String>,
    pub song_order: i32,
}

/// Broadcaster for music events using tokio broadcast channel
pub struct MusicEventBroadcaster {
    sender: broadcast::Sender<MusicEvent>,
}

impl MusicEventBroadcaster {
    /// Create a new broadcaster with default capacity
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self { sender }
    }

    /// Broadcast an event to all subscribers
    pub fn broadcast(&self, event: MusicEvent) {
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to receive events
    pub fn subscribe(&self) -> broadcast::Receiver<MusicEvent> {
        self.sender.subscribe()
    }
}

impl Default for MusicEventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create PlaylistEventData from a Playlist
impl From<&crate::models::Playlist> for PlaylistEventData {
    fn from(playlist: &crate::models::Playlist) -> Self {
        Self {
            id: playlist.id.to_string(),
            name: playlist.name.clone(),
            description: playlist.description.clone(),
            cover_url: playlist.cover_url.clone(),
            song_count: playlist.song_count,
            total_duration: playlist.total_duration,
        }
    }
}

/// Helper to create SongEventData from a Song
impl From<&crate::models::Song> for SongEventData {
    fn from(song: &crate::models::Song) -> Self {
        Self {
            id: song.id.to_string(),
            playlist_id: song.playlist_id.to_string(),
            youtube_id: song.youtube_id.clone(),
            title: song.title.clone(),
            artist: song.artist.clone(),
            album: song.album.clone(),
            duration: song.duration,
            cover_url: song.cover_url.clone(),
            song_order: song.song_order,
        }
    }
}
