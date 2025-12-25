//! =============================================================================
//! Application Configuration
//! =============================================================================
//! Loads and manages application configuration from environment variables.
//! =============================================================================

use anyhow::{Context, Result};
use serde::Deserialize;

/// =============================================================================
/// Config - Root application configuration
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Ollama configuration
    pub ollama: OllamaConfig,
    
    /// Searxng configuration
    pub searxng: SearxngConfig,
    
    /// Microservices configuration (API Gateway)
    pub microservices: MicroservicesConfig,
}

impl Config {
    /// =========================================================================
    /// Load configuration from environment variables
    /// =========================================================================
    /// Reads configuration from environment variables with sensible defaults.
    /// 
    /// # Environment Variables
    /// 
    /// * `SERVER_HOST` - Server bind address (default: 0.0.0.0)
    /// * `SERVER_PORT` - Server port (default: 3000)
    /// * `DATABASE_URL` - SurrealDB connection URL
    /// * `DATABASE_NS` - SurrealDB namespace
    /// * `DATABASE_DB` - SurrealDB database name
    /// * `DATABASE_USER` - SurrealDB username
    /// * `DATABASE_PASS` - SurrealDB password
    /// * `OLLAMA_URL` - Ollama API URL
    /// * `SEARXNG_URL` - Searxng API URL
    /// 
    /// # Returns
    /// 
    /// * `Ok(Config)` - Loaded configuration
    /// * `Err` - If required environment variables are missing
    /// =========================================================================
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            ollama: OllamaConfig::from_env()?,
            searxng: SearxngConfig::from_env()?,
            microservices: MicroservicesConfig::from_env(),
        })
    }
}

/// =============================================================================
/// ServerConfig - HTTP server configuration
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// Request timeout in seconds
    #[allow(dead_code)]
    pub request_timeout_secs: u64,
    
    /// Maximum request body size in bytes
    #[allow(dead_code)]
    pub max_body_size: usize,
}

impl ServerConfig {
    /// Load server configuration from environment
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("Invalid SERVER_PORT")?,
            request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_body_size: std::env::var("MAX_BODY_SIZE")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                .parse()
                .unwrap_or(10_485_760),
        })
    }
}

/// =============================================================================
/// DatabaseConfig - SurrealDB configuration
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL (ws://host:port)
    pub url: String,
    
    /// Namespace
    pub namespace: String,
    
    /// Database name
    pub database: String,
    
    /// Username
    pub username: String,
    
    /// Password
    pub password: String,
}

impl DatabaseConfig {
    /// Load database configuration from environment
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
            namespace: std::env::var("DATABASE_NS")
                .unwrap_or_else(|_| "neuro".to_string()),
            database: std::env::var("DATABASE_DB")
                .unwrap_or_else(|_| "memories".to_string()),
            username: std::env::var("DATABASE_USER")
                .unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("DATABASE_PASS")
                .unwrap_or_else(|_| "root".to_string()),
        })
    }
}

/// =============================================================================
/// OllamaConfig - Ollama API configuration
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaConfig {
    /// Ollama API URL
    pub url: String,
    
    /// Request timeout in seconds
    pub timeout_secs: u64,
    
    /// Default model for generation
    pub default_model: String,
    
    /// Embedding model
    pub embedding_model: String,
}

impl OllamaConfig {
    /// Load Ollama configuration from environment
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            timeout_secs: std::env::var("OLLAMA_TIMEOUT_SECS")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap_or(120),
            default_model: std::env::var("OLLAMA_DEFAULT_MODEL")
                .unwrap_or_else(|_| "qwen2.5-coder:7b".to_string()),
            embedding_model: std::env::var("OLLAMA_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),
        })
    }
}

/// =============================================================================
/// SearxngConfig - Searxng API configuration
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct SearxngConfig {
    /// Searxng API URL
    pub url: String,
    
    /// Request timeout in seconds
    pub timeout_secs: u64,
    
    /// Maximum results per search
    pub max_results: usize,
}

impl SearxngConfig {
    /// Load Searxng configuration from environment
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: std::env::var("SEARXNG_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            timeout_secs: std::env::var("SEARXNG_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_results: std::env::var("SEARXNG_MAX_RESULTS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        })
    }
}

/// =============================================================================
/// Microservices URLs for API Gateway
/// =============================================================================
#[derive(Debug, Clone, Deserialize)]
pub struct MicroservicesConfig {
    /// Checklists service URL
    pub checklists_url: String,
    /// Music service URL
    pub music_url: String,
}

impl MicroservicesConfig {
    /// Load microservices configuration from environment
    pub fn from_env() -> Self {
        Self {
            checklists_url: std::env::var("CHECKLISTS_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            music_url: std::env::var("MUSIC_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".to_string()),
        }
    }
}
