//! =============================================================================
//! Data Models
//! =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;

/// Memory types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Fact,
    Preference,
    Entity,
    Goal,
    Skill,
    Event,
    Opinion,
    Experience,
    Task,
    Episodic,
    Semantic,
    Procedural,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryType::Fact => write!(f, "fact"),
            MemoryType::Preference => write!(f, "preference"),
            MemoryType::Entity => write!(f, "entity"),
            MemoryType::Goal => write!(f, "goal"),
            MemoryType::Skill => write!(f, "skill"),
            MemoryType::Event => write!(f, "event"),
            MemoryType::Opinion => write!(f, "opinion"),
            MemoryType::Experience => write!(f, "experience"),
            MemoryType::Task => write!(f, "task"),
            MemoryType::Episodic => write!(f, "episodic"),
            MemoryType::Semantic => write!(f, "semantic"),
            MemoryType::Procedural => write!(f, "procedural"),
        }
    }
}

impl std::str::FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fact" => Ok(MemoryType::Fact),
            "preference" => Ok(MemoryType::Preference),
            "entity" => Ok(MemoryType::Entity),
            "goal" => Ok(MemoryType::Goal),
            "skill" => Ok(MemoryType::Skill),
            "event" => Ok(MemoryType::Event),
            "opinion" => Ok(MemoryType::Opinion),
            "experience" => Ok(MemoryType::Experience),
            "task" => Ok(MemoryType::Task),
            "episodic" => Ok(MemoryType::Episodic),
            "semantic" => Ok(MemoryType::Semantic),
            "procedural" => Ok(MemoryType::Procedural),
            _ => Err(format!("Unknown memory type: {}", s)),
        }
    }
}

/// Memory metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetadata {
    #[serde(default)]
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub language: Option<String>,
    pub confidence: Option<f64>,
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub custom: serde_json::Value,
}

/// Memory node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub content: String,
    pub memory_type: MemoryType,
    #[serde(default)]
    pub vector: Vec<f32>,
    #[serde(default)]
    pub metadata: MemoryMetadata,
    pub importance_score: f64,
    pub access_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database record for deserialization
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryRecord {
    pub id: Thing,
    pub content: String,
    pub memory_type: String,
    #[serde(default)]
    pub vector: Vec<f32>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(default = "default_importance")]
    pub importance_score: f64,
    #[serde(default)]
    pub access_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_importance() -> f64 {
    0.5
}

impl MemoryRecord {
    pub fn to_memory(self) -> Memory {
        let id = match &self.id.id {
            surrealdb::sql::Id::String(s) => Uuid::parse_str(s).unwrap_or_default(),
            _ => Uuid::new_v4(),
        };

        let memory_type = self.memory_type.parse().unwrap_or(MemoryType::Fact);
        let metadata: MemoryMetadata = serde_json::from_value(self.metadata).unwrap_or_default();

        Memory {
            id,
            content: self.content,
            memory_type,
            vector: self.vector,
            metadata,
            importance_score: self.importance_score,
            access_count: self.access_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Relation between memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub relation_type: String,
    pub confidence: f64,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Database record for relation
#[derive(Debug, Clone, Deserialize)]
pub struct RelationRecord {
    pub id: Thing,
    #[serde(rename = "in")]
    pub from: Thing,
    #[serde(rename = "out")]
    pub to: Thing,
    #[serde(default = "default_relation_type")]
    pub relation_type: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

fn default_relation_type() -> String {
    "related_to".to_string()
}

fn default_confidence() -> f64 {
    1.0
}

impl RelationRecord {
    pub fn to_relation(self) -> Relation {
        let from_id = match &self.from.id {
            surrealdb::sql::Id::String(s) => Uuid::parse_str(s).unwrap_or_default(),
            _ => Uuid::new_v4(),
        };
        let to_id = match &self.to.id {
            surrealdb::sql::Id::String(s) => Uuid::parse_str(s).unwrap_or_default(),
            _ => Uuid::new_v4(),
        };

        Relation {
            from_id,
            to_id,
            relation_type: self.relation_type,
            confidence: self.confidence,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateMemoryRequest {
    pub content: String,
    pub memory_type: Option<String>,
    pub metadata: Option<MemoryMetadata>,
    pub importance_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemoryRequest {
    pub content: Option<String>,
    pub memory_type: Option<String>,
    pub metadata: Option<MemoryMetadata>,
    pub importance_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMemoriesRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub threshold: Option<f64>,
    pub memory_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub memory: Memory,
    pub similarity: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationRequest {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub relation_type: String,
    pub confidence: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct GraphStats {
    pub total_memories: usize,
    pub total_relations: usize,
    pub memories_by_type: std::collections::HashMap<String, usize>,
}
