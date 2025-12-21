//! =============================================================================
//! SurrealDB Memory Repository
//! =============================================================================
//! Implements the MemoryRepository port using SurrealDB.
//! Handles all memory persistence and graph operations.
//! =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::domain::{
    entities::memory::{MemoryNode, MemoryQuery},
    errors::DomainError,
    ports::memory_repository::{
        GraphExport, GraphStats, MemoryRepository, RelationDirection,
    },
    value_objects::relation::{GraphEdge, Relation},
};
use crate::infrastructure::database::DatabasePool;

/// =============================================================================
/// SurrealDbRepository - SurrealDB implementation of MemoryRepository
/// =============================================================================
/// Provides persistence for memory nodes and graph relations using SurrealDB.
/// Supports vector similarity search for the GraphRAG pattern.
/// =============================================================================
#[derive(Clone)]
pub struct SurrealDbRepository {
    /// Database connection pool
    pool: DatabasePool,
}

/// Internal struct for SurrealDB record representation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryRecord {
    id: String,
    content: String,
    vector: Vec<f32>,
    memory_type: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    access_count: u64,
    importance_score: f64,
}

impl SurrealDbRepository {
    /// =========================================================================
    /// Create a new SurrealDbRepository
    /// =========================================================================
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Convert domain MemoryNode to database record
    fn to_record(memory: &MemoryNode) -> MemoryRecord {
        MemoryRecord {
            id: memory.id.to_string(),
            content: memory.content.clone(),
            vector: memory.vector.clone(),
            memory_type: format!("{:?}", memory.memory_type).to_lowercase(),
            metadata: serde_json::to_value(&memory.metadata).unwrap_or_default(),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
            access_count: memory.access_count,
            importance_score: memory.importance_score,
        }
    }

    /// Convert database record to domain MemoryNode
    fn from_record(record: MemoryRecord) -> Result<MemoryNode, DomainError> {
        let memory_type = match record.memory_type.as_str() {
            "fact" => crate::domain::entities::memory::MemoryType::Fact,
            "preference" => crate::domain::entities::memory::MemoryType::Preference,
            "procedure" => crate::domain::entities::memory::MemoryType::Procedure,
            "conversation" => crate::domain::entities::memory::MemoryType::Conversation,
            "semantictag" => crate::domain::entities::memory::MemoryType::SemanticTag,
            "issue" => crate::domain::entities::memory::MemoryType::Issue,
            "insight" => crate::domain::entities::memory::MemoryType::Insight,
            "externalknowledge" => crate::domain::entities::memory::MemoryType::ExternalKnowledge,
            "codesnippet" => crate::domain::entities::memory::MemoryType::CodeSnippet,
            _ => crate::domain::entities::memory::MemoryType::General,
        };

        let metadata: crate::domain::entities::memory::MemoryMetadata = 
            serde_json::from_value(record.metadata).unwrap_or_default();

        Ok(MemoryNode {
            id: Uuid::parse_str(&record.id)
                .map_err(|e| DomainError::database(format!("Invalid UUID: {}", e)))?,
            content: record.content,
            vector: record.vector,
            memory_type,
            metadata,
            created_at: record.created_at,
            updated_at: record.updated_at,
            access_count: record.access_count,
            importance_score: record.importance_score,
        })
    }

    /// Get relation table name
    fn relation_table_name(relation: &Relation) -> &'static str {
        match relation {
            Relation::RelatedTo => "related_to",
            Relation::Causes => "causes",
            Relation::PartOf => "part_of",
            Relation::Follows => "follows",
            Relation::Contradicts => "contradicts",
            Relation::Supports => "supports",
            Relation::DerivedFrom => "derived_from",
            Relation::SameAs => "same_as",
            Relation::ContextOf => "context_of",
            Relation::References => "references_rel",
            Relation::Supersedes => "supersedes",
        }
    }
}

#[async_trait]
impl MemoryRepository for SurrealDbRepository {
    /// =========================================================================
    /// Create a new memory
    /// =========================================================================
    #[instrument(skip(self, memory), fields(memory_id = %memory.id))]
    async fn create(&self, memory: MemoryNode) -> Result<MemoryNode, DomainError> {
        let record = Self::to_record(&memory);
        let id = record.id.clone();

        let _: Option<MemoryRecord> = self.pool.client()
            .create(("memory", &id))
            .content(record)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!(memory_id = %id, "Memory created in database");

        Ok(memory)
    }

    /// =========================================================================
    /// Get memory by ID
    /// =========================================================================
    #[instrument(skip(self))]
    async fn get_by_id(&self, id: Uuid) -> Result<Option<MemoryNode>, DomainError> {
        let record: Option<MemoryRecord> = self.pool.client()
            .select(("memory", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        match record {
            Some(r) => Ok(Some(Self::from_record(r)?)),
            None => Ok(None),
        }
    }

    /// =========================================================================
    /// Update memory
    /// =========================================================================
    #[instrument(skip(self, memory), fields(memory_id = %memory.id))]
    async fn update(&self, memory: MemoryNode) -> Result<MemoryNode, DomainError> {
        let record = Self::to_record(&memory);
        let id = record.id.clone();

        let _: Option<MemoryRecord> = self.pool.client()
            .update(("memory", &id))
            .content(record)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!(memory_id = %id, "Memory updated in database");

        Ok(memory)
    }

    /// =========================================================================
    /// Delete memory
    /// =========================================================================
    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<bool, DomainError> {
        let result: Option<MemoryRecord> = self.pool.client()
            .delete(("memory", id.to_string()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!(memory_id = %id, deleted = result.is_some(), "Memory delete attempted");

        Ok(result.is_some())
    }

    /// =========================================================================
    /// Query memories
    /// =========================================================================
    #[instrument(skip(self, query))]
    async fn query(&self, query: MemoryQuery) -> Result<Vec<MemoryNode>, DomainError> {
        let mut sql = String::from("SELECT * FROM memory WHERE true");
        let mut bindings: Vec<(String, serde_json::Value)> = Vec::new();

        // Filter by memory types
        if !query.memory_types.is_empty() {
            let types: Vec<String> = query.memory_types
                .iter()
                .map(|t| format!("{:?}", t).to_lowercase())
                .collect();
            sql.push_str(" AND memory_type IN $types");
            bindings.push(("types".to_string(), serde_json::json!(types)));
        }

        // Filter by minimum importance
        if let Some(min_imp) = query.min_importance {
            sql.push_str(" AND importance_score >= $min_importance");
            bindings.push(("min_importance".to_string(), serde_json::json!(min_imp)));
        }

        // Filter by time range
        if let Some(after) = query.created_after {
            sql.push_str(" AND created_at >= $created_after");
            bindings.push(("created_after".to_string(), serde_json::json!(after.to_rfc3339())));
        }

        if let Some(before) = query.created_before {
            sql.push_str(" AND created_at <= $created_before");
            bindings.push(("created_before".to_string(), serde_json::json!(before.to_rfc3339())));
        }

        // Order and limit
        sql.push_str(" ORDER BY importance_score DESC, created_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" START {}", offset));
        }

        // Execute query
        let mut db_query = self.pool.client().query(&sql);
        for (name, value) in bindings {
            db_query = db_query.bind((name, value));
        }

        let mut response = db_query.await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<MemoryRecord> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        let memories: Result<Vec<MemoryNode>, DomainError> = records
            .into_iter()
            .map(Self::from_record)
            .collect();

        memories
    }

    /// =========================================================================
    /// Semantic search using vector similarity
    /// =========================================================================
    #[instrument(skip(self, query_vector))]
    async fn semantic_search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        min_similarity: f64,
    ) -> Result<Vec<(MemoryNode, f64)>, DomainError> {
        // SurrealDB vector search using cosine similarity
        // Note: This is a simplified implementation. In production,
        // you would use SurrealDB's vector index for efficiency.
        
        let sql = r#"
            SELECT *, 
                   vector::similarity::cosine(vector, $query_vector) AS similarity
            FROM memory
            WHERE vector::similarity::cosine(vector, $query_vector) >= $min_similarity
            ORDER BY similarity DESC
            LIMIT $limit
        "#;

        let mut response = self.pool.client()
            .query(sql)
            .bind(("query_vector", &query_vector))
            .bind(("min_similarity", min_similarity))
            .bind(("limit", limit))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        #[derive(Debug, Deserialize)]
        struct SearchResult {
            #[serde(flatten)]
            record: MemoryRecord,
            similarity: f64,
        }

        let results: Vec<SearchResult> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        let memories: Result<Vec<(MemoryNode, f64)>, DomainError> = results
            .into_iter()
            .map(|r| Ok((Self::from_record(r.record)?, r.similarity)))
            .collect();

        memories
    }

    /// =========================================================================
    /// Get all memories with pagination
    /// =========================================================================
    async fn get_all(&self, limit: usize, offset: usize) -> Result<Vec<MemoryNode>, DomainError> {
        let sql = format!(
            "SELECT * FROM memory ORDER BY created_at DESC LIMIT {} START {}",
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

    /// =========================================================================
    /// Count total memories
    /// =========================================================================
    async fn count(&self) -> Result<usize, DomainError> {
        let mut response = self.pool.client()
            .query("SELECT count() FROM memory GROUP ALL")
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: usize,
        }

        let result: Option<CountResult> = response.take(0)
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(result.map(|r| r.count).unwrap_or(0))
    }

    /// =========================================================================
    /// Create a relation
    /// =========================================================================
    #[instrument(skip(self, edge))]
    async fn create_relation(&self, edge: GraphEdge) -> Result<GraphEdge, DomainError> {
        let table = Self::relation_table_name(&edge.relation);
        let from_id = format!("memory:{}", edge.from_id);
        let to_id = format!("memory:{}", edge.to_id);

        let sql = format!(
            "RELATE {} -> {} -> {} SET confidence = $confidence, created_at = $created_at, metadata = $metadata",
            from_id, table, to_id
        );

        self.pool.client()
            .query(&sql)
            .bind(("confidence", edge.confidence))
            .bind(("created_at", edge.created_at.to_rfc3339()))
            .bind(("metadata", serde_json::to_value(&edge.metadata).unwrap_or_default()))
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!(
            from = %edge.from_id,
            to = %edge.to_id,
            relation = %edge.relation,
            "Relation created"
        );

        Ok(edge)
    }

    /// =========================================================================
    /// Get relations for a memory
    /// =========================================================================
    async fn get_relations(
        &self,
        memory_id: Uuid,
        relation_type: Option<Relation>,
        direction: RelationDirection,
    ) -> Result<Vec<GraphEdge>, DomainError> {
        let tables = match relation_type {
            Some(ref rel) => vec![Self::relation_table_name(rel)],
            None => vec![
                "related_to", "causes", "part_of", "follows", "contradicts",
                "supports", "derived_from", "same_as", "context_of", "references_rel", "supersedes"
            ],
        };

        let mut all_edges = Vec::new();
        let id_str = format!("memory:{}", memory_id);

        for table in tables {
            let sql = match direction {
                RelationDirection::Outgoing => format!(
                    "SELECT * FROM {} WHERE in = $id",
                    table
                ),
                RelationDirection::Incoming => format!(
                    "SELECT * FROM {} WHERE out = $id",
                    table
                ),
                RelationDirection::Both => format!(
                    "SELECT * FROM {} WHERE in = $id OR out = $id",
                    table
                ),
            };

            let mut response = self.pool.client()
                .query(&sql)
                .bind(("id", &id_str))
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            #[derive(Debug, Deserialize)]
            struct EdgeRecord {
                #[serde(rename = "in")]
                from: String,
                #[serde(rename = "out")]
                to: String,
                confidence: f64,
                created_at: chrono::DateTime<chrono::Utc>,
                metadata: Option<serde_json::Value>,
            }

            let records: Vec<EdgeRecord> = response.take(0)
                .unwrap_or_default();

            for record in records {
                let from_id = record.from.trim_start_matches("memory:").to_string();
                let to_id = record.to.trim_start_matches("memory:").to_string();

                let edge = GraphEdge {
                    from_id: Uuid::parse_str(&from_id)
                        .map_err(|e| DomainError::database(format!("Invalid UUID: {}", e)))?,
                    to_id: Uuid::parse_str(&to_id)
                        .map_err(|e| DomainError::database(format!("Invalid UUID: {}", e)))?,
                    relation: relation_type.clone().unwrap_or(Relation::RelatedTo),
                    confidence: record.confidence,
                    created_at: record.created_at,
                    metadata: record.metadata
                        .map(|v| serde_json::from_value(v).unwrap_or_default())
                        .unwrap_or_default(),
                };

                all_edges.push(edge);
            }
        }

        Ok(all_edges)
    }

    /// =========================================================================
    /// Get related memories via graph traversal
    /// =========================================================================
    async fn get_related_memories(
        &self,
        memory_id: Uuid,
        max_depth: usize,
        _relation_types: Option<Vec<Relation>>,
    ) -> Result<Vec<(MemoryNode, GraphEdge)>, DomainError> {
        // Use SurrealDB's graph traversal
        let sql = format!(
            "SELECT *, ->relates_to.out AS related FROM memory:{} FETCH related LIMIT {}",
            memory_id,
            max_depth * 10
        );

        let mut response = self.pool.client()
            .query(&sql)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let records: Vec<MemoryRecord> = response.take(0).unwrap_or_default();

        let mut results = Vec::new();
        for record in records {
            let memory = Self::from_record(record)?;
            let edge = GraphEdge::new(memory_id, memory.id, Relation::RelatedTo);
            results.push((memory, edge));
        }

        Ok(results)
    }

    /// =========================================================================
    /// Delete a relation
    /// =========================================================================
    async fn delete_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation: Relation,
    ) -> Result<bool, DomainError> {
        let table = Self::relation_table_name(&relation);
        let sql = format!(
            "DELETE {} WHERE in = memory:{} AND out = memory:{}",
            table, from_id, to_id
        );

        self.pool.client()
            .query(&sql)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(true)
    }

    /// =========================================================================
    /// Get graph statistics
    /// =========================================================================
    async fn get_graph_stats(&self) -> Result<GraphStats, DomainError> {
        // Count nodes
        let total_nodes = self.count().await?;

        // Count edges (simplified - counts all relation tables)
        let mut total_edges = 0usize;
        let tables = vec![
            "related_to", "causes", "part_of", "follows", "contradicts",
            "supports", "derived_from", "same_as", "context_of", "references_rel", "supersedes"
        ];

        let mut edges_by_type = std::collections::HashMap::new();

        for table in tables {
            let sql = format!("SELECT count() FROM {} GROUP ALL", table);
            let mut response = self.pool.client()
                .query(&sql)
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            #[derive(Debug, Deserialize)]
            struct CountResult {
                count: usize,
            }

            let result: Option<CountResult> = response.take(0).unwrap_or(None);
            let count = result.map(|r| r.count).unwrap_or(0);
            
            total_edges += count;
            edges_by_type.insert(table.to_string(), count);
        }

        // Count by type
        let mut response = self.pool.client()
            .query("SELECT memory_type, count() FROM memory GROUP BY memory_type")
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        #[derive(Debug, Deserialize)]
        struct TypeCount {
            memory_type: String,
            count: usize,
        }

        let type_counts: Vec<TypeCount> = response.take(0).unwrap_or_default();
        let nodes_by_type: std::collections::HashMap<String, usize> = type_counts
            .into_iter()
            .map(|tc| (tc.memory_type, tc.count))
            .collect();

        let avg_connections = if total_nodes > 0 {
            total_edges as f64 / total_nodes as f64
        } else {
            0.0
        };

        Ok(GraphStats {
            total_nodes,
            total_edges,
            nodes_by_type,
            edges_by_type,
            avg_connections,
            most_connected_node: None, // Would need additional query
        })
    }

    /// =========================================================================
    /// Export full graph
    /// =========================================================================
    async fn export_graph(&self) -> Result<GraphExport, DomainError> {
        let nodes = self.get_all(10000, 0).await?;
        
        let mut all_edges = Vec::new();
        for node in &nodes {
            let edges = self.get_relations(node.id, None, RelationDirection::Outgoing).await?;
            all_edges.extend(edges);
        }

        Ok(GraphExport {
            nodes,
            edges: all_edges,
            exported_at: chrono::Utc::now(),
        })
    }
}
