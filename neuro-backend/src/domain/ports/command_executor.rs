//! =============================================================================
//! Command Executor Port - Simplified
//! =============================================================================
//! Defines the interface for safe local command execution.
//! =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

/// =============================================================================
/// CommandExecutor - Interface for safe command execution
/// =============================================================================
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a shell command with safety controls
    async fn execute(&self, command: &str, options: Option<ExecutionOptions>) -> Result<CommandOutput, DomainError>;

    /// Validate if a command is allowed
    async fn validate(&self, command: &str) -> Result<bool, DomainError>;

    /// Check if a binary exists
    async fn binary_exists(&self, binary_name: &str) -> bool;
}

/// =============================================================================
/// ExecutionOptions - Options for command execution
/// =============================================================================
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    /// Working directory
    pub working_dir: Option<String>,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Environment variables
    pub env_vars: std::collections::HashMap<String, String>,
}

impl ExecutionOptions {
    pub fn with_timeout(timeout: u64) -> Self {
        Self {
            timeout_secs: Some(timeout),
            ..Default::default()
        }
    }

    pub fn with_working_dir(dir: &str) -> Self {
        Self {
            working_dir: Some(dir.to_string()),
            ..Default::default()
        }
    }
}

/// =============================================================================
/// CommandOutput - Result of command execution
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether timed out
    pub timed_out: bool,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}
