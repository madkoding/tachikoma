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
    pub fn from_env(default_port: u16) -> Self {
        dotenvy::dotenv().ok();
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(default_port),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
            database_user: env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string()),
            database_pass: env::var("DATABASE_PASS").unwrap_or_else(|_| "root".to_string()),
            database_ns: env::var("DATABASE_NS").unwrap_or_else(|_| "tachikoma".to_string()),
            database_db: env::var("DATABASE_DB").unwrap_or_else(|_| "tachikoma".to_string()),
        }
    }

    pub fn from_env_port(default_port: u16) -> Self {
        Self::from_env(default_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::from_env(8080);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.database_user, "root");
        assert_eq!(config.database_ns, "tachikoma");
    }

    #[test]
    fn test_config_backend_url_default() {
        let config = Config::from_env(3000);
        assert!(config.backend_url.contains("localhost"));
    }
}