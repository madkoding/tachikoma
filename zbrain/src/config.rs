//! Configuration management for Z-Brain CLI.
//!
//! Handles loading, saving, and managing user configuration stored in
//! the user's config directory (e.g., ~/.config/zbrain/config.toml).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Z-Brain configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// TACHIKOMA-OS API endpoint URL
    pub api_endpoint: String,

    /// Enable verbose output
    pub verbose: bool,

    /// Enable colored output
    pub colored: bool,

    /// Default conversation timeout in seconds
    pub timeout_secs: u64,

    /// History file path (relative to config dir)
    pub history_file: String,

    /// Maximum history entries to keep
    pub max_history: usize,

    /// User's preferred name (for personalization)
    pub user_name: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: "http://localhost:3000".to_string(),
            verbose: false,
            colored: true,
            timeout_secs: 60,
            history_file: "history.txt".to_string(),
            max_history: 1000,
            user_name: None,
        }
    }
}

impl Config {
    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "tachikoma-os", "zbrain")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        Ok(dirs.config_dir().join("config.toml"))
    }

    /// Get the history file path
    pub fn history_path(&self) -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "tachikoma-os", "zbrain")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        Ok(dirs.data_dir().join(&self.history_file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_endpoint, "http://localhost:3000");
        assert!(config.colored);
        assert!(!config.verbose);
    }
}
