//! =============================================================================
//! LLM Provider Port - Simplified
//! =============================================================================
//! Defines the abstract interface for Large Language Model operations.
//! =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

/// =============================================================================
/// LlmProvider - Abstract interface for LLM operations
/// =============================================================================
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate text completion
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<GenerationResult, DomainError>;

    /// Generate streaming completion (simplified - returns full result)
    async fn generate_stream(&self, prompt: &str, model: Option<&str>) -> Result<GenerationResult, DomainError> {
        self.generate(prompt, model).await
    }

    /// Generate text embedding
    async fn embed(&self, text: &str) -> Result<Vec<f32>, DomainError>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, DomainError>;

    /// Check if a model is available
    async fn is_model_available(&self, model_name: &str) -> bool;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, DomainError>;

    /// Pull/download a model
    async fn pull_model(&self, model_name: &str) -> Result<(), DomainError>;

    /// Check provider health
    async fn health_check(&self) -> Result<bool, DomainError>;
}

/// =============================================================================
/// GenerationResult - Result of text generation
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    /// The generated text
    pub content: String,
    /// Model that was used
    pub model: String,
    /// Number of tokens in the prompt
    pub prompt_tokens: u64,
    /// Number of tokens generated
    pub completion_tokens: u64,
    /// Finish reason
    pub finish_reason: String,
}

/// =============================================================================
/// ModelInfo - Information about an available model
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name/identifier
    pub name: String,
    /// Model size in bytes
    pub size: u64,
    /// When the model was modified
    pub modified_at: String,
    /// Parameter count estimate
    pub parameters: Option<u64>,
    /// Whether this is an embedding model
    pub is_embedding: bool,
}
