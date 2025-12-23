//! =============================================================================
//! Agent Orchestrator Service - Simplified
//! =============================================================================
//! Coordinates AI agent operations including tool execution and memory access.
//! =============================================================================

use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::application::services::{memory_service::MemoryService, model_manager::ModelManager};
use crate::domain::{
    entities::memory::MemoryType,
    errors::DomainError,
    ports::{
        command_executor::{CommandExecutor, CommandOutput, ExecutionOptions},
        llm_provider::LlmProvider,
        search_provider::{SearchOptions, SearchProvider, SearchResults},
    },
};

/// =============================================================================
/// AgentOrchestrator - AI Agent Coordination Service
/// =============================================================================
pub struct AgentOrchestrator {
    memory_service: Arc<MemoryService>,
    model_manager: Arc<ModelManager>,
    llm_provider: Arc<dyn LlmProvider>,
    search_provider: Arc<dyn SearchProvider>,
    command_executor: Arc<dyn CommandExecutor>,
}

impl AgentOrchestrator {
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

    /// Search the web for information
    #[instrument(skip(self))]
    pub async fn search_web(&self, query: &str) -> Result<SearchResults, DomainError> {
        info!(query = query, "Executing web search");
        let options = SearchOptions::with_limit(10);
        self.search_provider.search(query, Some(options)).await
    }

    /// Execute a local command safely
    #[instrument(skip(self))]
    pub async fn execute_command(
        &self,
        command: &str,
        working_dir: Option<String>,
    ) -> Result<CommandOutput, DomainError> {
        info!(command = command, "Executing command");

        // Validate command
        let allowed = self.command_executor.validate(command).await?;
        if !allowed {
            return Err(DomainError::command_blocked(command, "Command not allowed"));
        }

        let options = if let Some(dir) = working_dir {
            Some(ExecutionOptions::with_working_dir(&dir))
        } else {
            None
        };

        self.command_executor.execute(command, options).await
    }

    /// Store information in long-term memory
    #[instrument(skip(self, content), fields(content_len = content.len()))]
    pub async fn remember(&self, content: &str, memory_type: &str) -> Result<Uuid, DomainError> {
        info!(memory_type = memory_type, "Storing new memory");

        let mem_type = match memory_type.to_lowercase().as_str() {
            "fact" => MemoryType::Fact,
            "preference" => MemoryType::Preference,
            "procedure" => MemoryType::Procedure,
            "insight" => MemoryType::Insight,
            "context" => MemoryType::Context,
            "conversation" => MemoryType::Conversation,
            "task" => MemoryType::Task,
            "entity" => MemoryType::Entity,
            "goal" => MemoryType::Goal,
            "skill" => MemoryType::Skill,
            "event" => MemoryType::Event,
            "opinion" => MemoryType::Opinion,
            "experience" => MemoryType::Experience,
            _ => MemoryType::General,
        };

        let memory = self.memory_service
            .create_memory(content.to_string(), mem_type, None)
            .await?;

        info!(memory_id = %memory.id, "Memory stored successfully");
        Ok(memory.id)
    }

    /// Recall relevant memories
    #[instrument(skip(self))]
    pub async fn recall(&self, query: &str, limit: usize) -> Result<Vec<(Uuid, String, f64)>, DomainError> {
        debug!(query = query, limit = limit, "Recalling memories");

        let results = self.memory_service.search(query, limit).await?;

        let memories: Vec<(Uuid, String, f64)> = results
            .into_iter()
            .map(|(memory, similarity)| (memory.id, memory.content, similarity))
            .collect();

        debug!(query = query, results = memories.len(), "Memory recall completed");
        Ok(memories)
    }

    /// Check if search provider is healthy
    pub async fn is_search_healthy(&self) -> bool {
        self.search_provider.is_healthy().await
    }

    /// Check if LLM provider is healthy
    pub async fn is_llm_healthy(&self) -> bool {
        self.llm_provider.health_check().await.unwrap_or(false)
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
