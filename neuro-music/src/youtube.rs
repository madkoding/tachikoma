//! YouTube integration using yt-dlp

use regex::Regex;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::models::YouTubeMetadata;

pub struct YouTubeService {
    ytdlp_path: String,
    ffmpeg_path: String,
}

impl YouTubeService {
    pub fn new(config: &Config) -> Self {
        Self {
            ytdlp_path: config.ytdlp_path.clone(),
            ffmpeg_path: config.ffmpeg_path.clone(),
        }
    }

    /// Extract YouTube video ID from URL
    pub fn extract_video_id(url: &str) -> Option<String> {
        let patterns = [
            r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})",
            r"youtube\.com/v/([a-zA-Z0-9_-]{11})",
            r"youtube\.com/shorts/([a-zA-Z0-9_-]{11})",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(url) {
                    if let Some(id) = caps.get(1) {
                        return Some(id.as_str().to_string());
                    }
                }
            }
        }

        // Maybe it's already just an ID
        if url.len() == 11 && url.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Some(url.to_string());
        }

        None
    }

    /// Fetch video metadata using yt-dlp
    pub async fn get_metadata(&self, url: &str) -> Result<YouTubeMetadata, String> {
        let video_id = Self::extract_video_id(url)
            .ok_or_else(|| "Invalid YouTube URL".to_string())?;

        info!(video_id = %video_id, "Fetching YouTube metadata");

        let output = Command::new(&self.ytdlp_path)
            .args([
                "--dump-json",
                "--no-playlist",
                "--no-warnings",
                &format!("https://www.youtube.com/watch?v={}", video_id),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(stderr = %stderr, "yt-dlp failed");
            return Err(format!("yt-dlp failed: {}", stderr));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse yt-dlp output: {}", e))?;

        // Build YouTubeMetadata with fields matching backend expectations
        Ok(YouTubeMetadata {
            youtube_id: video_id,
            title: json["title"].as_str().unwrap_or("Unknown").to_string(),
            artist: json["uploader"].as_str().map(|s| s.to_string()),
            album: json["album"].as_str().map(|s| s.to_string()),
            duration: json["duration"].as_f64().map(|d| d as i64).or_else(|| json["duration"].as_i64()).unwrap_or(0),
            thumbnail_url: json["thumbnail"].as_str().map(|s| s.to_string()),
        })
    }

    /// Get direct audio stream URL
    pub async fn get_audio_stream_url(&self, video_id: &str) -> Result<StreamUrl, String> {
        info!(video_id = %video_id, "Fetching audio stream URL");

        let output = Command::new(&self.ytdlp_path)
            .args([
                "--format", "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio",
                "--get-url",
                "--no-playlist",
                "--no-warnings",
                &format!("https://www.youtube.com/watch?v={}", video_id),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(stderr = %stderr, "yt-dlp failed to get stream URL");
            return Err(format!("Failed to get stream URL: {}", stderr));
        }

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Get format info
        let format_output = Command::new(&self.ytdlp_path)
            .args([
                "--format", "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio",
                "--dump-json",
                "--no-playlist",
                "--no-warnings",
                &format!("https://www.youtube.com/watch?v={}", video_id),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .ok();

        let (format, bitrate, sample_rate) = format_output
            .and_then(|o| {
                let json_str = String::from_utf8_lossy(&o.stdout);
                serde_json::from_str::<serde_json::Value>(&json_str).ok()
            })
            .map(|json| {
                let format = json["ext"].as_str().unwrap_or("unknown").to_string();
                let bitrate = json["abr"].as_i64().unwrap_or(128) as i32;
                let sample_rate = json["asr"].as_i64().unwrap_or(44100) as i32;
                (format, bitrate, sample_rate)
            })
            .unwrap_or(("unknown".to_string(), 128, 44100));

        debug!(url = %url, format = %format, bitrate = %bitrate, "Got stream URL");

        Ok(StreamUrl {
            url,
            format,
            bitrate,
            sample_rate,
        })
    }

    /// Search YouTube for videos
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>, String> {
        info!(query = %query, max_results = %max_results, "Searching YouTube");

        // Request more results than needed since we'll filter some out
        let fetch_count = max_results * 3;

        let output = Command::new(&self.ytdlp_path)
            .args([
                "--dump-json",
                "--flat-playlist",
                "--no-warnings",
                &format!("ytsearch{}:{}", fetch_count, query),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Search failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results: Vec<SearchResult> = stdout
            .lines()
            .filter_map(|line| {
                serde_json::from_str::<serde_json::Value>(line).ok()
            })
            .filter_map(|json| {
                let title = json["title"].as_str()?.to_string();
                let duration = json["duration"].as_f64().map(|d| d as i64).or_else(|| json["duration"].as_i64()).unwrap_or(0);
                
                // Filter out lives (duration 0) and videos > 30 minutes (1800 seconds)
                if duration == 0 || duration > 1800 {
                    return None;
                }
                
                // Filter out compilations, MV, AMV, playlists, and long videos
                let title_lower = title.to_lowercase();
                let excluded_patterns = [
                    "compilacion", "compilación", "compilation", "compilaciones",
                    "mix ", " mix", "megamix", "mega mix",
                    " mv", "mv ", "(mv)", "[mv]",
                    " amv", "amv ", "(amv)", "[amv]",
                    "music video", "official video",
                    "1 hour", "1hour", "2 hour", "2hour",
                    "full album", "álbum completo", "album completo",
                    "playlist", "play list",
                    "live", "en vivo", "en directo", "concierto", "concert",
                ];
                
                for pattern in excluded_patterns {
                    if title_lower.contains(pattern) {
                        return None;
                    }
                }
                
                Some(SearchResult {
                    video_id: json["id"].as_str()?.to_string(),
                    title,
                    channel: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
                    duration,
                    thumbnail: format!(
                        "https://i.ytimg.com/vi/{}/mqdefault.jpg",
                        json["id"].as_str()?
                    ),
                    view_count: json["view_count"].as_f64().map(|v| v as i64).or_else(|| json["view_count"].as_i64()),
                })
            })
            .take(max_results)
            .collect();

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct StreamUrl {
    pub url: String,
    pub format: String,
    pub bitrate: i32,
    pub sample_rate: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub title: String,
    pub channel: String,
    pub duration: i64,
    pub thumbnail: String,
    pub view_count: Option<i64>,
}
