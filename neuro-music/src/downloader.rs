//! Song downloader module - Downloads songs to OGG format for offline playback

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::fs;
use tracing::{info, error, warn};

use crate::config::Config;
use crate::youtube::YouTubeService;

/// Manages song downloads in OGG format
pub struct Downloader {
    config: Config,
    youtube: Arc<YouTubeService>,
}

impl Downloader {
    pub fn new(config: Config, youtube: Arc<YouTubeService>) -> Self {
        Self { config, youtube }
    }

    /// Get the path where a song would be stored
    pub fn get_song_path(&self, youtube_id: &str) -> PathBuf {
        Path::new(&self.config.downloads_path).join(format!("{}.ogg", youtube_id))
    }

    /// Check if a song is already downloaded
    pub async fn is_downloaded(&self, youtube_id: &str) -> bool {
        let path = self.get_song_path(youtube_id);
        fs::metadata(&path).await.is_ok()
    }

    /// Ensure the downloads directory exists
    pub async fn ensure_downloads_dir(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.config.downloads_path).await
    }

    /// Download a song in OGG format with quality equivalent to 320kbps MP3
    /// Uses libopus at ~256kbps VBR which is perceptually equivalent to 320kbps MP3
    pub async fn download_song(&self, youtube_id: &str) -> Result<PathBuf, String> {
        // Check if already downloaded
        let output_path = self.get_song_path(youtube_id);
        if output_path.exists() {
            info!(youtube_id = %youtube_id, "Song already downloaded");
            return Ok(output_path);
        }

        // Ensure downloads directory exists
        self.ensure_downloads_dir().await
            .map_err(|e| format!("Failed to create downloads directory: {}", e))?;

        // Get stream URL from YouTube
        let stream_info = self.youtube.get_audio_stream_url(youtube_id).await
            .map_err(|e| format!("Failed to get stream URL: {}", e))?;

        info!(youtube_id = %youtube_id, "Starting download");

        // Create temp path for download
        let temp_path = output_path.with_extension("ogg.tmp");

        // Download and convert to OGG using ffmpeg
        // 256kbps Opus VBR is perceptually equivalent to 320kbps MP3
        let status = Command::new(&self.config.ffmpeg_path)
            .args([
                "-y",                           // Overwrite output
                "-i", &stream_info.url,         // Input from stream URL
                "-vn",                          // No video
                "-acodec", "libopus",           // Use Opus codec
                "-b:a", "256k",                 // 256kbps (equivalent to 320kbps MP3)
                "-vbr", "on",                   // Variable bitrate for better quality
                "-compression_level", "10",     // Best compression quality
                "-ar", "48000",                 // 48kHz sample rate
                "-ac", "2",                     // Stereo
                "-af", "loudnorm=I=-14:TP=-1:LRA=11",  // Normalize loudness
                "-f", "ogg",                    // OGG container
                temp_path.to_str().unwrap(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

        if !status.success() {
            // Clean up temp file
            let _ = fs::remove_file(&temp_path).await;
            return Err(format!("ffmpeg failed with status: {}", status));
        }

        // Rename temp file to final path
        fs::rename(&temp_path, &output_path).await
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        info!(youtube_id = %youtube_id, path = %output_path.display(), "Download complete");
        Ok(output_path)
    }

    /// Download a song in the background (spawns a task)
    pub fn download_in_background(self: Arc<Self>, youtube_id: String) {
        tokio::spawn(async move {
            match self.download_song(&youtube_id).await {
                Ok(path) => info!(youtube_id = %youtube_id, path = %path.display(), "Background download completed"),
                Err(e) => warn!(youtube_id = %youtube_id, error = %e, "Background download failed"),
            }
        });
    }

    /// Delete a downloaded song
    pub async fn delete_song(&self, youtube_id: &str) -> Result<(), std::io::Error> {
        let path = self.get_song_path(youtube_id);
        if path.exists() {
            fs::remove_file(&path).await?;
            info!(youtube_id = %youtube_id, "Deleted downloaded song");
        }
        Ok(())
    }
}
