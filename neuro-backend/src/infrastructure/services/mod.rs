//! =============================================================================
//! Infrastructure Services Module
//! =============================================================================
//! Contains concrete implementations of external service adapters.
//! These implement the port traits defined in the domain layer.
//! =============================================================================

pub mod ollama;
pub mod searxng;
pub mod command_executor;

pub use ollama::OllamaClient;
pub use searxng::SearxngClient;
pub use command_executor::SafeCommandExecutor;
