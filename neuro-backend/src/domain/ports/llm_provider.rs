//! =============================================================================
//! LLM Provider Port
//! =============================================================================
//! Defines the abstract interface for Large Language Model operations.
//! This port is implemented by the Ollama adapter in the infrastructure layer.
//! =============================================================================

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::domain::{
    errors::DomainError,
    value_objects::model_tier::{ModelConfig, ModelTier},
};

/// =============================================================================
/// LlmProvider - Abstract interface for LLM operations
/// =============================================================================
/// Defines all operations for interacting with large language models.
/// Supports text generation, embeddings, and streaming responses.
/// 
/// # Responsibilities
/// 
/// * Text generation (completion)
/// * Streaming generation
/// * Vector embedding generation
/// * Model management and health checking
/// 
/// # Implementation Notes
/// 
/// Implementations should handle:
/// - Connection retries and timeouts
/// - Model loading/unloading
/// - Token counting and context management
/// - Rate limiting if applicable
/// =============================================================================
#[async_trait]
pub trait LlmProvider: Send + Sync {
    // =========================================================================
    // Generation Operations
    // =========================================================================

    /// =========================================================================
    /// Generate text completion
    /// =========================================================================
    /// Generates a text response for the given prompt using the specified
    /// model configuration.
    /// 
    /// # Arguments
    /// 
    /// * `prompt` - The input prompt/question
    /// * `config` - Model configuration (tier, temperature, etc.)
    /// * `context` - Optional conversation context (previous messages)
    /// 
    /// # Returns
    /// 
    /// * `Ok(GenerationResult)` - The generated response with metadata
    /// * `Err(DomainError)` - If generation fails
    /// 
    /// # Errors
    /// 
    /// * `DomainError::ModelNotAvailable` - Requested model not loaded
    /// * `DomainError::InferenceError` - Generation failed
    /// * `DomainError::ContextTooLarge` - Input exceeds context window
    /// =========================================================================
    async fn generate(
        &self,
        prompt: &str,
        config: &ModelConfig,
        context: Option<&[ChatContext]>,
    ) -> Result<GenerationResult, DomainError>;

    /// =========================================================================
    /// Generate streaming text completion
    /// =========================================================================
    /// Same as `generate` but returns a stream of tokens for real-time display.
    /// 
    /// # Arguments
    /// 
    /// * `prompt` - The input prompt/question
    /// * `config` - Model configuration
    /// * `context` - Optional conversation context
    /// 
    /// # Returns
    /// 
    /// * `Ok(Stream)` - A stream of generation chunks
    /// * `Err(DomainError)` - If stream creation fails
    /// =========================================================================
    async fn generate_stream(
        &self,
        prompt: &str,
        config: &ModelConfig,
        context: Option<&[ChatContext]>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, DomainError>> + Send>>, DomainError>;

    /// =========================================================================
    /// Generate with tool calling support
    /// =========================================================================
    /// Generates a response that may include tool calls (function calls).
    /// The response includes both text and any tool invocations.
    /// 
    /// # Arguments
    /// 
    /// * `prompt` - The input prompt
    /// * `config` - Model configuration
    /// * `tools` - Available tools the model can call
    /// * `context` - Optional conversation context
    /// 
    /// # Returns
    /// 
    /// * `Ok(ToolGenerationResult)` - Response with potential tool calls
    /// * `Err(DomainError)` - If generation fails
    /// =========================================================================
    async fn generate_with_tools(
        &self,
        prompt: &str,
        config: &ModelConfig,
        tools: &[ToolDefinition],
        context: Option<&[ChatContext]>,
    ) -> Result<ToolGenerationResult, DomainError>;

    // =========================================================================
    // Embedding Operations
    // =========================================================================

    /// =========================================================================
    /// Generate text embedding
    /// =========================================================================
    /// Creates a vector embedding for the given text.
    /// Used for semantic search in the GraphRAG system.
    /// 
    /// # Arguments
    /// 
    /// * `text` - The text to embed
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<f32>)` - The embedding vector
    /// * `Err(DomainError)` - If embedding fails
    /// 
    /// # Vector Dimensions
    /// 
    /// The dimension depends on the embedding model:
    /// - nomic-embed-text: 768 dimensions
    /// - bge-large: 1024 dimensions
    /// =========================================================================
    async fn embed(&self, text: &str) -> Result<Vec<f32>, DomainError>;

    /// =========================================================================
    /// Generate embeddings for multiple texts
    /// =========================================================================
    /// Batch embedding generation for efficiency.
    /// 
    /// # Arguments
    /// 
    /// * `texts` - List of texts to embed
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<Vec<f32>>)` - List of embedding vectors
    /// * `Err(DomainError)` - If embedding fails
    /// =========================================================================
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, DomainError>;

    // =========================================================================
    // Model Management
    // =========================================================================

    /// =========================================================================
    /// Check if a model is available
    /// =========================================================================
    /// Verifies that the specified model is loaded and ready for inference.
    /// 
    /// # Arguments
    /// 
    /// * `model_name` - The name of the model to check
    /// 
    /// # Returns
    /// 
    /// `true` if the model is available, `false` otherwise
    /// =========================================================================
    async fn is_model_available(&self, model_name: &str) -> bool;

    /// =========================================================================
    /// List available models
    /// =========================================================================
    /// Returns a list of all models currently loaded and available.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<ModelInfo>)` - List of available models
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    async fn list_models(&self) -> Result<Vec<ModelInfo>, DomainError>;

    /// =========================================================================
    /// Pull/download a model
    /// =========================================================================
    /// Downloads a model if not already present.
    /// 
    /// # Arguments
    /// 
    /// * `model_name` - The name of the model to pull
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Model is now available
    /// * `Err(DomainError)` - If download fails
    /// =========================================================================
    async fn pull_model(&self, model_name: &str) -> Result<(), DomainError>;

    /// =========================================================================
    /// Get the best available model for a tier
    /// =========================================================================
    /// Selects the best available model for the specified tier.
    /// Falls back to lower tiers if the preferred model isn't available.
    /// 
    /// # Arguments
    /// 
    /// * `tier` - The desired model tier
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The name of the best available model
    /// * `Err(DomainError)` - If no suitable model is available
    /// =========================================================================
    async fn get_best_model_for_tier(&self, tier: ModelTier) -> Result<String, DomainError>;

    // =========================================================================
    // Health Check
    // =========================================================================

    /// =========================================================================
    /// Check provider health
    /// =========================================================================
    /// Verifies that the LLM provider is responsive and operational.
    /// 
    /// # Returns
    /// 
    /// * `Ok(HealthStatus)` - Current health status
    /// * `Err(DomainError)` - If health check fails
    /// =========================================================================
    async fn health_check(&self) -> Result<HealthStatus, DomainError>;
}

/// =============================================================================
/// ChatContext - Context for conversation-aware generation
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatContext {
    /// Role of the message (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: String,
}

impl ChatContext {
    /// Create a user context message
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    /// Create an assistant context message
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }

    /// Create a system context message
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }
}

/// =============================================================================
/// GenerationResult - Result of text generation
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationResult {
    /// The generated text
    pub content: String,
    /// Model that was used
    pub model: String,
    /// Number of tokens in the prompt
    pub prompt_tokens: usize,
    /// Number of tokens generated
    pub completion_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// Finish reason (stop, length, etc.)
    pub finish_reason: String,
}

/// =============================================================================
/// StreamChunk - A chunk of streaming generation
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamChunk {
    /// The text content of this chunk
    pub content: String,
    /// Whether this is the final chunk
    pub done: bool,
    /// Model being used
    pub model: Option<String>,
}

/// =============================================================================
/// ToolDefinition - Definition of a callable tool
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// JSON schema for the tool's parameters
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// Create a search web tool definition
    pub fn search_web() -> Self {
        Self {
            name: "search_web".to_string(),
            description: "Search the web for information. Use when you need current information or facts you don't know.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    /// Create an execute command tool definition
    pub fn execute_command() -> Self {
        Self {
            name: "execute_command".to_string(),
            description: "Execute a shell command on the local system. Use for system tasks or running scripts.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Optional working directory"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    /// Create a remember tool definition
    pub fn remember() -> Self {
        Self {
            name: "remember".to_string(),
            description: "Store important information in long-term memory for future reference.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The information to remember"
                    },
                    "memory_type": {
                        "type": "string",
                        "enum": ["fact", "preference", "procedure", "insight"],
                        "description": "Type of memory"
                    }
                },
                "required": ["content", "memory_type"]
            }),
        }
    }
}

/// =============================================================================
/// ToolCall - A tool call made by the model
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Name of the tool to call
    pub name: String,
    /// Arguments for the tool (JSON)
    pub arguments: serde_json::Value,
}

/// =============================================================================
/// ToolGenerationResult - Result with potential tool calls
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolGenerationResult {
    /// Text content (may be empty if tool call is primary)
    pub content: Option<String>,
    /// Tool calls made by the model
    pub tool_calls: Vec<ToolCall>,
    /// Generation metadata
    pub metadata: GenerationResult,
}

/// =============================================================================
/// ModelInfo - Information about an available model
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    /// Model name/tag
    pub name: String,
    /// Model size in bytes
    pub size: u64,
    /// When the model was modified
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Model family (llama, mistral, etc.)
    pub family: Option<String>,
    /// Parameter count
    pub parameter_size: Option<String>,
    /// Quantization level
    pub quantization_level: Option<String>,
}

/// =============================================================================
/// HealthStatus - Provider health status
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    /// Whether the provider is healthy
    pub healthy: bool,
    /// Provider name/version
    pub provider: String,
    /// Number of loaded models
    pub loaded_models: usize,
    /// Available GPU memory in MB
    pub gpu_memory_available_mb: Option<u64>,
    /// Total GPU memory in MB
    pub gpu_memory_total_mb: Option<u64>,
}
