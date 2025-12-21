//! =============================================================================
//! Domain Errors
//! =============================================================================
//! Defines all domain-specific error types for the NEURO-OS system.
//! Uses thiserror for ergonomic error handling.
//! =============================================================================

use thiserror::Error;
use uuid::Uuid;

/// =============================================================================
/// DomainError - Comprehensive error type for domain operations
/// =============================================================================
/// Represents all possible errors that can occur in the domain layer.
/// Each variant includes relevant context for debugging and user feedback.
/// =============================================================================
#[derive(Debug, Error)]
pub enum DomainError {
    // =========================================================================
    // Database Errors
    // =========================================================================
    
    /// Database connection or query error
    #[error("Database error: {message}")]
    DatabaseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Entity not found in database
    #[error("Entity not found: {entity_type} with ID {id}")]
    NotFound { entity_type: String, id: String },

    /// Duplicate entry error
    #[error("Duplicate entry: {entity_type} with ID {id} already exists")]
    DuplicateEntry { entity_type: String, id: String },

    /// Database transaction error
    #[error("Transaction error: {message}")]
    TransactionError { message: String },

    // =========================================================================
    // LLM/Model Errors
    // =========================================================================

    /// Model is not available or loaded
    #[error("Model not available: {model_name}")]
    ModelNotAvailable { model_name: String },

    /// Inference/generation error
    #[error("Inference error: {message}")]
    InferenceError { message: String },

    /// Context window exceeded
    #[error("Context too large: {tokens} tokens exceeds limit of {limit}")]
    ContextTooLarge { tokens: usize, limit: usize },

    /// Embedding generation error
    #[error("Embedding error: {message}")]
    EmbeddingError { message: String },

    /// Model download/pull error
    #[error("Failed to pull model {model_name}: {reason}")]
    ModelPullError { model_name: String, reason: String },

    // =========================================================================
    // Search Errors
    // =========================================================================

    /// Search engine error
    #[error("Search error: {message}")]
    SearchError { message: String },

    /// Rate limiting error
    #[error("Rate limited: too many requests, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    /// URL fetch error
    #[error("Failed to fetch URL {url}: {reason}")]
    FetchError { url: String, reason: String },

    // =========================================================================
    // Command Execution Errors
    // =========================================================================

    /// Command is not allowed (security)
    #[error("Command blocked: {command} - {reason}")]
    CommandBlocked { command: String, reason: String },

    /// Command execution timeout
    #[error("Command timeout: {command} exceeded {timeout_secs} seconds")]
    CommandTimeout { command: String, timeout_secs: u64 },

    /// Command failed with non-zero exit
    #[error("Command failed: {command} exited with code {exit_code}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    /// Command parsing error
    #[error("Invalid command syntax: {message}")]
    CommandParseError { message: String },

    // =========================================================================
    // Validation Errors
    // =========================================================================

    /// Input validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Invalid memory type
    #[error("Invalid memory type: {memory_type}")]
    InvalidMemoryType { memory_type: String },

    /// Invalid relation type
    #[error("Invalid relation type: {relation_type}")]
    InvalidRelationType { relation_type: String },

    // =========================================================================
    // Agent Errors
    // =========================================================================

    /// Agent task execution error
    #[error("Task execution error: {task_type} - {message}")]
    TaskExecutionError { task_type: String, message: String },

    /// Tool not found
    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    /// Tool invocation error
    #[error("Tool error: {tool_name} - {message}")]
    ToolError { tool_name: String, message: String },

    // =========================================================================
    // Configuration Errors
    // =========================================================================

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Missing environment variable
    #[error("Missing environment variable: {var_name}")]
    MissingEnvVar { var_name: String },

    // =========================================================================
    // Infrastructure Errors
    // =========================================================================

    /// HTTP/network error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl DomainError {
    // =========================================================================
    // Convenience Constructors
    // =========================================================================

    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::DatabaseError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a not found error
    pub fn not_found(entity_type: impl Into<String>, id: impl ToString) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.to_string(),
        }
    }

    /// Create a not found error for a memory
    pub fn memory_not_found(id: Uuid) -> Self {
        Self::not_found("Memory", id)
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a model not available error
    pub fn model_unavailable(model_name: impl Into<String>) -> Self {
        Self::ModelNotAvailable {
            model_name: model_name.into(),
        }
    }

    /// Create an inference error
    pub fn inference(message: impl Into<String>) -> Self {
        Self::InferenceError {
            message: message.into(),
        }
    }

    /// Create a command blocked error
    pub fn command_blocked(command: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CommandBlocked {
            command: command.into(),
            reason: reason.into(),
        }
    }

    /// Create a search error
    pub fn search(message: impl Into<String>) -> Self {
        Self::SearchError {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    // =========================================================================
    // Error Classification
    // =========================================================================

    /// Check if this error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::DatabaseError { .. }
                | Self::NetworkError { .. }
                | Self::RateLimited { .. }
                | Self::CommandTimeout { .. }
        )
    }

    /// Check if this error is a user error (vs system error)
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            Self::ValidationError { .. }
                | Self::InvalidMemoryType { .. }
                | Self::InvalidRelationType { .. }
                | Self::CommandBlocked { .. }
                | Self::CommandParseError { .. }
        )
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound { .. } => 404,
            Self::DuplicateEntry { .. } => 409,
            Self::ValidationError { .. } => 400,
            Self::InvalidMemoryType { .. } => 400,
            Self::InvalidRelationType { .. } => 400,
            Self::CommandBlocked { .. } => 403,
            Self::RateLimited { .. } => 429,
            Self::ModelNotAvailable { .. } => 503,
            _ => 500,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::NotFound { entity_type, .. } => {
                format!("The requested {} could not be found.", entity_type.to_lowercase())
            }
            Self::ValidationError { field, message } => {
                format!("Invalid {}: {}", field, message)
            }
            Self::CommandBlocked { reason, .. } => {
                format!("Command not allowed: {}", reason)
            }
            Self::RateLimited { retry_after_secs } => {
                format!("Too many requests. Please try again in {} seconds.", retry_after_secs)
            }
            Self::ModelNotAvailable { model_name } => {
                format!("The AI model '{}' is not currently available.", model_name)
            }
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
}

// =============================================================================
// Error Conversions
// =============================================================================

impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        Self::InternalError {
            message: format!("IO error: {}", err),
        }
    }
}

impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError {
            message: err.to_string(),
        }
    }
}

/// =============================================================================
/// ErrorResponse - API error response format
/// =============================================================================
/// Standard format for error responses in the REST API.
/// =============================================================================
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    /// Error type/code
    pub error: String,
    /// Human-readable message
    pub message: String,
    /// HTTP status code
    pub status: u16,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Create an error response from a DomainError
    pub fn from_domain_error(err: &DomainError, request_id: Option<String>) -> Self {
        Self {
            error: error_code(err),
            message: err.user_message(),
            status: err.status_code(),
            request_id,
            details: None,
        }
    }
}

/// Get error code string for a DomainError
fn error_code(err: &DomainError) -> String {
    match err {
        DomainError::DatabaseError { .. } => "DATABASE_ERROR",
        DomainError::NotFound { .. } => "NOT_FOUND",
        DomainError::DuplicateEntry { .. } => "DUPLICATE_ENTRY",
        DomainError::TransactionError { .. } => "TRANSACTION_ERROR",
        DomainError::ModelNotAvailable { .. } => "MODEL_NOT_AVAILABLE",
        DomainError::InferenceError { .. } => "INFERENCE_ERROR",
        DomainError::ContextTooLarge { .. } => "CONTEXT_TOO_LARGE",
        DomainError::EmbeddingError { .. } => "EMBEDDING_ERROR",
        DomainError::ModelPullError { .. } => "MODEL_PULL_ERROR",
        DomainError::SearchError { .. } => "SEARCH_ERROR",
        DomainError::RateLimited { .. } => "RATE_LIMITED",
        DomainError::FetchError { .. } => "FETCH_ERROR",
        DomainError::CommandBlocked { .. } => "COMMAND_BLOCKED",
        DomainError::CommandTimeout { .. } => "COMMAND_TIMEOUT",
        DomainError::CommandFailed { .. } => "COMMAND_FAILED",
        DomainError::CommandParseError { .. } => "COMMAND_PARSE_ERROR",
        DomainError::ValidationError { .. } => "VALIDATION_ERROR",
        DomainError::InvalidMemoryType { .. } => "INVALID_MEMORY_TYPE",
        DomainError::InvalidRelationType { .. } => "INVALID_RELATION_TYPE",
        DomainError::TaskExecutionError { .. } => "TASK_EXECUTION_ERROR",
        DomainError::ToolNotFound { .. } => "TOOL_NOT_FOUND",
        DomainError::ToolError { .. } => "TOOL_ERROR",
        DomainError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
        DomainError::MissingEnvVar { .. } => "MISSING_ENV_VAR",
        DomainError::NetworkError { .. } => "NETWORK_ERROR",
        DomainError::SerializationError { .. } => "SERIALIZATION_ERROR",
        DomainError::InternalError { .. } => "INTERNAL_ERROR",
    }
    .to_string()
}
