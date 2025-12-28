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
            .ok_or_else(|| "URL de YouTube inválida".to_string())?;

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
            .map_err(|e| format!("Error ejecutando yt-dlp: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
            error!(stderr = %stderr, "yt-dlp failed");
            
            // Parse common yt-dlp errors and return user-friendly messages
            if stderr.contains("private video") || stderr.contains("video is private") {
                return Err("Este video es privado".to_string());
            }
            if stderr.contains("video unavailable") || stderr.contains("video is unavailable") {
                return Err("Este video no está disponible".to_string());
            }
            if stderr.contains("video has been removed") {
                return Err("Este video fue eliminado".to_string());
            }
            if stderr.contains("age-restricted") || stderr.contains("sign in to confirm your age") {
                return Err("Este video tiene restricción de edad".to_string());
            }
            if stderr.contains("not available in your country") || stderr.contains("blocked in your country") {
                return Err("Este video no está disponible en tu región".to_string());
            }
            if stderr.contains("copyright") {
                return Err("Este video fue bloqueado por derechos de autor".to_string());
            }
            if stderr.contains("live event") || stderr.contains("premieres in") {
                return Err("Este video es un evento en vivo o estreno pendiente".to_string());
            }
            if stderr.contains("requires payment") || stderr.contains("rental") {
                return Err("Este video requiere pago".to_string());
            }
            
            return Err("No se pudo obtener información del video".to_string());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|_| "Error procesando respuesta de YouTube".to_string())?;

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

    /// Clean up video title by removing unnecessary tags like (Official Video), 4K, etc.
    /// Also extracts artist if present in "Artist - Title" format
    fn clean_title(title: &str, channel: &str) -> (String, Option<String>) {
        use regex::Regex;
        
        let mut cleaned = title.to_string();
        let mut artist: Option<String> = None;
        
        // First, try to extract "Artist - Title" pattern
        // Common patterns: "Artist - Song", "Artist – Song", "Artist | Song"
        if let Ok(re) = Regex::new(r"^([^|\-–—]+?)\s*[|\-–—]\s*(.+)$") {
            if let Some(caps) = re.captures(&cleaned) {
                let potential_artist = caps.get(1).map(|m| m.as_str().trim().to_string());
                let potential_title = caps.get(2).map(|m| m.as_str().trim().to_string());
                
                if let (Some(art), Some(tit)) = (potential_artist, potential_title) {
                    // Only use if the potential artist looks like a name (not too long)
                    // and doesn't look like part of the title
                    if art.len() < 50 && !art.to_lowercase().contains("feat") {
                        artist = Some(art);
                        cleaned = tit;
                    }
                }
            }
        }
        
        // If no artist extracted, use channel name as fallback
        if artist.is_none() && !channel.is_empty() && channel != "Unknown" {
            // Clean channel name (remove "VEVO", "Official", etc.)
            let channel_clean = Regex::new(r"(?i)(vevo|official|music|records|channel)$")
                .map(|re| re.replace_all(channel, "").trim().to_string())
                .unwrap_or_else(|_| channel.to_string());
            if !channel_clean.is_empty() {
                artist = Some(channel_clean);
            }
        }
        
        // Patterns to remove from title (case insensitive)
        let patterns = [
            // Video quality tags
            r"(?i)\s*\(?\[?(official\s*)?(music\s*)?(video|audio|visualizer)\]?\)?",
            r"(?i)\s*\(?\[?(video\s*)?(oficial|official)\]?\)?",
            r"(?i)\s*\(?\[?4k\s*(remaster(ed)?|hd|video)?\]?\)?",
            r"(?i)\s*\(?\[?(hd|hq|uhd|1080p|720p)\]?\)?",
            r"(?i)\s*\(?\[?remaster(ed)?\]?\)?",
            // Lyrics tags
            r"(?i)\s*\(?\[?(with\s*)?lyrics?\]?\)?",
            r"(?i)\s*\(?\[?con\s*letra\]?\)?",
            r"(?i)\s*\(?\[?letra\]?\)?",
            r"(?i)\s*\(?\[?lyric\s*video\]?\)?",
            // Audio tags
            r"(?i)\s*\(?\[?audio\s*(only)?\]?\)?",
            r"(?i)\s*\(?\[?audio\s*oficial\]?\)?",
            // Version tags that are noise
            r"(?i)\s*\(?\[?m/?v\]?\)?",
            // Trailing separators
            r"\s*[|\-–—]\s*$",
        ];
        
        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }
        
        // Clean up extra whitespace and trim
        let whitespace_re = Regex::new(r"\s+").unwrap();
        cleaned = whitespace_re.replace_all(&cleaned, " ").to_string();
        
        (cleaned.trim().to_string(), artist)
    }

    /// Check if a query is for classical music or opera (allows longer duration)
    fn is_classical_or_opera(query: &str) -> bool {
        let query_lower = query.to_lowercase();
        let classical_patterns = [
            // Genres
            "opera", "ópera", "classical", "clásica", "clasica",
            "symphony", "sinfonía", "sinfonia",
            "concerto", "concierto",
            "sonata", "nocturne", "nocturno",
            "requiem", "cantata", "oratorio",
            // Classical composers
            "beethoven", "mozart", "bach", "chopin", "vivaldi",
            "tchaikovsky", "brahms", "handel", "haydn", "schubert",
            "debussy", "ravel", "liszt", "mendelssohn", "schumann",
            "rachmaninoff", "rachmaninov", "prokofiev", "shostakovich",
            "mahler", "bruckner", "dvorak", "dvořák", "grieg", "sibelius",
            // Opera composers
            "verdi", "puccini", "wagner", "rossini", "donizetti",
            "bellini", "bizet", "massenet", "strauss",
            // Famous opera singers
            "pavarotti", "callas", "domingo", "carreras", "netrebko",
            "bocelli", "brightman", "fleming", "gheorghiu",
            // Orchestra/ensemble terms
            "philharmonic", "filarmónica", "orquesta", "orchestra",
            "aria", "overture", "obertura", "quartet", "cuarteto",
        ];
        
        classical_patterns.iter().any(|pattern| query_lower.contains(pattern))
    }

    /// Search YouTube for videos
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>, String> {
        // Determine max duration based on query type
        // Classical/Opera: up to 30 min, Others: up to 12 min
        let is_classical = Self::is_classical_or_opera(query);
        let max_duration = if is_classical { 1800 } else { 720 }; // 30 min for classical, 12 min for others
        
        info!(
            query = %query, 
            max_results = %max_results, 
            is_classical = %is_classical,
            max_duration_sec = %max_duration,
            "Searching YouTube"
        );

        // Request more results than needed since we'll filter some out
        // Use higher multiplier to ensure enough results after filtering
        let fetch_count = (max_results * 15).min(100); // 15x multiplier, cap at 100

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
                
                // Filter out lives (duration 0) and videos exceeding max duration
                // max_duration is 10 min (600s) for regular music, 30 min (1800s) for classical/opera
                if duration == 0 || duration > max_duration {
                    return None;
                }
                
                // Filter out compilations, karaoke, covers, and multi-hour videos
                let title_lower = title.to_lowercase();
                let excluded_patterns = [
                    // Compilations and mixes
                    "compilacion", "compilación", "compilation",
                    "megamix", "mega mix",
                    // Multi-hour videos
                    "1 hour", "1hour", "2 hour", "2hour", "3 hour", "3hour",
                    // Full albums
                    "full album", "álbum completo", "album completo",
                    // Playlists
                    "playlist", "play list",
                    // Low quality versions
                    "karaoke", "8d audio",
                ];
                
                for pattern in excluded_patterns {
                    if title_lower.contains(pattern) {
                        return None;
                    }
                }
                
                // Get channel name
                let channel = json["uploader"].as_str().unwrap_or("Unknown").to_string();
                
                // Clean the title and extract artist
                let (clean_title, extracted_artist) = Self::clean_title(&title, &channel);
                
                Some(SearchResult {
                    video_id: json["id"].as_str()?.to_string(),
                    title: clean_title,
                    artist: extracted_artist,
                    channel,
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
    pub artist: Option<String>,
    pub channel: String,
    pub duration: i64,
    pub thumbnail: String,
    pub view_count: Option<i64>,
}
