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
    pub ollama_url: String,
    pub memory_service_url: String,
    pub default_model: String,
    pub fast_model: String,
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
                .unwrap_or_else(|_| "neuro".to_string()),
            database_db: std::env::var("DATABASE_DB")
                .unwrap_or_else(|_| "chat".to_string()),
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            memory_service_url: std::env::var("MEMORY_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string()),
            default_model: std::env::var("DEFAULT_MODEL")
                .unwrap_or_else(|_| "qwen2.5-coder:7b".to_string()),
            fast_model: std::env::var("FAST_MODEL")
                .unwrap_or_else(|_| "qwen2.5:3b".to_string()),
        }
    }
}
