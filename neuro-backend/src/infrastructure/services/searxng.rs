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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(url: String) -> SearxngConfig {
        SearxngConfig {
            url,
            timeout_secs: 5,
            max_results: 5,
        }
    }

    #[tokio::test]
    async fn test_search_returns_results() {
        let server = MockServer::start().await;
        let config = test_config(server.uri());
        let client = SearxngClient::new(config);

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "rust async"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "query": "rust async",
                "results": [
                    {"title": "Rust Async", "url": "https://rust-lang.org", "content": "Async in Rust", "engine": "google"},
                    {"title": "Tokio", "url": "https://tokio.rs", "content": "Tokio runtime", "engine": "duckduckgo"}
                ],
                "number_of_results": 2
            })))
            .mount(&server)
            .await;

        let results = client.search("rust async", None).await.unwrap();
        assert_eq!(results.query, "rust async");
        assert_eq!(results.results.len(), 2);
        assert_eq!(results.results[0].title, "Rust Async");
    }

    #[tokio::test]
    async fn test_search_empty_results() {
        let server = MockServer::start().await;
        let config = test_config(server.uri());
        let client = SearxngClient::new(config);

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "query": "nonexistent",
                "results": [],
                "number_of_results": 0
            })))
            .mount(&server)
            .await;

        let results = client.search("nonexistent", None).await.unwrap();
        assert_eq!(results.results.len(), 0);
        assert_eq!(results.total_results, Some(0));
    }

    #[tokio::test]
    async fn test_search_http_error() {
        let server = MockServer::start().await;
        let config = test_config(server.uri());
        let client = SearxngClient::new(config);

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let result = client.search("test", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_max_results_truncation() {
        let server = MockServer::start().await;
        let config = test_config(server.uri());
        let client = SearxngClient::new(config);

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "query": "test",
                "results": [
                    {"title": "1", "url": "u1", "content": "c1", "engine": "e"},
                    {"title": "2", "url": "u2", "content": "c2", "engine": "e"},
                    {"title": "3", "url": "u3", "content": "c3", "engine": "e"}
                ],
                "number_of_results": 3
            })))
            .mount(&server)
            .await;

        let results = client.search("test", Some(SearchOptions::with_limit(2))).await.unwrap();
        assert_eq!(results.results.len(), 2);
    }

    #[tokio::test]
    async fn test_is_healthy_ok() {
        let server = MockServer::start().await;
        let config = test_config(server.uri());
        let client = SearxngClient::new(config);

        Mock::given(method("GET"))
            .and(path("/healthz"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        assert!(client.is_healthy().await);
    }

    #[tokio::test]
    async fn test_is_healthy_down() {
        let config = test_config("http://127.0.0.1:1".to_string());
        let client = SearxngClient::new(config);
        assert!(!client.is_healthy().await);
    }

    #[test]
    fn test_search_results_as_context() {
        let results = SearchResults {
            query: "rust".to_string(),
            results: vec![
                SearchResultItem {
                    title: "Rust".to_string(),
                    url: "https://rust-lang.org".to_string(),
                    snippet: "A language".to_string(),
                    engine: None,
                },
            ],
            total_results: Some(1),
        };
        let context = results.as_context(5);
        assert!(context.contains("Rust"));
        assert!(context.contains("rust-lang.org"));
    }
}
