//! =============================================================================
//! Command Executor - Safe Shell Command Execution
//! =============================================================================

use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error};

/// Command executor with safety restrictions
pub struct CommandExecutor {
    timeout_secs: u64,
    max_output_bytes: usize,
}

/// Command execution request
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Command execution result
#[derive(Debug, Serialize)]
pub struct ExecuteResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
    pub error: Option<String>,
}

/// Blocked commands that should never be executed
const BLOCKED_COMMANDS: &[&str] = &[
    "rm", "rmdir", "dd", "mkfs", "fdisk", "format",
    "shutdown", "reboot", "poweroff", "halt", "init",
    "kill", "killall", "pkill",
    "chmod", "chown", "chgrp",
    "sudo", "su", "doas",
    "passwd", "useradd", "userdel", "usermod",
    "wget", "curl", // Network downloads can be dangerous
    "nc", "netcat", "ncat",
    "ssh", "scp", "rsync",
    "mount", "umount",
    "iptables", "ip6tables", "nft",
    "systemctl", "service",
    "docker", "podman",
    "eval", "exec",
];

/// Dangerous patterns in arguments
const DANGEROUS_PATTERNS: &[&str] = &[
    "|", ";", "&&", "||", "`", "$(", "${",
    ">", ">>", "<", "<<",
    "/dev/", "/proc/", "/sys/",
    "../", "/..",
    "~root", "/root",
];

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            timeout_secs: 30,
            max_output_bytes: 1024 * 1024, // 1MB max output
        }
    }

    /// Check if a command is allowed to run
    pub fn is_command_allowed(&self, command: &str, args: &[String], allowed_list: &[String]) -> Result<(), String> {
        // Get the base command (without path)
        let base_command = command.rsplit('/').next().unwrap_or(command);

        // Check against blocklist
        if BLOCKED_COMMANDS.contains(&base_command) {
            return Err(format!("Command '{}' is blocked for security reasons", base_command));
        }

        // Check if command is in the allowed list (if list is not empty)
        if !allowed_list.is_empty() && !allowed_list.iter().any(|c| c == base_command) {
            return Err(format!(
                "Command '{}' is not in the allowed list. Allowed: {:?}",
                base_command, allowed_list
            ));
        }

        // Check arguments for dangerous patterns
        for arg in args {
            for pattern in DANGEROUS_PATTERNS {
                if arg.contains(pattern) {
                    return Err(format!(
                        "Argument contains dangerous pattern '{}': {}",
                        pattern, arg
                    ));
                }
            }
        }

        // Check command itself for dangerous patterns
        for pattern in DANGEROUS_PATTERNS {
            if command.contains(pattern) {
                return Err(format!(
                    "Command contains dangerous pattern '{}': {}",
                    pattern, command
                ));
            }
        }

        Ok(())
    }

    /// Execute a command safely
    pub async fn execute(
        &self,
        request: &ExecuteRequest,
        allowed_commands: &[String],
    ) -> ExecuteResult {
        debug!("Executing command: {} {:?}", request.command, request.args);

        // Validate command
        if let Err(e) = self.is_command_allowed(&request.command, &request.args, allowed_commands) {
            warn!("Command rejected: {}", e);
            return ExecuteResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                truncated: false,
                error: Some(e),
            };
        }

        // Build command
        let mut cmd = Command::new(&request.command);
        cmd.args(&request.args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set working directory if specified
        if let Some(ref dir) = request.working_dir {
            // Validate working directory
            if dir.contains("..") || dir.starts_with("/root") {
                return ExecuteResult {
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    truncated: false,
                    error: Some("Invalid working directory".to_string()),
                };
            }
            cmd.current_dir(dir);
        }

        // Spawn process
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to spawn command: {}", e);
                return ExecuteResult {
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    truncated: false,
                    error: Some(format!("Failed to start command: {}", e)),
                };
            }
        };

        // Wait with timeout
        let timeout = std::time::Duration::from_secs(
            request.timeout_secs.unwrap_or(self.timeout_secs)
        );

        let result = tokio::time::timeout(timeout, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let mut truncated = false;

                // Truncate output if too large
                if stdout.len() > self.max_output_bytes {
                    stdout.truncate(self.max_output_bytes);
                    stdout.push_str("\n... [output truncated]");
                    truncated = true;
                }
                if stderr.len() > self.max_output_bytes {
                    stderr.truncate(self.max_output_bytes);
                    stderr.push_str("\n... [output truncated]");
                    truncated = true;
                }

                ExecuteResult {
                    success: output.status.success(),
                    exit_code: output.status.code(),
                    stdout,
                    stderr,
                    truncated,
                    error: None,
                }
            }
            Ok(Err(e)) => {
                error!("Command execution failed: {}", e);
                ExecuteResult {
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    truncated: false,
                    error: Some(format!("Command failed: {}", e)),
                }
            }
            Err(_) => {
                warn!("Command timed out after {} seconds", timeout.as_secs());
                ExecuteResult {
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    truncated: false,
                    error: Some(format!("Command timed out after {} seconds", timeout.as_secs())),
                }
            }
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}
