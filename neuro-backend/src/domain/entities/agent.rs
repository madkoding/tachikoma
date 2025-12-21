//! =============================================================================
//! Agent Task Entity
//! =============================================================================
//! Represents tasks that the AI agent can execute.
//! Includes web search, command execution, and memory operations.
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// =============================================================================
/// AgentTask - Represents an executable task for the AI agent
/// =============================================================================
/// Tasks are operations that the AI decides to perform during a conversation.
/// Each task has a type, input parameters, and produces a result.
/// 
/// # Task Types
/// 
/// * `WebSearch` - Search the web using Searxng
/// * `ExecuteCommand` - Run a safe local command
/// * `RememberFact` - Store information in memory
/// * `RecallMemory` - Retrieve relevant memories
/// * `CodeGeneration` - Generate or modify code
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Unique identifier for the task
    pub id: Uuid,

    /// The type of task to execute
    pub task_type: TaskType,

    /// Input parameters for the task
    pub input: TaskInput,

    /// Current status of the task
    pub status: TaskStatus,

    /// Result of the task execution (if completed)
    pub result: Option<TaskResult>,

    /// ID of the conversation that spawned this task
    pub conversation_id: Option<Uuid>,

    /// When the task was created
    pub created_at: DateTime<Utc>,

    /// When the task was completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Model tier used for this task
    pub model_tier: Option<String>,
}

impl AgentTask {
    /// =========================================================================
    /// Create a new agent task
    /// =========================================================================
    pub fn new(task_type: TaskType, input: TaskInput) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            input,
            status: TaskStatus::Pending,
            result: None,
            conversation_id: None,
            created_at: Utc::now(),
            completed_at: None,
            model_tier: None,
        }
    }

    /// =========================================================================
    /// Create a web search task
    /// =========================================================================
    pub fn web_search(query: String) -> Self {
        Self::new(
            TaskType::WebSearch,
            TaskInput::WebSearch { query, max_results: 5 },
        )
    }

    /// =========================================================================
    /// Create a command execution task
    /// =========================================================================
    pub fn execute_command(command: String, working_dir: Option<String>) -> Self {
        Self::new(
            TaskType::ExecuteCommand,
            TaskInput::ExecuteCommand {
                command,
                working_dir,
                timeout_secs: 30,
            },
        )
    }

    /// =========================================================================
    /// Create a memory storage task
    /// =========================================================================
    pub fn remember(content: String, memory_type: String) -> Self {
        Self::new(
            TaskType::RememberFact,
            TaskInput::Remember {
                content,
                memory_type,
                tags: Vec::new(),
            },
        )
    }

    /// =========================================================================
    /// Create a memory recall task
    /// =========================================================================
    pub fn recall(query: String, limit: usize) -> Self {
        Self::new(
            TaskType::RecallMemory,
            TaskInput::Recall { query, limit },
        )
    }

    /// =========================================================================
    /// Mark the task as running
    /// =========================================================================
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
    }

    /// =========================================================================
    /// Mark the task as completed with a result
    /// =========================================================================
    pub fn complete(&mut self, result: TaskResult) {
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
    }

    /// =========================================================================
    /// Mark the task as failed
    /// =========================================================================
    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.result = Some(TaskResult::Error { message: error });
        self.completed_at = Some(Utc::now());
    }

    /// =========================================================================
    /// Get execution duration in milliseconds
    /// =========================================================================
    pub fn duration_ms(&self) -> Option<u64> {
        self.completed_at.map(|completed| {
            (completed - self.created_at).num_milliseconds() as u64
        })
    }

    /// =========================================================================
    /// Check if the task requires a large model
    /// =========================================================================
    /// Determines if this task is complex enough to warrant a larger model.
    /// Used by the ModelManager to select the appropriate model tier.
    /// =========================================================================
    pub fn requires_large_model(&self) -> bool {
        match &self.task_type {
            TaskType::CodeGeneration => true,
            TaskType::ComplexReasoning => true,
            TaskType::ExecuteCommand => {
                // Complex commands might need more reasoning
                if let TaskInput::ExecuteCommand { command, .. } = &self.input {
                    command.len() > 100 || command.contains('|') || command.contains("&&")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// =============================================================================
/// TaskType - Classification of agent tasks
/// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Search the web for information
    WebSearch,
    /// Execute a local command
    ExecuteCommand,
    /// Store information in memory
    RememberFact,
    /// Retrieve relevant memories
    RecallMemory,
    /// Generate or modify code
    CodeGeneration,
    /// Complex reasoning task
    ComplexReasoning,
    /// Simple query/response
    SimpleQuery,
}

/// =============================================================================
/// TaskInput - Input parameters for different task types
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskInput {
    /// Web search input
    WebSearch {
        query: String,
        max_results: usize,
    },

    /// Command execution input
    ExecuteCommand {
        command: String,
        working_dir: Option<String>,
        timeout_secs: u64,
    },

    /// Memory storage input
    Remember {
        content: String,
        memory_type: String,
        tags: Vec<String>,
    },

    /// Memory recall input
    Recall {
        query: String,
        limit: usize,
    },

    /// Code generation input
    CodeGeneration {
        prompt: String,
        language: String,
        context: Option<String>,
    },

    /// Simple text input
    Text {
        content: String,
    },
}

/// =============================================================================
/// TaskResult - Output of task execution
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskResult {
    /// Web search results
    WebSearchResults {
        results: Vec<SearchResult>,
        query: String,
    },

    /// Command execution output
    CommandOutput {
        stdout: String,
        stderr: String,
        exit_code: i32,
    },

    /// Memory storage confirmation
    MemoryStored {
        memory_id: Uuid,
        content_preview: String,
    },

    /// Retrieved memories
    MemoriesRecalled {
        memories: Vec<RecalledMemory>,
        query: String,
    },

    /// Generated code
    GeneratedCode {
        code: String,
        language: String,
        explanation: Option<String>,
    },

    /// Simple text response
    Text {
        content: String,
    },

    /// Error result
    Error {
        message: String,
    },
}

/// =============================================================================
/// TaskStatus - Current state of a task
/// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is waiting to be executed
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// =============================================================================
/// SearchResult - A single web search result
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Title of the search result
    pub title: String,
    /// URL of the result
    pub url: String,
    /// Snippet/description of the result
    pub snippet: String,
    /// Source engine (google, duckduckgo, etc.)
    pub engine: Option<String>,
}

/// =============================================================================
/// RecalledMemory - A memory retrieved from the database
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalledMemory {
    /// The memory ID
    pub id: Uuid,
    /// The memory content
    pub content: String,
    /// Similarity score to the query (0.0 - 1.0)
    pub similarity: f64,
    /// Memory type
    pub memory_type: String,
    /// When the memory was created
    pub created_at: DateTime<Utc>,
}
