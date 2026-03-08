//! =============================================================================
//! Configuration
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
    /// URL to tachikoma-backend - the only gateway to Ollama
    pub backend_url: String,
    pub memory_service_url: String,
    // Speculative decoding configuration (optional overrides, backend uses defaults)
    pub speculative_enabled: bool,
    pub draft_model: Option<String>,
    pub target_model: Option<String>,
    pub speculative_lookahead: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3003".to_string())
                .parse()
                .unwrap_or(3003),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
            database_user: std::env::var("DATABASE_USER")
                .unwrap_or_else(|_| "root".to_string()),
            database_pass: std::env::var("DATABASE_PASS")
                .unwrap_or_else(|_| "root".to_string()),
            database_ns: std::env::var("DATABASE_NS")
                .unwrap_or_else(|_| "tachikoma".to_string()),
            database_db: std::env::var("DATABASE_DB")
                .unwrap_or_else(|_| "chat".to_string()),
            // Backend URL - the ONLY gateway to Ollama
            backend_url: std::env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            memory_service_url: std::env::var("MEMORY_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string()),
            // Speculative decoding (optional overrides - backend has defaults)
            speculative_enabled: std::env::var("SPECULATIVE_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            // None means use backend defaults (Light tier for draft, Standard for target)
            draft_model: std::env::var("DRAFT_MODEL").ok(),
            target_model: std::env::var("TARGET_MODEL").ok(),
            speculative_lookahead: std::env::var("SPECULATIVE_LOOKAHEAD")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        }
    }
}
