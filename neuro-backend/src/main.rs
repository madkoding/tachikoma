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
use std::io::{self, Write};

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
    services::{OllamaClient, SafeCommandExecutor, SearxngClient, VoiceEngine, VoiceConfig},
};

// ANSI color codes for terminal output
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

/// Print a startup step with spinner animation
fn print_step(step: u8, total: u8, message: &str) {
    let progress = "█".repeat(step as usize) + &"░".repeat((total - step) as usize);
    print!("\r{CYAN}[{progress}]{RESET} {DIM}({step}/{total}){RESET} {message}");
    io::stdout().flush().ok();
}

/// Print a completed step
fn print_done(step: u8, total: u8, message: &str) {
    let progress = "█".repeat(step as usize) + &"░".repeat((total - step) as usize);
    println!("\r{CYAN}[{progress}]{RESET} {GREEN}✓{RESET} {message}");
}

/// Warm up the LLM model by sending a simple request
/// This preloads the model into GPU memory for faster first response
async fn warm_up_model(llm_provider: &Arc<dyn crate::domain::ports::llm_provider::LlmProvider + Send + Sync>) -> Result<()> {
    use std::time::Instant;
    use tokio::time::{interval, Duration};
    
    let start = Instant::now();
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    
    // Spawn warmup task
    let llm = llm_provider.clone();
    let warmup_handle = tokio::spawn(async move {
        llm.generate("hi", Some("ministral-3:3b")).await
    });
    
    // Animate spinner while waiting
    let mut ticker = interval(Duration::from_millis(80));
    let mut frame_idx = 0;
    
    loop {
        tokio::select! {
            result = &mut Box::pin(async { warmup_handle.is_finished() }) => {
                if result {
                    break;
                }
            }
            _ = ticker.tick() => {
                let elapsed = start.elapsed().as_secs_f64();
                let spinner = spinner_frames[frame_idx % spinner_frames.len()];
                print!("\r{CYAN}[███░░░░░]{RESET} {YELLOW}{spinner}{RESET} Loading model to GPU... {DIM}({elapsed:.1}s){RESET}  ");
                io::stdout().flush().ok();
                frame_idx += 1;
            }
        }
        
        if warmup_handle.is_finished() {
            break;
        }
    }
    
    // Get result
    match warmup_handle.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(anyhow::anyhow!("{}", e)),
        Err(e) => Err(anyhow::anyhow!("Task failed: {}", e)),
    }
}

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
    /// Voice engine for TTS synthesis
    pub voice_engine: Arc<VoiceEngine>,
    /// Event broadcaster for SSE
    pub event_broadcaster: Arc<crate::infrastructure::api::EventBroadcaster>,
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
    // Initialize structured logging with tracing (only warnings and errors)
    // -------------------------------------------------------------------------
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,neuro_backend=info".into()),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    // -------------------------------------------------------------------------
    // Pretty startup banner
    // -------------------------------------------------------------------------
    println!("\n{CYAN}{BOLD}╔═══════════════════════════════════════════════════════════╗{RESET}");
    println!("{CYAN}{BOLD}║{RESET}            {MAGENTA}🧠 NEURO-OS Backend v0.1.0{RESET}                   {CYAN}{BOLD}║{RESET}");
    println!("{CYAN}{BOLD}╚═══════════════════════════════════════════════════════════╝{RESET}\n");

    const TOTAL_STEPS: u8 = 8;

    // -------------------------------------------------------------------------
    // Initialize database connection pool
    // -------------------------------------------------------------------------
    print_step(1, TOTAL_STEPS, "Connecting to SurrealDB...");
    let database_pool = DatabasePool::new(&config.database).await?;
    print_done(1, TOTAL_STEPS, "SurrealDB connected");

    // -------------------------------------------------------------------------
    // Initialize external service clients
    // -------------------------------------------------------------------------
    print_step(2, TOTAL_STEPS, "Initializing Ollama client...");
    let ollama_client = OllamaClient::new(config.ollama.clone());
    let llm_provider: Arc<dyn LlmProvider + Send + Sync> = Arc::new(ollama_client);
    print_done(2, TOTAL_STEPS, "Ollama client ready");
    
    // Warm up the model (preload to GPU memory)
    print_step(3, TOTAL_STEPS, "Preloading LLM model to GPU...");
    io::stdout().flush().ok();
    if let Err(e) = warm_up_model(&llm_provider).await {
        println!("\r{CYAN}[███░░░░]{RESET} {YELLOW}⚠{RESET} Model warmup skipped: {}", e);
    } else {
        print_done(3, TOTAL_STEPS, "LLM model loaded to GPU");
    }

    print_step(4, TOTAL_STEPS, "Initializing Searxng client...");
    let searxng_client = SearxngClient::new(config.searxng.clone());
    let search_provider: Arc<dyn SearchProvider + Send + Sync> = Arc::new(searxng_client);
    print_done(4, TOTAL_STEPS, "Searxng client ready");

    print_step(5, TOTAL_STEPS, "Initializing command executor...");
    let command_executor: Arc<dyn CommandExecutor + Send + Sync> = 
        Arc::new(SafeCommandExecutor::new());
    print_done(5, TOTAL_STEPS, "Command executor ready");

    // -------------------------------------------------------------------------
    // Create memory repository
    // -------------------------------------------------------------------------
    print_step(6, TOTAL_STEPS, "Initializing memory repository...");
    let surreal_repository = Arc::new(SurrealDbRepository::new(database_pool.clone()));
    let memory_repository: Arc<dyn MemoryRepository + Send + Sync> = surreal_repository.clone();
    print_done(6, TOTAL_STEPS, "Memory repository ready");

    // -------------------------------------------------------------------------
    // Create event broadcaster for SSE (needed by services)
    // -------------------------------------------------------------------------
    let event_broadcaster = Arc::new(crate::infrastructure::api::EventBroadcaster::new(100));

    // -------------------------------------------------------------------------
    // Create application services
    // -------------------------------------------------------------------------
    print_step(7, TOTAL_STEPS, "Creating application services...");
    
    let model_manager = Arc::new(ModelManager::new(llm_provider.clone()));

    let memory_service = Arc::new(MemoryService::with_broadcaster(
        memory_repository,
        llm_provider.clone(),
        event_broadcaster.clone(),
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

    print_done(7, TOTAL_STEPS, "Application services ready");

    // -------------------------------------------------------------------------
    // Initialize Voice Engine (Kokoro-82M TTS)
    // -------------------------------------------------------------------------
    print_step(8, TOTAL_STEPS, "Initializing Voice Engine...");
    let voice_config = VoiceConfig::default();
    let voice_engine = Arc::new(VoiceEngine::new(voice_config));
    
    // Initialize in background (non-blocking)
    let ve_clone = voice_engine.clone();
    tokio::spawn(async move {
        if let Err(e) = ve_clone.initialize().await {
            tracing::warn!("Voice engine initialization failed: {}. TTS will be disabled.", e);
        }
    });
    print_done(8, TOTAL_STEPS, "Voice Engine initialized");

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
        voice_engine,
        event_broadcaster,
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
    
    println!("\n{GREEN}{BOLD}✓ NEURO-OS Backend ready!{RESET}");
    println!("{DIM}─────────────────────────────────────────────────────────{RESET}");
    println!("  {CYAN}▸{RESET} Server:   {YELLOW}http://{bind_addr}{RESET}");
    println!("  {CYAN}▸{RESET} Health:   {DIM}GET  /api/health{RESET}");
    println!("  {CYAN}▸{RESET} Chat:     {DIM}POST /api/chat{RESET}");
    println!("  {CYAN}▸{RESET} Voice:    {DIM}POST /api/voice/synthesize{RESET}");
    println!("{DIM}─────────────────────────────────────────────────────────{RESET}\n");

    axum::serve(listener, app).await?;

    Ok(())
}
