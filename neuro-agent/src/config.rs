//! =============================================================================
//! Configuration
//! =============================================================================

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub searxng_url: String,
    pub allowed_commands: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        // Default allowed commands for safe execution
        let default_allowed = vec![
            "ls", "cat", "head", "tail", "wc", "grep", "find", "which",
            "date", "cal", "uptime", "whoami", "pwd", "echo", "df", "du",
        ];

        let allowed_commands = env::var("ALLOWED_COMMANDS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| default_allowed.iter().map(|s| s.to_string()).collect());

        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3005".to_string())
                .parse()
                .unwrap_or(3005),
            searxng_url: env::var("SEARXNG_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            allowed_commands,
        }
    }
}
