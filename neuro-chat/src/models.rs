//! =============================================================================
//! Data Models
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;

// ============================================================================
// Conversation Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
    pub message_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationRecord {
    pub id: Thing,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub message_count: i32,
}

impl ConversationRecord {
    pub fn to_conversation(self) -> Conversation {
        let id = match &self.id.id {
            surrealdb::sql::Id::String(s) => Uuid::parse_str(s).unwrap_or_default(),
            _ => Uuid::new_v4(),
        };
        Conversation {
            id,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
            archived: self.archived,
            message_count: self.message_count,
        }
    }
}

// ============================================================================
// Message Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub model: Option<String>,
    pub tokens: Option<i32>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageRecord {
    pub id: Thing,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub tokens: Option<i32>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl ChatMessageRecord {
    pub fn to_message(self) -> ChatMessage {
        let id = match &self.id.id {
            surrealdb::sql::Id::String(s) => Uuid::parse_str(s).unwrap_or_default(),
            _ => Uuid::new_v4(),
        };
        let conversation_id = Uuid::parse_str(&self.conversation_id).unwrap_or_default();
        let role = match self.role.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => MessageRole::User,
        };
        ChatMessage {
            id,
            conversation_id,
            role,
            content: self.content,
            model: self.model,
            tokens: self.tokens,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub model: Option<String>,
    #[serde(default)]
    pub include_memories: bool,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub content: String,
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub model: String,
    pub tokens_prompt: i32,
    pub tokens_completion: i32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ConversationWithMessages {
    #[serde(flatten)]
    pub conversation: Conversation,
    pub messages: Vec<ChatMessage>,
}

// ============================================================================
// Ollama Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaChatResponse {
    pub message: OllamaMessage,
    pub done: bool,
    #[serde(default)]
    pub prompt_eval_count: i32,
    #[serde(default)]
    pub eval_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct OllamaStreamChunk {
    pub message: Option<OllamaMessage>,
    pub done: bool,
    #[serde(default)]
    pub prompt_eval_count: i32,
    #[serde(default)]
    pub eval_count: i32,
}

// ============================================================================
// Memory Models (for client calls)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub memory: Memory,
    pub similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub content: String,
    pub memory_type: String,
    pub importance_score: f64,
}

#[derive(Debug, Serialize)]
pub struct MemorySearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub threshold: Option<f64>,
}
