//! Configuration module for neuro-music

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub database_user: String,
    pub database_pass: String,
    pub database_namespace: String,
    pub database_name: String,
    pub ytdlp_path: String,
    pub ffmpeg_path: String,
    pub musicbrainz_api: String,
    pub coverart_api: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3002".to_string())
                .parse()
                .unwrap_or(3002),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string()),
            database_user: env::var("DATABASE_USER")
                .unwrap_or_else(|_| "root".to_string()),
            database_pass: env::var("DATABASE_PASS")
                .unwrap_or_else(|_| "root".to_string()),
            database_namespace: env::var("DATABASE_NAMESPACE")
                .unwrap_or_else(|_| "neuro".to_string()),
            database_name: env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "music".to_string()),
            ytdlp_path: env::var("YTDLP_PATH")
                .unwrap_or_else(|_| "yt-dlp".to_string()),
            ffmpeg_path: env::var("FFMPEG_PATH")
                .unwrap_or_else(|_| "ffmpeg".to_string()),
            musicbrainz_api: env::var("MUSICBRAINZ_API")
                .unwrap_or_else(|_| "https://musicbrainz.org/ws/2".to_string()),
            coverart_api: env::var("COVERART_API")
                .unwrap_or_else(|_| "https://coverartarchive.org".to_string()),
        }
    }
}
