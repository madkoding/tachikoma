//! =============================================================================
//! Database Connection and Operations
//! =============================================================================

use anyhow::Result;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{debug, error, info};

use crate::config::Config;

#[derive(Clone)]
pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    pub async fn connect(config: &Config) -> Result<Self> {
        info!("Connecting to SurrealDB at {}", config.database_url);

        // Remove ws:// prefix if present
        let host = config.database_url
            .replace("ws://", "")
            .replace("wss://", "");

        let client = Surreal::new::<Ws>(&host).await?;

        // Authenticate
        client
            .signin(Root {
                username: &config.database_user,
                password: &config.database_pass,
            })
            .await?;

        // Select namespace and database
        client
            .use_ns(&config.database_ns)
            .use_db(&config.database_db)
            .await?;

        info!(
            "Connected to namespace '{}' database '{}'",
            config.database_ns, config.database_db
        );

        Ok(Self { client })
    }

    pub fn client(&self) -> &Surreal<Client> {
        &self.client
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema...");

        let statements = vec![
            // Memory table
            "DEFINE TABLE memory SCHEMAFULL",
            "DEFINE FIELD content ON memory TYPE string",
            "DEFINE FIELD vector ON memory TYPE array<float>",
            "DEFINE FIELD memory_type ON memory TYPE string",
            "DEFINE FIELD metadata ON memory TYPE object",
            "DEFINE FIELD created_at ON memory TYPE datetime",
            "DEFINE FIELD updated_at ON memory TYPE datetime",
            "DEFINE FIELD access_count ON memory TYPE int DEFAULT 0",
            "DEFINE FIELD importance_score ON memory TYPE float DEFAULT 0.5",
            // Indexes
            "DEFINE INDEX memory_type_idx ON memory FIELDS memory_type",
            "DEFINE INDEX memory_created_idx ON memory FIELDS created_at",
            "DEFINE INDEX memory_importance_idx ON memory FIELDS importance_score",
            // Relation tables for graph
            "DEFINE TABLE related_to SCHEMAFULL",
            "DEFINE FIELD in ON related_to TYPE record<memory>",
            "DEFINE FIELD out ON related_to TYPE record<memory>",
            "DEFINE FIELD relation_type ON related_to TYPE string",
            "DEFINE FIELD confidence ON related_to TYPE float DEFAULT 1.0",
            "DEFINE FIELD created_at ON related_to TYPE datetime",
            "DEFINE FIELD metadata ON related_to TYPE object",
        ];

        for stmt in statements {
            match self.client.query(stmt).await {
                Ok(mut response) => {
                    let errors: Vec<surrealdb::Error> = response.take_errors().into_values().collect();
                    if !errors.is_empty() {
                        for err in &errors {
                            debug!("Schema statement note: {}", err);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to execute schema statement: {}", e);
                }
            }
        }

        info!("Database schema initialized");
        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        let result: Option<String> = self.client
            .query("RETURN 'healthy'")
            .await?
            .take(0)?;
        Ok(result.is_some())
    }
}
