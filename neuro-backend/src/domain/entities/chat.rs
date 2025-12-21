//! =============================================================================
//! Chat Message Entity
//! =============================================================================
//! Represents messages in the chat interface and conversation history.
//! Tracks the flow of communication between user and AI.
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// =============================================================================
/// ChatMessage - Conversation Message Entity
/// =============================================================================
/// Represents a single message in a conversation.
/// Can be from a user, assistant, or system.
/// 
/// # Fields
/// 
/// * `id` - Unique identifier for the message
/// * `conversation_id` - ID of the parent conversation
/// * `role` - The role of the message sender (user/assistant/system)
/// * `content` - The text content of the message
/// * `metadata` - Additional message metadata
/// * `created_at` - Timestamp when the message was created
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique identifier for the message
    pub id: Uuid,

    /// ID of the parent conversation this message belongs to
    pub conversation_id: Uuid,

    /// The role of the message sender
    pub role: MessageRole,

    /// The text content of the message
    pub content: String,

    /// Additional metadata about the message
    #[serde(default)]
    pub metadata: MessageMetadata,

    /// Timestamp when the message was created
    pub created_at: DateTime<Utc>,
}

impl ChatMessage {
    /// =========================================================================
    /// Create a new user message
    /// =========================================================================
    /// Creates a message from the user role.
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The ID of the conversation
    /// * `content` - The message content
    /// 
    /// # Returns
    /// 
    /// A new `ChatMessage` with user role
    /// =========================================================================
    pub fn user(conversation_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            role: MessageRole::User,
            content,
            metadata: MessageMetadata::default(),
            created_at: Utc::now(),
        }
    }

    /// =========================================================================
    /// Create a new assistant message
    /// =========================================================================
    /// Creates a message from the assistant role.
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The ID of the conversation
    /// * `content` - The message content
    /// 
    /// # Returns
    /// 
    /// A new `ChatMessage` with assistant role
    /// =========================================================================
    pub fn assistant(conversation_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            role: MessageRole::Assistant,
            content,
            metadata: MessageMetadata::default(),
            created_at: Utc::now(),
        }
    }

    /// =========================================================================
    /// Create a new system message
    /// =========================================================================
    /// Creates a message from the system role (instructions, context).
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The ID of the conversation
    /// * `content` - The system instruction content
    /// 
    /// # Returns
    /// 
    /// A new `ChatMessage` with system role
    /// =========================================================================
    pub fn system(conversation_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            role: MessageRole::System,
            content,
            metadata: MessageMetadata::default(),
            created_at: Utc::now(),
        }
    }

    /// =========================================================================
    /// Add tool call information to the message
    /// =========================================================================
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.metadata.tool_calls = tool_calls;
        self
    }
}

/// =============================================================================
/// MessageRole - Role of the message sender
/// =============================================================================
/// Defines who sent the message in the conversation.
/// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the AI assistant
    Assistant,
    /// System instructions or context
    System,
    /// Tool/function response
    Tool,
}

/// =============================================================================
/// MessageMetadata - Additional message attributes
/// =============================================================================
/// Contains supplementary information about a chat message.
/// =============================================================================
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Model used to generate the response (for assistant messages)
    pub model: Option<String>,

    /// Token count for the message
    pub token_count: Option<u32>,

    /// Generation time in milliseconds
    pub generation_time_ms: Option<u64>,

    /// Tool calls made during this message
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,

    /// Memory IDs that were used as context
    #[serde(default)]
    pub context_memory_ids: Vec<Uuid>,

    /// Whether the message was edited
    #[serde(default)]
    pub edited: bool,

    /// Error information if generation failed
    pub error: Option<String>,
}

/// =============================================================================
/// ToolCall - Represents a tool/function call by the assistant
/// =============================================================================
/// Tracks when the AI decides to use a tool (search, command, remember).
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for the tool call
    pub id: Uuid,

    /// Name of the tool that was called
    pub tool_name: String,

    /// Arguments passed to the tool (JSON)
    pub arguments: serde_json::Value,

    /// Result returned by the tool
    pub result: Option<String>,

    /// Whether the tool call succeeded
    pub success: bool,

    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,
}

impl ToolCall {
    /// =========================================================================
    /// Create a new tool call record
    /// =========================================================================
    pub fn new(tool_name: String, arguments: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            arguments,
            result: None,
            success: false,
            execution_time_ms: None,
        }
    }

    /// =========================================================================
    /// Mark the tool call as completed
    /// =========================================================================
    pub fn complete(mut self, result: String, execution_time_ms: u64) -> Self {
        self.result = Some(result);
        self.success = true;
        self.execution_time_ms = Some(execution_time_ms);
        self
    }

    /// =========================================================================
    /// Mark the tool call as failed
    /// =========================================================================
    pub fn fail(mut self, error: String) -> Self {
        self.result = Some(error);
        self.success = false;
        self
    }
}

/// =============================================================================
/// Conversation - Groups related messages
/// =============================================================================
/// Represents a conversation session with the AI.
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for the conversation
    pub id: Uuid,

    /// Title of the conversation (auto-generated or user-defined)
    pub title: Option<String>,

    /// Messages in the conversation
    #[serde(default)]
    pub messages: Vec<ChatMessage>,

    /// When the conversation was started
    pub created_at: DateTime<Utc>,

    /// When the conversation was last updated
    pub updated_at: DateTime<Utc>,

    /// Whether the conversation is archived
    #[serde(default)]
    pub archived: bool,
}

impl Conversation {
    /// =========================================================================
    /// Create a new conversation
    /// =========================================================================
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: None,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            archived: false,
        }
    }

    /// =========================================================================
    /// Create a conversation with a title
    /// =========================================================================
    pub fn with_title(title: String) -> Self {
        let mut conv = Self::new();
        conv.title = Some(title);
        conv
    }

    /// =========================================================================
    /// Add a message to the conversation
    /// =========================================================================
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// =========================================================================
    /// Get the last N messages (for context window)
    /// =========================================================================
    pub fn last_messages(&self, n: usize) -> &[ChatMessage] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

/// =============================================================================
/// ChatRequest - Incoming chat request from API
/// =============================================================================
/// Represents an incoming chat message from the user interface.
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// The user's message content
    pub message: String,

    /// Optional conversation ID (creates new if not provided)
    pub conversation_id: Option<Uuid>,

    /// Language preference (for i18n)
    #[serde(default = "default_language")]
    pub language: String,

    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
}

fn default_language() -> String {
    "en".to_string()
}

/// =============================================================================
/// ChatResponse - Outgoing chat response from API
/// =============================================================================
/// Represents the AI's response to a chat message.
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The conversation ID
    pub conversation_id: Uuid,

    /// The assistant's response message
    pub message: ChatMessage,

    /// Tools that were used during response generation
    #[serde(default)]
    pub tools_used: Vec<String>,

    /// Memory IDs that were used as context
    #[serde(default)]
    pub context_memories: Vec<Uuid>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}
