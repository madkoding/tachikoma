use anyhow::Result;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    pub async fn connect(url: &str, user: &str, pass: &str, ns: &str, db: &str) -> Result<Self> {
        info!("Connecting to SurrealDB at {}", url);

        let host = url.replace("ws://", "").replace("wss://", "");

        let client = Surreal::new::<Ws>(&host).await?;

        client
            .signin(Root {
                username: user,
                password: pass,
            })
            .await?;

        client.use_ns(ns).use_db(db).await?;

        info!("Connected to namespace '{}' database '{}'", ns, db);

        Ok(Self { client })
    }

    pub fn client(&self) -> &Surreal<Client> {
        &self.client
    }

    pub async fn initialize_schema(&self, statements: &[&str]) -> Result<()> {
        info!("Initializing database schema...");

        for stmt in statements {
            match self.client.query(*stmt).await {
                Ok(mut response) => {
                    let errors: Vec<surrealdb::Error> =
                        response.take_errors().into_values().collect();
                    for err in &errors {
                        debug!("Schema note: {}", err);
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
        let result: Option<String> = self.client.query("RETURN 'healthy'").await?.take(0)?;
        Ok(result.is_some())
    }
}