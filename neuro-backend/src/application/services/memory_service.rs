//! =============================================================================
//! Memory Service
//! =============================================================================
//! Application service for managing memories in the GraphRAG system.
//! Handles memory creation, retrieval, search, and graph operations.
//! 
//! # Responsibilities
//! 
//! * Memory CRUD operations
//! * Semantic search with embedding generation
//! * Graph relation management
//! * Memory importance calculation
//! =============================================================================

use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::domain::{
    entities::memory::{MemoryMetadata, MemoryNode, MemoryQuery, MemoryType},
    errors::DomainError,
    ports::{
        llm_provider::LlmProvider,
        memory_repository::{GraphExport, GraphStats, MemoryRepository, RelationDirection},
    },
    value_objects::relation::{GraphEdge, Relation},
};

/// =============================================================================
/// MemoryService - Memory Management Application Service
/// =============================================================================
/// Orchestrates memory operations by coordinating between the memory repository
/// and the LLM provider (for embeddings).
/// 
/// # Example Usage
/// 
/// ```rust
/// let service = MemoryService::new(memory_repo, llm_provider);
/// 
/// // Create a memory with auto-generated embedding
/// let memory = service.create_memory(
///     "User prefers dark mode".to_string(),
///     MemoryType::Preference,
///     None,
/// ).await?;
/// 
/// // Search memories semantically
/// let results = service.search("interface preferences", 5).await?;
/// ```
/// =============================================================================
pub struct MemoryService {
    /// Repository for memory persistence
    repository: Arc<dyn MemoryRepository>,
    
    /// LLM provider for embedding generation
    llm_provider: Arc<dyn LlmProvider>,
}

impl MemoryService {
    /// =========================================================================
    /// Create a new MemoryService
    /// =========================================================================
    /// Initializes the service with required dependencies.
    /// 
    /// # Arguments
    /// 
    /// * `repository` - Memory repository for persistence
    /// * `llm_provider` - LLM provider for embeddings
    /// 
    /// # Returns
    /// 
    /// A new MemoryService instance
    /// =========================================================================
    pub fn new(
        repository: Arc<dyn MemoryRepository>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            repository,
            llm_provider,
        }
    }

    // =========================================================================
    // Memory CRUD Operations
    // =========================================================================

    /// =========================================================================
    /// Create a new memory
    /// =========================================================================
    /// Creates a memory with auto-generated embedding vector.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The memory content text
    /// * `memory_type` - Classification of the memory
    /// * `metadata` - Optional additional metadata
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The created or merged memory with ID and embedding
    /// * `Err(DomainError)` - If creation fails
    /// 
    /// # Behavior
    /// 
    /// This function implements smart memory deduplication:
    /// 1. Generates embedding for the new content
    /// 2. Searches for similar existing memories (similarity > 0.50)
    /// 3. Uses LLM to determine if memories should be merged
    /// 4. If related memory exists, merges the new information into it
    /// 5. If no related memory exists, creates a new one
    /// =========================================================================
    #[instrument(skip(self, content), fields(content_len = content.len()))]
    pub async fn create_memory(
        &self,
        content: String,
        memory_type: MemoryType,
        metadata: Option<MemoryMetadata>,
    ) -> Result<MemoryNode, DomainError> {
        // Validate content
        if content.trim().is_empty() {
            return Err(DomainError::validation("content", "Content cannot be empty"));
        }

        // Generate embedding
        info!("Generating embedding for content: {}...", &content[..content.len().min(50)]);
        let vector = self.llm_provider.embed(&content).await?;
        info!("Embedding generated with {} dimensions", vector.len());

        // Search for similar existing memories (low threshold to find candidates)
        let similar_memories = self.repository
            .semantic_search(vector.clone(), 10, 0.30)
            .await?;
        
        info!("Found {} similar memories with threshold 0.30", similar_memories.len());

        // Filter by same memory type
        let candidates: Vec<_> = similar_memories
            .into_iter()
            .filter(|(m, _)| m.memory_type == memory_type)
            .collect();
        
        info!("After filtering by type: {} candidates", candidates.len());

        // Determine if we should merge based on similarity and LLM analysis
        if !candidates.is_empty() {
            // Get the most similar candidate
            let (best_match, best_similarity) = candidates
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();
            
            info!("Best match similarity: {:.2}", best_similarity);
            
            // High similarity (>0.60) = automatic merge without asking LLM
            // This catches cases like "Me gusta el morado" and "También me gusta el verde"
            let should_merge = if *best_similarity > 0.60 {
                info!("High similarity ({:.2}), auto-merging", best_similarity);
                true
            } else {
                // Medium similarity (0.30-0.60) = ask LLM
                info!("Medium similarity ({:.2}), asking LLM", best_similarity);
                self.should_merge_memories(&content, &best_match.content).await.unwrap_or(false)
            };
            
            if should_merge {
                let existing_memory = best_match.clone();
                let similarity = *best_similarity;
                info!(
                    existing_id = %existing_memory.id,
                    similarity = %similarity,
                    "Memories should be merged"
                );

                // Merge the content - combine old and new information
                let merged_content = self.merge_memory_content(
                    &existing_memory.content,
                    &content,
                ).await?;

                // Only update if content actually changed
                if merged_content != existing_memory.content {
                    // Re-generate embedding for merged content
                    let merged_vector = self.llm_provider.embed(&merged_content).await?;
                    
                    // Update the existing memory
                    let mut updated_memory = existing_memory.clone();
                    updated_memory.content = merged_content;
                    updated_memory.vector = merged_vector;
                    updated_memory.updated_at = chrono::Utc::now();
                    updated_memory.access_count += 1;
                    
                    // Merge metadata if provided
                    if let Some(new_meta) = metadata {
                        // Merge tags
                        for tag in new_meta.tags {
                            if !updated_memory.metadata.tags.contains(&tag) {
                                updated_memory.metadata.tags.push(tag);
                            }
                        }
                    }

                    let result = self.repository.update(updated_memory).await?;
                    info!(memory_id = %result.id, "Memory merged and updated successfully");
                    return Ok(result);
                } else {
                    // Content is essentially the same, just return the existing memory
                    info!(memory_id = %existing_memory.id, "Memory content unchanged, skipping duplicate");
                    return Ok(existing_memory);
                }
            }
        }

        // No related memory found, create new one
        let memory = if let Some(meta) = metadata {
            MemoryNode::with_metadata(content, vector, memory_type, meta)
        } else {
            MemoryNode::new(content, vector, memory_type)
        };
        
        info!("Memory node created with vector length: {}", memory.vector.len());

        // Persist to database
        let created = self.repository.create(memory).await?;

        info!(memory_id = %created.id, "Memory created successfully");

        Ok(created)
    }

    /// =========================================================================
    /// Find Memory to Merge using LLM
    /// =========================================================================
    /// Uses the LLM to intelligently determine if the new content should be
    /// merged with any existing memory candidate.
    /// =========================================================================
    /// =========================================================================
    /// Simple LLM check for medium-similarity cases
    /// =========================================================================
    async fn should_merge_memories(
        &self,
        new_content: &str,
        existing_content: &str,
    ) -> Result<bool, DomainError> {
        let prompt = format!(
            r#"¿Estas dos frases tratan del MISMO TEMA o están relacionadas?

FRASE 1: "{}"
FRASE 2: "{}"

Responde SOLO "SI" o "NO"."#,
            new_content,
            existing_content
        );

        let response = self.llm_provider.generate(&prompt, None).await?;
        let response_text = response.content.trim().to_uppercase();
        
        info!("LLM merge check response: '{}'", response_text);
        
        // Check if response contains "SI" or "SÍ"
        let should_merge = response_text.contains("SI") || response_text.contains("SÍ");
        info!("Should merge: {}", should_merge);
        
        Ok(should_merge)
    }

    /// =========================================================================
    /// Merge memory content using LLM
    /// =========================================================================
    /// Combines existing memory content with new information intelligently.
    /// Uses the LLM to merge related information without duplication.
    /// =========================================================================
    async fn merge_memory_content(
        &self,
        existing_content: &str,
        new_content: &str,
    ) -> Result<String, DomainError> {
        let merge_prompt = format!(
            r#"Merge these two related pieces of information into a single, coherent statement.
Combine all unique information without repetition.
Keep the result concise and natural.

Existing information: "{}"
New information: "{}"

Merged result (just the merged text, nothing else):"#,
            existing_content, new_content
        );

        let result = self.llm_provider
            .generate(&merge_prompt, None)
            .await?;

        // Clean up the response
        let merged = result.content.trim().trim_matches('"').to_string();
        
        info!(
            existing = %existing_content,
            new = %new_content,
            merged = %merged,
            "Memory content merged"
        );

        Ok(merged)
    }

    /// =========================================================================
    /// Get a memory by ID
    /// =========================================================================
    /// Retrieves a single memory by its unique identifier.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The memory UUID
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The found memory
    /// * `Err(DomainError::NotFound)` - If memory doesn't exist
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn get_memory(&self, id: Uuid) -> Result<MemoryNode, DomainError> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| DomainError::memory_not_found(id))
    }

    /// =========================================================================
    /// Update an existing memory
    /// =========================================================================
    /// Updates the content and/or metadata of an existing memory.
    /// Re-generates the embedding if content changes.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The memory ID to update
    /// * `content` - Optional new content
    /// * `memory_type` - Optional new type
    /// * `metadata` - Optional new metadata
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The updated memory
    /// * `Err(DomainError)` - If update fails
    /// =========================================================================
    #[instrument(skip(self, content))]
    pub async fn update_memory(
        &self,
        id: Uuid,
        content: Option<String>,
        memory_type: Option<MemoryType>,
        metadata: Option<MemoryMetadata>,
    ) -> Result<MemoryNode, DomainError> {
        // Get existing memory
        let mut memory = self.get_memory(id).await?;

        // Update content and re-embed if changed
        if let Some(new_content) = content {
            if new_content != memory.content {
                memory.vector = self.llm_provider.embed(&new_content).await?;
                memory.content = new_content;
            }
        }

        // Update type if provided
        if let Some(new_type) = memory_type {
            memory.memory_type = new_type;
        }

        // Update metadata if provided
        if let Some(new_meta) = metadata {
            memory.metadata = new_meta;
        }

        // Update timestamp
        memory.updated_at = chrono::Utc::now();

        // Persist changes
        let updated = self.repository.update(memory).await?;

        info!(memory_id = %id, "Memory updated successfully");

        Ok(updated)
    }

    /// =========================================================================
    /// Delete a memory
    /// =========================================================================
    /// Removes a memory and all its relations from the database.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The memory ID to delete
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Memory was deleted
    /// * `Err(DomainError)` - If deletion fails
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn delete_memory(&self, id: Uuid) -> Result<bool, DomainError> {
        let deleted = self.repository.delete(id).await?;

        if deleted {
            info!(memory_id = %id, "Memory deleted successfully");
        } else {
            debug!(memory_id = %id, "Memory not found for deletion");
        }

        Ok(deleted)
    }

    // =========================================================================
    // Search Operations
    // =========================================================================

    /// =========================================================================
    /// Semantic search for memories
    /// =========================================================================
    /// Finds memories similar to the query using vector similarity.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The search query text
    /// * `limit` - Maximum number of results
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<(MemoryNode, f64)>)` - Memories with similarity scores
    /// * `Err(DomainError)` - If search fails
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(MemoryNode, f64)>, DomainError> {
        // Generate query embedding
        let query_vector = self.llm_provider.embed(query).await?;

        // Search with default similarity threshold
        let results = self
            .repository
            .semantic_search(query_vector, limit, 0.5)
            .await?;

        debug!(query = query, results = results.len(), "Semantic search completed");

        Ok(results)
    }

    /// =========================================================================
    /// Advanced memory query
    /// =========================================================================
    /// Queries memories with various filters and options.
    /// 
    /// # Arguments
    /// 
    /// * `query` - Query parameters
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<MemoryNode>)` - Matching memories
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    #[instrument(skip(self, query))]
    pub async fn query_memories(
        &self,
        mut query: MemoryQuery,
    ) -> Result<Vec<MemoryNode>, DomainError> {
        // If query has text, generate embedding
        if let Some(text) = &query.query_text {
            let vector = self.llm_provider.embed(text).await?;
            query.query_vector = Some(vector);
        }

        self.repository.query(query).await
    }

    /// =========================================================================
    /// Get all memories with pagination
    /// =========================================================================
    /// Retrieves all memories with pagination support.
    /// 
    /// # Arguments
    /// 
    /// * `limit` - Maximum number of results
    /// * `offset` - Number of results to skip
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<MemoryNode>)` - List of memories
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    pub async fn get_all_memories(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MemoryNode>, DomainError> {
        self.repository.get_all(limit, offset).await
    }

    /// =========================================================================
    /// Get memory count
    /// =========================================================================
    /// Returns the total number of memories.
    /// =========================================================================
    pub async fn count_memories(&self) -> Result<usize, DomainError> {
        self.repository.count().await
    }

    // =========================================================================
    // Graph Operations
    // =========================================================================

    /// =========================================================================
    /// Create a relation between memories
    /// =========================================================================
    /// Adds a directed edge between two memory nodes.
    /// 
    /// # Arguments
    /// 
    /// * `from_id` - Source memory ID
    /// * `to_id` - Target memory ID
    /// * `relation` - Type of relation
    /// * `confidence` - Confidence score (0.0 - 1.0)
    /// 
    /// # Returns
    /// 
    /// * `Ok(GraphEdge)` - The created edge
    /// * `Err(DomainError)` - If creation fails
    /// =========================================================================
    #[instrument(skip(self))]
    pub async fn create_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation: Relation,
        confidence: f64,
    ) -> Result<GraphEdge, DomainError> {
        // Verify both memories exist
        self.get_memory(from_id).await?;
        self.get_memory(to_id).await?;

        // Create edge
        let edge = GraphEdge::with_confidence(from_id, to_id, relation, confidence);
        let created = self.repository.create_relation(edge).await?;

        info!(
            from = %from_id,
            to = %to_id,
            relation = %created.relation,
            "Relation created successfully"
        );

        Ok(created)
    }

    /// =========================================================================
    /// Get relations for a memory
    /// =========================================================================
    /// Retrieves all relations connected to a memory.
    /// 
    /// # Arguments
    /// 
    /// * `memory_id` - The memory ID
    /// * `relation_type` - Optional filter by relation type
    /// * `direction` - Incoming, outgoing, or both
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<GraphEdge>)` - List of relations
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    pub async fn get_relations(
        &self,
        memory_id: Uuid,
        relation_type: Option<Relation>,
        direction: RelationDirection,
    ) -> Result<Vec<GraphEdge>, DomainError> {
        self.repository
            .get_relations(memory_id, relation_type, direction)
            .await
    }

    /// =========================================================================
    /// Get related memories with graph traversal
    /// =========================================================================
    /// Finds memories related to a given memory through graph edges.
    /// 
    /// # Arguments
    /// 
    /// * `memory_id` - Starting memory ID
    /// * `max_depth` - Maximum traversal depth
    /// * `relation_types` - Optional filter for relation types
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<(MemoryNode, GraphEdge)>)` - Related memories with edges
    /// * `Err(DomainError)` - If traversal fails
    /// =========================================================================
    pub async fn get_related_memories(
        &self,
        memory_id: Uuid,
        max_depth: usize,
        relation_types: Option<Vec<Relation>>,
    ) -> Result<Vec<(MemoryNode, GraphEdge)>, DomainError> {
        self.repository
            .get_related_memories(memory_id, max_depth, relation_types)
            .await
    }

    /// =========================================================================
    /// Delete a relation
    /// =========================================================================
    /// Removes a relation between two memories.
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
    pub async fn delete_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation: Relation,
    ) -> Result<bool, DomainError> {
        self.repository.delete_relation(from_id, to_id, relation).await
    }

    // =========================================================================
    // Admin/Graph Operations
    // =========================================================================

    /// =========================================================================
    /// Get graph statistics
    /// =========================================================================
    /// Returns statistics about the memory graph for the admin dashboard.
    /// 
    /// # Returns
    /// 
    /// * `Ok(GraphStats)` - Graph statistics
    /// * `Err(DomainError)` - If query fails
    /// =========================================================================
    pub async fn get_graph_stats(&self) -> Result<GraphStats, DomainError> {
        self.repository.get_graph_stats().await
    }

    /// =========================================================================
    /// Export the full graph
    /// =========================================================================
    /// Exports all nodes and edges for visualization or backup.
    /// 
    /// # Returns
    /// 
    /// * `Ok(GraphExport)` - Full graph data
    /// * `Err(DomainError)` - If export fails
    /// =========================================================================
    pub async fn export_graph(&self) -> Result<GraphExport, DomainError> {
        self.repository.export_graph().await
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// =========================================================================
    /// Store a fact from conversation
    /// =========================================================================
    /// Convenience method for storing facts learned during conversation.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The fact to store
    /// * `source` - Source of the fact (e.g., "conversation", "web_search")
    /// * `conversation_id` - Optional conversation reference
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The created memory
    /// * `Err(DomainError)` - If creation fails
    /// =========================================================================
    pub async fn store_fact(
        &self,
        content: String,
        source: &str,
        conversation_id: Option<Uuid>,
    ) -> Result<MemoryNode, DomainError> {
        let mut metadata = MemoryMetadata::default().with_source(source);
        metadata.conversation_id = conversation_id;

        self.create_memory(content, MemoryType::Fact, Some(metadata)).await
    }

    /// =========================================================================
    /// Store a user preference
    /// =========================================================================
    /// Convenience method for storing user preferences.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The preference to store
    /// * `importance` - Importance score (0.0 - 1.0)
    /// 
    /// # Returns
    /// 
    /// * `Ok(MemoryNode)` - The created memory
    /// * `Err(DomainError)` - If creation fails
    /// =========================================================================
    pub async fn store_preference(
        &self,
        content: String,
        importance: f64,
    ) -> Result<MemoryNode, DomainError> {
        let metadata = MemoryMetadata::default()
            .with_source("user_preference")
            .with_importance(importance);

        self.create_memory(content, MemoryType::Preference, Some(metadata)).await
    }

    /// =========================================================================
    /// Find relevant context for a query
    /// =========================================================================
    /// Combines semantic search with graph traversal to find relevant context.
    /// Used for RAG (Retrieval-Augmented Generation).
    /// 
    /// # Arguments
    /// 
    /// * `query` - The query to find context for
    /// * `max_memories` - Maximum memories to return
    /// * `include_related` - Whether to include related memories via graph
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<MemoryNode>)` - Relevant memories for context
    /// * `Err(DomainError)` - If search fails
    /// =========================================================================
    pub async fn find_context(
        &self,
        query: &str,
        max_memories: usize,
        include_related: bool,
    ) -> Result<Vec<MemoryNode>, DomainError> {
        // Start with semantic search
        let mut results = self.search(query, max_memories).await?;

        if include_related && !results.is_empty() {
            // Get related memories for top results
            let mut related = Vec::new();
            for (memory, _) in results.iter().take(3) {
                if let Ok(rel) = self.get_related_memories(memory.id, 1, None).await {
                    for (mem, _) in rel {
                        if !results.iter().any(|(m, _)| m.id == mem.id) 
                            && !related.iter().any(|m: &MemoryNode| m.id == mem.id) 
                        {
                            related.push(mem);
                        }
                    }
                }
            }

            // Combine results
            let mut memories: Vec<MemoryNode> = results.into_iter().map(|(m, _)| m).collect();
            memories.extend(related);
            memories.truncate(max_memories);

            Ok(memories)
        } else {
            Ok(results.into_iter().map(|(m, _)| m).collect())
        }
    }
}
