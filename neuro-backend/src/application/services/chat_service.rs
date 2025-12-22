//! =============================================================================
//! Chat Service - Simplified
//! =============================================================================

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, error};
use uuid::Uuid;

use crate::application::services::{
    agent_orchestrator::AgentOrchestrator,
    memory_service::MemoryService,
    model_manager::ModelManager,
};
use crate::domain::{
    entities::chat::{ChatMessage, ChatRequest, ChatResponse, Conversation, MessageMetadata},
    errors::DomainError,
    ports::llm_provider::LlmProvider,
    value_objects::model_tier::ModelTier,
};
use crate::infrastructure::database::SurrealDbRepository;

/// =============================================================================
/// ChatService - Chat Conversation Management
/// =============================================================================
pub struct ChatService {
    #[allow(dead_code)]
    agent_orchestrator: Arc<AgentOrchestrator>,
    memory_service: Arc<MemoryService>,
    #[allow(dead_code)]
    model_manager: Arc<ModelManager>,
    llm_provider: Arc<dyn LlmProvider>,
    repository: Arc<SurrealDbRepository>,
    system_prompt: String,
}

impl ChatService {
    pub fn new(
        agent_orchestrator: Arc<AgentOrchestrator>,
        memory_service: Arc<MemoryService>,
        model_manager: Arc<ModelManager>,
        llm_provider: Arc<dyn LlmProvider>,
        repository: Arc<SurrealDbRepository>,
    ) -> Self {
        Self {
            agent_orchestrator,
            memory_service,
            model_manager,
            llm_provider,
            repository,
            system_prompt: Self::default_system_prompt(),
        }
    }

    fn default_system_prompt() -> String {
        r#"Eres Tachikoma, un asistente de IA amigable y conversacional creado por madKoding.
Tu personalidad es curiosa, empática y natural. Mantén conversaciones fluidas y recuerda el contexto de lo que el usuario te ha dicho.
Responde siempre en el mismo idioma que usa el usuario. Sé conciso pero amable."#.to_string()
    }

    /// Process a chat request
    #[instrument(skip(self, request), fields(message_len = request.message.len()))]
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, DomainError> {
        let start = Instant::now();
        let conversation_id = request.conversation_id.unwrap_or_else(Uuid::new_v4);

        // Create user message
        let user_message = ChatMessage::user(conversation_id, request.message.clone());

        // Get relevant memory context
        let context_memories = self.memory_service.search(&request.message, 5).await?;
        let memory_ids: Vec<Uuid> = context_memories.iter().map(|(m, _)| m.id).collect();

        debug!(memory_count = context_memories.len(), "Retrieved memory context");

        // Build prompt with context
        let prompt = self.build_prompt(&request.message, &context_memories);

        // Auto-select model based on message analysis
        let selected_model = self.select_model_for_task(&request.message);
        info!(model = %selected_model, "Auto-selected model for task");

        // Generate response with selected model
        let result = self.llm_provider.generate(&prompt, Some(&selected_model)).await?;

        // Create assistant message
        let mut assistant_message = ChatMessage::assistant(conversation_id, result.content);
        assistant_message.metadata = MessageMetadata {
            model: Some(result.model),
            context_memory_ids: memory_ids.clone(),
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
            prompt_tokens: Some(result.prompt_tokens),
            completion_tokens: Some(result.completion_tokens),
            token_count: Some((result.prompt_tokens + result.completion_tokens) as u32),
            ..Default::default()
        };

        // Update conversation history
        self.update_conversation(conversation_id, user_message, assistant_message.clone()).await;

        let response = ChatResponse {
            conversation_id,
            message: assistant_message,
            tools_used: vec![],
            context_memories: memory_ids,
            processing_time_ms: start.elapsed().as_millis() as u64,
        };

        info!(
            conversation_id = %conversation_id,
            processing_time_ms = response.processing_time_ms,
            "Chat completed"
        );

        Ok(response)
    }

    fn build_prompt(&self, user_message: &str, memories: &[(crate::domain::entities::memory::MemoryNode, f64)]) -> String {
        let mut prompt = self.system_prompt.clone();
        prompt.push_str("\n\n");

        if !memories.is_empty() {
            prompt.push_str("Relevant context from memory:\n");
            for (memory, score) in memories.iter().take(3) {
                prompt.push_str(&format!("- [{:.2}] {}\n", score, memory.content));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("User: ");
        prompt.push_str(user_message);
        prompt.push_str("\n\nAssistant: ");
        prompt
    }

    /// Update conversation - internal method
    async fn update_conversation(
        &self,
        conversation_id: Uuid,
        user_message: ChatMessage,
        assistant_message: ChatMessage,
    ) {
        self.update_conversation_direct(conversation_id, user_message, assistant_message).await;
    }

    /// Update conversation directly - public for streaming handler
    pub async fn update_conversation_direct(
        &self,
        conversation_id: Uuid,
        user_message: ChatMessage,
        assistant_message: ChatMessage,
    ) {
        tracing::info!(conversation_id = %conversation_id, "Saving conversation to database");
        
        // Get or create conversation
        let mut conversation = self.repository
            .get_conversation(conversation_id)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| {
                tracing::info!("Creating new conversation");
                let mut conv = Conversation::new();
                conv.id = conversation_id;
                // Generate title from first message
                conv.title = Some(user_message.content.chars().take(50).collect::<String>());
                conv
            });

        conversation.add_message(user_message.clone());
        conversation.add_message(assistant_message.clone());

        // Save conversation
        if let Err(e) = self.repository.save_conversation(&conversation).await {
            tracing::error!(error = %e, "Failed to save conversation");
        } else {
            tracing::info!("Conversation saved successfully");
        }

        // Save messages
        if let Err(e) = self.repository.save_message(&user_message).await {
            tracing::error!(error = %e, "Failed to save user message");
        } else {
            tracing::info!("User message saved");
        }
        if let Err(e) = self.repository.save_message(&assistant_message).await {
            tracing::error!(error = %e, "Failed to save assistant message");
        } else {
            tracing::info!("Assistant message saved");
        }
    }

    pub async fn get_conversation(&self, conversation_id: Uuid) -> Option<Conversation> {
        self.repository
            .get_conversation(conversation_id)
            .await
            .unwrap_or(None)
    }

    pub async fn list_conversations(&self) -> Vec<(Uuid, Option<String>, chrono::DateTime<chrono::Utc>)> {
        self.repository
            .list_conversations()
            .await
            .unwrap_or_default()
    }

    pub async fn delete_conversation(&self, conversation_id: Uuid) -> bool {
        self.repository
            .delete_conversation(conversation_id)
            .await
            .unwrap_or(false)
    }

    /// =========================================================================
    /// Auto-select model based on task analysis
    /// =========================================================================
    /// Analyzes the user message to determine the appropriate model tier:
    /// - Light (ministral-3:3b): Quick questions, simple tasks
    /// - Standard (qwen2.5-coder:7b): General coding, moderate complexity
    /// - Heavy (qwen2.5-coder:14b): Complex coding, deep reasoning
    /// =========================================================================
    pub fn select_model_for_task(&self, message: &str) -> String {
        let msg_lower = message.to_lowercase();
        let msg_len = message.len();

        // Detect code-related keywords
        let code_keywords = [
            "code", "function", "implement", "class", "struct", "enum",
            "bug", "fix", "error", "debug", "refactor", "optimize",
            "algorithm", "data structure", "api", "database", "sql",
            "rust", "python", "javascript", "typescript", "java", "c++",
            "async", "await", "thread", "mutex", "memory", "performance",
            "test", "unit test", "integration", "arquitectura", "design pattern",
            "código", "función", "implementar", "corregir", "arreglar",
        ];

        // Detect complex reasoning keywords
        let reasoning_keywords = [
            "explain", "why", "how does", "analyze", "compare", "evaluate",
            "design", "architecture", "tradeoff", "pros and cons", "best practice",
            "strategy", "approach", "solution", "complex", "difficult",
            "explica", "por qué", "cómo funciona", "analiza", "compara",
        ];

        // Detect quick response patterns
        let quick_patterns = [
            "hi", "hello", "hola", "thanks", "gracias", "yes", "no", "ok",
            "what is", "qué es", "define", "list", "name", "when", "where",
        ];

        let is_code_task = code_keywords.iter().any(|k| msg_lower.contains(k));
        let is_complex = reasoning_keywords.iter().any(|k| msg_lower.contains(k));
        let is_quick = quick_patterns.iter().any(|k| msg_lower.starts_with(k)) 
            || (msg_len < 50 && !is_code_task && !is_complex);

        // Determine tier
        let tier = if is_code_task && (is_complex || msg_len > 200) {
            ModelTier::Heavy
        } else if is_code_task || is_complex {
            ModelTier::Standard
        } else if is_quick {
            ModelTier::Light
        } else {
            ModelTier::Standard
        };

        debug!(
            message_len = msg_len,
            is_code_task = is_code_task,
            is_complex = is_complex,
            is_quick = is_quick,
            tier = %tier,
            "Task analysis complete"
        );

        tier.default_model().to_string()
    }
}
