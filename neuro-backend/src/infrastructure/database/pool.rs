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

        // Extract host from URL (remove ws:// prefix if present)
        let host = config.url
            .strip_prefix("ws://")
            .or_else(|| config.url.strip_prefix("wss://"))
            .unwrap_or(&config.url);

        // Connect to SurrealDB
        let client = Surreal::new::<Ws>(host).await?;

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
        // Note: SurrealDB 1.5.x doesn't support IF NOT EXISTS or TYPE RELATION syntax
        let schema = r#"
            -- Memory node table
            DEFINE TABLE memory SCHEMAFULL;
            
            DEFINE FIELD id ON memory TYPE string;
            DEFINE FIELD content ON memory TYPE string;
            DEFINE FIELD vector ON memory TYPE array<float>;
            DEFINE FIELD memory_type ON memory TYPE string;
            DEFINE FIELD metadata ON memory TYPE object;
            DEFINE FIELD created_at ON memory TYPE datetime;
            DEFINE FIELD updated_at ON memory TYPE datetime;
            DEFINE FIELD access_count ON memory TYPE int DEFAULT 0;
            DEFINE FIELD importance_score ON memory TYPE float DEFAULT 0.5;
            
            -- Indexes for efficient queries
            DEFINE INDEX memory_id_idx ON memory FIELDS id UNIQUE;
            DEFINE INDEX memory_type_idx ON memory FIELDS memory_type;
            DEFINE INDEX memory_created_idx ON memory FIELDS created_at;
            DEFINE INDEX memory_importance_idx ON memory FIELDS importance_score;
            
            -- Relation tables (graph edges) - using regular tables with from/to fields
            DEFINE TABLE related_to SCHEMAFULL;
            DEFINE FIELD in ON related_to TYPE record(memory);
            DEFINE FIELD out ON related_to TYPE record(memory);
            DEFINE FIELD confidence ON related_to TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON related_to TYPE datetime;
            DEFINE FIELD metadata ON related_to TYPE object;
            
            DEFINE TABLE causes SCHEMAFULL;
            DEFINE FIELD in ON causes TYPE record(memory);
            DEFINE FIELD out ON causes TYPE record(memory);
            DEFINE FIELD confidence ON causes TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON causes TYPE datetime;
            DEFINE FIELD metadata ON causes TYPE object;
            
            DEFINE TABLE part_of SCHEMAFULL;
            DEFINE FIELD in ON part_of TYPE record(memory);
            DEFINE FIELD out ON part_of TYPE record(memory);
            DEFINE FIELD confidence ON part_of TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON part_of TYPE datetime;
            DEFINE FIELD metadata ON part_of TYPE object;
            
            DEFINE TABLE follows SCHEMAFULL;
            DEFINE FIELD in ON follows TYPE record(memory);
            DEFINE FIELD out ON follows TYPE record(memory);
            DEFINE FIELD confidence ON follows TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON follows TYPE datetime;
            DEFINE FIELD metadata ON follows TYPE object;
            
            DEFINE TABLE contradicts SCHEMAFULL;
            DEFINE FIELD in ON contradicts TYPE record(memory);
            DEFINE FIELD out ON contradicts TYPE record(memory);
            DEFINE FIELD confidence ON contradicts TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON contradicts TYPE datetime;
            DEFINE FIELD metadata ON contradicts TYPE object;
            
            DEFINE TABLE supports SCHEMAFULL;
            DEFINE FIELD in ON supports TYPE record(memory);
            DEFINE FIELD out ON supports TYPE record(memory);
            DEFINE FIELD confidence ON supports TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON supports TYPE datetime;
            DEFINE FIELD metadata ON supports TYPE object;
            
            DEFINE TABLE derived_from SCHEMAFULL;
            DEFINE FIELD in ON derived_from TYPE record(memory);
            DEFINE FIELD out ON derived_from TYPE record(memory);
            DEFINE FIELD confidence ON derived_from TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON derived_from TYPE datetime;
            DEFINE FIELD metadata ON derived_from TYPE object;
            
            DEFINE TABLE same_as SCHEMAFULL;
            DEFINE FIELD in ON same_as TYPE record(memory);
            DEFINE FIELD out ON same_as TYPE record(memory);
            DEFINE FIELD confidence ON same_as TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON same_as TYPE datetime;
            DEFINE FIELD metadata ON same_as TYPE object;
            
            DEFINE TABLE context_of SCHEMAFULL;
            DEFINE FIELD in ON context_of TYPE record(memory);
            DEFINE FIELD out ON context_of TYPE record(memory);
            DEFINE FIELD confidence ON context_of TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON context_of TYPE datetime;
            DEFINE FIELD metadata ON context_of TYPE object;
            
            DEFINE TABLE references_rel SCHEMAFULL;
            DEFINE FIELD in ON references_rel TYPE record(memory);
            DEFINE FIELD out ON references_rel TYPE record(memory);
            DEFINE FIELD confidence ON references_rel TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON references_rel TYPE datetime;
            DEFINE FIELD metadata ON references_rel TYPE object;
            
            DEFINE TABLE supersedes SCHEMAFULL;
            DEFINE FIELD in ON supersedes TYPE record(memory);
            DEFINE FIELD out ON supersedes TYPE record(memory);
            DEFINE FIELD confidence ON supersedes TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON supersedes TYPE datetime;
            DEFINE FIELD metadata ON supersedes TYPE object;
            
            -- Conversation table
            -- Note: Do NOT define FIELD id - SurrealDB handles record IDs automatically
            DEFINE TABLE conversation SCHEMAFULL;
            DEFINE FIELD title ON conversation TYPE option<string>;
            DEFINE FIELD created_at ON conversation TYPE datetime;
            DEFINE FIELD updated_at ON conversation TYPE datetime;
            DEFINE FIELD archived ON conversation TYPE bool DEFAULT false;
            
            -- Chat message table
            -- Note: Do NOT define FIELD id - SurrealDB handles record IDs automatically
            DEFINE TABLE chat_message SCHEMAFULL;
            DEFINE FIELD conversation_id ON chat_message TYPE string;
            DEFINE FIELD role ON chat_message TYPE string;
            DEFINE FIELD content ON chat_message TYPE string;
            DEFINE FIELD metadata ON chat_message TYPE object;
            DEFINE FIELD created_at ON chat_message TYPE datetime;
            
            DEFINE INDEX message_conversation_idx ON chat_message FIELDS conversation_id;
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
