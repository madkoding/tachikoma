//! =============================================================================
//! Neuro-Music Microservice
//! =============================================================================
//! Music streaming service with YouTube integration, equalizer, and playlists.
//! Uses neuro-backend as data layer via HTTP client.
//! =============================================================================

mod audio_dsp;
mod backend_client;
mod config;
mod cover_art;
mod handlers;
mod models;
mod routes;
mod youtube;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::backend_client::BackendClient;
use crate::config::Config;
use crate::cover_art::CoverArtService;
use crate::youtube::YouTubeService;

/// Application state shared across handlers
pub struct AppState {
    pub client: BackendClient,
    pub config: Config,
    pub youtube: YouTubeService,
    pub cover_art: CoverArtService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_music=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    
    info!("🎵 Neuro-Music Microservice");
    info!("============================");
    info!("Port: {}", config.port);
    info!("Backend URL: {}", config.backend_url);
    info!("yt-dlp: {}", config.ytdlp_path);
    info!("ffmpeg: {}", config.ffmpeg_path);

    // Check dependencies
    check_dependencies(&config).await?;

    // Create backend client
    let client = BackendClient::new(&config);
    info!("✅ Backend client initialized");

    // Create services
    let youtube = YouTubeService::new(&config);
    let cover_art = CoverArtService::new(&config);

    // Create app state
    let state = Arc::new(AppState { 
        client, 
        config: config.clone(),
        youtube,
        cover_art,
    });

    // Build router
    let app = routes::create_router(state);

    // Start server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🚀 Server listening on {}", addr);
    info!("  ▸ Health: GET /health");
    info!("  ▸ Playlists: /api/music/playlists/*");
    info!("  ▸ Streaming: /api/music/stream/:song_id");
    info!("  ▸ YouTube: /api/music/youtube/*");
    info!("  ▸ Equalizer: /api/music/equalizer");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_dependencies(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::process::Command;

    // Check yt-dlp
    let ytdlp_check = Command::new(&config.ytdlp_path)
        .arg("--version")
        .output()
        .await;

    match ytdlp_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("✅ yt-dlp version: {}", version.trim());
        }
        _ => {
            return Err("yt-dlp not found. Install with: pip install yt-dlp".into());
        }
    }

    // Check ffmpeg
    let ffmpeg_check = Command::new(&config.ffmpeg_path)
        .arg("-version")
        .output()
        .await;

    match ffmpeg_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("unknown");
            info!("✅ ffmpeg: {}", first_line);
        }
        _ => {
            return Err("ffmpeg not found. Install with: apt install ffmpeg".into());
        }
    }

    Ok(())
}
