use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Database error: {message}")]
    DatabaseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Entity not found: {entity_type} with ID {id}")]
    NotFound { entity_type: String, id: String },

    #[error("Duplicate entry: {entity_type} with ID {id} already exists")]
    DuplicateEntry { entity_type: String, id: String },

    #[error("Inference error: {message}")]
    InferenceError { message: String },

    #[error("Search error: {message}")]
    SearchError { message: String },

    #[error("Command blocked: {command} - {reason}")]
    CommandBlocked { command: String, reason: String },

    #[error("Command failed: {command} exited with code {exit_code}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("Invalid command syntax: {message}")]
    CommandParseError { message: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl DomainError {
    pub fn database(message: impl Into<String>) -> Self {
        Self::DatabaseError {
            message: message.into(),
            source: None,
        }
    }

    pub fn not_found(entity_type: impl Into<String>, id: impl ToString) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.to_string(),
        }
    }

    pub fn memory_not_found(id: Uuid) -> Self {
        Self::not_found("Memory", id)
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn llm_error(message: impl Into<String>) -> Self {
        Self::InferenceError {
            message: message.into(),
        }
    }

    pub fn search(message: impl Into<String>) -> Self {
        Self::SearchError {
            message: message.into(),
        }
    }

    pub fn command_blocked(command: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CommandBlocked {
            command: command.into(),
            reason: reason.into(),
        }
    }

    pub fn command_error(message: impl Into<String>) -> Self {
        Self::CommandParseError {
            message: message.into(),
        }
    }

    pub fn is_retriable(&self) -> bool {
        matches!(self, Self::DatabaseError { .. } | Self::InternalError { .. })
    }

    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            Self::ValidationError { .. } | Self::CommandBlocked { .. } | Self::CommandParseError { .. }
        )
    }

    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound { .. } => 404,
            Self::DuplicateEntry { .. } => 409,
            Self::ValidationError { .. } => 400,
            Self::CommandBlocked { .. } => 403,
            _ => 500,
        }
    }

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
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
}

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

impl From<reqwest::Error> for DomainError {
    fn from(err: reqwest::Error) -> Self {
        Self::DatabaseError {
            message: format!("HTTP error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_status_code() {
        let err = DomainError::not_found("Memory", Uuid::new_v4());
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_validation_error_status_code() {
        let err = DomainError::validation("title", "is required");
        assert_eq!(err.status_code(), 400);
    }

    #[test]
    fn test_command_blocked_status_code() {
        let err = DomainError::command_blocked("rm -rf", "dangerous");
        assert_eq!(err.status_code(), 403);
    }

    #[test]
    fn test_is_retriable() {
        assert!(DomainError::database("conn lost").is_retriable());
        assert!(DomainError::InternalError { message: "oops".to_string() }.is_retriable());
        assert!(!DomainError::not_found("Memory", "x").is_retriable());
    }

    #[test]
    fn test_is_user_error() {
        assert!(DomainError::validation("f", "m").is_user_error());
        assert!(DomainError::command_blocked("cmd", "r").is_user_error());
        assert!(!DomainError::database("err").is_user_error());
    }

    #[test]
    fn test_user_message() {
        let err = DomainError::not_found("Memory", Uuid::new_v4());
        assert!(err.user_message().contains("memory"));
    }
}