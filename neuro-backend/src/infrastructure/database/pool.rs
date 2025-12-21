//! =============================================================================
//! Database Connection Pool
//! =============================================================================
//! Manages SurrealDB connections and initialization.
//! =============================================================================

use anyhow::Result;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{info, debug};

use crate::infrastructure::config::DatabaseConfig;

/// =============================================================================
/// DatabasePool - SurrealDB Connection Manager
/// =============================================================================
/// Manages the SurrealDB client connection and provides access to the database.
/// Handles connection initialization, authentication, and schema setup.
/// =============================================================================
#[derive(Clone)]
pub struct DatabasePool {
    /// The SurrealDB client
    client: Surreal<Client>,
}

impl DatabasePool {
    /// =========================================================================
    /// Create a new database connection pool
    /// =========================================================================
    /// Connects to SurrealDB, authenticates, and initializes the schema.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Database configuration
    /// 
    /// # Returns
    /// 
    /// * `Ok(DatabasePool)` - Connected pool
    /// * `Err` - If connection fails
    /// =========================================================================
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to SurrealDB at {}", config.url);

        // Connect to SurrealDB
        let client = Surreal::new::<Ws>(&config.url).await?;

        // Authenticate
        client
            .signin(Root {
                username: &config.username,
                password: &config.password,
            })
            .await?;

        debug!("Authenticated with SurrealDB");

        // Select namespace and database
        client
            .use_ns(&config.namespace)
            .use_db(&config.database)
            .await?;

        debug!(
            "Selected namespace '{}' and database '{}'",
            config.namespace, config.database
        );

        let pool = Self { client };

        // Initialize schema
        pool.initialize_schema().await?;

        Ok(pool)
    }

    /// =========================================================================
    /// Get the SurrealDB client
    /// =========================================================================
    /// Returns a reference to the underlying client for direct queries.
    /// =========================================================================
    pub fn client(&self) -> &Surreal<Client> {
        &self.client
    }

    /// =========================================================================
    /// Initialize database schema
    /// =========================================================================
    /// Creates necessary tables, indexes, and graph structures.
    /// This is idempotent and can be called multiple times safely.
    /// =========================================================================
    async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema...");

        // Define the memory table with vector index
        let schema = r#"
            -- Memory node table
            DEFINE TABLE IF NOT EXISTS memory SCHEMAFULL;
            
            DEFINE FIELD IF NOT EXISTS id ON memory TYPE string;
            DEFINE FIELD IF NOT EXISTS content ON memory TYPE string;
            DEFINE FIELD IF NOT EXISTS vector ON memory TYPE array;
            DEFINE FIELD IF NOT EXISTS memory_type ON memory TYPE string;
            DEFINE FIELD IF NOT EXISTS metadata ON memory TYPE object;
            DEFINE FIELD IF NOT EXISTS created_at ON memory TYPE datetime;
            DEFINE FIELD IF NOT EXISTS updated_at ON memory TYPE datetime;
            DEFINE FIELD IF NOT EXISTS access_count ON memory TYPE int DEFAULT 0;
            DEFINE FIELD IF NOT EXISTS importance_score ON memory TYPE float DEFAULT 0.5;
            
            -- Indexes for efficient queries
            DEFINE INDEX IF NOT EXISTS memory_id_idx ON memory FIELDS id UNIQUE;
            DEFINE INDEX IF NOT EXISTS memory_type_idx ON memory FIELDS memory_type;
            DEFINE INDEX IF NOT EXISTS memory_created_idx ON memory FIELDS created_at;
            DEFINE INDEX IF NOT EXISTS memory_importance_idx ON memory FIELDS importance_score;
            
            -- Relation tables (graph edges)
            DEFINE TABLE IF NOT EXISTS related_to SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON related_to TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON related_to TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON related_to TYPE object;
            
            DEFINE TABLE IF NOT EXISTS causes SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON causes TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON causes TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON causes TYPE object;
            
            DEFINE TABLE IF NOT EXISTS part_of SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON part_of TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON part_of TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON part_of TYPE object;
            
            DEFINE TABLE IF NOT EXISTS follows SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON follows TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON follows TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON follows TYPE object;
            
            DEFINE TABLE IF NOT EXISTS contradicts SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON contradicts TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON contradicts TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON contradicts TYPE object;
            
            DEFINE TABLE IF NOT EXISTS supports SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON supports TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON supports TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON supports TYPE object;
            
            DEFINE TABLE IF NOT EXISTS derived_from SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON derived_from TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON derived_from TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON derived_from TYPE object;
            
            DEFINE TABLE IF NOT EXISTS same_as SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON same_as TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON same_as TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON same_as TYPE object;
            
            DEFINE TABLE IF NOT EXISTS context_of SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON context_of TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON context_of TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON context_of TYPE object;
            
            DEFINE TABLE IF NOT EXISTS references_rel SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON references_rel TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON references_rel TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON references_rel TYPE object;
            
            DEFINE TABLE IF NOT EXISTS supersedes SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD IF NOT EXISTS confidence ON supersedes TYPE float DEFAULT 1.0;
            DEFINE FIELD IF NOT EXISTS created_at ON supersedes TYPE datetime;
            DEFINE FIELD IF NOT EXISTS metadata ON supersedes TYPE object;
            
            -- Conversation table
            DEFINE TABLE IF NOT EXISTS conversation SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS id ON conversation TYPE string;
            DEFINE FIELD IF NOT EXISTS title ON conversation TYPE option<string>;
            DEFINE FIELD IF NOT EXISTS created_at ON conversation TYPE datetime;
            DEFINE FIELD IF NOT EXISTS updated_at ON conversation TYPE datetime;
            DEFINE FIELD IF NOT EXISTS archived ON conversation TYPE bool DEFAULT false;
            
            DEFINE INDEX IF NOT EXISTS conversation_id_idx ON conversation FIELDS id UNIQUE;
            
            -- Chat message table
            DEFINE TABLE IF NOT EXISTS chat_message SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS id ON chat_message TYPE string;
            DEFINE FIELD IF NOT EXISTS conversation_id ON chat_message TYPE string;
            DEFINE FIELD IF NOT EXISTS role ON chat_message TYPE string;
            DEFINE FIELD IF NOT EXISTS content ON chat_message TYPE string;
            DEFINE FIELD IF NOT EXISTS metadata ON chat_message TYPE object;
            DEFINE FIELD IF NOT EXISTS created_at ON chat_message TYPE datetime;
            
            DEFINE INDEX IF NOT EXISTS message_id_idx ON chat_message FIELDS id UNIQUE;
            DEFINE INDEX IF NOT EXISTS message_conversation_idx ON chat_message FIELDS conversation_id;
        "#;

        // Execute schema definition
        self.client.query(schema).await?;

        info!("Database schema initialized successfully");

        Ok(())
    }

    /// =========================================================================
    /// Health check
    /// =========================================================================
    /// Verifies database connectivity.
    /// =========================================================================
    pub async fn health_check(&self) -> Result<bool> {
        let result: Option<String> = self.client
            .query("RETURN 'healthy'")
            .await?
            .take(0)?;

        Ok(result.is_some())
    }
}
