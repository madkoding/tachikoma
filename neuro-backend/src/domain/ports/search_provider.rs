//! =============================================================================
//! Search Provider Port
//! =============================================================================
//! Defines the abstract interface for web search operations.
//! This port is implemented by the Searxng adapter in the infrastructure layer.
//! =============================================================================

use async_trait::async_trait;

use crate::domain::errors::DomainError;

/// =============================================================================
/// SearchProvider - Abstract interface for web search
/// =============================================================================
/// Defines operations for searching the web and retrieving information.
/// Used by the Agent to gather external knowledge.
/// 
/// # Responsibilities
/// 
/// * Web search with result ranking
/// * Result filtering and categorization
/// * Content extraction from URLs
/// 
/// # Implementation Notes
/// 
/// Implementations should handle:
/// - Rate limiting and caching
/// - Result deduplication
/// - Safe content filtering
/// =============================================================================
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// =========================================================================
    /// Search the web
    /// =========================================================================
    /// Performs a web search and returns ranked results.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The search query string
    /// * `options` - Optional search configuration
    /// 
    /// # Returns
    /// 
    /// * `Ok(SearchResults)` - The search results
    /// * `Err(DomainError)` - If search fails
    /// 
    /// # Errors
    /// 
    /// * `DomainError::SearchError` - Search engine unavailable
    /// * `DomainError::RateLimited` - Too many requests
    /// =========================================================================
    async fn search(
        &self,
        query: &str,
        options: Option<SearchOptions>,
    ) -> Result<SearchResults, DomainError>;

    /// =========================================================================
    /// Search with specific engines
    /// =========================================================================
    /// Performs a search using only the specified search engines.
    /// 
    /// # Arguments
    /// 
    /// * `query` - The search query string
    /// * `engines` - List of engines to use (e.g., ["google", "duckduckgo"])
    /// * `options` - Optional search configuration
    /// 
    /// # Returns
    /// 
    /// * `Ok(SearchResults)` - The search results
    /// * `Err(DomainError)` - If search fails
    /// =========================================================================
    async fn search_with_engines(
        &self,
        query: &str,
        engines: &[String],
        options: Option<SearchOptions>,
    ) -> Result<SearchResults, DomainError>;

    /// =========================================================================
    /// Fetch and extract content from a URL
    /// =========================================================================
    /// Retrieves the content of a web page and extracts the main text.
    /// 
    /// # Arguments
    /// 
    /// * `url` - The URL to fetch
    /// 
    /// # Returns
    /// 
    /// * `Ok(WebContent)` - Extracted content
    /// * `Err(DomainError)` - If fetch fails
    /// =========================================================================
    async fn fetch_url(&self, url: &str) -> Result<WebContent, DomainError>;

    /// =========================================================================
    /// Check search provider health
    /// =========================================================================
    /// Verifies that the search provider is operational.
    /// 
    /// # Returns
    /// 
    /// `true` if the provider is healthy, `false` otherwise
    /// =========================================================================
    async fn is_healthy(&self) -> bool;
}

/// =============================================================================
/// SearchOptions - Configuration for search queries
/// =============================================================================
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub max_results: Option<usize>,

    /// Language filter (ISO 639-1 code)
    pub language: Option<String>,

    /// Time range filter
    pub time_range: Option<TimeRange>,

    /// Safe search level
    pub safe_search: Option<SafeSearchLevel>,

    /// Categories to search (general, images, news, etc.)
    pub categories: Option<Vec<String>>,

    /// Page number for pagination
    pub page: Option<usize>,
}

impl SearchOptions {
    /// Create default options with a result limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            max_results: Some(limit),
            ..Default::default()
        }
    }

    /// Create options for code/technical searches
    pub fn for_code() -> Self {
        Self {
            max_results: Some(10),
            categories: Some(vec!["it".to_string()]),
            ..Default::default()
        }
    }

    /// Create options for news searches
    pub fn for_news() -> Self {
        Self {
            max_results: Some(10),
            categories: Some(vec!["news".to_string()]),
            time_range: Some(TimeRange::Week),
            ..Default::default()
        }
    }
}

/// =============================================================================
/// TimeRange - Time filter for search results
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TimeRange {
    /// Last 24 hours
    Day,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// Last 365 days
    Year,
}

impl TimeRange {
    /// Convert to Searxng time_range parameter
    pub fn to_param(&self) -> &'static str {
        match self {
            TimeRange::Day => "day",
            TimeRange::Week => "week",
            TimeRange::Month => "month",
            TimeRange::Year => "year",
        }
    }
}

/// =============================================================================
/// SafeSearchLevel - Safe search filtering level
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SafeSearchLevel {
    /// No filtering
    Off,
    /// Moderate filtering
    Moderate,
    /// Strict filtering
    Strict,
}

impl Default for SafeSearchLevel {
    fn default() -> Self {
        Self::Moderate
    }
}

impl SafeSearchLevel {
    /// Convert to Searxng safesearch parameter
    pub fn to_param(&self) -> u8 {
        match self {
            SafeSearchLevel::Off => 0,
            SafeSearchLevel::Moderate => 1,
            SafeSearchLevel::Strict => 2,
        }
    }
}

/// =============================================================================
/// SearchResults - Container for search results
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResults {
    /// The search query used
    pub query: String,

    /// List of search result items
    pub results: Vec<SearchResultItem>,

    /// Total number of results available (if known)
    pub total_results: Option<usize>,

    /// Search duration in milliseconds
    pub search_time_ms: u64,

    /// Suggestions for related queries
    pub suggestions: Vec<String>,

    /// Corrections for typos in query
    pub corrections: Vec<String>,
}

impl SearchResults {
    /// Check if results are empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the top N results
    pub fn top(&self, n: usize) -> Vec<&SearchResultItem> {
        self.results.iter().take(n).collect()
    }

    /// Format results as a text summary for LLM context
    pub fn as_context(&self, max_results: usize) -> String {
        let mut context = format!("Web search results for '{}':\n\n", self.query);

        for (i, result) in self.results.iter().take(max_results).enumerate() {
            context.push_str(&format!(
                "{}. {}\n   URL: {}\n   {}\n\n",
                i + 1,
                result.title,
                result.url,
                result.snippet
            ));
        }

        context
    }
}

/// =============================================================================
/// SearchResultItem - A single search result
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResultItem {
    /// Title of the result
    pub title: String,

    /// URL of the result
    pub url: String,

    /// Snippet/description of the result
    pub snippet: String,

    /// Search engine that provided this result
    pub engine: Option<String>,

    /// Score/ranking from the search engine
    pub score: Option<f64>,

    /// Category of the result
    pub category: Option<String>,

    /// Thumbnail URL (for image results)
    pub thumbnail: Option<String>,

    /// Publication date (for news results)
    pub published_date: Option<String>,
}

impl SearchResultItem {
    /// Format as a concise string
    pub fn as_brief(&self) -> String {
        format!("{}: {}", self.title, self.snippet)
    }
}

/// =============================================================================
/// WebContent - Extracted content from a web page
/// =============================================================================
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebContent {
    /// The URL that was fetched
    pub url: String,

    /// Page title
    pub title: Option<String>,

    /// Main text content (cleaned)
    pub content: String,

    /// Word count of the content
    pub word_count: usize,

    /// Language of the content (detected)
    pub language: Option<String>,

    /// Whether the fetch was successful
    pub success: bool,

    /// Error message if fetch failed
    pub error: Option<String>,
}

impl WebContent {
    /// Create a successful content result
    pub fn success(url: String, title: Option<String>, content: String) -> Self {
        let word_count = content.split_whitespace().count();
        Self {
            url,
            title,
            content,
            word_count,
            language: None,
            success: true,
            error: None,
        }
    }

    /// Create a failed content result
    pub fn error(url: String, error: String) -> Self {
        Self {
            url,
            title: None,
            content: String::new(),
            word_count: 0,
            language: None,
            success: false,
            error: Some(error),
        }
    }

    /// Truncate content to a maximum length
    pub fn truncated(&self, max_chars: usize) -> String {
        if self.content.len() <= max_chars {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..max_chars])
        }
    }
}
