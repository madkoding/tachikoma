//! =============================================================================
//! Agent Orchestrator Service
//! =============================================================================
//! Coordinates AI agent operations including tool execution and memory access.
//! Implements the Agent trait methods: search_web(), execute_command(), remember().
//! 
//! # Agent Architecture
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        AGENT ORCHESTRATOR                               │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │   User Query ──▶ [Intent Analysis] ──▶ [Tool Selection] ──▶ [Execute]  │
//! │                         │                    │                │         │
//! │                         ▼                    ▼                ▼         │
//! │                  ┌───────────┐        ┌───────────┐    ┌───────────┐   │
//! │                  │  Memory   │        │   Tools   │    │  Response │   │
//! │                  │  Context  │        │   Pool    │    │  Builder  │   │
//! │                  └───────────┘        └───────────┘    └───────────┘   │
//! │                                             │                          │
//! │                    ┌────────────────────────┼────────────────────┐     │
//! │                    ▼                        ▼                    ▼     │
//! │              search_web()           execute_cmd()         remember()   │
//! │                                                                        │
//! └────────────────────────────────────────────────────────────────────────┘
//! ```
//! =============================================================================

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::application::services::{memory_service::MemoryService, model_manager::ModelManager};
use crate::domain::{
    entities::{
        agent::{AgentTask, RecalledMemory, SearchResult, TaskInput, TaskResult, TaskStatus, TaskType},
        memory::MemoryType,
    },
    errors::DomainError,
    ports::{
        command_executor::{CommandExecutor, CommandOutput, ExecutionOptions, ValidationResult},
        llm_provider::{ChatContext, GenerationResult, LlmProvider, ToolCall, ToolDefinition},
        search_provider::{SearchOptions, SearchProvider, SearchResults},
    },
};

/// =============================================================================
/// AgentOrchestrator - AI Agent Coordination Service
/// =============================================================================
/// Manages the execution of agent tasks including tool calls, memory access,
/// and response generation. Acts as the central coordinator for all AI
/// agent operations.
/// 
/// # Responsibilities
/// 
/// * Task routing and execution
/// * Tool invocation (search, command, memory)
/// * Response generation with context
/// * Error handling and retry logic
/// 
/// # Example Usage
/// 
/// ```rust
/// let orchestrator = AgentOrchestrator::new(
///     memory_service,
///     model_manager,
///     llm_provider,
///     search_provider,
///     command_executor,
/// );
/// 
/// // Execute a task
/// let result = orchestrator.execute_task(task).await?;
/// 
/// // Use specific tools
/// let search_results = orchestrator.search_web("rust async").await?;
/// let output = orchestrator.execute_command("ls -la", None).await?;
/// orchestrator.remember("User prefers Rust", "preference").await?;
/// ```
/// =============================================================================
pub struct AgentOrchestrator {
    /// Memory service for memory operations
    memory_service: Arc<MemoryService>,
    
    /// Model manager for model selection
    model_manager: Arc<ModelManager>,
    
    /// LLM provider for generation
    llm_provider: Arc<dyn LlmProvider>,
    
    /// Search provider for web search
    search_provider: Arc<dyn SearchProvider>,
    
    /// Command executor for local commands
    command_executor: Arc<dyn CommandExecutor>,
}

impl AgentOrchestrator {
    /// =========================================================================
    /// Create a new AgentOrchestrator
    /// =========================================================================
    /// Initializes the orchestrator with all required service dependencies.
    /// 
    /// # Arguments
    /// 
    /// * `memory_service` - Service for memory operations
    /// * `model_manager` - Manager for model selection
    /// * `llm_provider` - Provider for LLM operations
    /// * `search_provider` - Provider for web search
    /// * `command_executor` - Executor for local commands
    /// 
    /// # Returns
    /// 
    /// A new AgentOrchestrator instance
    /// =========================================================================
    pub fn new(
        memory_service: Arc<MemoryService>,
        model_manager: Arc<ModelManager>,
        llm_provider: Arc<dyn LlmProvider>,
        search_provider: Arc<dyn SearchProvider>,
        command_executor: Arc<dyn CommandExecutor>,
    ) -> Self {
        Self {
            memory_service,
            model_manager,
            llm_provider,
            search_provider,
            command_executor,
        }
    }

    // =========================================================================
    // Core Agent Methods (Agent Trait Implementation)
    // =========================================================================

    /// =========================================================================
    /// Search the web for information
    /// =========================================================================
    /// Performs a web search using Searxng and returns formatted results.
    /// This is one of the primary tools available to the AI agent.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The search query string
    /// 
    /// # Returns
    /// 
    /// * `Ok(SearchResults)` - The search results
    /// * `Err(DomainError)` - If search fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let results = orchestrator.search_web("rust async best practices").await?;
    /// for result in results.results {
    ///     println!("{}: {}", result.title, result.url);
    /// }
    /// ```
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn search_web(&self, query: &str) -> Result<SearchResults, DomainError> {
        info!(query = query, "Executing web search");
        
        let start = Instant::now();
        
        let options = SearchOptions::with_limit(10);
        let results = self.search_provider.search(query, Some(options)).await?;

        let duration = start.elapsed();
        info!(
            query = query,
            results = results.results.len(),
            duration_ms = duration.as_millis(),
            "Web search completed"
        );

        Ok(results)
    }

    /// =========================================================================
    /// Execute a local command safely
    /// =========================================================================
    /// Executes a shell command with security validation and resource limits.
    /// Commands are validated against an allowlist before execution.
    /// 
    /// # Arguments
    /// 
    /// * `command` - The shell command to execute
    /// * `working_dir` - Optional working directory
    /// 
    /// # Returns
    /// 
    /// * `Ok(CommandOutput)` - The command output
    /// * `Err(DomainError)` - If execution fails or is blocked
    /// 
    /// # Security
    /// 
    /// Commands are validated against a blocklist of dangerous patterns.
    /// Only commands in the allowlist are permitted.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let output = orchestrator.execute_command("ls -la", Some("/home/user")).await?;
    /// println!("Exit code: {}", output.exit_code);
    /// println!("Output: {}", output.stdout);
    /// ```
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn execute_command(
        &self,
        command: &str,
        working_dir: Option<String>,
    ) -> Result<CommandOutput, DomainError> {
        info!(command = command, "Executing command");

        // First validate the command
        let validation = self.validate_command(command).await?;
        if !validation.allowed {
            warn!(
                command = command,
                reason = %validation.reason,
                "Command blocked"
            );
            return Err(DomainError::command_blocked(command, &validation.reason));
        }

        // Build execution options
        let options = if let Some(dir) = working_dir {
            ExecutionOptions::with_working_dir(dir)
        } else {
            ExecutionOptions::default()
        };

        // Execute the command
        let start = Instant::now();
        let output = self.command_executor.execute(command, options).await?;
        let duration = start.elapsed();

        info!(
            command = command,
            exit_code = output.exit_code,
            duration_ms = duration.as_millis(),
            "Command execution completed"
        );

        Ok(output)
    }

    /// =========================================================================
    /// Store information in long-term memory
    /// =========================================================================
    /// Saves a piece of information to the memory system for future retrieval.
    /// The memory is automatically embedded for semantic search.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The information to remember
    /// * `memory_type` - Type classification (fact, preference, procedure, etc.)
    /// 
    /// # Returns
    /// 
    /// * `Ok(Uuid)` - The ID of the created memory
    /// * `Err(DomainError)` - If storage fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let memory_id = orchestrator.remember(
    ///     "User prefers dark mode in all applications",
    ///     "preference"
    /// ).await?;
    /// ```
    /// =========================================================================
    #[instrument(skip(self, content), fields(content_len = content.len()))]
    pub async fn remember(
        &self,
        content: &str,
        memory_type: &str,
    ) -> Result<Uuid, DomainError> {
        info!(memory_type = memory_type, "Storing new memory");

        // Parse memory type
        let mem_type = self.parse_memory_type(memory_type)?;

        // Create memory
        let memory = self.memory_service
            .create_memory(content.to_string(), mem_type, None)
            .await?;

        info!(memory_id = %memory.id, "Memory stored successfully");

        Ok(memory.id)
    }

    /// =========================================================================
    /// Recall relevant memories
    /// =========================================================================
    /// Searches the memory system for relevant information.
    /// Uses semantic search for best results.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The query to search for
    /// * `limit` - Maximum number of memories to return
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<RecalledMemory>)` - Relevant memories with similarity scores
    /// * `Err(DomainError)` - If recall fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let memories = orchestrator.recall("user interface preferences", 5).await?;
    /// for memory in memories {
    ///     println!("[{:.2}] {}", memory.similarity, memory.content);
    /// }
    /// ```
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn recall(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RecalledMemory>, DomainError> {
        debug!(query = query, limit = limit, "Recalling memories");

        let results = self.memory_service.search(query, limit).await?;

        let memories: Vec<RecalledMemory> = results
            .into_iter()
            .map(|(memory, similarity)| RecalledMemory {
                id: memory.id,
                content: memory.content,
                similarity,
                memory_type: format!("{:?}", memory.memory_type).to_lowercase(),
                created_at: memory.created_at,
            })
            .collect();

        debug!(
            query = query,
            results = memories.len(),
            "Memory recall completed"
        );

        Ok(memories)
    }

    // =========================================================================
    // Task Execution
    // =========================================================================

    /// =========================================================================
    /// Execute an agent task
    /// =========================================================================
    /// Routes and executes a task based on its type.
    /// 
    /// # Arguments
    /// 
    /// * `task` - The task to execute
    /// 
    /// # Returns
    /// 
    /// * `Ok(AgentTask)` - The completed task with result
    /// * `Err(DomainError)` - If execution fails
    /// =========================================================================
    #[instrument(skip(self, task), fields(task_id = %task.id, task_type = ?task.task_type))]
    pub async fn execute_task(&self, mut task: AgentTask) -> Result<AgentTask, DomainError> {
        info!("Executing agent task");
        task.start();

        let result = match &task.task_type {
            TaskType::WebSearch => self.execute_web_search_task(&task.input).await,
            TaskType::ExecuteCommand => self.execute_command_task(&task.input).await,
            TaskType::RememberFact => self.execute_remember_task(&task.input).await,
            TaskType::RecallMemory => self.execute_recall_task(&task.input).await,
            TaskType::CodeGeneration => self.execute_code_generation_task(&task.input).await,
            TaskType::ComplexReasoning => self.execute_reasoning_task(&task.input).await,
            TaskType::SimpleQuery => self.execute_simple_query_task(&task.input).await,
        };

        match result {
            Ok(task_result) => {
                task.complete(task_result);
                info!(task_id = %task.id, "Task completed successfully");
            }
            Err(e) => {
                error!(task_id = %task.id, error = %e, "Task failed");
                task.fail(e.to_string());
            }
        }

        Ok(task)
    }

    /// Execute a web search task
    async fn execute_web_search_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::WebSearch { query, max_results } = input {
            let options = SearchOptions::with_limit(*max_results);
            let results = self.search_provider.search(query, Some(options)).await?;

            let search_results: Vec<SearchResult> = results
                .results
                .into_iter()
                .map(|r| SearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r.snippet,
                    engine: r.engine,
                })
                .collect();

            Ok(TaskResult::WebSearchResults {
                results: search_results,
                query: query.clone(),
            })
        } else {
            Err(DomainError::internal("Invalid input for web search task"))
        }
    }

    /// Execute a command execution task
    async fn execute_command_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::ExecuteCommand { command, working_dir, timeout_secs } = input {
            let mut options = ExecutionOptions::with_timeout(*timeout_secs);
            if let Some(dir) = working_dir {
                options.working_dir = Some(dir.clone());
            }

            let output = self.command_executor.execute(command, options).await?;

            Ok(TaskResult::CommandOutput {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_code: output.exit_code,
            })
        } else {
            Err(DomainError::internal("Invalid input for command task"))
        }
    }

    /// Execute a remember task
    async fn execute_remember_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::Remember { content, memory_type, tags } = input {
            let mem_type = self.parse_memory_type(memory_type)?;
            
            let mut metadata = crate::domain::entities::memory::MemoryMetadata::default();
            metadata.tags = tags.clone();

            let memory = self.memory_service
                .create_memory(content.clone(), mem_type, Some(metadata))
                .await?;

            Ok(TaskResult::MemoryStored {
                memory_id: memory.id,
                content_preview: content.chars().take(50).collect(),
            })
        } else {
            Err(DomainError::internal("Invalid input for remember task"))
        }
    }

    /// Execute a recall task
    async fn execute_recall_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::Recall { query, limit } = input {
            let memories = self.recall(query, *limit).await?;

            Ok(TaskResult::MemoriesRecalled {
                memories,
                query: query.clone(),
            })
        } else {
            Err(DomainError::internal("Invalid input for recall task"))
        }
    }

    /// Execute a code generation task
    async fn execute_code_generation_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::CodeGeneration { prompt, language, context } = input {
            // Use heavy model for code generation
            let config = self.model_manager
                .select_model(true, true, false)
                .await?;

            let full_prompt = if let Some(ctx) = context {
                format!(
                    "Context:\n{}\n\nTask: Generate {} code for:\n{}",
                    ctx, language, prompt
                )
            } else {
                format!("Generate {} code for:\n{}", language, prompt)
            };

            let result = self.llm_provider.generate(&full_prompt, &config, None).await?;

            Ok(TaskResult::GeneratedCode {
                code: result.content,
                language: language.clone(),
                explanation: None,
            })
        } else {
            Err(DomainError::internal("Invalid input for code generation task"))
        }
    }

    /// Execute a reasoning task
    async fn execute_reasoning_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::Text { content } = input {
            let config = self.model_manager
                .select_model(false, true, false)
                .await?;

            let result = self.llm_provider.generate(content, &config, None).await?;

            Ok(TaskResult::Text {
                content: result.content,
            })
        } else {
            Err(DomainError::internal("Invalid input for reasoning task"))
        }
    }

    /// Execute a simple query task
    async fn execute_simple_query_task(&self, input: &TaskInput) -> Result<TaskResult, DomainError> {
        if let TaskInput::Text { content } = input {
            let config = self.model_manager
                .select_model(false, false, true)
                .await?;

            let result = self.llm_provider.generate(content, &config, None).await?;

            Ok(TaskResult::Text {
                content: result.content,
            })
        } else {
            Err(DomainError::internal("Invalid input for simple query task"))
        }
    }

    // =========================================================================
    // Generation with Tools
    // =========================================================================

    /// =========================================================================
    /// Generate a response with tool access
    /// =========================================================================
    /// Generates an AI response with the ability to call tools.
    /// Handles the full agent loop including tool execution.
    /// 
    /// # Arguments
    /// 
    /// * `prompt` - The user prompt
    /// * `context` - Optional conversation context
    /// * `max_tool_calls` - Maximum number of tool calls to allow
    /// 
    /// # Returns
    /// 
    /// * `Ok((String, Vec<String>))` - Response and list of tools used
    /// * `Err(DomainError)` - If generation fails
    /// =========================================================================
    #[instrument(skip(self, prompt, context))]
    pub async fn generate_with_tools(
        &self,
        prompt: &str,
        context: Option<&[ChatContext]>,
        max_tool_calls: usize,
    ) -> Result<(String, Vec<String>), DomainError> {
        let tools = self.get_available_tools();
        let config = self.model_manager.select_model(false, false, false).await?;

        let mut tools_used = Vec::new();
        let mut accumulated_context = Vec::new();

        // Add existing context
        if let Some(ctx) = context {
            accumulated_context.extend_from_slice(ctx);
        }

        // Add the user prompt
        accumulated_context.push(ChatContext::user(prompt.to_string()));

        // Agent loop
        for iteration in 0..max_tool_calls {
            debug!(iteration = iteration, "Agent iteration");

            let result = self.llm_provider
                .generate_with_tools(prompt, &config, &tools, Some(&accumulated_context))
                .await?;

            // Check if we have tool calls
            if result.tool_calls.is_empty() {
                // No tool calls, return the response
                return Ok((
                    result.content.unwrap_or_default(),
                    tools_used,
                ));
            }

            // Execute tool calls
            for tool_call in result.tool_calls {
                info!(tool = %tool_call.name, "Executing tool call");
                tools_used.push(tool_call.name.clone());

                let tool_result = self.execute_tool_call(&tool_call).await?;

                // Add tool result to context
                accumulated_context.push(ChatContext::assistant(format!(
                    "Tool {} called with result: {}",
                    tool_call.name, tool_result
                )));
            }
        }

        // Max iterations reached, generate final response
        let final_result = self.llm_provider
            .generate(prompt, &config, Some(&accumulated_context))
            .await?;

        Ok((final_result.content, tools_used))
    }

    /// Get available tool definitions
    fn get_available_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::search_web(),
            ToolDefinition::execute_command(),
            ToolDefinition::remember(),
        ]
    }

    /// Execute a single tool call
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<String, DomainError> {
        match tool_call.name.as_str() {
            "search_web" => {
                let query = tool_call.arguments["query"]
                    .as_str()
                    .ok_or_else(|| DomainError::validation("query", "Missing query parameter"))?;

                let results = self.search_web(query).await?;
                Ok(results.as_context(5))
            }
            "execute_command" => {
                let command = tool_call.arguments["command"]
                    .as_str()
                    .ok_or_else(|| DomainError::validation("command", "Missing command parameter"))?;

                let working_dir = tool_call.arguments["working_dir"]
                    .as_str()
                    .map(|s| s.to_string());

                let output = self.execute_command(command, working_dir).await?;
                Ok(output.as_context())
            }
            "remember" => {
                let content = tool_call.arguments["content"]
                    .as_str()
                    .ok_or_else(|| DomainError::validation("content", "Missing content parameter"))?;

                let memory_type = tool_call.arguments["memory_type"]
                    .as_str()
                    .unwrap_or("general");

                let memory_id = self.remember(content, memory_type).await?;
                Ok(format!("Memory stored with ID: {}", memory_id))
            }
            _ => Err(DomainError::ToolNotFound {
                tool_name: tool_call.name.clone(),
            }),
        }
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Parse memory type from string
    fn parse_memory_type(&self, type_str: &str) -> Result<MemoryType, DomainError> {
        match type_str.to_lowercase().as_str() {
            "fact" => Ok(MemoryType::Fact),
            "preference" => Ok(MemoryType::Preference),
            "procedure" => Ok(MemoryType::Procedure),
            "conversation" => Ok(MemoryType::Conversation),
            "semantic_tag" => Ok(MemoryType::SemanticTag),
            "issue" => Ok(MemoryType::Issue),
            "insight" => Ok(MemoryType::Insight),
            "external_knowledge" | "external" => Ok(MemoryType::ExternalKnowledge),
            "code_snippet" | "code" => Ok(MemoryType::CodeSnippet),
            "general" | _ => Ok(MemoryType::General),
        }
    }

    /// Validate a command before execution
    async fn validate_command(&self, command: &str) -> Result<ValidationResult, DomainError> {
        self.command_executor.validate(command).await
    }

    /// Check if search provider is healthy
    pub async fn is_search_healthy(&self) -> bool {
        self.search_provider.is_healthy().await
    }

    /// Get memory service reference
    pub fn memory_service(&self) -> &Arc<MemoryService> {
        &self.memory_service
    }

    /// Get model manager reference
    pub fn model_manager(&self) -> &Arc<ModelManager> {
        &self.model_manager
    }
}
