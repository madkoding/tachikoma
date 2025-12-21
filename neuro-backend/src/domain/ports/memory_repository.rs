//! =============================================================================
//! Memory Repository Port
//! =============================================================================
//! Defines the abstract interface for memory persistence operations.
//! This port is implemented by the SurrealDB adapter in the infrastructure layer.
//! =============================================================================

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    entities::memory::{MemoryNode, MemoryQuery},
    errors::DomainError,
    value_objects::relation::{GraphEdge, Relation},
};

/// =============================================================================
/// MemoryRepository - Abstract interface for memory storage
/// =============================================================================
/// Defines all operations for storing, retrieving, and managing memory nodes
/// in the GraphRAG system. Implementations handle the actual persistence.
/// 
/// # Responsibilities
/// 
/// * CRUD operations for memory nodes
/// * Semantic search using vector embeddings
/// * Graph traversal for related memories
/// * Relation management between nodes
/// 
/// # Implementation Notes
/// 
/// Implementations should handle:
/// - Connection pooling and retry logic
/// - Vector index optimization
/// - Graph edge management
/// - Transaction support where available
/// =============================================================================
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    // =========================================================================
    // CRUD Operations
    // =========================================================================

    /// =========================================================================
    /// Create a new memory node
    /// =========================================================================
    /// Persists a new memory node to the database. The node should have a
    /// unique ID and valid vector embedding.
    /// 
    /// # Arguments
    /// 
    /// * `memory` - The memory node to create
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The created memory with any server-generated fields
    /// * `Err(DomainError)` - If creation fails
    /// 
    /// # Errors
    /// 
    /// * `DomainError::DatabaseError` - Database connection or query failed
    /// * `DomainError::DuplicateEntry` - Memory with same ID already exists
    /// =========================================================================
    async fn create(&self, memory: MemoryNode) -> Result<MemoryNode, DomainError>;

    /// =========================================================================
    /// Get a memory by its unique ID
    /// =========================================================================
    /// Retrieves a single memory node by its UUID.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The unique identifier of the memory
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(MemoryNode))` - The found memory
    /// * `Ok(None)` - No memory found with the given ID
    /// * `Err(DomainError)` - If the query fails
    /// =========================================================================
    async fn get_by_id(&self, id: Uuid) -> Result<Option<MemoryNode>, DomainError>;

    /// =========================================================================
    /// Update an existing memory node
    /// =========================================================================
    /// Updates all fields of an existing memory node. The node must already
    /// exist in the database.
    /// 
    /// # Arguments
    /// 
    /// * `memory` - The memory node with updated fields
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The updated memory
    /// * `Err(DomainError)` - If update fails
    /// 
    /// # Errors
    /// 
    /// * `DomainError::NotFound` - Memory with given ID doesn't exist
    /// * `DomainError::DatabaseError` - Database operation failed
    /// =========================================================================
    async fn update(&self, memory: MemoryNode) -> Result<MemoryNode, DomainError>;

    /// =========================================================================
    /// Delete a memory by ID
    /// =========================================================================
    /// Removes a memory node and all its relations from the database.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The unique identifier of the memory to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Memory was deleted
    /// * `Ok(false)` - Memory didn't exist
    /// * `Err(DomainError)` - If deletion fails
    /// =========================================================================
    async fn delete(&self, id: Uuid) -> Result<bool, DomainError>;

    // =========================================================================
    // Query Operations
    // =========================================================================

    /// =========================================================================
    /// Query memories with filters
    /// =========================================================================
    /// Retrieves memories matching the given query parameters.
    /// Supports filtering by type, tags, importance, and time range.
    /// 
    /// # Arguments
    /// 
    /// * `query` - Query parameters for filtering
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<MemoryNode>)` - Matching memories (may be empty)
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    async fn query(&self, query: MemoryQuery) -> Result<Vec<MemoryNode>, DomainError>;

    /// =========================================================================
    /// Semantic search using vector similarity
    /// =========================================================================
    /// Finds memories similar to the given query vector using cosine
    /// similarity or another vector distance metric.
    /// 
    /// # Arguments
    /// 
    /// * `query_vector` - The embedding vector to search with
    /// * `limit` - Maximum number of results to return
    /// * `min_similarity` - Minimum similarity threshold (0.0 - 1.0)
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<(MemoryNode, f64)>)` - Memories with similarity scores
    /// * `Err(DomainError)` - If search fails
    /// =========================================================================
    async fn semantic_search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        min_similarity: f64,
    ) -> Result<Vec<(MemoryNode, f64)>, DomainError>;

    /// =========================================================================
    /// Get all memories (with optional pagination)
    /// =========================================================================
    /// Retrieves all memories with optional limit and offset.
    /// Used for admin dashboard and bulk operations.
    /// 
    /// # Arguments
    /// 
    /// * `limit` - Maximum number of memories to return
    /// * `offset` - Number of memories to skip
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<MemoryNode>)` - List of memories
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    async fn get_all(&self, limit: usize, offset: usize) -> Result<Vec<MemoryNode>, DomainError>;

    /// =========================================================================
    /// Get total count of memories
    /// =========================================================================
    /// Returns the total number of memories in the database.
    /// 
    /// # Returns
    /// 
    /// * `Ok(usize)` - Total count
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    async fn count(&self) -> Result<usize, DomainError>;

    // =========================================================================
    // Graph Operations
    // =========================================================================

    /// =========================================================================
    /// Create a relation between two memories
    /// =========================================================================
    /// Adds a directed edge between two memory nodes in the graph.
    /// 
    /// # Arguments
    /// 
    /// * `edge` - The graph edge defining the relation
    /// 
    /// # Returns
    /// 
    /// * `Ok(GraphEdge)` - The created edge
    /// * `Err(DomainError)` - If creation fails
    /// =========================================================================
    async fn create_relation(&self, edge: GraphEdge) -> Result<GraphEdge, DomainError>;

    /// =========================================================================
    /// Get relations for a memory
    /// =========================================================================
    /// Retrieves all relations (edges) connected to a memory node.
    /// 
    /// # Arguments
    /// 
    /// * `memory_id` - The ID of the memory to get relations for
    /// * `relation_type` - Optional filter for relation type
    /// * `direction` - Whether to get outgoing, incoming, or both relations
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<GraphEdge>)` - List of relations
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    async fn get_relations(
        &self,
        memory_id: Uuid,
        relation_type: Option<Relation>,
        direction: RelationDirection,
    ) -> Result<Vec<GraphEdge>, DomainError>;

    /// =========================================================================
    /// Get related memories (graph traversal)
    /// =========================================================================
    /// Traverses the graph to find memories related to the given node.
    /// Supports multi-hop traversal with depth limit.
    /// 
    /// # Arguments
    /// 
    /// * `memory_id` - The starting memory node
    /// * `max_depth` - Maximum graph traversal depth
    /// * `relation_types` - Optional filter for relation types
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<(MemoryNode, GraphEdge)>)` - Related memories with their edges
    /// * `Err(DomainError)` - If traversal fails
    /// =========================================================================
    async fn get_related_memories(
        &self,
        memory_id: Uuid,
        max_depth: usize,
        relation_types: Option<Vec<Relation>>,
    ) -> Result<Vec<(MemoryNode, GraphEdge)>, DomainError>;

    /// =========================================================================
    /// Delete a relation between memories
    /// =========================================================================
    /// Removes an edge from the graph.
    /// 
    /// # Arguments
    /// 
    /// * `from_id` - Source memory ID
    /// * `to_id` - Target memory ID
    /// * `relation` - Type of relation to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Relation was deleted
    /// * `Ok(false)` - Relation didn't exist
    /// * `Err(DomainError)` - If deletion fails
    /// =========================================================================
    async fn delete_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation: Relation,
    ) -> Result<bool, DomainError>;

    // =========================================================================
    // Admin Operations
    // =========================================================================

    /// =========================================================================
    /// Get graph statistics
    /// =========================================================================
    /// Returns statistics about the memory graph for admin dashboard.
    /// 
    /// # Returns
    /// 
    /// * `Ok(GraphStats)` - Statistics about nodes and edges
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    async fn get_graph_stats(&self) -> Result<GraphStats, DomainError>;

    /// =========================================================================
    /// Export full graph structure
    /// =========================================================================
    /// Exports all nodes and edges for visualization or backup.
    /// 
    /// # Returns
    /// 
    /// * `Ok(GraphExport)` - Full graph data
    /// * `Err(DomainError)` - If export fails
    /// =========================================================================
    async fn export_graph(&self) -> Result<GraphExport, DomainError>;
}

/// =============================================================================
/// RelationDirection - Direction for relation queries
/// =============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationDirection {
    /// Relations where this node is the source
    Outgoing,
    /// Relations where this node is the target
    Incoming,
    /// Both incoming and outgoing relations
    Both,
}

/// =============================================================================
/// GraphStats - Statistics about the memory graph
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphStats {
    /// Total number of memory nodes
    pub total_nodes: usize,
    /// Total number of relations/edges
    pub total_edges: usize,
    /// Count by memory type
    pub nodes_by_type: std::collections::HashMap<String, usize>,
    /// Count by relation type
    pub edges_by_type: std::collections::HashMap<String, usize>,
    /// Average connections per node
    pub avg_connections: f64,
    /// Most connected node ID
    pub most_connected_node: Option<Uuid>,
}

/// =============================================================================
/// GraphExport - Full export of graph data
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphExport {
    /// All memory nodes
    pub nodes: Vec<MemoryNode>,
    /// All edges/relations
    pub edges: Vec<GraphEdge>,
    /// Export timestamp
    pub exported_at: chrono::DateTime<chrono::Utc>,
}
