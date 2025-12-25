use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;

use crate::config::Config;

pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: config.backend_url.clone(),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        debug!("GET {}", url);
        
        self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        debug!("POST {}", url);
        
        self.client
            .post(&url)
            .json(body)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        debug!("PUT {}", url);
        
        self.client
            .put(&url)
            .json(body)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn delete(&self, path: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        debug!("DELETE {}", url);
        
        self.client
            .delete(&url)
            .send()
            .await?;
        
        Ok(())
    }
}
