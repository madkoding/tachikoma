//! =============================================================================
//! Configuration
//! =============================================================================
//! 
//! TACHIKOMA-OS Memory Service configuration.
//! All LLM/embedding operations go through tachikoma-backend's /api/llm/* endpoints.
//! =============================================================================

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_user: String,
    pub database_pass: String,
    pub database_ns: String,
    pub database_db: String,
    /// Backend URL - the gateway to all LLM operations (embeddings, etc.)
    pub backend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3004".to_string())
                .parse()
                .unwrap_or(3004),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
            database_user: std::env::var("DATABASE_USER")
                .unwrap_or_else(|_| "root".to_string()),
            database_pass: std::env::var("DATABASE_PASS")
                .unwrap_or_else(|_| "root".to_string()),
            database_ns: std::env::var("DATABASE_NS")
                .unwrap_or_else(|_| "tachikoma".to_string()),
            database_db: std::env::var("DATABASE_DB")
                .unwrap_or_else(|_| "memories".to_string()),
            backend_url: std::env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}
