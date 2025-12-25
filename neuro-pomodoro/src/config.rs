//! Configuration module

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server port
    pub port: u16,
    /// Backend URL for data layer
    pub backend_url: String,
    /// Default work duration in minutes
    pub default_work_minutes: u32,
    /// Default short break duration in minutes
    pub default_short_break_minutes: u32,
    /// Default long break duration in minutes
    pub default_long_break_minutes: u32,
    /// Pomodoros before long break
    pub pomodoros_before_long_break: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3010".to_string())
                .parse()
                .unwrap_or(3010),
            backend_url: std::env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            default_work_minutes: std::env::var("DEFAULT_WORK_MINUTES")
                .unwrap_or_else(|_| "25".to_string())
                .parse()
                .unwrap_or(25),
            default_short_break_minutes: std::env::var("DEFAULT_SHORT_BREAK_MINUTES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            default_long_break_minutes: std::env::var("DEFAULT_LONG_BREAK_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            pomodoros_before_long_break: std::env::var("POMODOROS_BEFORE_LONG_BREAK")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
        }
    }
}
