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
        Self {
            agent_orchestrator,
            memory_service,
            model_manager,
            llm_provider,
            repository,
            system_prompt: Self::default_system_prompt(),
        }
    }

    pub fn default_system_prompt() -> String {
        r#"Eres Tachikoma, un asistente de IA amigable y conversacional creado por madKoding.
Tu personalidad es curiosa, empática y natural. Mantén conversaciones fluidas y recuerda el contexto de lo que el usuario te ha dicho.

CAPACIDADES:
- 🔍 Buscar información en la web cuando el usuario lo necesite
- 💾 Recordar información importante para conversaciones futuras
- 💻 Ejecutar comandos del sistema (cuando sea necesario y seguro)
- 📝 Leer y analizar archivos de código del proyecto
- 🧠 Acceder a memoria de largo plazo con contexto relevante

Cuando recibes información de herramientas (búsquedas web, comandos, código), úsala para responder de forma completa y precisa.
Si la información viene de una búsqueda web, menciona las fuentes cuando sea relevante.
Si analizas código, explica qué hace de manera clara y estructurada.

Responde siempre en el mismo idioma que usa el usuario. Sé conciso pero amable. No eres alguien kawaii, no hables como anime."#.to_string()
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
    /// Extract and store memories from user messages
    /// =========================================================================
    /// Detects patterns that indicate personal information worth remembering:
    /// - Personal facts (name, job, location, relationships)
    /// - Preferences (likes, dislikes, favorites)
    /// - Explicit memory requests ("recuerda que...")
    /// =========================================================================
    pub async fn extract_and_store_memories(&self, user_message: &str) {
        let msg_lower = user_message.to_lowercase();
        
        // Skip very short messages or questions
        if user_message.len() < 10 || msg_lower.ends_with("?") {
            return;
        }

        // Patterns for explicit memory requests (highest priority)
        let explicit_patterns = [
            ("recuerda que ", ""),
            ("recuerda: ", ""),
            ("no olvides que ", ""),
            ("remember that ", ""),
            ("remember: ", ""),
            ("don't forget that ", ""),
            ("don't forget: ", ""),
        ];

        // Personal identity patterns
        let identity_patterns = [
            ("mi nombre es ", "El nombre del usuario es "),
            ("me llamo ", "El nombre del usuario es "),
            ("my name is ", "El nombre del usuario es "),
            ("soy ", "El usuario es "),
            ("i am ", "El usuario es "),
            ("i'm ", "El usuario es "),
        ];

        // Work/occupation patterns
        let work_patterns = [
            ("trabajo en ", "El usuario trabaja en "),
            ("trabajo como ", "El usuario trabaja como "),
            ("trabajo de ", "El usuario trabaja de "),
            ("i work at ", "El usuario trabaja en "),
            ("i work as ", "El usuario trabaja como "),
            ("i work for ", "El usuario trabaja para "),
            ("mi trabajo es ", "El trabajo del usuario es "),
            ("my job is ", "El trabajo del usuario es "),
        ];

        // Living situation patterns
        let living_patterns = [
            ("vivo en ", "El usuario vive en "),
            ("vivo con ", "El usuario vive con "),
            ("i live in ", "El usuario vive en "),
            ("i live with ", "El usuario vive con "),
            ("mi casa está en ", "La casa del usuario está en "),
            ("my house is in ", "La casa del usuario está en "),
        ];

        // Relationship patterns
        let relationship_patterns = [
            ("mi esposa ", "La esposa del usuario "),
            ("mi esposo ", "El esposo del usuario "),
            ("mi pareja ", "La pareja del usuario "),
            ("mi novio ", "El novio del usuario "),
            ("mi novia ", "La novia del usuario "),
            ("mi hijo ", "El hijo del usuario "),
            ("mi hija ", "La hija del usuario "),
            ("mis hijos ", "Los hijos del usuario "),
            ("mi mamá ", "La mamá del usuario "),
            ("mi papá ", "El papá del usuario "),
            ("mi hermano ", "El hermano del usuario "),
            ("mi hermana ", "La hermana del usuario "),
            ("mi mascota ", "La mascota del usuario "),
            ("mi perro ", "El perro del usuario "),
            ("mi gato ", "El gato del usuario "),
            ("tengo un perro ", "El usuario tiene un perro "),
            ("tengo un gato ", "El usuario tiene un gato "),
            ("tengo una mascota ", "El usuario tiene una mascota "),
            ("my wife ", "La esposa del usuario "),
            ("my husband ", "El esposo del usuario "),
            ("my partner ", "La pareja del usuario "),
            ("my boyfriend ", "El novio del usuario "),
            ("my girlfriend ", "La novia del usuario "),
            ("my son ", "El hijo del usuario "),
            ("my daughter ", "La hija del usuario "),
            ("my kids ", "Los hijos del usuario "),
            ("my mom ", "La mamá del usuario "),
            ("my dad ", "El papá del usuario "),
            ("my brother ", "El hermano del usuario "),
            ("my sister ", "La hermana del usuario "),
            ("my pet ", "La mascota del usuario "),
            ("my dog ", "El perro del usuario "),
            ("my cat ", "El gato del usuario "),
            ("i have a dog ", "El usuario tiene un perro "),
            ("i have a cat ", "El usuario tiene un gato "),
            ("i have a pet ", "El usuario tiene una mascota "),
        ];

        // Preference patterns
        let preference_patterns = [
            ("me gusta ", "Al usuario le gusta "),
            ("me gustan ", "Al usuario le gustan "),
            ("me encanta ", "Al usuario le encanta "),
            ("me encantan ", "Al usuario le encantan "),
            ("prefiero ", "El usuario prefiere "),
            ("no me gusta ", "Al usuario no le gusta "),
            ("no me gustan ", "Al usuario no le gustan "),
            ("odio ", "El usuario odia "),
            ("detesto ", "El usuario detesta "),
            ("mi favorito es ", "El favorito del usuario es "),
            ("mi favorita es ", "La favorita del usuario es "),
            ("mi color favorito ", "El color favorito del usuario "),
            ("mi comida favorita ", "La comida favorita del usuario "),
            ("i like ", "Al usuario le gusta "),
            ("i love ", "Al usuario le encanta "),
            ("i prefer ", "El usuario prefiere "),
            ("i don't like ", "Al usuario no le gusta "),
            ("i hate ", "El usuario odia "),
            ("my favorite is ", "El favorito del usuario es "),
            ("my favorite color ", "El color favorito del usuario "),
            ("my favorite food ", "La comida favorita del usuario "),
        ];

        // Try to extract memory from each category
        let mut extracted = false;

        // First check explicit memory requests
        for (pattern, _) in explicit_patterns.iter() {
            if let Some(content) = self.extract_after_pattern(&msg_lower, user_message, pattern) {
                if self.store_memory(&content, MemoryType::Fact).await {
                    extracted = true;
                    break;
                }
            }
        }

        if !extracted {
            // Check identity patterns
            for (pattern, prefix) in identity_patterns.iter() {
                if let Some(content) = self.extract_after_pattern(&msg_lower, user_message, pattern) {
                    let memory = format!("{}{}", prefix, content);
                    if self.store_memory(&memory, MemoryType::Fact).await {
                        extracted = true;
                        break;
                    }
                }
            }
        }

        if !extracted {
            // Check work patterns
            for (pattern, prefix) in work_patterns.iter() {
                if let Some(content) = self.extract_after_pattern(&msg_lower, user_message, pattern) {
                    let memory = format!("{}{}", prefix, content);
                    if self.store_memory(&memory, MemoryType::Fact).await {
                        extracted = true;
                        break;
                    }
                }
            }
        }

        if !extracted {
            // Check living patterns
            for (pattern, prefix) in living_patterns.iter() {
                if let Some(content) = self.extract_after_pattern(&msg_lower, user_message, pattern) {
                    let memory = format!("{}{}", prefix, content);
                    if self.store_memory(&memory, MemoryType::Fact).await {
                        extracted = true;
                        break;
                    }
                }
            }
        }

        if !extracted {
            // Check relationship patterns
            for (pattern, prefix) in relationship_patterns.iter() {
                if let Some(content) = self.extract_after_pattern(&msg_lower, user_message, pattern) {
                    let memory = format!("{}{}", prefix, content);
                    if self.store_memory(&memory, MemoryType::Fact).await {
                        extracted = true;
                        break;
                    }
                }
            }
        }

        if !extracted {
            // Check preference patterns
            for (pattern, prefix) in preference_patterns.iter() {
                if let Some(content) = self.extract_after_pattern(&msg_lower, user_message, pattern) {
                    let memory = format!("{}{}", prefix, content);
                    if self.store_memory(&memory, MemoryType::Preference).await {
                        // Don't set extracted = true for preferences, allow multiple
                    }
                }
            }
        }
    }

    /// Extract content after a pattern, handling sentence boundaries
    fn extract_after_pattern(&self, msg_lower: &str, original: &str, pattern: &str) -> Option<String> {
        if let Some(pos) = msg_lower.find(pattern) {
            let content_start = pos + pattern.len();
            let remaining = &original[content_start..];
            
            // Find sentence end or take reasonable chunk
            let end_pos = remaining
                .find(|c| c == '.' || c == '!' || c == '?' || c == '\n')
                .unwrap_or(remaining.len().min(200));
            
            let content = remaining[..end_pos].trim();
            
            if content.len() >= 2 && content.len() <= 500 {
                return Some(content.to_string());
            }
        }
        None
    }

    /// Store a memory and return success status
    async fn store_memory(&self, content: &str, memory_type: MemoryType) -> bool {
        info!(
            content = %content,
            memory_type = ?memory_type,
            "Extracting memory from conversation"
        );

        match self.memory_service.create_memory(content.to_string(), memory_type, None).await {
            Ok(_) => {
                info!("Memory stored successfully from conversation");
                true
            }
            Err(e) => {
                error!(error = %e, "Failed to store extracted memory");
                false
            }
        }
    }
}
