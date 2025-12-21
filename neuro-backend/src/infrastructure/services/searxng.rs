//! =============================================================================
//! Searxng Client - Search Provider Implementation
//! =============================================================================
//! Implements the SearchProvider port using Searxng metasearch engine.
//! Provides privacy-respecting web search for AI agents.
//! =============================================================================

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, warn};

use crate::domain::{
    errors::DomainError,
    ports::search_provider::{SearchProvider, SearchQuery, SearchResult, SearchResponse},
};
use crate::infrastructure::config::SearxngConfig;

/// =============================================================================
/// SearxngClient - HTTP client for Searxng API
/// =============================================================================
/// Provides web search through a self-hosted Searxng instance.
/// Supports multiple search categories and result filtering.
/// =============================================================================
#[derive(Clone)]
pub struct SearxngClient {
    /// HTTP client for API requests
    client: Client,
    /// Configuration settings
    config: SearxngConfig,
}

/// Searxng API response format
#[derive(Debug, Deserialize)]
struct SearxngResponse {
    query: String,
    results: Vec<SearxngResult>,
    #[serde(default)]
    number_of_results: u64,
    #[serde(default)]
    suggestions: Vec<String>,
    #[serde(default)]
    infoboxes: Vec<SearxngInfobox>,
}

/// Individual search result from Searxng
#[derive(Debug, Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    engine: String,
    #[serde(default)]
    engines: Vec<String>,
    #[serde(default)]
    score: f64,
    #[serde(default)]
    category: String,
    #[serde(default)]
    publishedDate: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
}

/// Infobox data from Searxng
#[derive(Debug, Deserialize)]
struct SearxngInfobox {
    #[serde(default)]
    infobox: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    urls: Vec<SearxngInfoboxUrl>,
}

#[derive(Debug, Deserialize)]
struct SearxngInfoboxUrl {
    title: String,
    url: String,
}

/// Request parameters for Searxng
#[derive(Debug, Serialize)]
struct SearxngRequest {
    q: String,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    engines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pageno: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safesearch: Option<u8>,
}

impl SearxngClient {
    /// =========================================================================
    /// Create a new SearxngClient
    /// =========================================================================
    pub fn new(config: SearxngConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Build the search URL
    fn search_url(&self) -> String {
        format!("{}/search", self.config.host)
    }

    /// Convert category string to Searxng format
    fn format_categories(categories: &[String]) -> String {
        categories.join(",")
    }

    /// Convert time range to Searxng format
    fn format_time_range(time_range: &Option<String>) -> Option<String> {
        time_range.as_ref().map(|tr| {
            match tr.to_lowercase().as_str() {
                "day" | "24h" => "day",
                "week" | "7d" => "week",
                "month" | "30d" => "month",
                "year" | "365d" => "year",
                _ => "all",
            }.to_string()
        })
    }
}

#[async_trait]
impl SearchProvider for SearxngClient {
    /// =========================================================================
    /// Perform a web search
    /// =========================================================================
    #[instrument(skip(self, query), fields(q = %query.query))]
    async fn search(&self, query: SearchQuery) -> Result<SearchResponse, DomainError> {
        let categories = if query.categories.is_empty() {
            None
        } else {
            Some(Self::format_categories(&query.categories))
        };

        let request_params = SearxngRequest {
            q: query.query.clone(),
            format: "json".to_string(),
            categories,
            engines: None,
            language: query.language.clone(),
            pageno: query.page,
            time_range: Self::format_time_range(&query.time_range),
            safesearch: query.safe_search.map(|s| if s { 1 } else { 0 }),
        };

        let url = self.search_url();
        debug!(url = %url, query = %query.query, "Performing search");

        let response = self.client
            .get(&url)
            .query(&request_params)
            .send()
            .await
            .map_err(|e| DomainError::SearchError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DomainError::SearchError(format!(
                "Searxng API error: {} - {}",
                status, body
            )));
        }

        let searxng_response: SearxngResponse = response.json().await
            .map_err(|e| DomainError::SearchError(format!("Failed to parse response: {}", e)))?;

        // Convert to domain types
        let results: Vec<SearchResult> = searxng_response.results
            .into_iter()
            .take(query.limit.unwrap_or(10) as usize)
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.content,
                source: if r.engines.is_empty() {
                    r.engine
                } else {
                    r.engines.join(", ")
                },
                score: Some(r.score),
                published_date: r.publishedDate,
                category: Some(r.category),
                thumbnail_url: r.thumbnail,
            })
            .collect();

        let total_results = searxng_response.number_of_results as usize;

        debug!(
            query = %query.query,
            results = results.len(),
            total = total_results,
            "Search completed"
        );

        Ok(SearchResponse {
            query: searxng_response.query,
            results,
            total_results,
            suggestions: searxng_response.suggestions,
            page: query.page.unwrap_or(1),
        })
    }

    /// =========================================================================
    /// Search for images
    /// =========================================================================
    #[instrument(skip(self))]
    async fn search_images(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, DomainError> {
        let search_query = SearchQuery {
            query: query.to_string(),
            categories: vec!["images".to_string()],
            language: None,
            limit: Some(limit as u32),
            page: Some(1),
            time_range: None,
            safe_search: Some(true),
        };

        let response = self.search(search_query).await?;
        Ok(response.results)
    }

    /// =========================================================================
    /// Search for news
    /// =========================================================================
    #[instrument(skip(self))]
    async fn search_news(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, DomainError> {
        let search_query = SearchQuery {
            query: query.to_string(),
            categories: vec!["news".to_string()],
            language: None,
            limit: Some(limit as u32),
            page: Some(1),
            time_range: Some("week".to_string()),
            safe_search: Some(true),
        };

        let response = self.search(search_query).await?;
        Ok(response.results)
    }

    /// =========================================================================
    /// Search for code/development resources
    /// =========================================================================
    #[instrument(skip(self))]
    async fn search_code(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, DomainError> {
        // Include code-related terms in search
        let code_query = format!("{} site:github.com OR site:stackoverflow.com OR site:docs.rs", query);
        
        let search_query = SearchQuery {
            query: code_query,
            categories: vec!["it".to_string()],
            language: None,
            limit: Some(limit as u32),
            page: Some(1),
            time_range: None,
            safe_search: Some(true),
        };

        let response = self.search(search_query).await?;
        Ok(response.results)
    }

    /// =========================================================================
    /// Get available search categories
    /// =========================================================================
    async fn get_categories(&self) -> Result<Vec<String>, DomainError> {
        // Searxng standard categories
        Ok(vec![
            "general".to_string(),
            "images".to_string(),
            "videos".to_string(),
            "news".to_string(),
            "map".to_string(),
            "music".to_string(),
            "it".to_string(),
            "science".to_string(),
            "files".to_string(),
            "social media".to_string(),
        ])
    }

    /// =========================================================================
    /// Check search provider health
    /// =========================================================================
    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<bool, DomainError> {
        let url = format!("{}/healthz", self.config.host);
        
        let result = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match result {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                // Try alternate health check via simple search
                warn!(error = %e, "Health check endpoint failed, trying search");
                
                let search_result = self.search(SearchQuery {
                    query: "test".to_string(),
                    categories: vec![],
                    language: None,
                    limit: Some(1),
                    page: Some(1),
                    time_range: None,
                    safe_search: None,
                }).await;

                Ok(search_result.is_ok())
            }
        }
    }
}
