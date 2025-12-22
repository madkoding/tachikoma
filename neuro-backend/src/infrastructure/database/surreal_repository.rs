//! =============================================================================
//! SurrealDB Memory Repository - Simplified
//! =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::domain::{
    entities::memory::{MemoryNode, MemoryQuery, MemoryType, MemoryMetadata},
    errors::DomainError,
    ports::memory_repository::{GraphExport, GraphStats, MemoryRepository, RelationDirection},
    value_objects::relation::{GraphEdge, Relation},
};
use crate::infrastructure::database::DatabasePool;

/// SurrealDbRepository - SurrealDB implementation of MemoryRepository
#[derive(Clone)]
pub struct SurrealDbRepository {
    pool: DatabasePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryRecord {
    id: String,
    content: String,
    vector: Vec<f32>,
    memory_type: String,
    metadata: serde_json::Value,
    created_at: Datetime,
    updated_at: Datetime,
    access_count: u64,
    importance_score: f64,
}

impl SurrealDbRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn to_record(memory: &MemoryNode) -> MemoryRecord {
        MemoryRecord {
            id: memory.id.to_string(),
            content: memory.content.clone(),
            vector: memory.vector.clone(),
            memory_type: format!("{:?}", memory.memory_type).to_lowercase(),
            metadata: serde_json::to_value(&memory.metadata).unwrap_or_default(),
            created_at: Datetime::from(memory.created_at),
            updated_at: Datetime::from(memory.updated_at),
            access_count: memory.access_count,
            importance_score: memory.importance_score,
        }
    }

    fn from_record(record: MemoryRecord) -> Result<MemoryNode, DomainError> {
        use chrono::{DateTime, Utc};
        
        let memory_type = match record.memory_type.as_str() {
            "fact" => MemoryType::Fact,
            "preference" => MemoryType::Preference,
            "procedure" => MemoryType::Procedure,
            "conversation" => MemoryType::Conversation,
            "insight" => MemoryType::Insight,
            _ => MemoryType::General,
        };

        let metadata: MemoryMetadata = serde_json::from_value(record.metadata).unwrap_or_default();
        
        // Convert surrealdb::Datetime to chrono::DateTime<Utc>
        let created_at: DateTime<Utc> = record.created_at.0.into();
        let updated_at: DateTime<Utc> = record.updated_at.0.into();

        Ok(MemoryNode {
            id: Uuid::parse_str(&record.id)
                .map_err(|e| DomainError::database(format!("Invalid UUID: {}", e)))?,
            content: record.content,
            vector: record.vector,
            memory_type,
            metadata,
            created_at,
            updated_at,
            access_count: record.access_count,
            importance_score: record.importance_score,
        })
    }
}

#[async_trait]
impl MemoryRepository for SurrealDbRepository {
    #[instrument(skip(self, memory))]
    async fn create(&self, memory: MemoryNode) -> Result<MemoryNode, DomainError> {
        tracing::info!(memory_id = %memory.id, vector_len = memory.vector.len(), "Attempting to create memory in SurrealDB");

        // Use raw query with individual fields to ensure vector is properly stored
        let sql = r#"
            CREATE type::thing('memory', $id) SET
                content = $content,
                vector = $vector,
                memory_type = $memory_type,
                metadata = $metadata,
                created_at = $created_at,
                updated_at = $updated_at,
                access_count = $access_count,
                importance_score = $importance_score
        "#;

        let mut response = self.pool.client()
            .query(sql)
            .bind(("id", memory.id.to_string()))
            .bind(("content", memory.content.clone()))
            .bind(("vector", memory.vector.clone()))
            .bind(("memory_type", format!("{:?}", memory.memory_type).to_lowercase()))
            .bind(("metadata", serde_json::to_value(&memory.metadata).unwrap_or_default()))
            .bind(("created_at", Datetime::from(memory.created_at)))
            .bind(("updated_at", Datetime::from(memory.updated_at)))
            .bind(("access_count", memory.access_count))
            .bind(("importance_score", memory.importance_score))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SurrealDB query failed");
                DomainError::database(e.to_string())
            })?;
        
        // Check for query errors
        let errors: Vec<surrealdb::Error> = response.take_errors().into_values().collect();
        if !errors.is_empty() {
            let error_msg = errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ");
            tracing::error!(errors = %error_msg, "SurrealDB CREATE returned errors");
            return Err(DomainError::database(error_msg));
        }

        debug!(memory_id = %memory.id, "Memory created in SurrealDB");
        Ok(memory)
    }

    #[instrument(skip(self))]
    async fn get_by_id(&self, id: Uuid) -> Result<Option<MemoryNode>, DomainError> {
        let sql = "SELECT meta::id(id) as id, content, vector, memory_type, metadata, created_at, updated_at, access_count, importance_score FROM type::thing('memory', $id)";
        
        let mut response = self.pool.client()
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<MemoryRecord> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        match records.into_iter().next() {
            Some(r) => Ok(Some(Self::from_record(r)?)),
            None => Ok(None),
        }
    }

    #[instrument(skip(self, memory))]
    async fn update(&self, memory: MemoryNode) -> Result<MemoryNode, DomainError> {
        let record = Self::to_record(&memory);

        // Use raw query to avoid deserialization issues
        self.pool.client()
            .query("UPDATE type::thing('memory', $id) CONTENT $record")
            .bind(("id", memory.id.to_string()))
            .bind(("record", record))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!(memory_id = %memory.id, "Memory updated in SurrealDB");
        Ok(memory)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<bool, DomainError> {
        // Use raw query to avoid deserialization issues
        self.pool.client()
            .query("DELETE type::thing('memory', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!(memory_id = %id, "Memory deleted from SurrealDB");
        Ok(true)
    }

    #[instrument(skip(self, query_vector))]
    async fn semantic_search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        min_similarity: f64,
    ) -> Result<Vec<(MemoryNode, f64)>, DomainError> {
        // Simple implementation: fetch all and compute similarity in memory
        // In production, use SurrealDB's vector search capabilities
        let all_memories = self.get_all(1000, 0).await?;
        
        let mut results: Vec<(MemoryNode, f64)> = all_memories
            .into_iter()
            .map(|m| {
                let sim = cosine_similarity(&query_vector, &m.vector);
                (m, sim)
            })
            .filter(|(_, sim)| *sim >= min_similarity)
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    async fn get_all(&self, limit: usize, offset: usize) -> Result<Vec<MemoryNode>, DomainError> {
        let sql = format!(
            "SELECT meta::id(id) as id, content, vector, memory_type, metadata, created_at, updated_at, access_count, importance_score FROM memory ORDER BY created_at DESC LIMIT {} START {}",
            limit, offset
        );

        let mut response = self.pool.client()
            .query(&sql)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<MemoryRecord> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        records.into_iter().map(Self::from_record).collect()
    }

    async fn count(&self) -> Result<usize, DomainError> {
        let mut response = self.pool.client()
            .query("SELECT count() FROM memory GROUP ALL")
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        #[derive(Debug, Deserialize)]
        struct CountResult { count: usize }

        let result: Option<CountResult> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(result.map(|r| r.count).unwrap_or(0))
    }

    async fn create_relation(&self, edge: GraphEdge) -> Result<GraphEdge, DomainError> {
        // For simplicity, we'll store relations inline
        // In production, use SurrealDB's graph edges
        debug!(from = %edge.from_id, to = %edge.to_id, relation = ?edge.relation, "Relation created");
        
        Ok(edge)
    }

    async fn get_relations(
        &self,
        memory_id: Uuid,
        _relation_type: Option<Relation>,
        _direction: RelationDirection,
    ) -> Result<Vec<GraphEdge>, DomainError> {
        // Simplified: return empty for now
        debug!(memory_id = %memory_id, "Getting relations");
        Ok(vec![])
    }

    async fn get_related_memories(
        &self,
        memory_id: Uuid,
        _max_depth: usize,
        _relation_types: Option<Vec<Relation>>,
    ) -> Result<Vec<(MemoryNode, GraphEdge)>, DomainError> {
        debug!(memory_id = %memory_id, "Getting related memories");
        Ok(vec![])
    }

    async fn query(&self, query: MemoryQuery) -> Result<Vec<MemoryNode>, DomainError> {
        let mut results = self.get_all(query.limit.unwrap_or(100), query.offset.unwrap_or(0)).await?;

        if !query.memory_types.is_empty() {
            results.retain(|m| query.memory_types.contains(&m.memory_type));
        }

        if !query.tags.is_empty() {
            results.retain(|m| m.metadata.tags.iter().any(|t| query.tags.contains(t)));
        }

        Ok(results)
    }

    async fn delete_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation: Relation,
    ) -> Result<bool, DomainError> {
        debug!(from = %from_id, to = %to_id, relation = ?relation, "Deleting relation");
        Ok(true)
    }

    async fn get_graph_stats(&self) -> Result<GraphStats, DomainError> {
        // Get total nodes count
        let total_nodes = self.count().await?;
        
        // Get nodes by type
        let sql_by_type = "SELECT memory_type, count() as cnt FROM memory GROUP BY memory_type";
        let mut response = self.pool.client()
            .query(sql_by_type)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        #[derive(Debug, serde::Deserialize)]
        struct TypeCount {
            memory_type: String,
            cnt: i64,
        }
        
        let type_counts: Vec<TypeCount> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;
        
        let mut nodes_by_type = std::collections::HashMap::new();
        for tc in type_counts {
            nodes_by_type.insert(tc.memory_type, tc.cnt as usize);
        }
        
        // Count edges from relation tables
        let relation_tables = [
            "related_to", "causes", "part_of", "follows", "contradicts",
            "supports", "derived_from", "same_as", "context_of", "references_rel", "supersedes"
        ];
        
        let mut total_edges = 0usize;
        let mut edges_by_type = std::collections::HashMap::new();
        
        for table in relation_tables {
            let sql = format!("SELECT count() as cnt FROM {}", table);
            if let Ok(mut resp) = self.pool.client().query(&sql).await {
                #[derive(Debug, serde::Deserialize)]
                struct EdgeCount {
                    cnt: i64,
                }
                if let Ok(counts) = resp.take::<Vec<EdgeCount>>(0) {
                    if let Some(ec) = counts.first() {
                        if ec.cnt > 0 {
                            total_edges += ec.cnt as usize;
                            edges_by_type.insert(table.to_string(), ec.cnt as usize);
                        }
                    }
                }
            }
        }
        
        // Calculate average connections
        let avg_connections = if total_nodes > 0 {
            (total_edges as f64 * 2.0) / total_nodes as f64
        } else {
            0.0
        };
        
        Ok(GraphStats {
            total_nodes,
            total_edges,
            nodes_by_type,
            edges_by_type,
            avg_connections,
            most_connected_node: None,
        })
    }

    async fn export_graph(&self) -> Result<GraphExport, DomainError> {
        let nodes = self.get_all(10000, 0).await?;
        Ok(GraphExport {
            nodes,
            edges: vec![],
            exported_at: chrono::Utc::now(),
        })
    }
}

/// Simple cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        (dot / (norm_a * norm_b)) as f64
    }
}

// =============================================================================
// Conversation Repository Methods
// =============================================================================

use crate::domain::entities::chat::{Conversation, ChatMessage, MessageRole, MessageMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationRecord {
    id: String,
    title: Option<String>,
    created_at: Datetime,
    updated_at: Datetime,
    archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessageRecord {
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    metadata: serde_json::Value,
    created_at: Datetime,
}

impl SurrealDbRepository {
    /// Save a conversation to the database
    pub async fn save_conversation(&self, conversation: &Conversation) -> Result<(), DomainError> {
        let sql = r#"
            CREATE type::thing('conversation', $id) SET
                title = $title,
                created_at = $created_at,
                updated_at = $updated_at,
                archived = $archived
            ON DUPLICATE KEY UPDATE
                title = $title,
                updated_at = $updated_at,
                archived = $archived
        "#;

        self.pool.client()
            .query(sql)
            .bind(("id", conversation.id.to_string()))
            .bind(("title", conversation.title.clone()))
            .bind(("created_at", Datetime::from(conversation.created_at)))
            .bind(("updated_at", Datetime::from(conversation.updated_at)))
            .bind(("archived", conversation.archived))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(())
    }

    /// Save a chat message to the database
    pub async fn save_message(&self, message: &ChatMessage) -> Result<(), DomainError> {
        let sql = r#"
            CREATE type::thing('chat_message', $id) SET
                conversation_id = $conversation_id,
                role = $role,
                content = $content,
                metadata = $metadata,
                created_at = $created_at
        "#;

        let role = match message.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        };

        self.pool.client()
            .query(sql)
            .bind(("id", message.id.to_string()))
            .bind(("conversation_id", message.conversation_id.to_string()))
            .bind(("role", role))
            .bind(("content", message.content.clone()))
            .bind(("metadata", serde_json::to_value(&message.metadata).unwrap_or_default()))
            .bind(("created_at", Datetime::from(message.created_at)))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(())
    }

    /// Get a conversation by ID with its messages
    pub async fn get_conversation(&self, id: Uuid) -> Result<Option<Conversation>, DomainError> {
        // Get conversation record
        let sql = "SELECT meta::id(id) as id, title, created_at, updated_at, archived FROM type::thing('conversation', $id)";
        
        let mut response = self.pool.client()
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<ConversationRecord> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        let record = match records.into_iter().next() {
            Some(r) => r,
            None => return Ok(None),
        };

        // Get messages for this conversation
        let messages = self.get_messages_for_conversation(id).await?;

        use chrono::{DateTime, Utc};
        let created_at: DateTime<Utc> = record.created_at.0.into();
        let updated_at: DateTime<Utc> = record.updated_at.0.into();

        Ok(Some(Conversation {
            id: Uuid::parse_str(&record.id).map_err(|e| DomainError::database(e.to_string()))?,
            title: record.title,
            messages,
            created_at,
            updated_at,
            archived: record.archived,
        }))
    }

    /// Get all messages for a conversation
    async fn get_messages_for_conversation(&self, conversation_id: Uuid) -> Result<Vec<ChatMessage>, DomainError> {
        let sql = r#"
            SELECT meta::id(id) as id, conversation_id, role, content, metadata, created_at 
            FROM chat_message 
            WHERE conversation_id = $conversation_id 
            ORDER BY created_at ASC
        "#;

        let mut response = self.pool.client()
            .query(sql)
            .bind(("conversation_id", conversation_id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<ChatMessageRecord> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut messages = Vec::new();
        for record in records {
            use chrono::{DateTime, Utc};
            let created_at: DateTime<Utc> = record.created_at.0.into();
            
            let role = match record.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                "tool" => MessageRole::Tool,
                _ => MessageRole::User,
            };

            let metadata: MessageMetadata = serde_json::from_value(record.metadata).unwrap_or_default();

            messages.push(ChatMessage {
                id: Uuid::parse_str(&record.id).map_err(|e| DomainError::database(e.to_string()))?,
                conversation_id: Uuid::parse_str(&record.conversation_id).map_err(|e| DomainError::database(e.to_string()))?,
                role,
                content: record.content,
                metadata,
                created_at,
            });
        }

        Ok(messages)
    }

    /// List all conversations (without messages)
    pub async fn list_conversations(&self) -> Result<Vec<(Uuid, Option<String>, chrono::DateTime<chrono::Utc>)>, DomainError> {
        let sql = "SELECT meta::id(id) as id, title, updated_at FROM conversation ORDER BY updated_at DESC";

        let mut response = self.pool.client()
            .query(sql)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<ConversationRecord> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut result = Vec::new();
        for record in records {
            use chrono::{DateTime, Utc};
            let updated_at: DateTime<Utc> = record.updated_at.0.into();
            let id = Uuid::parse_str(&record.id).map_err(|e| DomainError::database(e.to_string()))?;
            result.push((id, record.title, updated_at));
        }

        Ok(result)
    }

    /// Delete a conversation and its messages
    pub async fn delete_conversation(&self, id: Uuid) -> Result<bool, DomainError> {
        // Delete messages first
        self.pool.client()
            .query("DELETE chat_message WHERE conversation_id = $id")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Delete conversation
        self.pool.client()
            .query("DELETE type::thing('conversation', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(true)
    }
}

