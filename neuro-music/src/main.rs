use std::sync::Arc;
use tracing::info;

use neuro_common::{init_tracing, serve};

mod backend_client;
mod config;
mod cover_art;
mod downloader;
mod events;
mod handlers;
mod metadata_enricher;
mod models;
mod routes;
mod youtube;

use crate::backend_client::BackendClient;
use crate::config::Config;
use crate::cover_art::CoverArtService;
use crate::downloader::Downloader;
use crate::events::MusicEventBroadcaster;
use crate::metadata_enricher::MetadataEnricher;
use crate::youtube::YouTubeService;

pub struct AppState {
    pub client: BackendClient,
    pub config: Config,
    pub youtube: Arc<YouTubeService>,
    pub cover_art: CoverArtService,
    pub downloader: Arc<Downloader>,
    pub metadata_enricher: MetadataEnricher,
    pub event_broadcaster: Arc<MusicEventBroadcaster>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing("tachikoma_music=info,tower_http=debug");

    let config = Config::from_env();
    info!("Music service | port={} backend={}", config.port, config.backend_url);

    check_dependencies(&config).await?;

    let client = BackendClient::new(&config);
    let youtube = Arc::new(YouTubeService::new(&config));
    let cover_art = CoverArtService::new(&config);
    let downloader = Arc::new(Downloader::new(config.clone(), youtube.clone()));
    let metadata_enricher = MetadataEnricher::new(&config);

    if let Err(e) = downloader.ensure_downloads_dir().await {
        info!("Could not create downloads directory: {}", e);
    }

    let event_broadcaster = Arc::new(MusicEventBroadcaster::new());

    let state = Arc::new(AppState {
        client,
        config: config.clone(),
        youtube,
        cover_art,
        downloader,
        metadata_enricher,
        event_broadcaster,
    });

    let app = routes::create_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    serve(&addr, app, "Music").await?;
    Ok(())
}

async fn check_dependencies(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::process::Command;

    let ytdlp_check = Command::new(&config.ytdlp_path).arg("--version").output().await;
    match ytdlp_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("yt-dlp version: {}", version.trim());
        }
        _ => return Err("yt-dlp not found. Install with: pip install yt-dlp".into()),
    }

    let ffmpeg_check = Command::new(&config.ffmpeg_path).arg("-version").output().await;
    match ffmpeg_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("unknown");
            info!("ffmpeg: {}", first_line);
        }
        _ => return Err("ffmpeg not found. Install with: apt install ffmpeg".into()),
    }

    Ok(())
}