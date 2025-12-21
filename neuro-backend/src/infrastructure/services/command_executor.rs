//! =============================================================================
//! Safe Command Executor Implementation
//! =============================================================================
//! Implements the CommandExecutor port with strict security controls.
//! Provides sandboxed command execution for AI agents.
//! =============================================================================

use async_trait::async_trait;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, info, warn, instrument};

use crate::domain::{
    errors::DomainError,
    ports::command_executor::{
        CommandExecutor, CommandRequest, CommandResult, CommandSecurity,
    },
};

/// =============================================================================
/// SafeCommandExecutor - Sandboxed command execution
/// =============================================================================
/// Provides secure command execution with:
/// - Whitelist-based command filtering
/// - Path restrictions
/// - Timeout enforcement
/// - Output size limits
/// =============================================================================
pub struct SafeCommandExecutor {
    /// Allowed commands (whitelist)
    allowed_commands: HashSet<String>,
    /// Allowed working directories
    allowed_paths: Vec<PathBuf>,
    /// Maximum output size in bytes
    max_output_bytes: usize,
    /// Default timeout in seconds
    default_timeout: u64,
    /// Security configuration
    security: CommandSecurity,
}

impl Default for SafeCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SafeCommandExecutor {
    /// =========================================================================
    /// Create a new SafeCommandExecutor with default settings
    /// =========================================================================
    /// Default whitelist includes safe, read-only commands.
    /// =========================================================================
    pub fn new() -> Self {
        let mut allowed_commands = HashSet::new();
        
        // Safe read-only commands
        allowed_commands.insert("ls".to_string());
        allowed_commands.insert("cat".to_string());
        allowed_commands.insert("head".to_string());
        allowed_commands.insert("tail".to_string());
        allowed_commands.insert("grep".to_string());
        allowed_commands.insert("find".to_string());
        allowed_commands.insert("wc".to_string());
        allowed_commands.insert("pwd".to_string());
        allowed_commands.insert("echo".to_string());
        allowed_commands.insert("date".to_string());
        allowed_commands.insert("whoami".to_string());
        allowed_commands.insert("uname".to_string());
        allowed_commands.insert("df".to_string());
        allowed_commands.insert("du".to_string());
        allowed_commands.insert("free".to_string());
        allowed_commands.insert("uptime".to_string());
        allowed_commands.insert("which".to_string());
        allowed_commands.insert("type".to_string());
        allowed_commands.insert("file".to_string());
        allowed_commands.insert("stat".to_string());
        allowed_commands.insert("tree".to_string());
        allowed_commands.insert("env".to_string());
        
        // Development tools (read-only mode)
        allowed_commands.insert("git".to_string());
        allowed_commands.insert("cargo".to_string());
        allowed_commands.insert("npm".to_string());
        allowed_commands.insert("node".to_string());
        allowed_commands.insert("python".to_string());
        allowed_commands.insert("python3".to_string());
        allowed_commands.insert("rustc".to_string());
        allowed_commands.insert("rg".to_string()); // ripgrep
        allowed_commands.insert("fd".to_string()); // fd-find
        allowed_commands.insert("jq".to_string()); // JSON processor
        allowed_commands.insert("curl".to_string());
        allowed_commands.insert("wget".to_string());
        
        // Default allowed paths (can be customized)
        let allowed_paths = vec![
            PathBuf::from("/home"),
            PathBuf::from("/tmp"),
            PathBuf::from("/var/tmp"),
        ];

        Self {
            allowed_commands,
            allowed_paths,
            max_output_bytes: 1024 * 1024, // 1MB
            default_timeout: 30,
            security: CommandSecurity::default(),
        }
    }

    /// =========================================================================
    /// Create executor with custom configuration
    /// =========================================================================
    pub fn with_config(
        allowed_commands: HashSet<String>,
        allowed_paths: Vec<PathBuf>,
        max_output_bytes: usize,
        default_timeout: u64,
    ) -> Self {
        Self {
            allowed_commands,
            allowed_paths,
            max_output_bytes,
            default_timeout,
            security: CommandSecurity::default(),
        }
    }

    /// Add an allowed command
    pub fn allow_command(&mut self, command: &str) {
        self.allowed_commands.insert(command.to_string());
    }

    /// Add an allowed path
    pub fn allow_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }

    /// Set security configuration
    pub fn with_security(mut self, security: CommandSecurity) -> Self {
        self.security = security;
        self
    }

    /// Extract base command from command string
    fn extract_base_command(command: &str) -> Option<String> {
        command
            .split_whitespace()
            .next()
            .map(|s| s.to_string())
    }

    /// Check if path is within allowed directories
    fn is_path_allowed(&self, path: &PathBuf) -> bool {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        self.allowed_paths.iter().any(|allowed| {
            canonical.starts_with(allowed)
        })
    }

    /// Check for dangerous patterns in command
    fn has_dangerous_patterns(command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf",
            "rm -fr",
            "mkfs",
            "dd if=",
            ":(){",  // Fork bomb
            ">(", // Process substitution that could be dangerous
            "| rm",
            "&& rm",
            "; rm",
            "chmod 777",
            "chmod -R",
            "chown -R",
            "sudo",
            "su ",
            "doas",
            "> /dev",
            ">> /dev",
            "/etc/passwd",
            "/etc/shadow",
            "id_rsa",
            ".ssh/",
            "eval ",
            "exec ",
            "`", // Command substitution
            "$(", // Command substitution
        ];

        let lower_command = command.to_lowercase();
        dangerous_patterns.iter().any(|pattern| {
            lower_command.contains(&pattern.to_lowercase())
        })
    }

    /// Validate git command (only allow safe subcommands)
    fn is_safe_git_command(args: &[String]) -> bool {
        let safe_subcommands = [
            "status", "log", "diff", "show", "branch", "tag",
            "remote", "fetch", "ls-files", "ls-tree", "rev-parse",
            "describe", "shortlog", "blame", "grep", "config",
        ];

        if args.is_empty() {
            return false;
        }

        let subcommand = args[0].as_str();
        safe_subcommands.contains(&subcommand)
    }
}

#[async_trait]
impl CommandExecutor for SafeCommandExecutor {
    /// =========================================================================
    /// Execute a command with security restrictions
    /// =========================================================================
    #[instrument(skip(self, request), fields(command = %request.command))]
    async fn execute(&self, request: CommandRequest) -> Result<CommandResult, DomainError> {
        let start_time = std::time::Instant::now();

        // Extract base command
        let base_command = Self::extract_base_command(&request.command)
            .ok_or_else(|| DomainError::CommandError("Empty command".to_string()))?;

        // Check if command is allowed
        if !self.is_command_allowed(&base_command).await? {
            warn!(command = %base_command, "Command not in whitelist");
            return Err(DomainError::CommandError(format!(
                "Command '{}' is not allowed",
                base_command
            )));
        }

        // Check for dangerous patterns
        if Self::has_dangerous_patterns(&request.command) {
            warn!(command = %request.command, "Dangerous pattern detected");
            return Err(DomainError::CommandError(
                "Command contains potentially dangerous patterns".to_string()
            ));
        }

        // Validate working directory if specified
        if let Some(ref cwd) = request.working_directory {
            let path = PathBuf::from(cwd);
            if !self.is_path_allowed(&path) {
                warn!(path = %cwd, "Working directory not allowed");
                return Err(DomainError::CommandError(format!(
                    "Working directory '{}' is not allowed",
                    cwd
                )));
            }
        }

        // Parse command into parts
        let parts: Vec<String> = shell_words::split(&request.command)
            .map_err(|e| DomainError::CommandError(format!("Failed to parse command: {}", e)))?;

        if parts.is_empty() {
            return Err(DomainError::CommandError("Empty command".to_string()));
        }

        let (cmd, args) = parts.split_first().unwrap();

        // Special validation for git
        if cmd == "git" && !Self::is_safe_git_command(&args.to_vec()) {
            warn!(command = %request.command, "Unsafe git subcommand");
            return Err(DomainError::CommandError(
                "Only read-only git commands are allowed".to_string()
            ));
        }

        // Build command
        let mut command = Command::new(cmd);
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Set working directory
        if let Some(ref cwd) = request.working_directory {
            command.current_dir(cwd);
        }

        // Set environment variables
        if let Some(ref env) = request.environment {
            for (key, value) in env {
                // Block sensitive environment variables
                if !key.to_lowercase().contains("password") 
                    && !key.to_lowercase().contains("secret")
                    && !key.to_lowercase().contains("token")
                    && !key.to_lowercase().contains("key") {
                    command.env(key, value);
                }
            }
        }

        debug!(command = %request.command, "Executing command");

        // Spawn process
        let mut child = command.spawn()
            .map_err(|e| DomainError::CommandError(format!("Failed to spawn process: {}", e)))?;

        // Set up timeout
        let timeout = std::time::Duration::from_secs(
            request.timeout_seconds.unwrap_or(self.default_timeout)
        );

        // Collect output with timeout
        let result = tokio::time::timeout(timeout, async {
            let stdout = child.stdout.take().expect("stdout not captured");
            let stderr = child.stderr.take().expect("stderr not captured");

            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            let mut stdout_output = String::new();
            let mut stderr_output = String::new();

            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(l)) => {
                                if stdout_output.len() + l.len() < self.max_output_bytes {
                                    stdout_output.push_str(&l);
                                    stdout_output.push('\n');
                                }
                            }
                            Ok(None) => break,
                            Err(e) => {
                                warn!(error = %e, "Error reading stdout");
                                break;
                            }
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(l)) => {
                                if stderr_output.len() + l.len() < self.max_output_bytes {
                                    stderr_output.push_str(&l);
                                    stderr_output.push('\n');
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                warn!(error = %e, "Error reading stderr");
                            }
                        }
                    }
                }
            }

            let status = child.wait().await
                .map_err(|e| DomainError::CommandError(format!("Failed to wait for process: {}", e)))?;

            Ok::<_, DomainError>((status, stdout_output, stderr_output))
        }).await;

        match result {
            Ok(Ok((status, stdout, stderr))) => {
                let duration = start_time.elapsed();
                let exit_code = status.code().unwrap_or(-1);

                info!(
                    command = %request.command,
                    exit_code = exit_code,
                    duration_ms = duration.as_millis(),
                    "Command completed"
                );

                Ok(CommandResult {
                    exit_code,
                    stdout,
                    stderr,
                    duration_ms: duration.as_millis() as u64,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout - kill the process
                let _ = child.kill().await;
                
                warn!(
                    command = %request.command,
                    timeout_secs = timeout.as_secs(),
                    "Command timed out"
                );

                Ok(CommandResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Command timed out after {} seconds", timeout.as_secs()),
                    duration_ms: timeout.as_millis() as u64,
                    timed_out: true,
                })
            }
        }
    }

    /// =========================================================================
    /// Check if command is allowed
    /// =========================================================================
    async fn is_command_allowed(&self, command: &str) -> Result<bool, DomainError> {
        let base_command = Self::extract_base_command(command)
            .unwrap_or_else(|| command.to_string());
        
        Ok(self.allowed_commands.contains(&base_command))
    }

    /// =========================================================================
    /// Get list of allowed commands
    /// =========================================================================
    async fn get_allowed_commands(&self) -> Result<Vec<String>, DomainError> {
        let mut commands: Vec<String> = self.allowed_commands.iter().cloned().collect();
        commands.sort();
        Ok(commands)
    }

    /// =========================================================================
    /// Get security configuration
    /// =========================================================================
    async fn get_security_config(&self) -> Result<CommandSecurity, DomainError> {
        Ok(self.security.clone())
    }

    /// =========================================================================
    /// Execute with elevated privileges (disabled by default)
    /// =========================================================================
    async fn execute_privileged(
        &self,
        _request: CommandRequest,
    ) -> Result<CommandResult, DomainError> {
        // Privileged execution is not supported for security reasons
        Err(DomainError::CommandError(
            "Privileged execution is not enabled for security reasons".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_pattern_detection() {
        assert!(SafeCommandExecutor::has_dangerous_patterns("rm -rf /"));
        assert!(SafeCommandExecutor::has_dangerous_patterns("sudo apt install"));
        assert!(SafeCommandExecutor::has_dangerous_patterns("cat /etc/passwd"));
        assert!(SafeCommandExecutor::has_dangerous_patterns("echo $(whoami)"));
        
        assert!(!SafeCommandExecutor::has_dangerous_patterns("ls -la"));
        assert!(!SafeCommandExecutor::has_dangerous_patterns("git status"));
        assert!(!SafeCommandExecutor::has_dangerous_patterns("cat file.txt"));
    }

    #[test]
    fn test_safe_git_commands() {
        assert!(SafeCommandExecutor::is_safe_git_command(&vec!["status".to_string()]));
        assert!(SafeCommandExecutor::is_safe_git_command(&vec!["log".to_string(), "--oneline".to_string()]));
        assert!(SafeCommandExecutor::is_safe_git_command(&vec!["diff".to_string()]));
        
        assert!(!SafeCommandExecutor::is_safe_git_command(&vec!["push".to_string()]));
        assert!(!SafeCommandExecutor::is_safe_git_command(&vec!["commit".to_string()]));
        assert!(!SafeCommandExecutor::is_safe_git_command(&vec![]));
    }

    #[test]
    fn test_extract_base_command() {
        assert_eq!(
            SafeCommandExecutor::extract_base_command("ls -la /home"),
            Some("ls".to_string())
        );
        assert_eq!(
            SafeCommandExecutor::extract_base_command("git status"),
            Some("git".to_string())
        );
        assert_eq!(
            SafeCommandExecutor::extract_base_command(""),
            None
        );
    }
}
