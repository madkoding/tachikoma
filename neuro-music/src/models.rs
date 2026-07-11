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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_playlist() -> Playlist {
        Playlist {
            id: Uuid::new_v4(),
            name: "My Mix".to_string(),
            description: None,
            cover_url: None,
            is_suggestions: false,
            is_favorites: false,
            last_suggestions_update: None,
            shuffle: false,
            repeat_mode: RepeatMode::Off,
            song_count: 0,
            total_duration: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_song(playlist_id: Uuid) -> Song {
        Song {
            id: Uuid::new_v4(),
            playlist_id,
            youtube_id: "dQw4w9WgXcQ".to_string(),
            youtube_url: "https://youtu.be/dQw4w9WgXcQ".to_string(),
            title: "Test".to_string(),
            artist: None,
            album: None,
            duration: 200,
            cover_url: None,
            thumbnail_url: None,
            song_order: 0,
            play_count: 0,
            is_liked: false,
            last_played: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn playlist_serde_roundtrip() {
        let p = sample_playlist();
        let json = serde_json::to_string(&p).unwrap();
        let back: Playlist = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, back.id);
        assert_eq!(format!("{}", p.repeat_mode), format!("{}", back.repeat_mode));
    }

    #[test]
    fn song_serde_roundtrip() {
        let s = sample_song(Uuid::new_v4());
        let json = serde_json::to_string(&s).unwrap();
        let back: Song = serde_json::from_str(&json).unwrap();
        assert_eq!(s.youtube_id, back.youtube_id);
        assert_eq!(s.duration, back.duration);
    }

    #[test]
    fn repeat_mode_default_and_from_str() {
        assert!(matches!(RepeatMode::default(), RepeatMode::Off));
        let mode: RepeatMode = "one".parse().unwrap();
        assert!(matches!(mode, RepeatMode::One));
        assert!("bad".parse::<RepeatMode>().is_err());
    }

    #[test]
    fn repeat_mode_serde_snake_case() {
        let json = serde_json::to_string(&RepeatMode::All).unwrap();
        assert_eq!(json, "\"all\"");
        let back: RepeatMode = serde_json::from_str("\"one\"").unwrap();
        assert!(matches!(back, RepeatMode::One));
    }

    #[test]
    fn equalizer_settings_default_and_presets() {
        let def = EqualizerSettings::default();
        assert!(def.enabled);
        assert!(def.preset.is_none());
        assert_eq!(def.bands, [0.0; 16]);

        let bass = EqualizerSettings::preset_bass_boost();
        assert_eq!(bass.preset.as_deref(), Some("bass_boost"));
        assert_eq!(bass.bands[0], 8.0);

        let flat = EqualizerSettings::preset_flat();
        assert!(flat.preset.is_none());
    }
}


