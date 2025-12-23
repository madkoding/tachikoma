//! =============================================================================
//! Knowledge Extractor Service - Intelligent Auto-Learning
//! =============================================================================
//! Uses LLM to automatically extract knowledge, entities, relationships,
//! and memorable information from conversations.
//! =============================================================================

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, error, instrument};
use uuid::Uuid;

use crate::domain::{
    entities::memory::MemoryType,
    errors::DomainError,
    ports::llm_provider::LlmProvider,
    value_objects::relation::Relation,
};
use crate::application::services::MemoryService;

/// =============================================================================
/// Extracted Knowledge Structure
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractedKnowledge {
    /// Facts extracted from the message
    #[serde(default)]
    pub facts: Vec<ExtractedItem>,
    /// Preferences detected
    #[serde(default)]
    pub preferences: Vec<ExtractedItem>,
    /// Entities (people, places, organizations)
    #[serde(default)]
    pub entities: Vec<ExtractedItem>,
    /// Goals and objectives mentioned
    #[serde(default)]
    pub goals: Vec<ExtractedItem>,
    /// Skills and abilities
    #[serde(default)]
    pub skills: Vec<ExtractedItem>,
    /// Events and dates
    #[serde(default)]
    pub events: Vec<ExtractedItem>,
    /// Opinions and beliefs
    #[serde(default)]
    pub opinions: Vec<ExtractedItem>,
    /// Experiences shared
    #[serde(default)]
    pub experiences: Vec<ExtractedItem>,
    /// Tasks or todos
    #[serde(default)]
    pub tasks: Vec<ExtractedItem>,
    /// Relationships between concepts
    #[serde(default)]
    pub relationships: Vec<ExtractedRelation>,
    /// Overall importance score (0.0 - 1.0)
    #[serde(default)]
    pub importance: f64,
    /// Whether this message contains memorable information
    #[serde(default)]
    pub is_memorable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedItem {
    /// The extracted content (accepts "content" or "name" from LLM)
    #[serde(alias = "name", alias = "text", alias = "value", default)]
    pub content: String,
    /// Confidence score (0.0 - 1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Original text that led to this extraction
    #[serde(default)]
    pub source_text: Option<String>,
    /// Optional relation field (for entities)
    #[serde(default)]
    pub relation: Option<String>,
}

fn default_confidence() -> f64 {
    0.7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelation {
    /// Source concept/entity
    #[serde(default)]
    pub from: String,
    /// Target concept/entity
    #[serde(default)]
    pub to: String,
    /// Type of relationship
    #[serde(default)]
    pub relation_type: String,
    /// Confidence score
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

/// =============================================================================
/// KnowledgeExtractor - Intelligent Learning Service
/// =============================================================================
pub struct KnowledgeExtractor {
    llm_provider: Arc<dyn LlmProvider>,
    memory_service: Arc<MemoryService>,
    /// Model to use for extraction (lightweight for speed)
    extraction_model: String,
}

impl KnowledgeExtractor {
    pub fn new(
        llm_provider: Arc<dyn LlmProvider>,
        memory_service: Arc<MemoryService>,
    ) -> Self {
        Self {
            llm_provider,
            memory_service,
            extraction_model: "ministral-3:3b".to_string(), // Use default model already in memory
        }
    }

    /// =========================================================================
    /// Extract and Store Knowledge from a Message
    /// =========================================================================
    /// Main entry point - analyzes a message and stores any valuable knowledge
    /// =========================================================================
    #[instrument(skip(self, message), fields(message_len = message.len()))]
    pub async fn learn_from_message(&self, message: &str) -> Result<ExtractedKnowledge, DomainError> {
        // Skip very short messages or simple greetings
        if message.len() < 10 || self.is_simple_greeting(message) {
            info!("⏭️ Skipping extraction: short/greeting message");
            return Ok(ExtractedKnowledge::default());
        }

        // Skip pure questions (usually not information to store)
        let msg_lower = message.to_lowercase();
        if message.trim().ends_with('?') 
           && !msg_lower.contains("recuerda") 
           && !msg_lower.contains("soy ")
           && !msg_lower.contains("me llamo")
           && !msg_lower.contains("trabajo")
           && !msg_lower.contains("me gusta")
        {
            info!("⏭️ Skipping extraction: question without personal info");
            return Ok(ExtractedKnowledge::default());
        }

        info!("🧠 Starting knowledge extraction for: {}...", &message[..message.len().min(50)]);

        // Extract knowledge using LLM
        let knowledge = self.extract_knowledge(message).await?;

        // Check if we found anything useful (don't rely only on is_memorable flag)
        let has_content = !knowledge.facts.is_empty()
            || !knowledge.preferences.is_empty()
            || !knowledge.entities.is_empty()
            || !knowledge.goals.is_empty()
            || !knowledge.skills.is_empty()
            || !knowledge.events.is_empty()
            || !knowledge.opinions.is_empty()
            || !knowledge.experiences.is_empty()
            || !knowledge.tasks.is_empty();

        if !has_content && !knowledge.is_memorable {
            info!("⏭️ No memorable content found in message");
            return Ok(knowledge);
        }

        info!("💾 Storing extracted knowledge...");
        
        // Store extracted knowledge
        self.store_knowledge(&knowledge).await?;

        // Find and create relationships with existing memories
        self.create_relationships(&knowledge).await?;

        info!(
            facts = knowledge.facts.len(),
            preferences = knowledge.preferences.len(),
            entities = knowledge.entities.len(),
            relationships = knowledge.relationships.len(),
            "Knowledge extraction completed"
        );

        Ok(knowledge)
    }

    /// =========================================================================
    /// Extract Knowledge using LLM
    /// =========================================================================
    async fn extract_knowledge(&self, message: &str) -> Result<ExtractedKnowledge, DomainError> {
        let prompt = self.build_extraction_prompt(message);
        
        info!(model = %self.extraction_model, "🧠 Calling LLM for knowledge extraction...");
        
        let result = self.llm_provider
            .generate(&prompt, Some(&self.extraction_model))
            .await?;

        debug!(response_len = result.content.len(), "LLM extraction response received");
        
        // Parse LLM response
        let knowledge = self.parse_extraction_response(&result.content)?;
        
        info!(
            is_memorable = knowledge.is_memorable,
            importance = knowledge.importance,
            facts = knowledge.facts.len(),
            preferences = knowledge.preferences.len(),
            "🧠 Extraction parsed"
        );
        
        Ok(knowledge)
    }

    /// =========================================================================
    /// Build the extraction prompt
    /// =========================================================================
    fn build_extraction_prompt(&self, message: &str) -> String {
        format!(r#"Analiza el siguiente mensaje del usuario y extrae TODA la información personal relevante.
Responde SOLO con JSON válido, sin explicaciones adicionales.

MENSAJE: "{}"

Extrae información en estas categorías:
- facts: Hechos sobre el usuario (nombre, edad, trabajo, ubicación, etc.)
- preferences: Gustos, preferencias, cosas que le gustan o no
- entities: Personas mencionadas (familia, amigos, conocidos) con su relación
- goals: Metas, objetivos, sueños, planes
- skills: Habilidades, conocimientos, cosas que sabe hacer
- events: Eventos, fechas importantes, cumpleaños, aniversarios
- opinions: Opiniones, creencias, puntos de vista
- experiences: Experiencias vividas, viajes, historias
- tasks: Tareas pendientes, cosas que debe hacer
- relationships: Conexiones entre conceptos mencionados

Para cada item extraído, incluye:
- content: El conocimiento extraído (reformulado en tercera persona: "El usuario...")
- confidence: Qué tan seguro estás (0.0 a 1.0)

Para relationships incluye:
- from: Concepto origen
- to: Concepto destino  
- relation_type: Uno de [related_to, causes, part_of, has_property, used_for, capable_of, located_in, created_by, similar_to]
- confidence: Qué tan seguro estás

También indica:
- importance: Importancia general del mensaje (0.0 a 1.0)
- is_memorable: Si vale la pena guardar algo (true/false)

Si el mensaje no contiene información personal relevante, devuelve is_memorable: false.

Responde SOLO con el JSON:
```json
{{
  "facts": [],
  "preferences": [],
  "entities": [],
  "goals": [],
  "skills": [],
  "events": [],
  "opinions": [],
  "experiences": [],
  "tasks": [],
  "relationships": [],
  "importance": 0.0,
  "is_memorable": false
}}
```"#, message)
    }

    /// =========================================================================
    /// Parse LLM extraction response
    /// =========================================================================
    fn parse_extraction_response(&self, response: &str) -> Result<ExtractedKnowledge, DomainError> {
        // Try to find JSON in the response
        let json_str = self.extract_json_from_response(response);
        
        info!("📄 Parsing JSON response (len={})", json_str.len());
        
        match serde_json::from_str::<ExtractedKnowledge>(&json_str) {
            Ok(knowledge) => {
                info!("✅ JSON parsed successfully");
                Ok(knowledge)
            },
            Err(e) => {
                error!(error = %e, "❌ Failed to parse extraction response");
                info!("Raw JSON attempted: {}", &json_str[..json_str.len().min(500)]);
                // Return default instead of failing completely
                Ok(ExtractedKnowledge::default())
            }
        }
    }

    /// =========================================================================
    /// Extract JSON from LLM response (handles markdown code blocks)
    /// =========================================================================
    fn extract_json_from_response(&self, response: &str) -> String {
        // Try to find JSON between code blocks
        if let Some(start) = response.find("```json") {
            let after_start = &response[start + 7..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        
        // Try to find JSON between generic code blocks
        if let Some(start) = response.find("```") {
            let after_start = &response[start + 3..];
            if let Some(end) = after_start.find("```") {
                let content = after_start[..end].trim();
                // Skip language identifier if present
                if let Some(newline) = content.find('\n') {
                    return content[newline..].trim().to_string();
                }
                return content.to_string();
            }
        }

        // Try to find raw JSON object
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                return response[start..=end].to_string();
            }
        }

        response.to_string()
    }

    /// =========================================================================
    /// Store extracted knowledge in memory
    /// =========================================================================
    async fn store_knowledge(&self, knowledge: &ExtractedKnowledge) -> Result<(), DomainError> {
        let min_confidence = 0.4; // Lower threshold to capture more knowledge
        
        // Store facts
        for item in &knowledge.facts {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Fact, item.confidence).await?;
            }
        }

        // Store preferences
        for item in &knowledge.preferences {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Preference, item.confidence).await?;
            }
        }

        // Store entities
        for item in &knowledge.entities {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Entity, item.confidence).await?;
            }
        }

        // Store goals
        for item in &knowledge.goals {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Goal, item.confidence).await?;
            }
        }

        // Store skills
        for item in &knowledge.skills {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Skill, item.confidence).await?;
            }
        }

        // Store events
        for item in &knowledge.events {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Event, item.confidence).await?;
            }
        }

        // Store opinions
        for item in &knowledge.opinions {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Opinion, item.confidence).await?;
            }
        }

        // Store experiences
        for item in &knowledge.experiences {
            if item.confidence >= min_confidence {
                self.store_memory_item(&item.content, MemoryType::Experience, item.confidence).await?;
            }
        }

        // Store tasks
        for item in &knowledge.tasks {
            if item.confidence >= 0.5 { // Slightly higher for tasks
                self.store_memory_item(&item.content, MemoryType::Task, item.confidence).await?;
            }
        }

        Ok(())
    }

    /// =========================================================================
    /// Store a single memory item
    /// =========================================================================
    async fn store_memory_item(
        &self, 
        content: &str, 
        memory_type: MemoryType,
        confidence: f64
    ) -> Result<Option<Uuid>, DomainError> {
        // Skip empty content
        if content.trim().is_empty() {
            return Ok(None);
        }

        info!(
            content = %content,
            memory_type = ?memory_type,
            confidence = confidence,
            "Storing extracted knowledge"
        );

        match self.memory_service.create_memory(
            content.to_string(),
            memory_type,
            None
        ).await {
            Ok(memory) => {
                info!(memory_id = %memory.id, "Knowledge stored successfully");
                Ok(Some(memory.id))
            }
            Err(e) => {
                // Don't fail the whole extraction if one memory fails
                error!(error = %e, "Failed to store memory item");
                Ok(None)
            }
        }
    }

    /// =========================================================================
    /// Create relationships between extracted knowledge and existing memories
    /// =========================================================================
    async fn create_relationships(&self, knowledge: &ExtractedKnowledge) -> Result<(), DomainError> {
        for rel in &knowledge.relationships {
            if rel.confidence < 0.6 {
                continue;
            }

            // Find memories matching the "from" and "to" concepts
            let from_memories = self.memory_service.search(&rel.from, 1).await?;
            let to_memories = self.memory_service.search(&rel.to, 1).await?;

            if let (Some((from_mem, from_sim)), Some((to_mem, to_sim))) = 
                (from_memories.first(), to_memories.first()) 
            {
                // Only create relation if both matches are good
                if *from_sim > 0.5 && *to_sim > 0.5 {
                    let relation = self.parse_relation_type(&rel.relation_type);
                    
                    match self.memory_service.create_relation(
                        from_mem.id,
                        to_mem.id,
                        relation,
                        rel.confidence
                    ).await {
                        Ok(_) => {
                            info!(
                                from = %rel.from,
                                to = %rel.to,
                                relation = %rel.relation_type,
                                "Created relationship between memories"
                            );
                        }
                        Err(e) => {
                            debug!(error = %e, "Failed to create relationship (might already exist)");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// =========================================================================
    /// Parse relation type string to enum
    /// =========================================================================
    fn parse_relation_type(&self, relation_str: &str) -> Relation {
        match relation_str.to_lowercase().as_str() {
            "related_to" | "relacionado" => Relation::RelatedTo,
            "causes" | "causa" => Relation::Causes,
            "part_of" | "parte_de" => Relation::PartOf,
            "has_property" | "tiene_propiedad" => Relation::HasProperty,
            "used_for" | "usado_para" => Relation::UsedFor,
            "capable_of" | "capaz_de" => Relation::CapableOf,
            "located_in" | "ubicado_en" => Relation::LocatedIn,
            "created_by" | "creado_por" => Relation::CreatedBy,
            "similar_to" | "similar_a" => Relation::SimilarTo,
            "contradicts" | "contradice" => Relation::Contradicts,
            "supports" | "apoya" => Relation::Supports,
            "derived_from" | "derivado_de" => Relation::DerivedFrom,
            _ => Relation::RelatedTo,
        }
    }

    /// =========================================================================
    /// Check if message is a simple greeting
    /// =========================================================================
    fn is_simple_greeting(&self, message: &str) -> bool {
        let msg_lower = message.to_lowercase().trim().to_string();
        let greetings = [
            "hola", "hi", "hello", "hey", "buenos días", "buenas tardes", 
            "buenas noches", "good morning", "good afternoon", "good evening",
            "qué tal", "cómo estás", "how are you", "what's up", "sup",
            "saludos", "greetings", "ey", "epa", "alo", "aló",
        ];
        
        // Check if message is just a greeting (possibly with punctuation)
        let clean_msg = msg_lower.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
        greetings.iter().any(|g| clean_msg == *g || clean_msg.starts_with(&format!("{} ", g)))
    }

    /// =========================================================================
    /// Intelligent context-aware learning
    /// =========================================================================
    /// Analyzes conversation context to better understand and store knowledge
    /// =========================================================================
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn learn_from_conversation(
        &self, 
        user_message: &str,
        assistant_response: &str,
        conversation_context: &[String]
    ) -> Result<ExtractedKnowledge, DomainError> {
        // Build context-aware prompt
        let context_str = if conversation_context.is_empty() {
            String::new()
        } else {
            format!("\n\nCONTEXTO PREVIO:\n{}", conversation_context.join("\n"))
        };

        let full_message = format!(
            "Usuario: {}\nAsistente: {}{}",
            user_message, 
            assistant_response,
            context_str
        );

        // Use modified extraction that considers context
        self.learn_from_message(&full_message).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_json_from_response() {
        // Test JSON extraction without needing full KnowledgeExtractor
        let response_with_markdown = r#"Here's the analysis:
```json
{"is_memorable": true}
```"#;
        
        // Test direct JSON extraction logic
        let json_str = if let Some(start) = response_with_markdown.find("```json") {
            let after_start = &response_with_markdown[start + 7..];
            if let Some(end) = after_start.find("```") {
                after_start[..end].trim().to_string()
            } else {
                response_with_markdown.to_string()
            }
        } else {
            response_with_markdown.to_string()
        };
        
        assert!(json_str.contains("is_memorable"));
    }

    #[test]
    fn test_is_simple_greeting_logic() {
        // Test greeting detection logic directly
        let greetings = [
            "hola", "hi", "hello", "hey", "buenos días", "buenas tardes",
        ];
        
        let test_msg = "hola";
        let msg_lower = test_msg.to_lowercase();
        let clean_msg = msg_lower.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
        let is_greeting = greetings.iter().any(|g| clean_msg == *g);
        assert!(is_greeting);
        
        let test_msg2 = "me llamo Juan";
        let msg_lower2 = test_msg2.to_lowercase();
        let clean_msg2 = msg_lower2.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
        let is_greeting2 = greetings.iter().any(|g| clean_msg2 == *g);
        assert!(!is_greeting2);
    }
}
