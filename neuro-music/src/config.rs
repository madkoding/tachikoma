//! Configuration module for tachikoma-music

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub backend_url: String,
    pub ytdlp_path: String,
    pub ffmpeg_path: String,
    pub musicbrainz_api: String,
    pub coverart_api: String,
    pub downloads_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .or_else(|_| env::var("MUSIC_PORT"))
                .unwrap_or_else(|_| "3002".to_string())
                .parse()
                .unwrap_or(3002),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string()),
            ytdlp_path: env::var("YTDLP_PATH")
                .unwrap_or_else(|_| "yt-dlp".to_string()),
            ffmpeg_path: env::var("FFMPEG_PATH")
                .unwrap_or_else(|_| "ffmpeg".to_string()),
            musicbrainz_api: env::var("MUSICBRAINZ_API")
                .unwrap_or_else(|_| "https://musicbrainz.org/ws/2".to_string()),
            coverart_api: env::var("COVERART_API")
                .unwrap_or_else(|_| "https://coverartarchive.org".to_string()),
            downloads_path: env::var("DOWNLOADS_PATH")
                .unwrap_or_else(|_| "/data/downloads".to_string()),
        }
    }
}
