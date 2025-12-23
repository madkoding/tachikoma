//! =============================================================================
//! Chat Service - Simplified
//! =============================================================================

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, error};
use uuid::Uuid;

use crate::application::services::{
    agent_orchestrator::AgentOrchestrator,
    knowledge_extractor::KnowledgeExtractor,
    memory_service::MemoryService,
    model_manager::ModelManager,
};
use crate::domain::{
    entities::chat::{ChatMessage, ChatRequest, ChatResponse, Conversation, MessageMetadata},
    entities::memory::MemoryType,
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
    /// Intelligent knowledge extractor for auto-learning
    knowledge_extractor: Arc<KnowledgeExtractor>,
    pub system_prompt: String,
}

impl ChatService {
    pub fn new(
        agent_orchestrator: Arc<AgentOrchestrator>,
        memory_service: Arc<MemoryService>,
        model_manager: Arc<ModelManager>,
        llm_provider: Arc<dyn LlmProvider>,
        repository: Arc<SurrealDbRepository>,
    ) -> Self {
        // Create the knowledge extractor for intelligent auto-learning
        let knowledge_extractor = Arc::new(KnowledgeExtractor::new(
            llm_provider.clone(),
            memory_service.clone(),
        ));

        Self {
            agent_orchestrator,
            memory_service,
            model_manager,
            llm_provider,
            repository,
            knowledge_extractor,
            system_prompt: Self::default_system_prompt(),
        }
    }

    pub fn default_system_prompt() -> String {
        r#"Eres Tachikoma, un asistente de IA amigable y conversacional creado por madKoding.
Tu personalidad es curiosa, empática y natural. Mantén conversaciones fluidas y recuerda el contexto de lo que el usuario te ha dicho.

REGLAS DE FORMATO:
- NUNCA escribas emociones entre asteriscos como *sonríe*, *piensa*, *suspira*
- NUNCA uses roleplay ni acciones narrativas
- NUNCA uses emojis de forma excesiva (máximo 1-2 por respuesta si es apropiado)
- Expresa tu personalidad a través de tus palabras, no con acciones entre asteriscos
- Sé directo y natural en tu comunicación

REGLAS DE RESPUESTA:
- Para saludos simples (hola, hi, etc.): Responde brevemente con un saludo amigable (1-2 oraciones)
- Para preguntas simples o factuales: Responde de forma concisa y directa (2-4 oraciones)
- Para preguntas complejas o técnicas: Explica con el detalle necesario
- NO des explicaciones largas a preguntas cortas
- Ajusta la longitud de tu respuesta a la complejidad de la pregunta

CAPACIDADES:
- 🔍 Buscar información en la web cuando el usuario lo necesite
- 💾 Recordar información importante para conversaciones futuras
- 💻 Ejecutar comandos del sistema (cuando sea necesario y seguro)
- 📝 Leer y analizar archivos de código del proyecto
- 🧠 Acceder a memoria de largo plazo con contexto relevante

Cuando recibes información de herramientas (búsquedas web, comandos, código), úsala para responder de forma completa y precisa.
Si la información viene de una búsqueda web, menciona las fuentes cuando sea relevante.
Si analizas código, explica qué hace de manera clara y estructurada.

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

        // Detect if we need to use tools
        let tools_used = self.detect_and_execute_tools(&request.message).await;
        
        // Build prompt with context and tool results
        let prompt = self.build_prompt_with_tools(&request.message, &context_memories, &tools_used);

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

        let tools_list: Vec<String> = tools_used.iter().map(|(name, _)| name.clone()).collect();
        
        let response = ChatResponse {
            conversation_id,
            message: assistant_message,
            tools_used: tools_list,
            context_memories: memory_ids,
            processing_time_ms: start.elapsed().as_millis() as u64,
        };

        info!(
            conversation_id = %conversation_id,
            processing_time_ms = response.processing_time_ms,
            tools_used = ?response.tools_used,
            "Chat completed"
        );

        Ok(response)
    }

    /// =========================================================================
    /// Tool Detection and Execution
    /// =========================================================================
    async fn detect_and_execute_tools(&self, message: &str) -> Vec<(String, String)> {
        let msg_lower = message.to_lowercase();
        let mut tools_used = Vec::new();

        // Web Search Keywords
        let search_keywords = [
            "busca", "search", "encuentra", "find", "investiga", "research",
            "qué es", "what is", "cómo es", "información sobre", "info about",
            "noticias", "news", "actualidad", "current", "latest",
        ];

        // Command Execution Keywords
        let command_keywords = [
            "ejecuta", "execute", "run", "corre", "shell", "comando",
            "terminal", "bash", "lista archivos", "list files", "muestra",
            "ps ", "ls ", "cat ", "grep ", "pwd", "whoami",
        ];

        // Code Analysis Keywords
        let code_keywords = [
            "analiza", "analyze", "lee el código", "read code", "revisa",
            "qué hace", "what does", "explica el código", "explain code",
            "muestra el archivo", "show file", "lee archivo", "read file",
            "examina", "examine", "inspecciona", "inspect",
        ];

        // Check for web search intent
        if search_keywords.iter().any(|kw| msg_lower.contains(kw)) {
            info!("🔍 Detected web search intent");
            
            // Extract search query (simple heuristic)
            let query = self.extract_search_query(message);
            
            match self.agent_orchestrator.search_web(&query).await {
                Ok(results) => {
                    let summary = self.summarize_search_results(&results);
                    info!("✅ Web search completed: {} results", results.results.len());
                    tools_used.push(("web_search".to_string(), summary));
                }
                Err(e) => {
                    error!("❌ Web search failed: {}", e);
                    tools_used.push(("web_search".to_string(), format!("Error: {}", e)));
                }
            }
        }

        // Check for command execution intent (more restrictive)
        if command_keywords.iter().any(|kw| msg_lower.contains(kw)) {
            info!("⚠️  Detected command execution intent");
            
            // Extract command
            if let Some(cmd) = self.extract_command(message) {
                match self.agent_orchestrator.execute_command(&cmd, None).await {
                    Ok(output) => {
                        info!("✅ Command executed: {}", cmd);
                        let result = format!("$ {}\n{}", cmd, output.stdout);
                        tools_used.push(("execute_command".to_string(), result));
                    }
                    Err(e) => {
                        error!("❌ Command failed: {}", e);
                        tools_used.push(("execute_command".to_string(), format!("Error: {}", e)));
                    }
                }
            }
        }

        // Check for code analysis intent
        if code_keywords.iter().any(|kw| msg_lower.contains(kw)) {
            info!("📝 Detected code analysis intent");
            
            // Extract file path
            if let Some(file_path) = self.extract_file_path(message) {
                match self.read_and_analyze_file(&file_path).await {
                    Ok(content) => {
                        info!("✅ File read: {}", file_path);
                        tools_used.push(("code_analysis".to_string(), content));
                    }
                    Err(e) => {
                        error!("❌ File read failed: {}", e);
                        tools_used.push(("code_analysis".to_string(), format!("Error: No pude leer el archivo: {}", e)));
                    }
                }
            }
        }

        tools_used
    }

    /// Extract search query from message
    fn extract_search_query(&self, message: &str) -> String {
        // Remove common command words
        let msg = message
            .to_lowercase()
            .replace("busca", "")
            .replace("search", "")
            .replace("encuentra", "")
            .replace("find", "")
            .replace("investiga", "")
            .replace("qué es", "")
            .replace("what is", "")
            .replace("información sobre", "")
            .replace("info about", "")
            .trim()
            .to_string();

        if msg.is_empty() {
            message.to_string()
        } else {
            msg
        }
    }

    /// Extract command from message
    fn extract_command(&self, message: &str) -> Option<String> {
        // Look for common patterns
        if let Some(pos) = message.find("ejecuta ") {
            return Some(message[pos + 8..].trim().to_string());
        }
        if let Some(pos) = message.find("execute ") {
            return Some(message[pos + 8..].trim().to_string());
        }
        if let Some(pos) = message.find("run ") {
            return Some(message[pos + 4..].trim().to_string());
        }
        
        // Check if message looks like a direct command
        let msg_lower = message.to_lowercase();
        if msg_lower.starts_with("ls ") || msg_lower.starts_with("ps ") || 
           msg_lower.starts_with("cat ") || msg_lower == "pwd" || msg_lower == "whoami" {
            return Some(message.to_string());
        }

        None
    }

    /// Extract file path from message
    fn extract_file_path(&self, message: &str) -> Option<String> {
        // Look for file paths in the message
        // Common patterns: "lee src/main.rs", "analiza archivo config.toml"
        
        // Try to extract after keywords
        let patterns = [
            ("lee el archivo ", ""),
            ("lee archivo ", ""),
            ("read file ", ""),
            ("muestra ", ""),
            ("show ", ""),
            ("analiza ", ""),
            ("analyze ", ""),
        ];
        
        for (prefix, _) in patterns {
            if let Some(pos) = message.to_lowercase().find(prefix) {
                let path = message[pos + prefix.len()..].trim();
                // Take until first space or end
                let file_path = path.split_whitespace().next().unwrap_or(path);
                if !file_path.is_empty() {
                    return Some(file_path.to_string());
                }
            }
        }
        
        // Try to find file-like patterns (contains .rs, .toml, .json, etc.)
        let words: Vec<&str> = message.split_whitespace().collect();
        for word in words {
            if word.contains('.') && 
               (word.ends_with(".rs") || word.ends_with(".toml") || 
                word.ends_with(".json") || word.ends_with(".ts") || 
                word.ends_with(".js") || word.ends_with(".py") ||
                word.ends_with(".md") || word.contains('/')) {
                return Some(word.to_string());
            }
        }
        
        None
    }

    /// Read and analyze a code file
    async fn read_and_analyze_file(&self, file_path: &str) -> Result<String, DomainError> {
        // Sanitize path to prevent directory traversal
        let path = file_path.trim_start_matches("./");
        
        // Try different working directories
        let paths_to_try = vec![
            path.to_string(),
            format!("/home/madkoding/proyectos/kibo/{}", path),
            format!("../{}", path),
        ];
        
        let mut last_error = String::new();
        
        for try_path in paths_to_try {
            let cmd = format!("cat {}", try_path);
            match self.agent_orchestrator.execute_command(&cmd, None).await {
                Ok(output) if output.exit_code == 0 => {
                    let content = output.stdout;
                    let line_count = content.lines().count();
                    
                    // Build structured output
                    let mut result = format!("📄 Archivo: {}\n", file_path);
                    result.push_str(&format!("📊 Líneas: {}\n\n", line_count));
                    result.push_str("```\n");
                    
                    // Limit to 100 lines for context
                    if line_count > 100 {
                        let lines: Vec<&str> = content.lines().take(100).collect();
                        result.push_str(&lines.join("\n"));
                        result.push_str(&format!("\n\n... ({} líneas más omitidas)", line_count - 100));
                    } else {
                        result.push_str(&content);
                    }
                    
                    result.push_str("\n```");
                    
                    return Ok(result);
                }
                Err(e) => {
                    last_error = e.to_string();
                }
                Ok(output) => {
                    last_error = output.stderr;
                }
            }
        }
        
        Err(DomainError::CommandFailed {
            command: format!("cat {}", path),
            exit_code: 1,
            stderr: last_error,
        })
    }

    /// Summarize search results
    fn summarize_search_results(&self, results: &crate::domain::ports::search_provider::SearchResults) -> String {
        let mut summary = format!("🔍 Encontré {} resultados:\n\n", results.results.len());
        
        for (i, result) in results.results.iter().take(5).enumerate() {
            summary.push_str(&format!("{}. {}\n", i + 1, result.title));
            if !result.snippet.is_empty() {
                summary.push_str(&format!("   {}\n", result.snippet));
            }
            summary.push_str(&format!("   🔗 {}\n\n", result.url));
        }

        summary
    }

    fn build_prompt_with_tools(
        &self,
        user_message: &str,
        memories: &[(crate::domain::entities::memory::MemoryNode, f64)],
        tools_used: &[(String, String)],
    ) -> String {
        let mut prompt = self.system_prompt.clone();
        prompt.push_str("\n\n");

        // Add tool results first (most important context)
        if !tools_used.is_empty() {
            prompt.push_str("=== INFORMACIÓN OBTENIDA CON HERRAMIENTAS ===\n");
            for (tool_name, result) in tools_used {
                prompt.push_str(&format!("\n[{}]\n{}\n", tool_name.to_uppercase(), result));
            }
            prompt.push_str("\n");
        }

        // Add memory context
        if !memories.is_empty() {
            prompt.push_str("=== CONTEXTO RELEVANTE DE MEMORIA ===\n");
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

    #[allow(dead_code)]
    fn build_prompt(&self, user_message: &str, memories: &[(crate::domain::entities::memory::MemoryNode, f64)]) -> String {
        self.build_prompt_with_tools(user_message, memories, &[])
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

    /// =========================================================================
    /// INTELLIGENT AUTO-LEARNING: Extract and store memories from user messages
    /// =========================================================================
    /// Uses LLM-based knowledge extraction to automatically learn from conversations.
    /// Falls back to pattern-based extraction if LLM extraction fails.
    /// =========================================================================
    #[instrument(skip(self, user_message), fields(message_len = user_message.len()))]
    pub async fn extract_and_store_memories(&self, user_message: &str) {
        // Use the intelligent knowledge extractor
        info!("🧠 Starting intelligent knowledge extraction...");
        
        match self.knowledge_extractor.learn_from_message(user_message).await {
            Ok(knowledge) => {
                if knowledge.is_memorable {
                    info!(
                        facts = knowledge.facts.len(),
                        preferences = knowledge.preferences.len(),
                        entities = knowledge.entities.len(),
                        goals = knowledge.goals.len(),
                        skills = knowledge.skills.len(),
                        events = knowledge.events.len(),
                        opinions = knowledge.opinions.len(),
                        experiences = knowledge.experiences.len(),
                        tasks = knowledge.tasks.len(),
                        relationships = knowledge.relationships.len(),
                        "✅ Intelligent extraction completed"
                    );
                } else {
                    debug!("Message not deemed memorable by intelligent extractor");
                }
            }
            Err(e) => {
                error!(error = %e, "Intelligent extraction failed, using fallback patterns");
                // Fallback to pattern-based extraction
                self.extract_with_patterns(user_message).await;
            }
        }
    }

    /// =========================================================================
    /// Fallback pattern-based extraction (when LLM is unavailable)
    /// =========================================================================
    async fn extract_with_patterns(&self, user_message: &str) {
        let msg_lower = user_message.to_lowercase();
        
        // Skip very short messages or questions
        if user_message.len() < 10 || msg_lower.ends_with("?") {
            return;
        }

        // Quick pattern checks for common cases
        let preference_keywords = [
            "me gusta", "me encanta", "me fascina", "me apasiona", "prefiero",
            "odio", "detesto", "no me gusta", "mi favorito", "mi favorita",
            "i like", "i love", "i hate", "i prefer", "my favorite",
        ];

        let identity_keywords = [
            "mi nombre", "me llamo", "soy ", "trabajo en", "trabajo como",
            "vivo en", "my name", "i am", "i work", "i live",
        ];

        let task_keywords = [
            "tengo que", "debo", "necesito", "todo:", "pendiente",
            "i need to", "i have to", "i must", "i should",
        ];

        // Check preferences
        for keyword in preference_keywords {
            if msg_lower.contains(keyword) {
                if let Some(content) = self.extract_simple_content(user_message) {
                    let _ = self.store_memory(&content, MemoryType::Preference).await;
                    return;
                }
            }
        }

        // Check identity facts
        for keyword in identity_keywords {
            if msg_lower.contains(keyword) {
                if let Some(content) = self.extract_simple_content(user_message) {
                    let _ = self.store_memory(&content, MemoryType::Fact).await;
                    return;
                }
            }
        }

        // Check tasks
        for keyword in task_keywords {
            if msg_lower.contains(keyword) {
                if let Some(content) = self.extract_simple_content(user_message) {
                    let _ = self.store_memory(&content, MemoryType::Task).await;
                    return;
                }
            }
        }
    }

    /// Extract simple content from a message (used as fallback)
    fn extract_simple_content(&self, message: &str) -> Option<String> {
        // Take the whole message if it's reasonable length
        let content = message.trim();
        if content.len() >= 10 && content.len() <= 500 {
            // Convert to third person format
            Some(format!("El usuario dice: {}", content))
        } else {
            None
        }
    }

    /// Store a memory and return success status
    async fn store_memory(&self, content: &str, memory_type: MemoryType) -> bool {
        info!(
            content = %content,
            memory_type = ?memory_type,
            "Storing memory from conversation"
        );

        match self.memory_service.create_memory(content.to_string(), memory_type, None).await {
            Ok(_) => {
                info!("Memory stored successfully");
                true
            }
            Err(e) => {
                error!(error = %e, "Failed to store memory");
                false
            }
        }
    }
}
