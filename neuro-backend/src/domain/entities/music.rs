//! =============================================================================
//! Music Domain Entities
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Repeat mode for playlists
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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

/// Playlist entity
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

/// Song entity
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

/// Playlist with songs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistWithSongs {
    #[serde(flatten)]
    pub playlist: Playlist,
    pub songs: Vec<Song>,
}

/// Request to create a playlist
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlaylist {
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    #[serde(default)]
    pub is_favorites: bool,
    #[serde(default)]
    pub is_suggestions: bool,
}

/// Request to update a playlist
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdatePlaylist {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub shuffle: Option<bool>,
    pub repeat_mode: Option<RepeatMode>,
}

/// Request to create a song
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSong {
    pub youtube_url: String,
    pub cover_url: Option<String>,
}

/// YouTube metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMetadata {
    pub youtube_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub thumbnail_url: Option<String>,
}

/// Request to update a song
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateSong {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub cover_url: Option<String>,
    pub song_order: Option<i32>,
    pub is_liked: Option<bool>,
}

/// Listening history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListeningEntry {
    pub id: Uuid,
    pub song_id: Uuid,
    pub youtube_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub listened_at: DateTime<Utc>,
}

/// Equalizer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerSettings {
    pub enabled: bool,
    pub preset: Option<String>,
    pub bands: Vec<f32>,
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: None,
            bands: vec![0.0; 16],
        }
    }
}
