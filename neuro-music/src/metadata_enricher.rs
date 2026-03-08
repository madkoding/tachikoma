//! Metadata enrichment service
//! Fetches accurate song metadata from MusicBrainz, falls back to LLM inference
//! Includes in-memory cache and MusicBrainz rate limiting

use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::config::Config;

/// Enriched metadata for a song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedMetadata {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub source: MetadataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSource {
    MusicBrainz,
    LlmInference,
    Original,
}

pub struct MetadataEnricher {
    client: Client,
    musicbrainz_api: String,
    backend_url: String,
    /// In-memory cache for metadata (key: "title:artist")
    cache: Cache<String, EnrichedMetadata>,
    /// Rate limiter: tracks last MusicBrainz request time
    last_musicbrainz_request: Arc<Mutex<Instant>>,
}

impl MetadataEnricher {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .user_agent("TachikomaOS-Music/1.0 (https://github.com/tachikoma-os)")
            .build()
            .unwrap_or_default();

        // Cache: 10k entries, TTL 7 days
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(86400 * 7))
            .build();

        // Rate limiter: allow immediate first request
        let last_musicbrainz_request = Arc::new(Mutex::new(
            Instant::now() - Duration::from_secs(2)
        ));

        info!("✅ Metadata enricher initialized with cache (10k entries, 7d TTL) and rate limiting (1.1s)");

        Self {
            client,
            musicbrainz_api: config.musicbrainz_api.clone(),
            backend_url: config.backend_url.clone(),
            cache,
            last_musicbrainz_request,
        }
    }

    /// Enrich metadata for a song
    /// Tries cache first, then MusicBrainz, then LLM inference
    pub async fn enrich(&self, title: &str, artist: Option<&str>) -> EnrichedMetadata {
        // Build cache key (normalized)
        let cache_key = format!("{}:{}", 
            title.to_lowercase().trim(), 
            artist.unwrap_or("").to_lowercase().trim()
        );

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            debug!(title = %title, "Cache HIT for metadata");
            return cached;
        }
        debug!(title = %title, "Cache MISS, fetching metadata");

        // First try MusicBrainz
        if let Some(enriched) = self.search_musicbrainz(title, artist).await {
            info!(
                title = %title,
                enriched_title = %enriched.title,
                enriched_artist = ?enriched.artist,
                "Found metadata in MusicBrainz"
            );
            // Store in cache
            self.cache.insert(cache_key, enriched.clone()).await;
            return enriched;
        }

        // Fall back to LLM inference
        if let Some(enriched) = self.infer_with_llm(title, artist).await {
            info!(
                title = %title,
                enriched_title = %enriched.title,
                enriched_artist = ?enriched.artist,
                "Inferred metadata with LLM"
            );
            // Store in cache
            self.cache.insert(cache_key, enriched.clone()).await;
            return enriched;
        }

        // Return original data if all else fails
        info!(title = %title, "Using original metadata");
        let original = EnrichedMetadata {
            title: title.to_string(),
            artist: artist.map(|s| s.to_string()),
            album: None,
            source: MetadataSource::Original,
        };
        // Cache original too to avoid repeated failed lookups
        self.cache.insert(cache_key, original.clone()).await;
        original
    }

    /// Rate limiter for MusicBrainz API (1 request per 1.1 seconds)
    async fn rate_limit_musicbrainz(&self) {
        let mut last_request = self.last_musicbrainz_request.lock().await;
        let elapsed = last_request.elapsed();
        let min_interval = Duration::from_millis(1100); // 1.1 sec to be safe

        if elapsed < min_interval {
            let wait_time = min_interval - elapsed;
            debug!(wait_ms = wait_time.as_millis(), "Rate limiting MusicBrainz request");
            tokio::time::sleep(wait_time).await;
        }

        *last_request = Instant::now();
    }

    /// Search MusicBrainz for recording metadata
    /// Rate-limited to 1 request per 1.1 seconds
    async fn search_musicbrainz(&self, title: &str, artist: Option<&str>) -> Option<EnrichedMetadata> {
        // Apply rate limiting before request
        self.rate_limit_musicbrainz().await;

        // Clean up title for search (remove common YouTube suffixes)
        let clean_title = Self::clean_youtube_title(title);
        
        let query = match artist {
            Some(a) if !a.is_empty() => format!("recording:\"{}\" AND artist:\"{}\"", clean_title, a),
            _ => format!("recording:\"{}\"", clean_title),
        };

        let url = format!(
            "{}/recording?query={}&limit=5&fmt=json",
            self.musicbrainz_api,
            urlencoding::encode(&query)
        );

        debug!(url = %url, "Searching MusicBrainz");

        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "MusicBrainz request failed");
                return None;
            }
        };

        if !response.status().is_success() {
            warn!(status = %response.status(), "MusicBrainz search failed");
            return None;
        }

        let json: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                warn!(error = %e, "Failed to parse MusicBrainz response");
                return None;
            }
        };

        let recordings = json["recordings"].as_array()?;
        
        // Find the best match
        for recording in recordings {
            let rec_title = recording["title"].as_str()?;
            
            // Get artist from artist-credit
            let artist_credit = recording["artist-credit"].as_array()?;
            let artist_name = artist_credit
                .first()
                .and_then(|ac| ac["artist"]["name"].as_str())
                .map(|s| s.to_string());

            // Get album from releases
            let album_name = recording["releases"]
                .as_array()
                .and_then(|releases| releases.first())
                .and_then(|release| release["title"].as_str())
                .map(|s| s.to_string());

            // Check if this is a good match (score > 80)
            let score = recording["score"].as_i64().unwrap_or(0);
            if score >= 80 {
                return Some(EnrichedMetadata {
                    title: rec_title.to_string(),
                    artist: artist_name,
                    album: album_name,
                    source: MetadataSource::MusicBrainz,
                });
            }
        }

        // If no high-score match, try the first result anyway
        if let Some(first) = recordings.first() {
            let rec_title = first["title"].as_str()?;
            let artist_credit = first["artist-credit"].as_array()?;
            let artist_name = artist_credit
                .first()
                .and_then(|ac| ac["artist"]["name"].as_str())
                .map(|s| s.to_string());
            let album_name = first["releases"]
                .as_array()
                .and_then(|releases| releases.first())
                .and_then(|release| release["title"].as_str())
                .map(|s| s.to_string());

            return Some(EnrichedMetadata {
                title: rec_title.to_string(),
                artist: artist_name,
                album: album_name,
                source: MetadataSource::MusicBrainz,
            });
        }

        None
    }

    /// Use LLM to infer metadata from YouTube title
    async fn infer_with_llm(&self, title: &str, channel: Option<&str>) -> Option<EnrichedMetadata> {
        let prompt = format!(
            r#"Analiza este título de video de YouTube y extrae la información de la canción.

Título del video: "{}"
Canal de YouTube: "{}"

Responde SOLO con un JSON válido con este formato exacto (sin markdown, sin explicaciones):
{{"title": "nombre de la canción", "artist": "nombre del artista", "album": "nombre del álbum o null si no se puede determinar"}}

Reglas:
- Extrae el nombre REAL de la canción sin "(Official Video)", "(Lyrics)", "ft.", "feat.", etc.
- El artista debe ser el nombre real del artista/banda, no el canal de YouTube
- Si hay colaboraciones (ft., feat., &, x, with), pon solo el artista principal
- Si no puedes determinar el álbum, usa null
- Si el canal es "VEVO" o termina en "VEVO", el artista es el nombre antes de VEVO
- Limpia caracteres especiales innecesarios del título"#,
            title,
            channel.unwrap_or("Desconocido")
        );

        let url = format!("{}/api/llm/chat", self.backend_url);
        
        let body = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": "Eres un experto en música que extrae metadatos de títulos de videos de YouTube. Respondes SOLO con JSON válido, sin markdown ni explicaciones adicionales."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "model": "light"
        });

        debug!(url = %url, "Calling LLM for metadata inference");

        let response = match self.client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "LLM request failed");
                return None;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            warn!(status = %status, error = %text, "LLM request failed");
            return None;
        }

        let json: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                warn!(error = %e, "Failed to parse LLM response");
                return None;
            }
        };

        // Extract content from LLM response
        let content = json["message"]["content"]
            .as_str()
            .or_else(|| json["content"].as_str())
            .or_else(|| json["response"].as_str())?;

        // Parse the JSON from the LLM response
        // Try to extract JSON from the response (it might have markdown code blocks)
        let clean_content = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: serde_json::Value = match serde_json::from_str(clean_content) {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, content = %content, "Failed to parse LLM JSON output");
                return None;
            }
        };

        let inferred_title = parsed["title"].as_str()?.to_string();
        let inferred_artist = parsed["artist"].as_str().map(|s| s.to_string());
        let inferred_album = parsed["album"].as_str().map(|s| s.to_string());

        Some(EnrichedMetadata {
            title: inferred_title,
            artist: inferred_artist,
            album: inferred_album,
            source: MetadataSource::LlmInference,
        })
    }

    /// Clean common YouTube title suffixes
    fn clean_youtube_title(title: &str) -> String {
        let patterns_to_remove = [
            "(Official Video)",
            "(Official Music Video)",
            "(Official Audio)",
            "(Official Lyric Video)",
            "(Lyrics)",
            "(Lyric Video)",
            "(Audio)",
            "(Video Oficial)",
            "(Audio Oficial)",
            "(Letra)",
            "(HD)",
            "(HQ)",
            "(4K)",
            "[Official Video]",
            "[Official Music Video]",
            "[Official Audio]",
            "[Lyrics]",
            "[Audio]",
            "[HD]",
            "[HQ]",
            "| Official Video",
            "| Official Music Video",
            "| Official Audio",
            "- Official Video",
            "- Official Music Video",
            "- Official Audio",
            "(Visualizer)",
            "[Visualizer]",
            "(Performance Video)",
            "(Music Video)",
            "(MV)",
            "[MV]",
        ];

        let mut result = title.to_string();
        for pattern in patterns_to_remove {
            result = result.replace(pattern, "");
            // Also try case-insensitive
            let lower_pattern = pattern.to_lowercase();
            let lower_result = result.to_lowercase();
            if let Some(idx) = lower_result.find(&lower_pattern) {
                result = format!("{}{}", &result[..idx], &result[idx + pattern.len()..]);
            }
        }

        // Trim and clean up extra spaces
        result
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Enrich multiple search results in batch (for search results)
    pub async fn enrich_search_results(&self, results: Vec<SearchResultToEnrich>) -> Vec<EnrichedSearchResult> {
        let mut enriched = Vec::with_capacity(results.len());
        
        for result in results {
            let metadata = self.enrich(&result.title, result.channel.as_deref()).await;
            
            // Use enriched artist if found, otherwise use the extracted artist from title
            let final_artist = metadata.artist.or(result.artist);
            
            enriched.push(EnrichedSearchResult {
                video_id: result.video_id,
                original_title: result.title,
                title: metadata.title,
                artist: final_artist,
                album: metadata.album,
                channel: result.channel,
                duration: result.duration,
                thumbnail: result.thumbnail,
                view_count: result.view_count,
                source: metadata.source,
            });
        }
        
        enriched
    }
}

/// Input for batch enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultToEnrich {
    pub video_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub channel: Option<String>,
    pub duration: i64,
    pub thumbnail: String,
    pub view_count: Option<i64>,
}

/// Enriched search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedSearchResult {
    pub video_id: String,
    pub original_title: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub channel: Option<String>,
    pub duration: i64,
    pub thumbnail: String,
    pub view_count: Option<i64>,
    pub source: MetadataSource,
}
