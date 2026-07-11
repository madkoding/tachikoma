//! =============================================================================
//! Searxng Client - Web Search Integration
//! =============================================================================

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// Searxng search client
pub struct SearxngClient {
    client: Client,
    base_url: String,
}

/// Search request parameters
#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pageno: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<String>,
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

/// Search response from Searxng
#[derive(Debug, Deserialize)]
pub struct SearxngResponse {
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub number_of_results: u64,
    #[serde(default)]
    pub suggestions: Vec<String>,
    #[serde(default)]
    pub answers: Vec<String>,
}

/// API response format
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub suggestions: Vec<String>,
    pub answers: Vec<String>,
}

impl SearxngClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Perform a web search
    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse, String> {
        debug!("Searching Searxng for: {}", request.query);

        let mut url = format!("{}/search", self.base_url);
        
        // Build query parameters
        let mut params = vec![
            ("q", request.query.clone()),
            ("format", "json".to_string()),
        ];

        if let Some(ref categories) = request.categories {
            params.push(("categories", categories.join(",")));
        }
        if let Some(ref engines) = request.engines {
            params.push(("engines", engines.join(",")));
        }
        if let Some(ref lang) = request.language {
            params.push(("language", lang.clone()));
        }
        if let Some(page) = request.pageno {
            params.push(("pageno", page.to_string()));
        }
        if let Some(ref time_range) = request.time_range {
            params.push(("time_range", time_range.clone()));
        }

        // Add query string
        let query_string = params.iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        url = format!("{}?{}", url, query_string);

        debug!("Searxng URL: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("Searxng request failed: {}", e);
                format!("Search request failed: {}", e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Searxng returned error {}: {}", status, body);
            return Err(format!("Search failed with status {}", status));
        }

        let searxng_response: SearxngResponse = response
            .json()
            .await
            .map_err(|e| {
                error!("Failed to parse Searxng response: {}", e);
                format!("Failed to parse search response: {}", e)
            })?;

        debug!("Got {} results from Searxng", searxng_response.results.len());

        Ok(SearchResponse {
            results: searxng_response.results,
            total: searxng_response.number_of_results,
            suggestions: searxng_response.suggestions,
            answers: searxng_response.answers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_result_serde_roundtrip() {
        let r = SearchResult {
            title: "Rust".to_string(),
            url: "https://rust-lang.org".to_string(),
            content: Some("desc".to_string()),
            engine: Some("google".to_string()),
            score: Some(1.5),
            category: Some("it".to_string()),
            thumbnail: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Rust");
        assert_eq!(back.score, Some(1.5));
    }

    #[test]
    fn search_result_minimal_defaults() {
        let json = r#"{"title":"T","url":"U"}"#;
        let r: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(r.title, "T");
        assert!(r.content.is_none());
        assert!(r.engine.is_none());
        assert!(r.score.is_none());
    }

    #[test]
    fn searxng_response_deserialize_defaults() {
        let json = r#"{"results":[]}"#;
        let resp: SearxngResponse = serde_json::from_str(json).unwrap();
        assert!(resp.results.is_empty());
        assert_eq!(resp.number_of_results, 0);
        assert!(resp.suggestions.is_empty());
        assert!(resp.answers.is_empty());
    }

    #[test]
    fn search_response_serialize() {
        let resp = SearchResponse {
            results: vec![],
            total: 42,
            suggestions: vec!["rust".to_string()],
            answers: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":42"));
        assert!(json.contains("\"suggestions\":[\"rust\"]"));
    }
}
