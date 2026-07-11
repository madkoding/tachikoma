use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::error;

pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    pub fn new(backend_url: &str, api_prefix: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/api/data/{}", backend_url.trim_end_matches('/'), api_prefix),
        }
    }

    pub fn from_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    async fn check_error(&self, response: reqwest::Response) -> Result<reqwest::Response> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Backend error {}: {}", status, text);
            return Err(anyhow::anyhow!("Backend error {}: {}", status, text));
        }
        Ok(response)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.get(&url).send().await?;
        let response = self.check_error(response).await?;
        Ok(response.json().await?)
    }

    pub async fn get_opt<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.get(&url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let response = self.check_error(response).await?;
        Ok(Some(response.json().await?))
    }

    pub async fn post<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.post(&url).json(body).send().await?;
        let response = self.check_error(response).await?;
        Ok(response.json().await?)
    }

    pub async fn patch<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Option<T>> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.patch(&url).json(body).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let response = self.check_error(response).await?;
        Ok(Some(response.json().await?))
    }

    pub async fn delete(&self, path: &str) -> Result<bool> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.delete(&url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        Ok(response.status().is_success())
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = self.base_url.replace("/api/data/", "/api/");
        let url = format!("{}/health", url.rsplit_once('/').map(|(b, _)| b).unwrap_or(&url));
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_builds_url() {
        let client = BackendClient::new("http://localhost:3000", "checklists");
        assert!(client.base_url().ends_with("/api/data/checklists"));
    }

    #[test]
    fn test_new_strips_trailing_slash() {
        let client = BackendClient::new("http://localhost:3000/", "music");
        assert_eq!(client.base_url(), "http://localhost:3000/api/data/music");
    }
}