//! =============================================================================
//! API Routes
//! =============================================================================
//! Defines all HTTP routes and creates the Axum router.
//! =============================================================================

use axum::{
    middleware,
    routing::{any, delete, get, patch, post, put},
    Router,
    http::header,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::infrastructure::api::handlers;
use crate::infrastructure::api::middleware::{logging_middleware, request_id_middleware};
use crate::AppState;

/// =============================================================================
/// Create the API router
/// =============================================================================
/// Builds the complete Axum router with all routes and middleware.
/// =============================================================================
pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS configuration - permissive for development
    // Allows SSE connections from remote hosts
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([
            header::CONTENT_TYPE,
            header::CACHE_CONTROL,
            header::CONNECTION,
        ]);

    // Build routes
    let api_routes = Router::new()
        // Health & System
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::readiness_check))
        .route("/live", get(handlers::liveness_check))
        .route("/models", get(handlers::list_models))
        .route("/system/info", get(handlers::system_info))
        
        // Chat
        .route("/chat", post(handlers::send_message))
        .route("/chat/stream", post(handlers::stream_message))
        .route("/chat/conversations", get(handlers::list_conversations))
        .route("/chat/conversations/:id", get(handlers::get_conversation))
        .route("/chat/conversations/:id", delete(handlers::delete_conversation))
        
        // Voice Synthesis
        .route("/voice/status", get(handlers::voice_status))
        .route("/voice/synthesize", post(handlers::synthesize_voice))
        .route("/voice/stream", post(handlers::stream_voice))
        
        // Memories
        .route("/memories", get(handlers::list_memories))
        .route("/memories", post(handlers::create_memory))
        .route("/memories/search", post(handlers::search_memories))
        .route("/memories/:id", get(handlers::get_memory))
        .route("/memories/:id", patch(handlers::update_memory))
        .route("/memories/:id", delete(handlers::delete_memory))
        .route("/memories/:id/relations", get(handlers::get_memory_relations))
        .route("/memories/:id/related", get(handlers::get_related_memories))
        .route("/memories/relations", post(handlers::create_relation))
        .route("/memories/:from_id/relations/:to_id", delete(handlers::delete_relation))
        
        // Graph Admin
        .route("/admin/graph/stats", get(handlers::get_graph_stats))
        .route("/admin/graph/export", get(handlers::export_graph))
        .route("/admin/graph/events", get(handlers::subscribe_graph_events))
        
        // Agent Tools
        .route("/agent/search", post(handlers::web_search))
        .route("/agent/search/categories", get(handlers::get_search_categories))
        .route("/agent/execute", post(handlers::execute_command))
        .route("/agent/commands", get(handlers::get_allowed_commands))
        
        // =====================================================================
        // LLM Gateway - ONLY interface to Ollama
        // =====================================================================
        // All microservices (neuro-chat, neuro-memory, etc.) must use these
        // endpoints instead of connecting directly to Ollama.
        // =====================================================================
        .route("/llm/health", get(handlers::llm_health))
        .route("/llm/embed", post(handlers::llm_embed))
        .route("/llm/embed/batch", post(handlers::llm_embed_batch))
        .route("/llm/chat", post(handlers::llm_chat))
        .route("/llm/chat/stream", post(handlers::llm_chat_stream))
        .route("/llm/chat/speculative/stream", post(handlers::llm_speculative_stream))
        .route("/llm/generate", post(handlers::llm_generate))
        
        // =====================================================================
        // Data Layer - Direct Database Access for Microservices
        // =====================================================================
        // Checklists data layer
        .route("/data/checklists", get(handlers::list_checklists))
        .route("/data/checklists", post(handlers::create_checklist))
        .route("/data/checklists/:id", get(handlers::get_checklist))
        .route("/data/checklists/:id", patch(handlers::update_checklist))
        .route("/data/checklists/:id", delete(handlers::delete_checklist))
        .route("/data/checklists/:id/items", get(handlers::list_checklist_items))
        .route("/data/checklists/:id/items", post(handlers::create_checklist_item))
        .route("/data/checklists/items/:id", patch(handlers::update_checklist_item))
        .route("/data/checklists/items/:id/toggle", post(handlers::toggle_checklist_item))
        .route("/data/checklists/items/:id", delete(handlers::delete_checklist_item))
        
        // Music data layer - Playlists
        .route("/data/playlists", get(handlers::list_playlists))
        .route("/data/playlists", post(handlers::create_playlist))
        .route("/data/playlists/:id", get(handlers::get_playlist))
        .route("/data/playlists/:id", patch(handlers::update_playlist))
        .route("/data/playlists/:id", delete(handlers::delete_playlist))
        .route("/data/playlists/:id/songs", get(handlers::list_playlist_songs))
        .route("/data/playlists/:id/songs", post(handlers::create_song))
        .route("/data/playlists/:id/reorder", post(handlers::reorder_songs))
        .route("/data/playlists/:id/suggestions-timestamp", post(handlers::update_suggestions_timestamp))
        
        // Music data layer - Songs
        .route("/data/songs/liked", get(handlers::get_liked_songs))
        .route("/data/songs/by-youtube-id", get(handlers::get_song_by_youtube_id))
        .route("/data/songs/:id", get(handlers::get_song))
        .route("/data/songs/:id", patch(handlers::update_song))
        .route("/data/songs/:id", delete(handlers::delete_song))
        .route("/data/songs/:id/play", post(handlers::increment_song_play_count))
        
        // Music data layer - History & Stats
        .route("/data/music/history", get(handlers::get_listening_history))
        .route("/data/music/history", post(handlers::add_listening_entry))
        .route("/data/music/top-songs", get(handlers::get_most_played_songs))
        .route("/data/music/equalizer", get(handlers::get_equalizer_settings))
        .route("/data/music/equalizer", put(handlers::save_equalizer_settings))
        
        // Kanban data layer - Boards
        .route("/data/kanban/boards", get(handlers::list_boards))
        .route("/data/kanban/boards", post(handlers::create_board))
        .route("/data/kanban/boards/:id", get(handlers::get_board))
        .route("/data/kanban/boards/:id", patch(handlers::update_board))
        .route("/data/kanban/boards/:id", delete(handlers::delete_board))
        // Kanban data layer - Columns
        .route("/data/kanban/boards/:board_id/columns", post(handlers::create_column))
        .route("/data/kanban/columns/:column_id", patch(handlers::update_column))
        .route("/data/kanban/columns/:column_id/reorder", post(handlers::reorder_column))
        .route("/data/kanban/columns/:column_id", delete(handlers::delete_column))
        // Kanban data layer - Cards
        .route("/data/kanban/columns/:column_id/cards", post(handlers::create_card))
        .route("/data/kanban/cards/:card_id", patch(handlers::update_card))
        .route("/data/kanban/cards/:card_id/move", post(handlers::move_card))
        .route("/data/kanban/cards/:card_id", delete(handlers::delete_card))
        
        // =====================================================================
        // API Gateway - Proxy to Microservices
        // =====================================================================
        // Checklists microservice proxy
        .route("/checklists", any(handlers::proxy_checklists))
        .route("/checklists/*path", any(handlers::proxy_checklists))
        // Music microservice proxy
        .route("/music", any(handlers::proxy_music))
        .route("/music/*path", any(handlers::proxy_music))
        // Pomodoro microservice proxy
        .route("/pomodoro", any(handlers::proxy_pomodoro))
        .route("/pomodoro/*path", any(handlers::proxy_pomodoro))
        // Kanban microservice proxy
        .route("/kanban", any(handlers::proxy_kanban))
        .route("/kanban/*path", any(handlers::proxy_kanban))
        // Note microservice proxy
        .route("/notes", any(handlers::proxy_note))
        .route("/notes/*path", any(handlers::proxy_note))
        // Docs microservice proxy
        .route("/docs", any(handlers::proxy_docs))
        .route("/docs/*path", any(handlers::proxy_docs))
        // Calendar microservice proxy
        .route("/calendar", any(handlers::proxy_calendar))
        .route("/calendar/*path", any(handlers::proxy_calendar))
        // Image microservice proxy
        .route("/images", any(handlers::proxy_image))
        .route("/images/*path", any(handlers::proxy_image))
        // Voice microservice proxy
        .route("/voice/proxy", any(handlers::proxy_voice))
        .route("/voice/proxy/*path", any(handlers::proxy_voice));

    // Compose final router
    Router::new()
        .nest("/api", api_routes)
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(logging_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
