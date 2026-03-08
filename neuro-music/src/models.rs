//! Data models for playlists and songs
//! No longer depends on SurrealDB - uses backend data layer

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Playlist
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub is_suggestions: bool,
    pub is_favorites: bool,
    pub last_suggestions_update: Option<DateTime<Utc>>,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
    pub song_count: i32,
    pub total_duration: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    #[default]
    Off,
    One,
    All,
}

impl std::fmt::Display for RepeatMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepeatMode::Off => write!(f, "off"),
            RepeatMode::One => write!(f, "one"),
            RepeatMode::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for RepeatMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(RepeatMode::Off),
            "one" => Ok(RepeatMode::One),
            "all" => Ok(RepeatMode::All),
            _ => Err(format!("Unknown repeat mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylist {
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePlaylist {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub shuffle: Option<bool>,
    pub repeat_mode: Option<RepeatMode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistWithSongs {
    #[serde(flatten)]
    pub playlist: Playlist,
    pub songs: Vec<Song>,
}

// =============================================================================
// Song
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: Uuid,
    pub playlist_id: Uuid,
    pub youtube_id: String,
    pub youtube_url: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub cover_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub song_order: i32,
    pub play_count: i32,
    pub is_liked: bool,
    pub last_played: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSong {
    pub youtube_url: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSong {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub cover_url: Option<String>,
    pub song_order: Option<i32>,
    pub is_liked: Option<bool>,
}

// =============================================================================
// YouTube Metadata (fetched via yt-dlp)
// =============================================================================

/// Metadata from yt-dlp (for deserialization)
#[derive(Debug, Clone, Deserialize)]
pub struct YtDlpMetadata {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
}

/// Metadata to send to backend (matches backend's YouTubeMetadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMetadata {
    pub youtube_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub thumbnail_url: Option<String>,
}

impl From<YtDlpMetadata> for YouTubeMetadata {
    fn from(m: YtDlpMetadata) -> Self {
        YouTubeMetadata {
            youtube_id: m.id,
            title: m.title,
            artist: m.uploader,
            album: m.album,
            duration: m.duration,
            thumbnail_url: m.thumbnail,
        }
    }
}

// =============================================================================
// Playback State
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub current_song_id: Option<Uuid>,
    pub current_playlist_id: Option<Uuid>,
    pub is_playing: bool,
    pub position: f64,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            current_song_id: None,
            current_playlist_id: None,
            is_playing: false,
            position: 0.0,
            volume: 0.8,
            shuffle: false,
            repeat_mode: RepeatMode::Off,
        }
    }
}

// =============================================================================
// Equalizer Settings
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerSettings {
    pub enabled: bool,
    pub preset: Option<String>,
    pub bands: [f32; 16],
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: None,
            bands: [0.0; 16],
        }
    }
}

impl EqualizerSettings {
    pub fn preset_flat() -> Self {
        Self::default()
    }

    pub fn preset_bass_boost() -> Self {
        Self {
            enabled: true,
            preset: Some("bass_boost".to_string()),
            bands: [8.0, 7.0, 6.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn preset_treble_boost() -> Self {
        Self {
            enabled: true,
            preset: Some("treble_boost".to_string()),
            bands: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 6.0, 7.0, 7.0, 8.0, 8.0],
        }
    }

    pub fn preset_vocal() -> Self {
        Self {
            enabled: true,
            preset: Some("vocal".to_string()),
            bands: [-2.0, -2.0, -1.0, 0.0, 3.0, 5.0, 5.0, 4.0, 3.0, 2.0, 0.0, -1.0, -1.0, -2.0, -2.0, -2.0],
        }
    }

    pub fn preset_rock() -> Self {
        Self {
            enabled: true,
            preset: Some("rock".to_string()),
            bands: [5.0, 4.0, 3.0, 1.0, -1.0, -2.0, 0.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 4.0, 4.0, 3.0],
        }
    }

    pub fn preset_electronic() -> Self {
        Self {
            enabled: true,
            preset: Some("electronic".to_string()),
            bands: [6.0, 5.0, 4.0, 2.0, 0.0, -2.0, -1.0, 0.0, 2.0, 4.0, 5.0, 5.0, 4.0, 4.0, 5.0, 6.0],
        }
    }

    pub fn preset_acoustic() -> Self {
        Self {
            enabled: true,
            preset: Some("acoustic".to_string()),
            bands: [3.0, 3.0, 2.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0, 1.0, 2.0, 3.0, 3.0, 2.0, 2.0, 2.0],
        }
    }
}

// =============================================================================
// Listening History
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListeningEntry {
    pub song_id: Uuid,
    pub youtube_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub listened_at: DateTime<Utc>,
}

// =============================================================================
// Search & Suggestions
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeSearchResult {
    pub youtube_id: String,
    pub title: String,
    pub channel: String,
    pub duration: i64,
    pub thumbnail: String,
    pub view_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSuggestion {
    pub youtube_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub thumbnail: String,
    pub duration: i64,
    pub reason: String,
    pub confidence: f32,
}

// =============================================================================
// API Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamInfo {
    pub song_id: Uuid,
    pub stream_url: String,
    pub format: String,
    pub bitrate: i32,
    pub sample_rate: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtSearchResult {
    pub source: String,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
