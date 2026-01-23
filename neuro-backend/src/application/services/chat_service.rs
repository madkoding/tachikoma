//! =============================================================================
//! Chat Service - Simplified
//! =============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, error};
use uuid::Uuid;
use reqwest::Client;
use serde::Deserialize;

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

/// LLM-classified user intent
#[derive(Debug, Deserialize)]
struct UserIntent {
    tool: String,
    #[serde(default)]
    parameters: HashMap<String, String>,
    #[serde(default = "default_confidence")]
    confidence: f32,
}

fn default_confidence() -> f32 {
    0.5
}

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
    /// HTTP client for calling microservices
    http_client: Client,
    /// Music service URL
    music_service_url: String,
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

        // Get music service URL from environment
        let music_service_url = std::env::var("MUSIC_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string());

        Self {
            agent_orchestrator,
            memory_service,
            model_manager,
            llm_provider,
            repository,
            knowledge_extractor,
            system_prompt: Self::default_system_prompt(),
            http_client: Client::new(),
            music_service_url,
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
- 🎵 Crear playlists de música automáticamente buscando canciones en YouTube

Cuando recibes información de herramientas (búsquedas web, comandos, código), úsala para responder de forma completa y precisa.
Si la información viene de una búsqueda web, menciona las fuentes cuando sea relevante.
Si analizas código, explica qué hace de manera clara y estructurada.
Si creas una playlist, informa al usuario qué canciones se agregaron.

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
    /// Quick Tool Check - Instant keyword detection for UI feedback
    /// =========================================================================
    /// Returns true if the message likely needs a tool (for showing "thinking" UI)
    pub fn quick_tool_check(&self, message: &str) -> bool {
        let intent = self.fallback_keyword_detection(message);
        intent.tool != "none" && intent.confidence >= 0.7
    }
    
    /// =========================================================================
    /// Tool Detection and Execution - Fast Keywords + LLM Fallback
    /// =========================================================================
    /// Public method to detect and execute tools based on user message.
    /// Uses fast keyword detection first, then LLM only for ambiguous cases.
    pub async fn detect_and_execute_tools(&self, message: &str) -> Vec<(String, String)> {
        let mut tools_used = Vec::new();
        
        info!("🔍 Starting tool detection for message: {}", message);

        // STEP 1: Try fast keyword detection first (instant)
        let keyword_intent = self.fallback_keyword_detection(message);
        
        let intent = if keyword_intent.tool != "none" && keyword_intent.confidence >= 0.7 {
            // Keywords matched with high confidence - use it directly
            info!("⚡ Fast keyword detection: tool={}", keyword_intent.tool);
            keyword_intent
        } else {
            // STEP 2: Use LLM only for ambiguous cases
            match self.classify_intent(message).await {
                Ok(intent) => {
                    info!("🧠 LLM classified intent: tool={}, confidence={}", intent.tool, intent.confidence);
                    intent
                }
                Err(e) => {
                    // LLM classification failed, use keyword result anyway
                    info!("⚠️ LLM classification failed: {}, using keyword result", e);
                    keyword_intent
                }
            }
        };

        // Skip if no tool needed
        if intent.tool == "none" {
            info!("ℹ️ No tool required for this message (classified as: none)");
            return tools_used;
        }

        info!("🔧 Executing tool: {}", intent.tool);

        // Execute the appropriate tool based on classification
        match intent.tool.as_str() {
            "create_playlist" => {
                info!("🎵 Executing playlist creation");
                match self.create_playlist_from_request(message).await {
                    Ok(result) => {
                        info!("✅ Playlist created successfully");
                        tools_used.push(("create_playlist".to_string(), result));
                    }
                    Err(e) => {
                        error!("❌ Playlist creation failed: {}", e);
                        tools_used.push(("create_playlist".to_string(), format!("Error al crear playlist: {}", e)));
                    }
                }
            }
            "web_search" => {
                info!("🔍 Executing web search");
                let query = intent.parameters.get("query")
                    .map(|s| s.as_str())
                    .unwrap_or(message);
                
                match self.agent_orchestrator.search_web(query).await {
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
            "execute_command" => {
                info!("⚠️ Executing command");
                if let Some(cmd) = intent.parameters.get("command") {
                    match self.agent_orchestrator.execute_command(cmd, None).await {
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
            "read_file" => {
                info!("📝 Reading file");
                if let Some(file_path) = intent.parameters.get("file_path") {
                    match self.read_and_analyze_file(file_path).await {
                        Ok(content) => {
                            info!("✅ File read: {}", file_path);
                            tools_used.push(("code_analysis".to_string(), content));
                        }
                        Err(e) => {
                            error!("❌ File read failed: {}", e);
                            tools_used.push(("code_analysis".to_string(), format!("Error: {}", e)));
                        }
                    }
                }
            }
            "create_checklist" => {
                info!("📋 Creating checklist");
                match self.create_checklist_from_request(message).await {
                    Ok(result) => {
                        info!("✅ Checklist created successfully");
                        tools_used.push(("create_checklist".to_string(), result));
                    }
                    Err(e) => {
                        error!("❌ Checklist creation failed: {}", e);
                        tools_used.push(("create_checklist".to_string(), format!("Error al crear checklist: {}", e)));
                    }
                }
            }
            _ => {
                debug!("Unknown tool: {}", intent.tool);
            }
        }

        tools_used
    }

    /// Fallback keyword-based detection when LLM classification fails
    fn fallback_keyword_detection(&self, message: &str) -> UserIntent {
        let msg_lower = message.to_lowercase();
        // Normalize: remove accents
        let msg_norm: String = msg_lower.chars().map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            _ => c,
        }).collect();

        // Check for playlist creation
        let playlist_verbs = ["crea", "arma", "haz", "genera", "pon", "create", "make"];
        let playlist_nouns = ["playlist", "lista", "mix", "musica", "canciones"];
        
        let has_playlist_verb = playlist_verbs.iter().any(|v| msg_norm.contains(v));
        let has_playlist_noun = playlist_nouns.iter().any(|n| msg_norm.contains(n));
        
        if has_playlist_verb && has_playlist_noun {
            info!("🎯 Fallback detected: create_playlist");
            return UserIntent {
                tool: "create_playlist".to_string(),
                parameters: HashMap::new(),
                confidence: 0.7,
            };
        }

        // Check for checklist creation - more flexible detection
        let checklist_verbs = ["crea", "arma", "haz", "genera", "create", "make", "agrega", "add", "necesito", "quiero", "hazme", "dame"];
        let checklist_nouns = ["checklist", "lista de tareas", "to-do", "todo", "pendientes", "tareas", "task list", "lista para"];
        
        let has_checklist_verb = checklist_verbs.iter().any(|v| msg_norm.contains(v));
        let has_checklist_noun = checklist_nouns.iter().any(|n| msg_norm.contains(n));
        
        // Also detect if just the noun is present (user might say "checklist para X")
        let checklist_only = msg_norm.contains("checklist") || msg_norm.contains("to-do") || msg_norm.contains("todo list");
        
        if (has_checklist_verb && has_checklist_noun) || checklist_only {
            info!("🎯 Fallback detected: create_checklist");
            return UserIntent {
                tool: "create_checklist".to_string(),
                parameters: HashMap::new(),
                confidence: 0.7,
            };
        }

        // Check for web search
        let search_keywords = ["busca", "buscar", "search", "investiga", "noticias", "news"];
        if search_keywords.iter().any(|k| msg_norm.contains(k)) {
            info!("🎯 Fallback detected: web_search");
            return UserIntent {
                tool: "web_search".to_string(),
                parameters: [("query".to_string(), message.to_string())].into_iter().collect(),
                confidence: 0.7,
            };
        }

        // Default: no tool
        UserIntent {
            tool: "none".to_string(),
            parameters: HashMap::new(),
            confidence: 0.5,
        }
    }

    /// Use LLM to classify the user's intent and determine which tool to use
    async fn classify_intent(&self, message: &str) -> Result<UserIntent, DomainError> {
        // Simple, direct classification prompt optimized for small models
        let classification_prompt = format!(
            r#"Clasifica este mensaje en UNA categoría:

MENSAJE: "{}"

CATEGORÍAS:
- create_playlist: si pide crear playlist, música, canciones, mix
- create_checklist: si pide crear checklist, lista de tareas, to-do, pendientes
- web_search: si pide buscar información, noticias
- execute_command: si pide ejecutar comandos de terminal
- read_file: si pide leer archivos de código
- none: conversación normal, saludos, preguntas generales

Responde SOLO con la categoría, una sola palabra:"#,
            message
        );

        // Use Light model (ministral:3b) for fast classification - it's a simple task
        let light_model = ModelTier::Light.default_model();
        let result = self.llm_provider.generate(&classification_prompt, Some(light_model)).await?;
        
        // Parse the simple response
        let content = result.content.trim().to_lowercase();
        info!("🤖 LLM classification response: {}", content);
        
        // Extract tool from response
        let tool = if content.contains("create_playlist") || content.contains("playlist") {
            "create_playlist"
        } else if content.contains("create_checklist") || content.contains("checklist") || content.contains("to-do") || content.contains("todo") {
            "create_checklist"
        } else if content.contains("web_search") || content.contains("search") || content.contains("buscar") {
            "web_search"
        } else if content.contains("execute_command") || content.contains("command") || content.contains("ejecutar") {
            "execute_command"
        } else if content.contains("read_file") || content.contains("file") || content.contains("archivo") {
            "read_file"
        } else {
            "none"
        };
        
        info!("🎯 Classified as: {}", tool);
        
        Ok(UserIntent {
            tool: tool.to_string(),
            parameters: HashMap::new(),
            confidence: 0.8,
        })
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
    /// - Light (qwen3:0.6b): Default for most tasks, quick questions
    /// - Standard (qwen3:0.6b): Complex reasoning, analysis
    /// - Heavy (qwen3:0.6b): Code generation, deep technical tasks
    /// =========================================================================
    pub fn select_model_for_task(&self, message: &str) -> String {
        let msg_lower = message.to_lowercase();
        let msg_len = message.len();

        // Detect code-related keywords - ONLY these trigger heavy model
        let code_keywords = [
            "code", "function", "implement", "class", "struct", "enum",
            "bug", "fix", "error", "debug", "refactor", "optimize",
            "algorithm", "data structure", "api", "database", "sql",
            "rust", "python", "javascript", "typescript", "java", "c++",
            "async", "await", "thread", "mutex", "memory", "performance",
            "test", "unit test", "integration", "arquitectura", "design pattern",
            "código", "función", "implementar", "corregir", "arreglar",
        ];

        // Detect complex reasoning keywords - these trigger standard model
        let reasoning_keywords = [
            "explain in detail", "analyze deeply", "compare and contrast",
            "pros and cons", "best practice", "architecture design",
            "explica en detalle", "analiza profundamente",
        ];

        let is_code_task = code_keywords.iter().any(|k| msg_lower.contains(k));
        let is_complex = reasoning_keywords.iter().any(|k| msg_lower.contains(k)) 
            || msg_len > 500; // Only very long messages trigger complex

        // Determine tier - DEFAULT TO LIGHT
        let tier = if is_code_task && (is_complex || msg_len > 300) {
            ModelTier::Heavy
        } else if is_code_task {
            ModelTier::Standard
        } else if is_complex {
            ModelTier::Standard
        } else {
            // DEFAULT: Use Light model for most conversations
            ModelTier::Light
        };

        debug!(
            message_len = msg_len,
            is_code_task = is_code_task,
            is_complex = is_complex,
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
        info!("🧠 Starting intelligent knowledge extraction for: {}...", &user_message[..user_message.len().min(50)]);
        
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
                    // LLM said not memorable, but try pattern fallback anyway for common cases
                    info!("🔄 LLM extraction returned is_memorable=false, trying pattern fallback...");
                    self.extract_with_patterns(user_message).await;
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
                info!("📝 Pattern matched preference keyword: '{}'", keyword);
                if let Some(content) = self.extract_simple_content(user_message) {
                    info!("💾 Storing preference via pattern: {}", content);
                    let stored = self.store_memory(&content, MemoryType::Preference).await;
                    if stored {
                        info!("✅ Preference stored successfully via pattern fallback");
                    }
                    return;
                }
            }
        }

        // Check identity facts
        for keyword in identity_keywords {
            if msg_lower.contains(keyword) {
                info!("📝 Pattern matched identity keyword: '{}'", keyword);
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
            // Convert to third person format - better reformulation
            let reformulated = content
                .replace("me gusta", "le gusta")
                .replace("Me gusta", "Al usuario le gusta")
                .replace("me encanta", "le encanta")
                .replace("Me encanta", "Al usuario le encanta")
                .replace("mi favorito", "su favorito")
                .replace("mi favorita", "su favorita")
                .replace("prefiero", "prefiere")
                .replace("odio", "odia")
                .replace("no me gusta", "no le gusta");
            
            // Add "Al usuario" prefix if not already reformulated
            if reformulated.starts_with("le ") || reformulated.starts_with("su ") {
                Some(format!("Al usuario {}", reformulated))
            } else if reformulated == content {
                Some(format!("El usuario dice: {}", content))
            } else {
                Some(reformulated)
            }
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

    /// =========================================================================
    /// Playlist Creation Tool
    /// =========================================================================
    
    /// Create a playlist from a user request using LLM to generate metadata and search terms
    async fn create_playlist_from_request(&self, user_request: &str) -> Result<String, String> {
        info!("🎵 Starting playlist creation from request: {}", user_request);
        
        // Step 1: Use LLM to generate playlist metadata and search tags
        let metadata = self.generate_playlist_metadata(user_request).await?;
        info!("📝 Generated metadata: {:?}", metadata);
        
        // Step 2: Create the playlist immediately WITHOUT cover (fast!)
        let playlist = self.create_music_playlist(&metadata.title, &metadata.description, None).await?;
        let playlist_id = playlist.id.clone();
        info!("✅ Playlist created with ID: {}", playlist_id);
        
        // Step 3: Spawn background task to search cover and add songs (don't wait for it)
        let music_service_url = self.music_service_url.clone();
        let http_client = self.http_client.clone();
        let search_tags = metadata.search_tags.clone();
        let playlist_title = metadata.title.clone();
        let bg_playlist_id = playlist_id.clone();
        
        // Helper function for URL encoding (moved outside closure for accessibility)
        fn url_encode(s: &str) -> String {
            s.chars()
                .map(|c| match c {
                    ' ' => "%20".to_string(),
                    '&' => "%26".to_string(),
                    '=' => "%3D".to_string(),
                    '?' => "%3F".to_string(),
                    '#' => "%23".to_string(),
                    '+' => "%2B".to_string(),
                    _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => c.to_string(),
                    _ => format!("%{:02X}", c as u8),
                })
                .collect()
        }
        
        tokio::spawn(async move {
            info!("🎵 Background task: Adding songs to playlist {}", bg_playlist_id);
            let mut added_count = 0;
            let max_songs = 10;
            let songs_per_tag = 3;
            let mut cover_updated = false;
            
            for tag in &search_tags {
                if added_count >= max_songs {
                    break;
                }
                
                info!("🔍 Background: Searching YouTube for: {}", tag);
                
                // Search YouTube
                let search_url = format!(
                    "{}/api/music/youtube/search?q={}&limit={}",
                    music_service_url,
                    url_encode(tag),
                    songs_per_tag
                );
                
                let search_response = match http_client.get(&search_url).send().await {
                    Ok(resp) if resp.status().is_success() => resp,
                    Ok(resp) => {
                        error!("  ❌ Background: Search failed with status: {}", resp.status());
                        continue;
                    }
                    Err(e) => {
                        error!("  ❌ Background: Search request failed: {}", e);
                        continue;
                    }
                };
                
                let results: Vec<YouTubeSearchResult> = match search_response.json().await {
                    Ok(r) => r,
                    Err(e) => {
                        error!("  ❌ Background: Failed to parse search results: {}", e);
                        continue;
                    }
                };
                
                // Update playlist cover with first result's thumbnail
                if !cover_updated && !results.is_empty() {
                    if let Some(thumbnail) = &results[0].thumbnail {
                        let update_url = format!("{}/api/music/playlists/{}", music_service_url, bg_playlist_id);
                        let body = serde_json::json!({ "cover_url": thumbnail });
                        if http_client.patch(&update_url).json(&body).send().await.is_ok() {
                            info!("🎨 Background: Updated playlist cover");
                            cover_updated = true;
                        }
                    }
                }
                
                for result in results {
                    if added_count >= max_songs {
                        break;
                    }
                    
                    // Add song to playlist
                    let add_url = format!("{}/api/music/playlists/{}/songs", music_service_url, bg_playlist_id);
                    let youtube_url = format!("https://www.youtube.com/watch?v={}", result.video_id);
                    
                    let body = serde_json::json!({
                        "youtube_url": youtube_url,
                        "title": result.title,
                        "artist": result.channel.as_deref().unwrap_or("Unknown Artist")
                    });
                    
                    match http_client.post(&add_url).json(&body).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            info!("  ✅ Background: Added: {}", result.title);
                            added_count += 1;
                        }
                        Ok(resp) if resp.status().as_u16() == 409 => {
                            debug!("  ⚠️ Background: Song already exists: {}", result.title);
                        }
                        Ok(resp) => {
                            debug!("  ⚠️ Background: Failed to add song ({}): {}", resp.status(), result.title);
                        }
                        Err(e) => {
                            debug!("  ⚠️ Background: Request failed for {}: {}", result.title, e);
                        }
                    }
                }
            }
            
            info!("🎵 Background task completed: Added {} songs to '{}'", added_count, playlist_title);
        });
        
        // Step 5: Return immediately with initial summary
        let summary = format!(
            "🎵 ¡Playlist creada!\n\n\
            **{}**\n\
            _{}_\n\n\
            Las canciones se están agregando en segundo plano. Puedes ver el progreso en la playlist.\n\n\
            Tags de búsqueda: {}",
            metadata.title,
            metadata.description,
            metadata.search_tags.join(", ")
        );
        
        Ok(summary)
    }
    
    /// Generate playlist metadata using LLM
    async fn generate_playlist_metadata(&self, user_request: &str) -> Result<PlaylistMetadata, String> {
        let prompt = format!(
            r#"El usuario quiere crear una playlist de música con este pedido: "{}"

Genera un JSON con:
1. "title": Un título creativo y descriptivo para la playlist (máximo 50 caracteres)
2. "description": Una descripción breve de la playlist (máximo 100 caracteres)  
3. "search_tags": Un array de exactamente 5 términos de búsqueda para YouTube que encuentren canciones relacionadas con el pedido. Cada tag debe ser específico y variado para obtener diversidad de canciones.

Responde SOLO con el JSON, sin explicaciones ni markdown. Ejemplo de formato:
{{"title": "Rock de los 80s", "description": "Los mejores hits del rock ochentero", "search_tags": ["rock 80s hits", "def leppard songs", "bon jovi best", "guns n roses classics", "journey greatest hits"]}}"#,
            user_request
        );
        
        let response = self.llm_provider.generate(&prompt, None).await
            .map_err(|e| format!("Error generando metadata: {}", e))?;
        
        // Parse the JSON response - response.content contains the text
        let json_str = response.content.trim();
        
        // Try to extract JSON if wrapped in markdown
        let json_str = if json_str.starts_with("```") {
            json_str
                .lines()
                .skip(1)
                .take_while(|l: &&str| !l.starts_with("```"))
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            json_str.to_string()
        };
        
        serde_json::from_str::<PlaylistMetadata>(&json_str)
            .map_err(|e| format!("Error parseando respuesta del LLM: {}. Response: {}", e, json_str))
    }
    
    /// Create a playlist in the music service
    async fn create_music_playlist(&self, title: &str, description: &str, cover_url: Option<&str>) -> Result<PlaylistResponse, String> {
        let url = format!("{}/api/music/playlists", self.music_service_url);
        
        let body = serde_json::json!({
            "name": title,
            "description": description,
            "cover_url": cover_url
        });
        
        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Error conectando con servicio de música: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Error creando playlist ({}): {}", status, text));
        }
        
        response.json::<PlaylistResponse>().await
            .map_err(|e| format!("Error parseando respuesta de playlist: {}", e))
    }

    /// =========================================================================
    /// Checklist Creation Tool
    /// =========================================================================
    
    /// Create a checklist from a user request using templates or LLM
    async fn create_checklist_from_request(&self, user_request: &str) -> Result<String, String> {
        info!("📋 Starting checklist creation from request: {}", user_request);
        
        // Step 1: Try to use template first (instant), fallback to LLM
        let metadata = if let Some(template) = self.match_checklist_template(user_request) {
            info!("⚡ Using template for checklist: {}", template.title);
            template
        } else {
            info!("🧠 No template match, using LLM to generate checklist");
            self.generate_checklist_metadata(user_request).await?
        };
        
        info!("📝 Checklist metadata ready: {:?}", metadata);
        
        // Step 2: Create the checklist in the checklists service
        let checklists_service_url = std::env::var("CHECKLISTS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());
        
        let url = format!("{}/api/checklists", checklists_service_url);
        
        let body = serde_json::json!({
            "title": metadata.title,
            "description": metadata.description,
            "priority": metadata.priority,
            "items": metadata.items.iter().map(|item| {
                serde_json::json!({
                    "content": item,
                    "is_completed": false
                })
            }).collect::<Vec<_>>()
        });
        
        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Error conectando con servicio de checklists: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Error creando checklist ({}): {}", status, text));
        }
        
        let checklist_response: serde_json::Value = response.json().await
            .map_err(|e| format!("Error parseando respuesta de checklist: {}", e))?;
        
        // Step 3: Generate result summary
        let checklist_id = checklist_response.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let summary = format!(
            "📋 Checklist creada exitosamente!\n\n\
            **{}**\n\
            _{}_\n\n\
            Se agregaron {} items:\n{}\n\n\
            ID: {}",
            metadata.title,
            metadata.description,
            metadata.items.len(),
            metadata.items.iter()
                .enumerate()
                .map(|(i, s)| format!("{}. [ ] {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n"),
            checklist_id
        );
        
        info!("✅ Checklist created with ID: {}", checklist_id);
        Ok(summary)
    }
    
    /// Match user request against predefined checklist templates (instant)
    fn match_checklist_template(&self, user_request: &str) -> Option<ChecklistMetadata> {
        let msg = user_request.to_lowercase();
        let msg_norm: String = msg.chars().map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            _ => c,
        }).collect();
        
        // Travel checklist
        if msg_norm.contains("viaje") || msg_norm.contains("viajar") || msg_norm.contains("vacaciones") || msg_norm.contains("travel") {
            return Some(ChecklistMetadata {
                title: "Lista para viaje".to_string(),
                description: "Preparativos esenciales para tu viaje".to_string(),
                priority: 4,
                items: vec![
                    "Revisar documentos (pasaporte, ID, visas)".to_string(),
                    "Reservar vuelos y alojamiento".to_string(),
                    "Preparar maleta con ropa adecuada".to_string(),
                    "Llevar medicamentos esenciales".to_string(),
                    "Cargar dispositivos electrónicos".to_string(),
                    "Notificar al banco sobre viaje".to_string(),
                    "Verificar seguro de viaje".to_string(),
                    "Imprimir confirmaciones importantes".to_string(),
                ],
            });
        }
        
        // Cleaning checklist
        if msg_norm.contains("limpiar") || msg_norm.contains("limpieza") || msg_norm.contains("asear") || msg_norm.contains("cleaning") || msg_norm.contains("casa") {
            return Some(ChecklistMetadata {
                title: "Limpieza del hogar".to_string(),
                description: "Tareas de limpieza para mantener el hogar ordenado".to_string(),
                priority: 3,
                items: vec![
                    "Barrer y trapear pisos".to_string(),
                    "Limpiar baños (inodoro, lavabo, ducha)".to_string(),
                    "Lavar platos y limpiar cocina".to_string(),
                    "Sacar la basura".to_string(),
                    "Ordenar habitaciones".to_string(),
                    "Cambiar sábanas y toallas".to_string(),
                    "Limpiar ventanas y espejos".to_string(),
                    "Aspirar alfombras y sofás".to_string(),
                ],
            });
        }
        
        // Study/Learning checklist
        if msg_norm.contains("estudiar") || msg_norm.contains("estudio") || msg_norm.contains("aprender") || msg_norm.contains("examen") || msg_norm.contains("study") {
            return Some(ChecklistMetadata {
                title: "Plan de estudio".to_string(),
                description: "Organiza tu sesión de estudio".to_string(),
                priority: 4,
                items: vec![
                    "Revisar temario o syllabus".to_string(),
                    "Organizar material de estudio".to_string(),
                    "Leer y tomar notas".to_string(),
                    "Hacer resúmenes de temas clave".to_string(),
                    "Practicar con ejercicios".to_string(),
                    "Repasar conceptos difíciles".to_string(),
                    "Hacer simulacros o tests".to_string(),
                    "Descansar entre sesiones".to_string(),
                ],
            });
        }
        
        // Cooking/Recipe checklist
        if msg_norm.contains("cocinar") || msg_norm.contains("receta") || msg_norm.contains("comida") || msg_norm.contains("cook") || msg_norm.contains("cena") || msg_norm.contains("almuerzo") {
            return Some(ChecklistMetadata {
                title: "Preparar comida".to_string(),
                description: "Pasos para preparar una comida".to_string(),
                priority: 3,
                items: vec![
                    "Decidir el menú".to_string(),
                    "Revisar ingredientes disponibles".to_string(),
                    "Hacer lista de compras".to_string(),
                    "Comprar ingredientes faltantes".to_string(),
                    "Preparar ingredientes (cortar, medir)".to_string(),
                    "Cocinar siguiendo la receta".to_string(),
                    "Servir y presentar".to_string(),
                    "Limpiar cocina después".to_string(),
                ],
            });
        }
        
        // Exercise/Workout checklist
        if msg_norm.contains("ejercicio") || msg_norm.contains("entrenar") || msg_norm.contains("gym") || msg_norm.contains("workout") || msg_norm.contains("fitness") {
            return Some(ChecklistMetadata {
                title: "Rutina de ejercicio".to_string(),
                description: "Plan para tu sesión de entrenamiento".to_string(),
                priority: 3,
                items: vec![
                    "Calentar 5-10 minutos".to_string(),
                    "Ejercicios de cardio".to_string(),
                    "Ejercicios de fuerza".to_string(),
                    "Ejercicios de core/abdomen".to_string(),
                    "Estiramientos finales".to_string(),
                    "Hidratarse bien".to_string(),
                    "Registrar progreso".to_string(),
                ],
            });
        }
        
        // Shopping checklist
        if msg_norm.contains("compras") || msg_norm.contains("supermercado") || msg_norm.contains("shopping") || msg_norm.contains("comprar") {
            return Some(ChecklistMetadata {
                title: "Lista de compras".to_string(),
                description: "Items para comprar".to_string(),
                priority: 3,
                items: vec![
                    "Frutas y verduras".to_string(),
                    "Lácteos y huevos".to_string(),
                    "Carnes o proteínas".to_string(),
                    "Pan y cereales".to_string(),
                    "Productos de limpieza".to_string(),
                    "Artículos de higiene personal".to_string(),
                    "Snacks y bebidas".to_string(),
                ],
            });
        }
        
        // Work/Project checklist
        if msg_norm.contains("proyecto") || msg_norm.contains("trabajo") || msg_norm.contains("presentacion") || msg_norm.contains("project") || msg_norm.contains("work") {
            return Some(ChecklistMetadata {
                title: "Planificación de proyecto".to_string(),
                description: "Pasos para completar tu proyecto".to_string(),
                priority: 4,
                items: vec![
                    "Definir objetivos claros".to_string(),
                    "Investigar y recopilar información".to_string(),
                    "Crear estructura o outline".to_string(),
                    "Desarrollar contenido principal".to_string(),
                    "Revisar y editar".to_string(),
                    "Preparar materiales de soporte".to_string(),
                    "Hacer prueba o ensayo".to_string(),
                    "Entrega final".to_string(),
                ],
            });
        }
        
        // Event planning checklist
        if msg_norm.contains("evento") || msg_norm.contains("fiesta") || msg_norm.contains("celebracion") || msg_norm.contains("party") || msg_norm.contains("cumpleanos") {
            return Some(ChecklistMetadata {
                title: "Organizar evento".to_string(),
                description: "Preparativos para tu evento o celebración".to_string(),
                priority: 4,
                items: vec![
                    "Definir fecha y hora".to_string(),
                    "Crear lista de invitados".to_string(),
                    "Enviar invitaciones".to_string(),
                    "Reservar lugar si es necesario".to_string(),
                    "Planificar menú o catering".to_string(),
                    "Organizar decoración".to_string(),
                    "Preparar música o entretenimiento".to_string(),
                    "Confirmar asistencias".to_string(),
                ],
            });
        }
        
        // Morning routine
        if msg_norm.contains("manana") || msg_norm.contains("despertar") || msg_norm.contains("morning") || msg_norm.contains("rutina matutina") {
            return Some(ChecklistMetadata {
                title: "Rutina matutina".to_string(),
                description: "Actividades para empezar bien el día".to_string(),
                priority: 3,
                items: vec![
                    "Despertar a la hora planeada".to_string(),
                    "Estirar o hacer ejercicio ligero".to_string(),
                    "Ducharse y arreglarse".to_string(),
                    "Desayunar saludable".to_string(),
                    "Revisar agenda del día".to_string(),
                    "Preparar lo necesario para salir".to_string(),
                ],
            });
        }
        
        // No template match
        None
    }
    
    /// Generate checklist metadata using LLM (fallback when no template matches)
    async fn generate_checklist_metadata(&self, user_request: &str) -> Result<ChecklistMetadata, String> {
        let prompt = format!(
            r#"El usuario quiere crear una lista de tareas con este pedido: "{}"

Genera un JSON con:
1. "title": Un título claro para la lista de tareas (máximo 50 caracteres)
2. "description": Una descripción breve de la lista (máximo 100 caracteres)
3. "priority": Prioridad del 1 al 5 (5 = más urgente)
4. "items": Un array de tareas específicas y accionables (mínimo 3, máximo 10 items)

Responde SOLO con el JSON, sin explicaciones ni markdown. Ejemplo de formato:
{{"title": "Preparar presentación", "description": "Tareas para la presentación del lunes", "priority": 4, "items": ["Revisar diapositivas", "Preparar notas", "Practicar timing", "Verificar proyector"]}}"#,
            user_request
        );
        
        // Use Light model for this simple task
        let light_model = ModelTier::Light.default_model();
        let response = self.llm_provider.generate(&prompt, Some(light_model)).await
            .map_err(|e| format!("Error generando metadata de checklist: {}", e))?;
        
        // Parse the JSON response
        let json_str = response.content.trim();
        
        // Try to extract JSON if wrapped in markdown
        let json_str = if json_str.starts_with("```") {
            json_str
                .lines()
                .skip(1)
                .take_while(|l: &&str| !l.starts_with("```"))
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            json_str.to_string()
        };
        
        serde_json::from_str::<ChecklistMetadata>(&json_str)
            .map_err(|e| format!("Error parseando respuesta del LLM: {}. Response: {}", e, json_str))
    }
}

// =============================================================================
// Playlist Creation Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct PlaylistMetadata {
    title: String,
    description: String,
    search_tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PlaylistResponse {
    id: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeSearchResult {
    video_id: String,
    title: String,
    channel: Option<String>,
    thumbnail: Option<String>,
}

// =============================================================================
// Checklist Creation Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct ChecklistMetadata {
    title: String,
    description: String,
    #[serde(default = "default_priority")]
    priority: i32,
    items: Vec<String>,
}

fn default_priority() -> i32 {
    3
}
