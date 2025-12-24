//! Configuration module

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub database_user: String,
    pub database_pass: String,
    pub database_namespace: String,
    pub database_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string()),
            database_user: env::var("DATABASE_USER")
                .unwrap_or_else(|_| "root".to_string()),
            database_pass: env::var("DATABASE_PASS")
                .unwrap_or_else(|_| "root".to_string()),
            database_namespace: env::var("DATABASE_NAMESPACE")
                .unwrap_or_else(|_| "neuro".to_string()),
            database_name: env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "checklists".to_string()),
        }
    }
}
