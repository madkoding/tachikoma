//! =============================================================================
//! Relation Value Object
//! =============================================================================
//! Represents the types of relations between memory nodes in the graph.
//! These relations enable the GraphRAG pattern by connecting memories.
//! 
//! # Graph Structure
//! 
//! ```text
//!   ┌─────────────┐         CAUSES          ┌─────────────┐
//!   │   Memory A  │ ───────────────────────▶│   Memory B  │
//!   │  (Problem)  │                         │ (Solution)  │
//!   └─────────────┘                         └─────────────┘
//!         │                                        │
//!         │ RELATED_TO                             │ PART_OF
//!         ▼                                        ▼
//!   ┌─────────────┐                         ┌─────────────┐
//!   │   Memory C  │                         │   Memory D  │
//!   │  (Context)  │                         │  (Whole)    │
//!   └─────────────┘                         └─────────────┘
//! ```
//! =============================================================================

use serde::{Deserialize, Serialize};

/// =============================================================================
/// Relation - Types of connections between memory nodes
/// =============================================================================
/// Defines the semantic relationships that can exist between memories.
/// These relations are stored as graph edges in SurrealDB.
/// 
/// # Relation Types
/// 
/// * `RelatedTo` - General semantic relationship
/// * `Causes` - Causal relationship (A causes B)
/// * `PartOf` - Composition relationship (A is part of B)
/// * `Follows` - Temporal/sequential relationship (A comes before B)
/// * `Contradicts` - Conflicting information
/// * `Supports` - Supporting/confirming relationship
/// * `DerivedFrom` - Inference relationship (B was derived from A)
/// * `SameAs` - Duplicate or equivalent information
/// * `ContextOf` - Contextual relationship (A provides context for B)
/// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// =======================================================================
    /// General semantic relationship
    /// =======================================================================
    /// Used when two memories are related but the specific type of
    /// relationship is not clearly defined.
    /// 
    /// # Example
    /// - "User works with Rust" ←→ "User prefers functional programming"
    /// =======================================================================
    RelatedTo,

    /// =======================================================================
    /// Causal relationship
    /// =======================================================================
    /// Indicates that one memory is a cause or reason for another.
    /// Direction: Source CAUSES Target
    /// 
    /// # Example
    /// - "Server crashed" → CAUSES → "Data was lost"
    /// =======================================================================
    Causes,

    /// =======================================================================
    /// Composition relationship
    /// =======================================================================
    /// Indicates that one memory is a component or subset of another.
    /// Direction: Source PART_OF Target (Source is contained in Target)
    /// 
    /// # Example
    /// - "User authentication" → PART_OF → "Security system"
    /// =======================================================================
    PartOf,

    /// =======================================================================
    /// Temporal/sequential relationship
    /// =======================================================================
    /// Indicates temporal or logical sequence.
    /// Direction: Source FOLLOWS Target (Source comes after Target)
    /// 
    /// # Example
    /// - "Deploy to production" → FOLLOWS → "Run tests"
    /// =======================================================================
    Follows,

    /// =======================================================================
    /// Conflicting information
    /// =======================================================================
    /// Indicates that two memories contain contradictory information.
    /// This is important for maintaining consistency.
    /// 
    /// # Example
    /// - "API uses REST" ←→ CONTRADICTS ←→ "API uses GraphQL"
    /// =======================================================================
    Contradicts,

    /// =======================================================================
    /// Supporting relationship
    /// =======================================================================
    /// Indicates that one memory supports or confirms another.
    /// Direction: Source SUPPORTS Target
    /// 
    /// # Example
    /// - "Performance tests passed" → SUPPORTS → "System is stable"
    /// =======================================================================
    Supports,

    /// =======================================================================
    /// Inference relationship
    /// =======================================================================
    /// Indicates that one memory was inferred or derived from another.
    /// Direction: Target DERIVED_FROM Source
    /// 
    /// # Example
    /// - "User is a developer" → DERIVED_FROM → "User writes Rust code"
    /// =======================================================================
    DerivedFrom,

    /// =======================================================================
    /// Duplicate/equivalent relationship
    /// =======================================================================
    /// Indicates that two memories contain the same or equivalent information.
    /// Used for deduplication and consolidation.
    /// 
    /// # Example
    /// - "Server runs on port 8080" ←→ SAME_AS ←→ "API available at :8080"
    /// =======================================================================
    SameAs,

    /// =======================================================================
    /// Contextual relationship
    /// =======================================================================
    /// Indicates that one memory provides context for understanding another.
    /// Direction: Source CONTEXT_OF Target
    /// 
    /// # Example
    /// - "Working on NEURO-OS project" → CONTEXT_OF → "Need GraphRAG system"
    /// =======================================================================
    ContextOf,

    /// =======================================================================
    /// Reference relationship
    /// =======================================================================
    /// Indicates that one memory references or mentions another.
    /// 
    /// # Example
    /// - "See documentation for API" → REFERENCES → "API Documentation"
    /// =======================================================================
    References,

    /// =======================================================================
    /// Supersedes relationship
    /// =======================================================================
    /// Indicates that one memory replaces or supersedes another.
    /// Direction: Source SUPERSEDES Target (Source is newer/better)
    /// 
    /// # Example
    /// - "Use Rust 2021 edition" → SUPERSEDES → "Use Rust 2018 edition"
    /// =======================================================================
    Supersedes,
}

impl Relation {
    /// =========================================================================
    /// Get the inverse of a relation
    /// =========================================================================
    /// Returns the inverse relation when traversing the graph in reverse.
    /// Some relations are symmetric (same in both directions).
    /// 
    /// # Returns
    /// 
    /// The inverse relation type
    /// =========================================================================
    pub fn inverse(&self) -> Self {
        match self {
            Relation::RelatedTo => Relation::RelatedTo,     // Symmetric
            Relation::Causes => Relation::DerivedFrom,       // A causes B → B derived from A
            Relation::PartOf => Relation::PartOf,            // Inverse is "Contains"
            Relation::Follows => Relation::Follows,          // Inverse is "Precedes"
            Relation::Contradicts => Relation::Contradicts,  // Symmetric
            Relation::Supports => Relation::DerivedFrom,     // A supports B → B derived from A
            Relation::DerivedFrom => Relation::Causes,       // A derived from B → B causes A
            Relation::SameAs => Relation::SameAs,            // Symmetric
            Relation::ContextOf => Relation::RelatedTo,      // Inverse is generic
            Relation::References => Relation::References,    // Inverse is "ReferencedBy"
            Relation::Supersedes => Relation::Supersedes,    // Inverse is "SupersededBy"
        }
    }

    /// =========================================================================
    /// Check if the relation is symmetric
    /// =========================================================================
    /// Symmetric relations have the same meaning in both directions.
    /// 
    /// # Returns
    /// 
    /// `true` if the relation is symmetric
    /// =========================================================================
    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            Relation::RelatedTo | Relation::Contradicts | Relation::SameAs
        )
    }

    /// =========================================================================
    /// Get the relation weight for graph algorithms
    /// =========================================================================
    /// Returns a weight value used in graph traversal algorithms.
    /// Higher weights indicate stronger relationships.
    /// 
    /// # Returns
    /// 
    /// A weight value between 0.0 and 1.0
    /// =========================================================================
    pub fn weight(&self) -> f64 {
        match self {
            Relation::SameAs => 1.0,         // Strongest
            Relation::PartOf => 0.9,
            Relation::Causes => 0.8,
            Relation::DerivedFrom => 0.8,
            Relation::Supports => 0.7,
            Relation::Supersedes => 0.7,
            Relation::ContextOf => 0.6,
            Relation::Follows => 0.5,
            Relation::References => 0.4,
            Relation::RelatedTo => 0.3,
            Relation::Contradicts => 0.2,    // Weakest (but important for consistency)
        }
    }

    /// =========================================================================
    /// Get all relation types as a list
    /// =========================================================================
    pub fn all_types() -> Vec<Self> {
        vec![
            Relation::RelatedTo,
            Relation::Causes,
            Relation::PartOf,
            Relation::Follows,
            Relation::Contradicts,
            Relation::Supports,
            Relation::DerivedFrom,
            Relation::SameAs,
            Relation::ContextOf,
            Relation::References,
            Relation::Supersedes,
        ]
    }

    /// =========================================================================
    /// Convert to SurrealDB table name
    /// =========================================================================
    /// Returns the table name used for this relation type in SurrealDB.
    /// =========================================================================
    pub fn to_table_name(&self) -> &'static str {
        match self {
            Relation::RelatedTo => "related_to",
            Relation::Causes => "causes",
            Relation::PartOf => "part_of",
            Relation::Follows => "follows",
            Relation::Contradicts => "contradicts",
            Relation::Supports => "supports",
            Relation::DerivedFrom => "derived_from",
            Relation::SameAs => "same_as",
            Relation::ContextOf => "context_of",
            Relation::References => "references",
            Relation::Supersedes => "supersedes",
        }
    }
}

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_table_name())
    }
}

/// =============================================================================
/// GraphEdge - Represents an edge in the memory graph
/// =============================================================================
/// Stores a relationship between two memory nodes.
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source memory node ID
    pub from_id: uuid::Uuid,

    /// Target memory node ID
    pub to_id: uuid::Uuid,

    /// Type of relation
    pub relation: Relation,

    /// Confidence score for inferred relations (0.0 - 1.0)
    pub confidence: f64,

    /// When the relation was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl GraphEdge {
    /// =========================================================================
    /// Create a new graph edge
    /// =========================================================================
    pub fn new(
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        relation: Relation,
    ) -> Self {
        Self {
            from_id,
            to_id,
            relation,
            confidence: 1.0,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// =========================================================================
    /// Create an edge with confidence score
    /// =========================================================================
    pub fn with_confidence(
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        relation: Relation,
        confidence: f64,
    ) -> Self {
        Self {
            from_id,
            to_id,
            relation,
            confidence: confidence.clamp(0.0, 1.0),
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_inverse() {
        assert_eq!(Relation::Causes.inverse(), Relation::DerivedFrom);
        assert_eq!(Relation::RelatedTo.inverse(), Relation::RelatedTo);
        assert_eq!(Relation::SameAs.inverse(), Relation::SameAs);
    }

    #[test]
    fn test_relation_symmetry() {
        assert!(Relation::RelatedTo.is_symmetric());
        assert!(Relation::SameAs.is_symmetric());
        assert!(!Relation::Causes.is_symmetric());
    }

    #[test]
    fn test_relation_weights() {
        assert!(Relation::SameAs.weight() > Relation::RelatedTo.weight());
        assert!(Relation::Causes.weight() > Relation::Contradicts.weight());
    }
}
