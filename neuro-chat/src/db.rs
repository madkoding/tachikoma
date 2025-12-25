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

        let host = config.database_url
            .replace("ws://", "")
            .replace("wss://", "");

        let client = Surreal::new::<Ws>(&host).await?;

        client
            .signin(Root {
                username: &config.database_user,
                password: &config.database_pass,
            })
            .await?;

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
            // Conversation table
            "DEFINE TABLE conversation SCHEMAFULL",
            "DEFINE FIELD title ON conversation TYPE option<string>",
            "DEFINE FIELD created_at ON conversation TYPE datetime",
            "DEFINE FIELD updated_at ON conversation TYPE datetime",
            "DEFINE FIELD archived ON conversation TYPE bool DEFAULT false",
            "DEFINE FIELD message_count ON conversation TYPE int DEFAULT 0",
            // Chat message table
            "DEFINE TABLE chat_message SCHEMAFULL",
            "DEFINE FIELD conversation_id ON chat_message TYPE string",
            "DEFINE FIELD role ON chat_message TYPE string",
            "DEFINE FIELD content ON chat_message TYPE string",
            "DEFINE FIELD model ON chat_message TYPE option<string>",
            "DEFINE FIELD tokens ON chat_message TYPE option<int>",
            "DEFINE FIELD metadata ON chat_message TYPE object",
            "DEFINE FIELD created_at ON chat_message TYPE datetime",
            "DEFINE INDEX message_conversation_idx ON chat_message FIELDS conversation_id",
            "DEFINE INDEX message_created_idx ON chat_message FIELDS created_at",
        ];

        for stmt in statements {
            match self.client.query(stmt).await {
                Ok(mut response) => {
                    let errors: Vec<surrealdb::Error> = response.take_errors().into_values().collect();
                    if !errors.is_empty() {
                        for err in &errors {
                            debug!("Schema note: {}", err);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to execute schema: {}", e);
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
