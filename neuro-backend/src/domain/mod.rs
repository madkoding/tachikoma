//! =============================================================================
//! Domain Layer - Core Business Logic
//! =============================================================================
//! This module contains the core domain entities and business rules.
//! The domain layer is completely independent of infrastructure concerns.
//! 
//! # Modules
//! 
//! - `entities`: Core domain objects (MemoryNode, ChatMessage, etc.)
//! - `value_objects`: Immutable domain values (Relation, ModelTier, etc.)
//! - `ports`: Abstract interfaces for external services (traits)
//! - `errors`: Domain-specific error types
//! =============================================================================

pub mod entities;
pub mod errors;
pub mod ports;
pub mod value_objects;

// Re-export commonly used types for convenience
pub use entities::{memory::MemoryNode, chat::ChatMessage, agent::AgentTask};
pub use value_objects::{relation::Relation, model_tier::ModelTier};
pub use ports::{
    memory_repository::MemoryRepository,
    llm_provider::LlmProvider,
    search_provider::SearchProvider,
    command_executor::CommandExecutor,
};
pub use errors::DomainError;
