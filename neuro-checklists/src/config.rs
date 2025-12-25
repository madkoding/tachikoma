//! Configuration module

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub backend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string()),
        }
    }
}
