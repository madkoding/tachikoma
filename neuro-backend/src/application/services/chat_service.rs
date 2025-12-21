//! =============================================================================
//! Chat Service
//! =============================================================================
//! Application service for handling chat conversations.
//! Coordinates between the agent orchestrator, memory service, and LLM.
//! =============================================================================

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::application::services::{
    agent_orchestrator::AgentOrchestrator,
    memory_service::MemoryService,
    model_manager::ModelManager,
};
use crate::domain::{
    entities::chat::{
        ChatMessage, ChatRequest, ChatResponse, Conversation, MessageMetadata, MessageRole,
    },
    errors::DomainError,
    ports::llm_provider::{ChatContext, LlmProvider},
};

/// =============================================================================
/// ChatService - Chat Conversation Management
/// =============================================================================
/// Handles chat conversations, coordinating between user input, AI generation,
/// memory context, and tool execution.
/// 
/// # Responsibilities
/// 
/// * Conversation management
/// * Context assembly from memories
/// * Response generation with tools
/// * Conversation history tracking
/// 
/// # Example Usage
/// 
/// ```rust
/// let service = ChatService::new(
///     agent_orchestrator,
///     memory_service,
///     model_manager,
///     llm_provider,
/// );
/// 
/// let request = ChatRequest {
///     message: "What is the capital of France?".to_string(),
///     conversation_id: None,
///     language: "en".to_string(),
///     stream: false,
/// };
/// 
/// let response = service.chat(request).await?;
/// println!("Assistant: {}", response.message.content);
/// ```
/// =============================================================================
pub struct ChatService {
    /// Agent orchestrator for tool execution
    agent_orchestrator: Arc<AgentOrchestrator>,
    
    /// Memory service for context retrieval
    memory_service: Arc<MemoryService>,
    
    /// Model manager for model selection
    model_manager: Arc<ModelManager>,
    
    /// LLM provider for generation
    llm_provider: Arc<dyn LlmProvider>,
    
    /// Active conversations (in-memory cache)
    conversations: RwLock<std::collections::HashMap<Uuid, Conversation>>,
    
    /// System prompt for the assistant
    system_prompt: String,
}

impl ChatService {
    /// =========================================================================
    /// Create a new ChatService
    /// =========================================================================
    pub fn new(
        agent_orchestrator: Arc<AgentOrchestrator>,
        memory_service: Arc<MemoryService>,
        model_manager: Arc<ModelManager>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            agent_orchestrator,
            memory_service,
            model_manager,
            llm_provider,
            conversations: RwLock::new(std::collections::HashMap::new()),
            system_prompt: Self::default_system_prompt(),
        }
    }

    /// Default system prompt for the assistant
    fn default_system_prompt() -> String {
        r#"You are NEURO-OS, an intelligent AI assistant with access to long-term memory and tools.

Your capabilities include:
- Remembering information from past conversations
- Searching the web for current information
- Executing safe local commands
- Providing helpful, accurate responses

Guidelines:
1. Be helpful, accurate, and concise
2. Use your memory to provide personalized responses
3. Search the web when you need current information
4. Execute commands when asked, but ensure they are safe
5. Always explain your reasoning when using tools

Remember: You have access to memories from previous conversations. Use them to provide better, more contextual responses."#
            .to_string()
    }

    /// =========================================================================
    /// Process a chat request
    /// =========================================================================
    /// Main entry point for chat interactions. Processes the user's message,
    /// retrieves relevant context, and generates a response.
    /// 
    /// # Arguments
    /// 
    /// * `request` - The incoming chat request
    /// 
    /// # Returns
    /// 
    /// * `Ok(ChatResponse)` - The assistant's response
    /// * `Err(DomainError)` - If processing fails
    /// =========================================================================
    #[instrument(skip(self, request), fields(conversation_id, message_len = request.message.len()))]
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, DomainError> {
        let start = Instant::now();

        // Get or create conversation
        let conversation_id = request.conversation_id.unwrap_or_else(Uuid::new_v4);
        tracing::Span::current().record("conversation_id", &conversation_id.to_string());

        // Create user message
        let user_message = ChatMessage::user(conversation_id, request.message.clone());

        // Get relevant memory context
        let context_memories = self.get_memory_context(&request.message, 5).await?;
        let memory_ids: Vec<Uuid> = context_memories.iter().map(|m| m.id).collect();

        debug!(
            memory_count = context_memories.len(),
            "Retrieved memory context"
        );

        // Build conversation context
        let chat_context = self.build_chat_context(
            conversation_id,
            &context_memories,
            &request.language,
        ).await;

        // Generate response with tools
        let (response_content, tools_used) = self.agent_orchestrator
            .generate_with_tools(&request.message, Some(&chat_context), 3)
            .await?;

        // Create assistant message
        let mut assistant_message = ChatMessage::assistant(conversation_id, response_content);
        assistant_message.metadata = MessageMetadata {
            model: Some(self.model_manager.get_max_available_tier().await.default_model().to_string()),
            context_memory_ids: memory_ids.clone(),
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
            ..Default::default()
        };

        // Update conversation history
        self.update_conversation(conversation_id, user_message, assistant_message.clone()).await;

        // Build response
        let response = ChatResponse {
            conversation_id,
            message: assistant_message,
            tools_used,
            context_memories: memory_ids,
            processing_time_ms: start.elapsed().as_millis() as u64,
        };

        info!(
            conversation_id = %conversation_id,
            processing_time_ms = response.processing_time_ms,
            tools = ?response.tools_used,
            "Chat completed"
        );

        Ok(response)
    }

    /// =========================================================================
    /// Get memory context for a query
    /// =========================================================================
    /// Retrieves relevant memories to use as context for the response.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The user's query
    /// * `max_memories` - Maximum number of memories to retrieve
    /// 
    /// # Returns
    /// 
    /// List of relevant memories
    /// =========================================================================
    async fn get_memory_context(
        &self,
        query: &str,
        max_memories: usize,
    ) -> Result<Vec<crate::domain::entities::memory::MemoryNode>, DomainError> {
        self.memory_service
            .find_context(query, max_memories, true)
            .await
    }

    /// =========================================================================
    /// Build chat context for generation
    /// =========================================================================
    /// Assembles the full context including system prompt, memories, and
    /// conversation history.
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The conversation ID
    /// * `memories` - Relevant memories to include
    /// * `language` - User's preferred language
    /// 
    /// # Returns
    /// 
    /// List of chat context messages
    /// =========================================================================
    async fn build_chat_context(
        &self,
        conversation_id: Uuid,
        memories: &[crate::domain::entities::memory::MemoryNode],
        language: &str,
    ) -> Vec<ChatContext> {
        let mut context = Vec::new();

        // Add system prompt with language preference
        let system_prompt = if language == "es" {
            format!("{}\n\nResponde en español.", self.system_prompt)
        } else {
            self.system_prompt.clone()
        };
        context.push(ChatContext::system(system_prompt));

        // Add memory context if available
        if !memories.is_empty() {
            let memory_context = self.format_memory_context(memories);
            context.push(ChatContext::system(format!(
                "Relevant information from your memory:\n{}",
                memory_context
            )));
        }

        // Add conversation history
        let conversations = self.conversations.read().await;
        if let Some(conversation) = conversations.get(&conversation_id) {
            // Add last few messages as context
            for msg in conversation.last_messages(10) {
                let chat_msg = match msg.role {
                    MessageRole::User => ChatContext::user(msg.content.clone()),
                    MessageRole::Assistant => ChatContext::assistant(msg.content.clone()),
                    MessageRole::System => ChatContext::system(msg.content.clone()),
                    MessageRole::Tool => ChatContext::assistant(format!("[Tool output]: {}", msg.content)),
                };
                context.push(chat_msg);
            }
        }

        context
    }

    /// Format memories as context string
    fn format_memory_context(
        &self,
        memories: &[crate::domain::entities::memory::MemoryNode],
    ) -> String {
        memories
            .iter()
            .enumerate()
            .map(|(i, m)| format!(
                "{}. [{}] {}",
                i + 1,
                format!("{:?}", m.memory_type).to_lowercase(),
                m.content
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// =========================================================================
    /// Update conversation with new messages
    /// =========================================================================
    /// Adds user and assistant messages to the conversation history.
    /// =========================================================================
    async fn update_conversation(
        &self,
        conversation_id: Uuid,
        user_message: ChatMessage,
        assistant_message: ChatMessage,
    ) {
        let mut conversations = self.conversations.write().await;
        
        let conversation = conversations
            .entry(conversation_id)
            .or_insert_with(Conversation::new);

        conversation.add_message(user_message);
        conversation.add_message(assistant_message);
    }

    /// =========================================================================
    /// Get conversation history
    /// =========================================================================
    /// Retrieves the full conversation history for a given ID.
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The conversation ID
    /// 
    /// # Returns
    /// 
    /// * `Some(Conversation)` - The conversation if found
    /// * `None` - If conversation doesn't exist
    /// =========================================================================
    pub async fn get_conversation(&self, conversation_id: Uuid) -> Option<Conversation> {
        let conversations = self.conversations.read().await;
        conversations.get(&conversation_id).cloned()
    }

    /// =========================================================================
    /// List all conversations
    /// =========================================================================
    /// Returns a list of all active conversations.
    /// 
    /// # Returns
    /// 
    /// List of conversation IDs with titles
    /// =========================================================================
    pub async fn list_conversations(&self) -> Vec<(Uuid, Option<String>, chrono::DateTime<chrono::Utc>)> {
        let conversations = self.conversations.read().await;
        conversations
            .values()
            .map(|c| (c.id, c.title.clone(), c.updated_at))
            .collect()
    }

    /// =========================================================================
    /// Delete a conversation
    /// =========================================================================
    /// Removes a conversation from history.
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The conversation ID to delete
    /// 
    /// # Returns
    /// 
    /// `true` if deleted, `false` if not found
    /// =========================================================================
    pub async fn delete_conversation(&self, conversation_id: Uuid) -> bool {
        let mut conversations = self.conversations.write().await;
        conversations.remove(&conversation_id).is_some()
    }

    /// =========================================================================
    /// Update system prompt
    /// =========================================================================
    /// Changes the system prompt used for all conversations.
    /// 
    /// # Arguments
    /// 
    /// * `prompt` - The new system prompt
    /// =========================================================================
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }

    /// =========================================================================
    /// Extract facts from conversation
    /// =========================================================================
    /// Analyzes conversation messages and extracts notable facts to store
    /// in memory for future reference.
    /// 
    /// # Arguments
    /// 
    /// * `conversation_id` - The conversation to analyze
    /// 
    /// # Returns
    /// 
    /// Number of facts extracted and stored
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn extract_and_store_facts(
        &self,
        conversation_id: Uuid,
    ) -> Result<usize, DomainError> {
        let conversation = self.get_conversation(conversation_id).await
            .ok_or_else(|| DomainError::not_found("Conversation", conversation_id))?;

        // Get the model for fact extraction
        let config = self.model_manager.select_model(false, true, false).await?;

        // Build prompt for fact extraction
        let messages: String = conversation.messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"Analyze the following conversation and extract any notable facts, preferences, or insights about the user that should be remembered for future conversations.

Conversation:
{}

List each fact on a new line, prefixed with the type in brackets:
[fact] - Factual information
[preference] - User preferences
[insight] - Insights about the user

Only include information that would be useful to remember. If no notable facts, respond with "NONE"."#,
            messages
        );

        let result = self.llm_provider.generate(&prompt, &config, None).await?;

        // Parse and store facts
        let mut stored_count = 0;
        for line in result.content.lines() {
            let line = line.trim();
            if line == "NONE" || line.is_empty() {
                continue;
            }

            // Parse fact type and content
            let (memory_type, content) = if line.starts_with("[fact]") {
                (crate::domain::entities::memory::MemoryType::Fact, line[6..].trim())
            } else if line.starts_with("[preference]") {
                (crate::domain::entities::memory::MemoryType::Preference, line[12..].trim())
            } else if line.starts_with("[insight]") {
                (crate::domain::entities::memory::MemoryType::Insight, line[9..].trim())
            } else {
                continue;
            };

            if !content.is_empty() {
                let mut metadata = crate::domain::entities::memory::MemoryMetadata::default();
                metadata.conversation_id = Some(conversation_id);
                metadata.source = Some("conversation_extraction".to_string());

                self.memory_service
                    .create_memory(content.to_string(), memory_type, Some(metadata))
                    .await?;

                stored_count += 1;
            }
        }

        info!(
            conversation_id = %conversation_id,
            facts_stored = stored_count,
            "Facts extracted and stored"
        );

        Ok(stored_count)
    }
}
