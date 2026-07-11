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
        let base = neuro_common::Config::from_env(3002);
        Self {
            port: base.port,
            backend_url: base.backend_url,
            ytdlp_path: env::var("YTDLP_PATH").unwrap_or_else(|_| "yt-dlp".to_string()),
            ffmpeg_path: env::var("FFMPEG_PATH").unwrap_or_else(|_| "ffmpeg".to_string()),
            musicbrainz_api: env::var("MUSICBRAINZ_API")
                .unwrap_or_else(|_| "https://musicbrainz.org/ws/2".to_string()),
            coverart_api: env::var("COVERART_API")
                .unwrap_or_else(|_| "https://coverartarchive.org".to_string()),
            downloads_path: env::var("DOWNLOADS_PATH")
                .unwrap_or_else(|_| "/data/downloads".to_string()),
        }
    }
}