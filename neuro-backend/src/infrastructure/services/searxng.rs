//! =============================================================================
//! Searxng Client - Search Provider Implementation
//! =============================================================================

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, instrument, warn};

use crate::domain::{
    errors::DomainError,
    ports::search_provider::{SearchProvider, SearchOptions, SearchResults, SearchResultItem},
};
use crate::infrastructure::config::SearxngConfig;

/// SearxngClient - HTTP client for Searxng API
#[derive(Clone)]
pub struct SearxngClient {
    client: Client,
    config: SearxngConfig,
}

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    query: String,
    results: Vec<SearxngResult>,
    #[serde(default)]
    number_of_results: u64,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    engine: String,
}

impl SearxngClient {
    pub fn new(config: SearxngConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, config }
    }

    fn search_url(&self) -> String {
        format!("{}/search", self.config.url)
    }
}

#[async_trait]
impl SearchProvider for SearxngClient {
    #[instrument(skip(self))]
    async fn search(&self, query: &str, options: Option<SearchOptions>) -> Result<SearchResults, DomainError> {
        let opts = options.unwrap_or_default();
        let max_results = opts.max_results.unwrap_or(self.config.max_results);

        let params: Vec<(&str, String)> = vec![
            ("q", query.to_string()),
            ("format", "json".to_string()),
        ];

        let url = self.search_url();
        debug!(url = %url, query = %query, "Performing search");

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| DomainError::search(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body: String = response.text().await.unwrap_or_default();
            return Err(DomainError::search(format!("Searxng API error: {} - {}", status, body)));
        }

        let searxng_response: SearxngResponse = response.json().await
            .map_err(|e| DomainError::search(format!("Failed to parse response: {}", e)))?;

        let results: Vec<SearchResultItem> = searxng_response.results
            .into_iter()
            .take(max_results)
            .map(|r| SearchResultItem {
                title: r.title,
                url: r.url,
                snippet: r.content,
                engine: Some(r.engine),
            })
            .collect();

        debug!(count = results.len(), "Search completed");

        Ok(SearchResults {
            query: searxng_response.query,
            results,
            total_results: Some(searxng_response.number_of_results as usize),
        })
    }

    #[instrument(skip(self))]
    async fn is_healthy(&self) -> bool {
        let url = format!("{}/healthz", self.config.url);
        match self.client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
            Ok(response) => response.status().is_success(),
            Err(e) => {
                warn!(error = %e, "Searxng health check failed");
                false
            }
        }
    }
}
