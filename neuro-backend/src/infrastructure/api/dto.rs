//! =============================================================================
//! Data Transfer Objects (DTOs)
//! =============================================================================
//! Request and response types for the REST API.
//! These are separate from domain entities to maintain API stability.
//! =============================================================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// =============================================================================
/// Chat DTOs
/// =============================================================================

/// Chat message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    /// User message content
    pub message: String,
    /// Optional conversation ID (creates new if not provided)
    #[serde(default)]
    pub conversation_id: Option<Uuid>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
    /// Optional system prompt override
    #[serde(default)]
    pub system_prompt: Option<String>,
}

/// Chat message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    /// Response content
    pub content: String,
    /// Conversation ID
    pub conversation_id: Uuid,
    /// Message ID
    pub message_id: Uuid,
    /// Model used for generation
    pub model: String,
    /// Tokens used in prompt
    pub tokens_prompt: u64,
    /// Tokens in completion
    pub tokens_completion: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Memories extracted from conversation
    #[serde(default)]
    pub extracted_memories: Vec<MemoryDto>,
}

/// Conversation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationDto {
    /// Conversation ID
    pub id: Uuid,
    /// Conversation title
    pub title: String,
    /// Message count
    pub message_count: usize,
    /// Created timestamp
    pub created_at: String,
    /// Last updated timestamp
    pub updated_at: String,
}

/// =============================================================================
/// Memory DTOs
/// =============================================================================

/// Memory node response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDto {
    /// Unique identifier
    pub id: Uuid,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: String,
    /// Importance score (0.0 - 1.0)
    pub importance_score: f64,
    /// Created timestamp
    pub created_at: String,
    /// Updated timestamp
    pub updated_at: String,
    /// Access count
    pub access_count: u64,
    /// Metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Create memory request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoryRequest {
    /// Memory content
    pub content: String,
    /// Memory type
    #[serde(default = "default_memory_type")]
    pub memory_type: String,
    /// Optional importance score
    #[serde(default)]
    pub importance_score: Option<f64>,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn default_memory_type() -> String {
    "general".to_string()
}

/// Update memory request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    /// New content (optional)
    #[serde(default)]
    pub content: Option<String>,
    /// New importance score (optional)
    #[serde(default)]
    pub importance_score: Option<f64>,
    /// New metadata (optional)
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Semantic search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    /// Search query
    pub query: String,
    /// Maximum results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Minimum similarity score (0.0 - 1.0)
    #[serde(default = "default_min_similarity")]
    pub min_similarity: f64,
    /// Optional memory type filter
    #[serde(default)]
    pub memory_types: Option<Vec<String>>,
}

fn default_limit() -> usize {
    10
}

fn default_min_similarity() -> f64 {
    0.7
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultDto {
    /// Memory node
    pub memory: MemoryDto,
    /// Similarity score
    pub similarity: f64,
}

/// =============================================================================
/// Graph DTOs
/// =============================================================================

/// Graph edge response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeDto {
    /// Source memory ID
    pub from_id: Uuid,
    /// Target memory ID
    pub to_id: Uuid,
    /// Relation type
    pub relation: String,
    /// Confidence score
    pub confidence: f64,
    /// Created timestamp
    pub created_at: String,
}

/// Create relation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationRequest {
    /// Source memory ID
    pub from_id: Uuid,
    /// Target memory ID
    pub to_id: Uuid,
    /// Relation type
    pub relation: String,
    /// Optional confidence score (defaults to 1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    1.0
}

/// Graph statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatsDto {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Nodes by type
    pub nodes_by_type: std::collections::HashMap<String, usize>,
    /// Edges by type
    pub edges_by_type: std::collections::HashMap<String, usize>,
    /// Average connections per node
    pub avg_connections: f64,
}

/// Full graph export response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportDto {
    /// All nodes
    pub nodes: Vec<MemoryDto>,
    /// All edges
    pub edges: Vec<GraphEdgeDto>,
    /// Export timestamp
    pub exported_at: String,
}

/// =============================================================================
/// Agent DTOs
/// =============================================================================

/// Web search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchRequest {
    /// Search query
    pub query: String,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Category filter
    #[serde(default)]
    pub category: Option<String>,
}

/// Web search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResultDto {
    /// Result title
    pub title: String,
    /// Result URL
    pub url: String,
    /// Result snippet
    pub snippet: String,
    /// Source engine
    pub source: String,
    /// Optional score
    #[serde(default)]
    pub score: Option<f64>,
}

/// Command execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecuteRequest {
    /// Command to execute
    pub command: String,
    /// Optional working directory
    #[serde(default)]
    pub working_directory: Option<String>,
    /// Optional timeout in seconds
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResultDto {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration in ms
    pub duration_ms: u64,
    /// Whether command timed out
    pub timed_out: bool,
}

/// =============================================================================
/// System DTOs
/// =============================================================================

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: String,
    /// Service statuses
    pub services: ServiceStatusDto,
    /// Version info
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Service status details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatusDto {
    /// Database status
    pub database: String,
    /// LLM provider status
    pub llm: String,
    /// Search provider status
    pub search: String,
}

/// Model info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoDto {
    /// Model identifier
    pub id: String,
    /// Model name
    pub name: String,
    /// Size in bytes
    #[serde(default)]
    pub size_bytes: Option<u64>,
    /// Parameter count
    #[serde(default)]
    pub parameters: Option<u64>,
    /// Context length
    #[serde(default)]
    pub context_length: Option<usize>,
    /// Is embedding model
    pub is_embedding_model: bool,
}

/// =============================================================================
/// Generic Response Wrappers
/// =============================================================================

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Response data
    pub data: Vec<T>,
    /// Total count
    pub total: usize,
    /// Current page
    pub page: usize,
    /// Items per page
    pub per_page: usize,
    /// Total pages
    pub total_pages: usize,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Optional details
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Add details to error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
