//! =============================================================================
//! Command Executor Port
//! =============================================================================
//! Defines the abstract interface for safe local command execution.
//! This is a critical security boundary for the AI agent.
//! =============================================================================

use async_trait::async_trait;

use crate::domain::errors::DomainError;

/// =============================================================================
/// CommandExecutor - Abstract interface for command execution
/// =============================================================================
/// Defines operations for safely executing local shell commands.
/// Implements security controls to prevent dangerous operations.
/// 
/// # Security Model
/// 
/// ```text
/// ┌─────────────────────────────────────────────────────────────────────────┐
/// │                      COMMAND EXECUTION PIPELINE                         │
/// ├─────────────────────────────────────────────────────────────────────────┤
/// │                                                                         │
/// │   User Input ──▶ [Allowlist Check] ──▶ [Sandbox] ──▶ [Timeout] ──▶ Run  │
/// │                         │                 │             │               │
/// │                         ▼                 ▼             ▼               │
/// │                      REJECT          Restricted     Terminate           │
/// │                    Dangerous          Resources      Long Runs          │
/// │                    Commands                                             │
/// │                                                                         │
/// └─────────────────────────────────────────────────────────────────────────┘
/// ```
/// 
/// # Responsibilities
/// 
/// * Command validation against allowlist
/// * Resource limiting (CPU, memory, time)
/// * Output capture and sanitization
/// * Audit logging of executed commands
/// 
/// # Implementation Notes
/// 
/// Implementations MUST:
/// - Validate commands against an allowlist
/// - Implement timeouts for all executions
/// - Capture both stdout and stderr
/// - Log all execution attempts
/// - Prevent shell injection attacks
/// =============================================================================
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// =========================================================================
    /// Execute a shell command
    /// =========================================================================
    /// Executes a command with safety controls and resource limits.
    /// 
    /// # Arguments
    /// 
    /// * `command` - The command to execute
    /// * `options` - Execution options (timeout, working dir, etc.)
    /// 
    /// # Returns
    /// 
    /// * `Ok(CommandOutput)` - The command output
    /// * `Err(DomainError)` - If execution fails or is blocked
    /// 
    /// # Errors
    /// 
    /// * `DomainError::CommandBlocked` - Command is not allowed
    /// * `DomainError::CommandTimeout` - Execution exceeded timeout
    /// * `DomainError::CommandFailed` - Command returned non-zero exit
    /// 
    /// # Security
    /// 
    /// Commands are validated against an allowlist before execution.
    /// Dangerous commands (rm -rf, sudo, etc.) are always blocked.
    /// =========================================================================
    async fn execute(
        &self,
        command: &str,
        options: ExecutionOptions,
    ) -> Result<CommandOutput, DomainError>;

    /// =========================================================================
    /// Check if a command is allowed
    /// =========================================================================
    /// Validates a command against the security allowlist without executing it.
    /// 
    /// # Arguments
    /// 
    /// * `command` - The command to validate
    /// 
    /// # Returns
    /// 
    /// * `Ok(ValidationResult)` - Validation result with details
    /// * `Err(DomainError)` - If validation fails
    /// =========================================================================
    async fn validate(&self, command: &str) -> Result<ValidationResult, DomainError>;

    /// =========================================================================
    /// Check if a binary exists in PATH
    /// =========================================================================
    /// Checks if a command/binary is available for execution.
    /// 
    /// # Arguments
    /// 
    /// * `binary_name` - Name of the binary to check
    /// 
    /// # Returns
    /// 
    /// `true` if the binary exists and is executable, `false` otherwise
    /// =========================================================================
    async fn binary_exists(&self, binary_name: &str) -> bool;

    /// =========================================================================
    /// Get the allowlist of permitted commands
    /// =========================================================================
    /// Returns the list of commands/patterns that are allowed.
    /// 
    /// # Returns
    /// 
    /// List of allowed command patterns
    /// =========================================================================
    fn get_allowlist(&self) -> &[AllowedCommand];

    /// =========================================================================
    /// Get the blocklist of forbidden commands
    /// =========================================================================
    /// Returns the list of commands/patterns that are always blocked.
    /// 
    /// # Returns
    /// 
    /// List of blocked command patterns
    /// =========================================================================
    fn get_blocklist(&self) -> &[BlockedPattern];
}

/// =============================================================================
/// ExecutionOptions - Options for command execution
/// =============================================================================
#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    /// Working directory for the command
    pub working_dir: Option<String>,

    /// Timeout in seconds (default: 30)
    pub timeout_secs: u64,

    /// Environment variables to set
    pub env_vars: std::collections::HashMap<String, String>,

    /// Maximum output size in bytes (default: 1MB)
    pub max_output_bytes: usize,

    /// Whether to capture stderr
    pub capture_stderr: bool,

    /// User to run as (if permitted)
    pub run_as_user: Option<String>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            working_dir: None,
            timeout_secs: 30,
            env_vars: std::collections::HashMap::new(),
            max_output_bytes: 1024 * 1024, // 1MB
            capture_stderr: true,
            run_as_user: None,
        }
    }
}

impl ExecutionOptions {
    /// Create options with custom timeout
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            timeout_secs,
            ..Default::default()
        }
    }

    /// Create options with working directory
    pub fn with_working_dir(working_dir: String) -> Self {
        Self {
            working_dir: Some(working_dir),
            ..Default::default()
        }
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }
}

/// =============================================================================
/// CommandOutput - Result of command execution
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Exit code
    pub exit_code: i32,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Whether the command was killed due to timeout
    pub timed_out: bool,

    /// Whether output was truncated
    pub truncated: bool,

    /// The command that was executed (sanitized)
    pub command: String,
}

impl CommandOutput {
    /// Check if the command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get combined output (stdout + stderr)
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }

    /// Format for LLM context
    pub fn as_context(&self) -> String {
        let status = if self.success() {
            "Success"
        } else {
            "Failed"
        };

        format!(
            "Command: {}\nStatus: {} (exit code: {})\nOutput:\n{}",
            self.command,
            status,
            self.exit_code,
            self.combined_output()
        )
    }
}

/// =============================================================================
/// ValidationResult - Result of command validation
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// Whether the command is allowed
    pub allowed: bool,

    /// Reason for the decision
    pub reason: String,

    /// Risk level of the command
    pub risk_level: RiskLevel,

    /// Parsed command components
    pub parsed: Option<ParsedCommand>,

    /// Suggestions for safer alternatives
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    /// Create an allowed result
    pub fn allowed(reason: &str, risk_level: RiskLevel) -> Self {
        Self {
            allowed: true,
            reason: reason.to_string(),
            risk_level,
            parsed: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a blocked result
    pub fn blocked(reason: &str) -> Self {
        Self {
            allowed: false,
            reason: reason.to_string(),
            risk_level: RiskLevel::Blocked,
            parsed: None,
            suggestions: Vec::new(),
        }
    }
}

/// =============================================================================
/// RiskLevel - Risk classification for commands
/// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel {
    /// Safe commands (ls, cat, echo, etc.)
    Safe,
    /// Low risk commands (grep, find, etc.)
    Low,
    /// Medium risk (network tools, package managers)
    Medium,
    /// High risk (file modifications, system changes)
    High,
    /// Always blocked (rm -rf, sudo, etc.)
    Blocked,
}

impl RiskLevel {
    /// Check if this risk level requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, RiskLevel::Medium | RiskLevel::High)
    }
}

/// =============================================================================
/// ParsedCommand - Parsed command components
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedCommand {
    /// The main command/binary
    pub program: String,
    /// Arguments passed to the command
    pub arguments: Vec<String>,
    /// Whether pipes are used
    pub has_pipes: bool,
    /// Whether redirects are used
    pub has_redirects: bool,
    /// Whether background execution is requested
    pub is_background: bool,
}

/// =============================================================================
/// AllowedCommand - Definition of an allowed command pattern
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllowedCommand {
    /// Command name or pattern
    pub pattern: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Description of what the command does
    pub description: String,
    /// Allowed arguments pattern (regex)
    pub allowed_args: Option<String>,
}

impl AllowedCommand {
    /// Create a new allowed command
    pub fn new(pattern: &str, risk_level: RiskLevel, description: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            risk_level,
            description: description.to_string(),
            allowed_args: None,
        }
    }
}

/// =============================================================================
/// BlockedPattern - Definition of a blocked command pattern
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockedPattern {
    /// Pattern to block (regex)
    pub pattern: String,
    /// Reason for blocking
    pub reason: String,
}

impl BlockedPattern {
    /// Create a new blocked pattern
    pub fn new(pattern: &str, reason: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            reason: reason.to_string(),
        }
    }
}

/// =============================================================================
/// Default allowlist for safe commands
/// =============================================================================
pub fn default_allowlist() -> Vec<AllowedCommand> {
    vec![
        // Information commands
        AllowedCommand::new("ls", RiskLevel::Safe, "List directory contents"),
        AllowedCommand::new("cat", RiskLevel::Safe, "Display file contents"),
        AllowedCommand::new("head", RiskLevel::Safe, "Display first lines of file"),
        AllowedCommand::new("tail", RiskLevel::Safe, "Display last lines of file"),
        AllowedCommand::new("pwd", RiskLevel::Safe, "Print working directory"),
        AllowedCommand::new("echo", RiskLevel::Safe, "Print text"),
        AllowedCommand::new("date", RiskLevel::Safe, "Display date and time"),
        AllowedCommand::new("whoami", RiskLevel::Safe, "Display current user"),
        AllowedCommand::new("hostname", RiskLevel::Safe, "Display hostname"),
        AllowedCommand::new("uname", RiskLevel::Safe, "Display system info"),
        AllowedCommand::new("df", RiskLevel::Safe, "Display disk space"),
        AllowedCommand::new("du", RiskLevel::Safe, "Display directory size"),
        AllowedCommand::new("free", RiskLevel::Safe, "Display memory usage"),
        AllowedCommand::new("uptime", RiskLevel::Safe, "Display system uptime"),
        // Search and text processing
        AllowedCommand::new("grep", RiskLevel::Low, "Search text patterns"),
        AllowedCommand::new("find", RiskLevel::Low, "Find files"),
        AllowedCommand::new("wc", RiskLevel::Safe, "Count words/lines"),
        AllowedCommand::new("sort", RiskLevel::Safe, "Sort text"),
        AllowedCommand::new("uniq", RiskLevel::Safe, "Filter unique lines"),
        AllowedCommand::new("cut", RiskLevel::Safe, "Cut text columns"),
        AllowedCommand::new("awk", RiskLevel::Low, "Text processing"),
        AllowedCommand::new("sed", RiskLevel::Low, "Stream editor"),
        // Development tools
        AllowedCommand::new("git", RiskLevel::Low, "Version control"),
        AllowedCommand::new("cargo", RiskLevel::Medium, "Rust package manager"),
        AllowedCommand::new("npm", RiskLevel::Medium, "Node package manager"),
        AllowedCommand::new("node", RiskLevel::Medium, "Node.js runtime"),
        AllowedCommand::new("python", RiskLevel::Medium, "Python runtime"),
        AllowedCommand::new("python3", RiskLevel::Medium, "Python 3 runtime"),
        AllowedCommand::new("rustc", RiskLevel::Medium, "Rust compiler"),
        // Network (read-only)
        AllowedCommand::new("ping", RiskLevel::Low, "Test network connectivity"),
        AllowedCommand::new("curl", RiskLevel::Low, "HTTP client"),
        AllowedCommand::new("wget", RiskLevel::Low, "Download files"),
    ]
}

/// =============================================================================
/// Default blocklist for dangerous commands
/// =============================================================================
pub fn default_blocklist() -> Vec<BlockedPattern> {
    vec![
        BlockedPattern::new(r"rm\s+-rf", "Recursive force delete is too dangerous"),
        BlockedPattern::new(r"sudo", "Elevated privileges not allowed"),
        BlockedPattern::new(r"su\s+", "Switching users not allowed"),
        BlockedPattern::new(r"chmod\s+777", "Overly permissive permissions"),
        BlockedPattern::new(r"mkfs", "Filesystem operations not allowed"),
        BlockedPattern::new(r"dd\s+", "Direct disk access not allowed"),
        BlockedPattern::new(r">\s*/dev/", "Writing to devices not allowed"),
        BlockedPattern::new(r"fork\s*bomb", "Fork bombs not allowed"),
        BlockedPattern::new(r":\(\)\{", "Fork bombs not allowed"),
        BlockedPattern::new(r"shutdown", "System shutdown not allowed"),
        BlockedPattern::new(r"reboot", "System reboot not allowed"),
        BlockedPattern::new(r"init\s+0", "System halt not allowed"),
        BlockedPattern::new(r"passwd", "Password changes not allowed"),
        BlockedPattern::new(r"useradd", "User management not allowed"),
        BlockedPattern::new(r"userdel", "User management not allowed"),
        BlockedPattern::new(r"eval\s+", "Eval not allowed (injection risk)"),
        BlockedPattern::new(r"\$\(.*\)", "Command substitution restricted"),
        BlockedPattern::new(r"`.*`", "Backtick substitution restricted"),
    ]
}
