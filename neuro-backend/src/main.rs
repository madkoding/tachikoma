//! =============================================================================
//! NEURO-OS Backend - Main Entry Point
//! =============================================================================
//! This is the main entry point for the NEURO-OS backend server.
//! It initializes all infrastructure components and starts the Axum HTTP server.
//! 
//! # Architecture
//! 
//! The application follows Hexagonal Architecture (Ports & Adapters):
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      INFRASTRUCTURE LAYER                       │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐│
//! │  │  Axum    │  │ SurrealDB│  │  Ollama  │  │    Searxng       ││
//! │  │  HTTP    │  │  Client  │  │  Client  │  │    Client        ││
//! │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────────┬─────────┘│
//! └───────┼─────────────┼────────────┼──────────────────┼──────────┘
//!         │             │            │                  │
//! ┌───────┴─────────────┴────────────┴──────────────────┴──────────┐
//! │                      APPLICATION LAYER                          │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
//! │  │ ChatService  │  │MemoryService │  │   AgentOrchestrator  │  │
//! │  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
//! └─────────┼─────────────────┼─────────────────────┼──────────────┘
//!           │                 │                     │
//! ┌─────────┴─────────────────┴─────────────────────┴──────────────┐
//! │                        DOMAIN LAYER                             │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
//! │  │MemoryNode    │  │   Relation   │  │      Agent           │  │
//! │  │ChatMessage   │  │   ModelTier  │  │      Tools           │  │
//! │  └──────────────┘  └──────────────┘  └──────────────────────┘  │
//! └────────────────────────────────────────────────────────────────┘
//! ```
//! =============================================================================

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

mod application;
mod domain;
mod infrastructure;

use crate::application::services::{
    AgentOrchestrator, ChatService, MemoryService, ModelManager,
};
use crate::domain::ports::{
    command_executor::CommandExecutor,
    llm_provider::LlmProvider,
    memory_repository::MemoryRepository,
    search_provider::SearchProvider,
};
use crate::infrastructure::{
    api::{create_router, handlers::system::init_start_time},
    config::Config,
    database::{DatabasePool, SurrealDbRepository},
    services::{OllamaClient, SafeCommandExecutor, SearxngClient},
};

/// =============================================================================
/// Application State
/// =============================================================================
/// Shared state containing all services and clients.
/// Passed to all HTTP handlers via Axum's state extractor.
/// =============================================================================
pub struct AppState {
    /// Database connection pool
    pub database_pool: DatabasePool,
    /// LLM provider (Ollama)
    pub llm_provider: Arc<dyn LlmProvider + Send + Sync>,
    /// Search provider (Searxng)
    pub search_provider: Arc<dyn SearchProvider + Send + Sync>,
    /// Command executor
    pub command_executor: Arc<dyn CommandExecutor + Send + Sync>,
    /// Memory service
    pub memory_service: Arc<MemoryService>,
    /// Chat service
    pub chat_service: Arc<ChatService>,
    /// Model manager
    pub model_manager: Arc<ModelManager>,
}

/// =============================================================================
/// Main Entry Point
/// =============================================================================
/// Initializes the NEURO-OS backend server with all required services.
/// 
/// # Initialization Order
/// 1. Load configuration from environment
/// 2. Initialize tracing/logging
/// 3. Connect to SurrealDB
/// 4. Initialize Ollama client
/// 5. Initialize Searxng client
/// 6. Create application services
/// 7. Create application state
/// 8. Start Axum HTTP server
/// =============================================================================
#[tokio::main]
async fn main() -> Result<()> {
    // -------------------------------------------------------------------------
    // Initialize start time for uptime tracking
    // -------------------------------------------------------------------------
    init_start_time();

    // -------------------------------------------------------------------------
    // Load environment variables from .env file (if present)
    // -------------------------------------------------------------------------
    dotenvy::dotenv().ok();

    // -------------------------------------------------------------------------
    // Load application configuration from environment
    // -------------------------------------------------------------------------
    let config = Config::from_env()?;

    // -------------------------------------------------------------------------
    // Initialize structured logging with tracing
    // -------------------------------------------------------------------------
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,neuro_backend=debug".into()),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    info!("🧠 NEURO-OS Backend starting...");
    info!("📋 Configuration loaded");

    // -------------------------------------------------------------------------
    // Initialize database connection pool
    // -------------------------------------------------------------------------
    info!("🔌 Connecting to SurrealDB...");
    let database_pool = DatabasePool::new(&config.database).await?;
    info!("✅ SurrealDB connection established");

    // -------------------------------------------------------------------------
    // Initialize external service clients
    // -------------------------------------------------------------------------
    info!("🤖 Initializing Ollama client...");
    let ollama_client = OllamaClient::new(config.ollama.clone());
    let llm_provider: Arc<dyn LlmProvider + Send + Sync> = Arc::new(ollama_client);
    info!("✅ Ollama client ready");

    info!("🔍 Initializing Searxng client...");
    let searxng_client = SearxngClient::new(config.searxng.clone());
    let search_provider: Arc<dyn SearchProvider + Send + Sync> = Arc::new(searxng_client);
    info!("✅ Searxng client ready");

    info!("🔐 Initializing command executor...");
    let command_executor: Arc<dyn CommandExecutor + Send + Sync> = 
        Arc::new(SafeCommandExecutor::new());
    info!("✅ Command executor ready");

    // -------------------------------------------------------------------------
    // Create memory repository
    // -------------------------------------------------------------------------
    info!("📦 Initializing memory repository...");
    let surreal_repository = Arc::new(SurrealDbRepository::new(database_pool.clone()));
    let memory_repository: Arc<dyn MemoryRepository + Send + Sync> = surreal_repository.clone();
    info!("✅ Memory repository ready");

    // -------------------------------------------------------------------------
    // Create application services
    // -------------------------------------------------------------------------
    info!("⚙️ Creating application services...");
    
    let model_manager = Arc::new(ModelManager::new(llm_provider.clone()));

    let memory_service = Arc::new(MemoryService::new(
        memory_repository,
        llm_provider.clone(),
    ));

    let agent_orchestrator = Arc::new(AgentOrchestrator::new(
        memory_service.clone(),
        model_manager.clone(),
        llm_provider.clone(),
        search_provider.clone(),
        command_executor.clone(),
    ));

    let chat_service = Arc::new(ChatService::new(
        agent_orchestrator,
        memory_service.clone(),
        model_manager.clone(),
        llm_provider.clone(),
        surreal_repository.clone(),
    ));

    info!("✅ Application services ready");

    // -------------------------------------------------------------------------
    // Create application state
    // -------------------------------------------------------------------------
    let app_state = Arc::new(AppState {
        database_pool,
        llm_provider,
        search_provider,
        command_executor,
        memory_service,
        chat_service,
        model_manager,
    });

    // -------------------------------------------------------------------------
    // Create the Axum application with all routes and middleware
    // -------------------------------------------------------------------------
    let app = create_router(app_state);

    // -------------------------------------------------------------------------
    // Start the HTTP server
    // -------------------------------------------------------------------------
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    
    info!("🚀 NEURO-OS Backend listening on http://{}", bind_addr);
    info!("📚 API endpoints:");
    info!("   - Health:  GET  /api/health");
    info!("   - Chat:    POST /api/chat");
    info!("   - Memory:  GET  /api/memories");
    info!("   - Graph:   GET  /api/admin/graph/stats");
    info!("   - Search:  POST /api/agent/search");

    axum::serve(listener, app).await?;

    Ok(())
}
