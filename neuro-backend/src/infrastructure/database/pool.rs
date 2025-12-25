//! =============================================================================
//! Database Connection Pool
//! =============================================================================
//! Manages SurrealDB connections and initialization.
//! =============================================================================

use anyhow::Result;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{info, debug, error};

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

        info!(
            "🗄️ Selected namespace '{}' and database '{}'",
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

        // Define schema statements separately for better error handling
        let statements = vec![
            // =====================================================================
            // Memory tables
            // =====================================================================
            "DEFINE TABLE memory SCHEMAFULL",
            "DEFINE FIELD content ON memory TYPE string",
            "DEFINE FIELD vector ON memory TYPE array<float>",
            "DEFINE FIELD memory_type ON memory TYPE string",
            "DEFINE FIELD metadata ON memory TYPE object",
            "DEFINE FIELD created_at ON memory TYPE datetime",
            "DEFINE FIELD updated_at ON memory TYPE datetime",
            "DEFINE FIELD access_count ON memory TYPE int DEFAULT 0",
            "DEFINE FIELD importance_score ON memory TYPE float DEFAULT 0.5",
            // Memory indexes
            "DEFINE INDEX memory_type_idx ON memory FIELDS memory_type",
            "DEFINE INDEX memory_created_idx ON memory FIELDS created_at",
            "DEFINE INDEX memory_importance_idx ON memory FIELDS importance_score",
            // Relation tables
            "DEFINE TABLE related_to SCHEMAFULL",
            "DEFINE FIELD in ON related_to TYPE record<memory>",
            "DEFINE FIELD out ON related_to TYPE record<memory>",
            "DEFINE FIELD confidence ON related_to TYPE float DEFAULT 1.0",
            "DEFINE FIELD created_at ON related_to TYPE datetime",
            "DEFINE FIELD metadata ON related_to TYPE object",
            
            // =====================================================================
            // Chat/Conversation tables
            // =====================================================================
            "DEFINE TABLE conversation SCHEMAFULL",
            "DEFINE FIELD title ON conversation TYPE option<string>",
            "DEFINE FIELD created_at ON conversation TYPE datetime",
            "DEFINE FIELD updated_at ON conversation TYPE datetime",
            "DEFINE FIELD archived ON conversation TYPE bool DEFAULT false",
            // Chat message table
            "DEFINE TABLE chat_message SCHEMAFULL",
            "DEFINE FIELD conversation_id ON chat_message TYPE string",
            "DEFINE FIELD role ON chat_message TYPE string",
            "DEFINE FIELD content ON chat_message TYPE string",
            "DEFINE FIELD metadata ON chat_message TYPE object",
            "DEFINE FIELD created_at ON chat_message TYPE datetime",
            "DEFINE INDEX message_conversation_idx ON chat_message FIELDS conversation_id",
            
            // =====================================================================
            // Checklist tables
            // =====================================================================
            "DEFINE TABLE checklist SCHEMAFULL",
            "DEFINE FIELD title ON TABLE checklist TYPE string",
            "DEFINE FIELD description ON TABLE checklist TYPE option<string>",
            "DEFINE FIELD priority ON TABLE checklist TYPE int DEFAULT 3",
            "DEFINE FIELD due_date ON TABLE checklist TYPE option<datetime>",
            "DEFINE FIELD notification_interval ON TABLE checklist TYPE option<int>",
            "DEFINE FIELD last_reminded ON TABLE checklist TYPE option<datetime>",
            "DEFINE FIELD is_archived ON TABLE checklist TYPE bool DEFAULT false",
            "DEFINE FIELD created_at ON TABLE checklist TYPE datetime DEFAULT time::now()",
            "DEFINE FIELD updated_at ON TABLE checklist TYPE datetime DEFAULT time::now()",
            // Checklist item table
            "DEFINE TABLE checklist_item SCHEMAFULL",
            "DEFINE FIELD checklist_id ON TABLE checklist_item TYPE string",
            "DEFINE FIELD content ON TABLE checklist_item TYPE string",
            "DEFINE FIELD is_completed ON TABLE checklist_item TYPE bool DEFAULT false",
            "DEFINE FIELD completed_at ON TABLE checklist_item TYPE option<datetime>",
            "DEFINE FIELD item_order ON TABLE checklist_item TYPE int DEFAULT 0",
            "DEFINE FIELD created_at ON TABLE checklist_item TYPE datetime DEFAULT time::now()",
            "DEFINE INDEX idx_item_checklist ON TABLE checklist_item COLUMNS checklist_id",
            
            // =====================================================================
            // Music tables
            // =====================================================================
            "DEFINE TABLE playlist SCHEMAFULL",
            "DEFINE FIELD name ON TABLE playlist TYPE string",
            "DEFINE FIELD description ON TABLE playlist TYPE option<string>",
            "DEFINE FIELD cover_url ON TABLE playlist TYPE option<string>",
            "DEFINE FIELD is_suggestions ON TABLE playlist TYPE bool DEFAULT false",
            "DEFINE FIELD shuffle ON TABLE playlist TYPE bool DEFAULT false",
            "DEFINE FIELD repeat_mode ON TABLE playlist TYPE string DEFAULT 'off'",
            "DEFINE FIELD song_count ON TABLE playlist TYPE int DEFAULT 0",
            "DEFINE FIELD total_duration ON TABLE playlist TYPE int DEFAULT 0",
            "DEFINE FIELD created_at ON TABLE playlist TYPE datetime DEFAULT time::now()",
            "DEFINE FIELD updated_at ON TABLE playlist TYPE datetime DEFAULT time::now()",
            // Song table
            "DEFINE TABLE song SCHEMAFULL",
            "DEFINE FIELD playlist_id ON TABLE song TYPE string",
            "DEFINE FIELD youtube_id ON TABLE song TYPE string",
            "DEFINE FIELD youtube_url ON TABLE song TYPE string",
            "DEFINE FIELD title ON TABLE song TYPE string",
            "DEFINE FIELD artist ON TABLE song TYPE option<string>",
            "DEFINE FIELD album ON TABLE song TYPE option<string>",
            "DEFINE FIELD duration ON TABLE song TYPE int DEFAULT 0",
            "DEFINE FIELD cover_url ON TABLE song TYPE option<string>",
            "DEFINE FIELD thumbnail_url ON TABLE song TYPE option<string>",
            "DEFINE FIELD song_order ON TABLE song TYPE int DEFAULT 0",
            "DEFINE FIELD play_count ON TABLE song TYPE int DEFAULT 0",
            "DEFINE FIELD last_played ON TABLE song TYPE option<datetime>",
            "DEFINE FIELD created_at ON TABLE song TYPE datetime DEFAULT time::now()",
            "DEFINE INDEX idx_song_playlist ON TABLE song COLUMNS playlist_id",
            // Listening history
            "DEFINE TABLE listening_history SCHEMAFULL",
            "DEFINE FIELD song_id ON TABLE listening_history TYPE string",
            "DEFINE FIELD youtube_id ON TABLE listening_history TYPE string",
            "DEFINE FIELD title ON TABLE listening_history TYPE string",
            "DEFINE FIELD artist ON TABLE listening_history TYPE option<string>",
            "DEFINE FIELD listened_at ON TABLE listening_history TYPE datetime DEFAULT time::now()",
            // Equalizer settings
            "DEFINE TABLE equalizer_settings SCHEMAFULL",
            "DEFINE FIELD enabled ON TABLE equalizer_settings TYPE bool DEFAULT true",
            "DEFINE FIELD preset ON TABLE equalizer_settings TYPE option<string>",
            "DEFINE FIELD bands ON TABLE equalizer_settings TYPE array DEFAULT [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]",
        ];

        for (i, stmt) in statements.iter().enumerate() {
            debug!("📝 Executing schema statement {}/{}: {}", i + 1, statements.len(), stmt);
            match self.client.query(*stmt).await {
                Ok(mut response) => {
                    let errors: Vec<surrealdb::Error> = response.take_errors().into_values().collect();
                    if !errors.is_empty() {
                        for err in &errors {
                            error!("❌ Schema error for '{}': {}", stmt, err);
                        }
                    } else {
                        debug!("✅ Success: {}", stmt);
                    }
                }
                Err(e) => {
                    error!("❌ Failed to execute '{}': {}", stmt, e);
                }
            }
        }

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
