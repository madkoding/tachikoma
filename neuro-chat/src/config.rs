use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub backend_url: String,
    pub database_url: String,
    pub database_user: String,
    pub database_pass: String,
    pub database_ns: String,
    pub database_db: String,
    pub memory_service_url: String,
    pub speculative_enabled: bool,
    pub draft_model: Option<String>,
    pub target_model: Option<String>,
    pub speculative_lookahead: usize,
}

impl Config {
    pub fn from_env() -> Self {
        let base = neuro_common::Config::from_env(3003);
        Self {
            host: base.host,
            port: base.port,
            backend_url: base.backend_url,
            database_url: base.database_url,
            database_user: base.database_user,
            database_pass: base.database_pass,
            database_ns: base.database_ns,
            database_db: env::var("DATABASE_DB").unwrap_or_else(|_| "chat".to_string()),
            memory_service_url: env::var("MEMORY_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string()),
            speculative_enabled: env::var("SPECULATIVE_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            draft_model: env::var("DRAFT_MODEL").ok(),
            target_model: env::var("TARGET_MODEL").ok(),
            speculative_lookahead: env::var("SPECULATIVE_LOOKAHEAD")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        }
    }
}