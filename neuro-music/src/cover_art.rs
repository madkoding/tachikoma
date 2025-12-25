//! Cover art fetching from MusicBrainz and Cover Art Archive

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::config::Config;

pub struct CoverArtService {
    client: Client,
    musicbrainz_api: String,
    coverart_api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtResult {
    pub url: String,
    pub source: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl CoverArtService {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .user_agent("NeuroOS-Music/1.0 (https://github.com/neuro-os)")
            .build()
            .unwrap_or_default();

        Self {
            client,
            musicbrainz_api: config.musicbrainz_api.clone(),
            coverart_api: config.coverart_api.clone(),
        }
    }

    /// Search for cover art by song title and artist
    pub async fn search_cover(&self, title: &str, artist: Option<&str>) -> Option<CoverArtResult> {
        // First try MusicBrainz
        if let Some(result) = self.search_musicbrainz(title, artist).await {
            return Some(result);
        }

        // Fallback to a generic cover
        None
    }

    /// Search MusicBrainz for release with cover art
    async fn search_musicbrainz(&self, title: &str, artist: Option<&str>) -> Option<CoverArtResult> {
        let query = match artist {
            Some(a) => format!("recording:\"{}\" AND artist:\"{}\"", title, a),
            None => format!("recording:\"{}\"", title),
        };

        let url = format!(
            "{}/recording?query={}&limit=5&fmt=json",
            self.musicbrainz_api,
            urlencoding::encode(&query)
        );

        debug!(url = %url, "Searching MusicBrainz");

        let response = self.client
            .get(&url)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            warn!("MusicBrainz search failed with status: {}", response.status());
            return None;
        }

        let json: serde_json::Value = response.json().await.ok()?;
        let recordings = json["recordings"].as_array()?;

        // Try to find a release with cover art
        for recording in recordings {
            if let Some(releases) = recording["releases"].as_array() {
                for release in releases {
                    if let Some(release_id) = release["id"].as_str() {
                        if let Some(cover) = self.get_cover_from_archive(release_id).await {
                            return Some(cover);
                        }
                    }
                }
            }
        }

        None
    }

    /// Get cover art from Cover Art Archive
    async fn get_cover_from_archive(&self, release_id: &str) -> Option<CoverArtResult> {
        let url = format!("{}/release/{}", self.coverart_api, release_id);

        debug!(release_id = %release_id, "Fetching from Cover Art Archive");

        let response = self.client
            .get(&url)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let json: serde_json::Value = response.json().await.ok()?;
        let images = json["images"].as_array()?;

        // Find front cover
        for image in images {
            let is_front = image["front"].as_bool().unwrap_or(false);
            if is_front {
                if let Some(url) = image["image"].as_str() {
                    info!(url = %url, "Found cover art");
                    return Some(CoverArtResult {
                        url: url.to_string(),
                        source: "musicbrainz".to_string(),
                        width: None,
                        height: None,
                    });
                }
            }
        }

        // Fallback to first image
        if let Some(first) = images.first() {
            if let Some(url) = first["image"].as_str() {
                return Some(CoverArtResult {
                    url: url.to_string(),
                    source: "musicbrainz".to_string(),
                    width: None,
                    height: None,
                });
            }
        }

        None
    }

    /// Get YouTube thumbnail as fallback cover
    pub fn get_youtube_thumbnail(video_id: &str, quality: ThumbnailQuality) -> String {
        let quality_str = match quality {
            ThumbnailQuality::Default => "default",
            ThumbnailQuality::Medium => "mqdefault",
            ThumbnailQuality::High => "hqdefault",
            ThumbnailQuality::Standard => "sddefault",
            ThumbnailQuality::MaxRes => "maxresdefault",
        };
        format!("https://i.ytimg.com/vi/{}/{}.jpg", video_id, quality_str)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThumbnailQuality {
    Default,    // 120x90
    Medium,     // 320x180
    High,       // 480x360
    Standard,   // 640x480
    MaxRes,     // 1280x720
}
