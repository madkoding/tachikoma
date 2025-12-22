//! =============================================================================
//! Search Provider Port - Simplified
//! =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

/// =============================================================================
/// SearchProvider - Interface for web search
/// =============================================================================
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Perform a web search
    async fn search(&self, query: &str, options: Option<SearchOptions>) -> Result<SearchResults, DomainError>;

    /// Check if provider is healthy
    async fn is_healthy(&self) -> bool;
}

/// Search options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    pub max_results: Option<usize>,
    pub language: Option<String>,
}

impl SearchOptions {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            max_results: Some(limit),
            language: None,
        }
    }
}

/// Search results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub total_results: Option<usize>,
}

impl SearchResults {
    pub fn as_context(&self, max_results: usize) -> String {
        let mut context = format!("Web search results for '{}':\n\n", self.query);
        for (i, result) in self.results.iter().take(max_results).enumerate() {
            context.push_str(&format!(
                "{}. {}\n   URL: {}\n   {}\n\n",
                i + 1, result.title, result.url, result.snippet
            ));
        }
        context
    }
}

/// Single search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub engine: Option<String>,
}
