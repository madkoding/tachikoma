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
}

impl Config {
    pub fn from_env() -> Self {
        let base = neuro_common::Config::from_env(3004);
        Self {
            host: base.host,
            port: base.port,
            backend_url: base.backend_url,
            database_url: base.database_url,
            database_user: base.database_user,
            database_pass: base.database_pass,
            database_ns: base.database_ns,
            database_db: env::var("DATABASE_DB").unwrap_or_else(|_| "memories".to_string()),
        }
    }
}