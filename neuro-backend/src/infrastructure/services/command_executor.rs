//! =============================================================================
//! Safe Command Executor Implementation
//! =============================================================================
//! Implements the CommandExecutor port with security controls.
//! =============================================================================

use async_trait::async_trait;
use std::collections::HashSet;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, warn, instrument};

use crate::domain::{
    errors::DomainError,
    ports::command_executor::{CommandExecutor, CommandOutput, ExecutionOptions},
};

/// =============================================================================
/// SafeCommandExecutor - Sandboxed command execution
/// =============================================================================
pub struct SafeCommandExecutor {
    allowed_commands: HashSet<String>,
    blocked_patterns: Vec<String>,
    default_timeout: u64,
    max_output_bytes: usize,
}

impl Default for SafeCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SafeCommandExecutor {
    pub fn new() -> Self {
        let mut allowed_commands = HashSet::new();

        // Safe read-only commands
        for cmd in &[
            "ls", "cat", "head", "tail", "grep", "find", "wc", "pwd", "echo",
            "date", "whoami", "uname", "df", "du", "free", "uptime", "which",
            "type", "file", "stat", "tree", "env", "hostname",
            // Dev tools
            "git", "cargo", "npm", "node", "python", "python3", "rustc",
            "rg", "fd", "jq", "curl", "wget",
        ] {
            allowed_commands.insert(cmd.to_string());
        }

        let blocked_patterns = vec![
            "rm -rf".to_string(),
            "sudo".to_string(),
            "su ".to_string(),
            "chmod 777".to_string(),
            "mkfs".to_string(),
            "dd ".to_string(),
            "> /dev/".to_string(),
            "shutdown".to_string(),
            "reboot".to_string(),
            "passwd".to_string(),
        ];

        Self {
            allowed_commands,
            blocked_patterns,
            default_timeout: 30,
            max_output_bytes: 1024 * 1024,
        }
    }

    fn extract_base_command(command: &str) -> Option<String> {
        command
            .trim()
            .split_whitespace()
            .next()
            .map(|s| s.to_string())
    }

    #[inline]
    fn contains_blocked_pattern(&self, command: &str) -> Option<&str> {
        // Usar find() es más idiomático y puede ser optimizado mejor por el compilador
        self.blocked_patterns
            .iter()
            .find(|pattern| command.contains(pattern.as_str()))
            .map(|s| s.as_str())
    }

    fn is_safe_git_command(args: &[&str]) -> bool {
        let safe_git_commands = ["status", "log", "diff", "show", "branch", "remote", "fetch", "pull", "clone"];
        if let Some(subcommand) = args.first() {
            safe_git_commands.contains(subcommand)
        } else {
            true
        }
    }
}

#[async_trait]
impl CommandExecutor for SafeCommandExecutor {
    #[instrument(skip(self))]
    async fn execute(&self, command: &str, options: Option<ExecutionOptions>) -> Result<CommandOutput, DomainError> {
        let opts = options.unwrap_or_default();
        let timeout = opts.timeout_secs.unwrap_or(self.default_timeout);

        // Validate command first
        if !self.validate(command).await? {
            return Err(DomainError::command_blocked(command, "Command not allowed"));
        }

        debug!(command = %command, "Executing command");

        let start = std::time::Instant::now();

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = &opts.working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in &opts.env_vars {
            cmd.env(key, value);
        }

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            cmd.output(),
        )
        .await;

        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(output)) => {
                let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();

                // Truncate if too large
                if stdout.len() > self.max_output_bytes {
                    stdout.truncate(self.max_output_bytes);
                    stdout.push_str("\n... [output truncated]");
                }
                if stderr.len() > self.max_output_bytes {
                    stderr.truncate(self.max_output_bytes);
                    stderr.push_str("\n... [output truncated]");
                }

                Ok(CommandOutput {
                    stdout,
                    stderr,
                    exit_code: output.status.code().unwrap_or(-1),
                    execution_time_ms: elapsed,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => {
                Err(DomainError::command_error(format!("Failed to execute command: {}", e)))
            }
            Err(_) => {
                warn!(command = %command, timeout = timeout, "Command timed out");
                Ok(CommandOutput {
                    stdout: String::new(),
                    stderr: "Command execution timed out".to_string(),
                    exit_code: -1,
                    execution_time_ms: elapsed,
                    timed_out: true,
                })
            }
        }
    }

    #[instrument(skip(self))]
    async fn validate(&self, command: &str) -> Result<bool, DomainError> {
        // Check for blocked patterns
        if let Some(pattern) = self.contains_blocked_pattern(command) {
            warn!(command = %command, pattern = %pattern, "Command blocked by pattern");
            return Ok(false);
        }

        // Extract and check base command
        let base_cmd = Self::extract_base_command(command)
            .ok_or_else(|| DomainError::command_error("Empty command"))?;

        if !self.allowed_commands.contains(&base_cmd) {
            warn!(command = %command, base = %base_cmd, "Command not in allowlist");
            return Ok(false);
        }

        // Special handling for git
        if base_cmd == "git" {
            let args: Vec<&str> = command.split_whitespace().skip(1).collect();
            if !Self::is_safe_git_command(&args) {
                warn!(command = %command, "Unsafe git command");
                return Ok(false);
            }
        }

        Ok(true)
    }

    #[instrument(skip(self))]
    async fn binary_exists(&self, binary_name: &str) -> bool {
        Command::new("which")
            .arg(binary_name)
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
