//! Backend client for data persistence

use reqwest::Client;
use serde::de::DeserializeOwned;
use tracing::{debug, error};

use crate::config::Config;
use crate::models::*;

/// Client for communicating with tachikoma-backend data layer
pub struct BackendClient {
    client: Client,
    base_url: String,
}

impl BackendClient {
    /// Create a new backend client
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: config.backend_url.clone(),
        }
    }

    /// Generic GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, BackendError> {
        let url = format!("{}/api/data{}", self.base_url, path);
        debug!("GET {}", url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Backend error: {} - {}", status, body);
            return Err(BackendError::ApiError(format!("{}: {}", status, body)));
        }
        
        Ok(response.json().await?)
    }

    /// Generic POST request
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BackendError> {
        let url = format!("{}/api/data{}", self.base_url, path);
        debug!("POST {}", url);
        
        let response = self.client.post(&url).json(body).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Backend error: {} - {}", status, body);
            return Err(BackendError::ApiError(format!("{}: {}", status, body)));
        }
        
        Ok(response.json().await?)
    }

    /// Generic PATCH request
    pub async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BackendError> {
        let url = format!("{}/api/data{}", self.base_url, path);
        debug!("PATCH {}", url);
        
        let response = self.client.patch(&url).json(body).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Backend error: {} - {}", status, body);
            return Err(BackendError::ApiError(format!("{}: {}", status, body)));
        }
        
        Ok(response.json().await?)
    }

    /// Generic DELETE request
    pub async fn delete(&self, path: &str) -> Result<(), BackendError> {
        let url = format!("{}/api/data{}", self.base_url, path);
        debug!("DELETE {}", url);
        
        let response = self.client.delete(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Backend error: {} - {}", status, body);
            return Err(BackendError::ApiError(format!("{}: {}", status, body)));
        }
        
        Ok(())
    }

    // =========================================================================
    // Pomodoro-specific methods
    // =========================================================================

    /// Get all sessions for today
    pub async fn get_today_sessions(&self) -> Result<Vec<PomodoroSession>, BackendError> {
        self.get("/pomodoro/sessions/today").await
    }

    /// Get session by ID
    pub async fn get_session(&self, id: &str) -> Result<Option<PomodoroSession>, BackendError> {
        match self.get(&format!("/pomodoro/sessions/{}", id)).await {
            Ok(session) => Ok(Some(session)),
            Err(BackendError::ApiError(e)) if e.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, session: &PomodoroSession) -> Result<PomodoroSession, BackendError> {
        self.post("/pomodoro/sessions", session).await
    }

    /// Update a session
    pub async fn update_session(&self, id: &str, update: &UpdateSessionRequest) -> Result<PomodoroSession, BackendError> {
        self.patch(&format!("/pomodoro/sessions/{}", id), update).await
    }

    /// Get user settings
    pub async fn get_settings(&self) -> Result<PomodoroSettings, BackendError> {
        match self.get("/pomodoro/settings").await {
            Ok(settings) => Ok(settings),
            Err(_) => Ok(PomodoroSettings::default()),
        }
    }

    /// Save user settings
    pub async fn save_settings(&self, settings: &PomodoroSettings) -> Result<PomodoroSettings, BackendError> {
        self.post("/pomodoro/settings", settings).await
    }

    /// Get daily stats
    pub async fn get_daily_stats(&self, date: &str) -> Result<DailyStats, BackendError> {
        match self.get(&format!("/pomodoro/stats/{}", date)).await {
            Ok(stats) => Ok(stats),
            Err(_) => Ok(DailyStats {
                date: date.to_string(),
                total_sessions: 0,
                completed_sessions: 0,
                total_work_minutes: 0,
                total_break_minutes: 0,
            }),
        }
    }

    /// Get stats for date range
    pub async fn get_stats_range(&self, start: &str, end: &str) -> Result<Vec<DailyStats>, BackendError> {
        self.get(&format!("/pomodoro/stats?start={}&end={}", start, end)).await
    }
}

/// Backend client errors
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API error: {0}")]
    ApiError(String),
}
