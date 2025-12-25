//! API Routes for neuro-music

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
        // Playlists
        .route("/playlists", get(handlers::list_playlists))
        .route("/playlists", post(handlers::create_playlist))
        .route("/playlists/:id", get(handlers::get_playlist))
        .route("/playlists/:id", patch(handlers::update_playlist))
        .route("/playlists/:id", delete(handlers::delete_playlist))
        
        // Songs
        .route("/playlists/:playlist_id/songs", post(handlers::add_song))
        .route("/playlists/:playlist_id/songs/:song_id", patch(handlers::update_song))
        .route("/playlists/:playlist_id/songs/:song_id", delete(handlers::delete_song))
        .route("/playlists/:playlist_id/reorder", post(handlers::reorder_songs))
        
        // Streaming
        .route("/stream/:song_id", get(handlers::stream_song))
        .route("/stream/:song_id/info", get(handlers::get_stream_info))
        
        // YouTube search & metadata
        .route("/youtube/search", get(handlers::search_youtube))
        .route("/youtube/metadata", get(handlers::get_youtube_metadata))
        
        // Cover art
        .route("/covers/search", get(handlers::search_cover))
        
        // Equalizer
        .route("/equalizer", get(handlers::get_equalizer))
        .route("/equalizer", put(handlers::update_equalizer))
        .route("/equalizer/preset", get(handlers::get_equalizer_preset))
        
        // History & Stats
        .route("/history", get(handlers::get_listening_history))
        .route("/stats/most-played", get(handlers::get_most_played));

    // Compose final router
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api/music", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
