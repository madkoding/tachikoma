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
        let base = neuro_common::Config::from_env(3005);

        let default_allowed = vec![
            "ls", "cat", "head", "tail", "wc", "grep", "find", "which",
            "date", "cal", "uptime", "whoami", "pwd", "echo", "df", "du",
        ];

        let allowed_commands = env::var("ALLOWED_COMMANDS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| default_allowed.iter().map(|s| s.to_string()).collect());

        Self {
            host: base.host,
            port: base.port,
            searxng_url: env::var("SEARXNG_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            allowed_commands,
        }
    }
}