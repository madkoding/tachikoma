//! In-memory store for Pomodoro sessions and settings
//! 
//! This provides a simple in-memory storage for development.
//! Data will be lost on restart.

use chrono::{Utc, NaiveDate};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::models::*;

/// In-memory store for Pomodoro data
pub struct MemoryStore {
    sessions: RwLock<HashMap<Uuid, PomodoroSession>>,
    settings: RwLock<PomodoroSettings>,
}

impl MemoryStore {
    /// Create a new memory store
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            settings: RwLock::new(PomodoroSettings::default()),
        }
    }

    // =========================================================================
    // Sessions
    // =========================================================================

    /// Get all sessions for today
    pub fn get_today_sessions(&self) -> Vec<PomodoroSession> {
        let today = Utc::now().date_naive();
        let sessions = self.sessions.read().unwrap();
        
        sessions.values()
            .filter(|s| {
                s.started_at
                    .map(|dt| dt.date_naive() == today)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get session by ID
    pub fn get_session(&self, id: &Uuid) -> Option<PomodoroSession> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(id).cloned()
    }

    /// Create a new session
    pub fn create_session(&self, session: &PomodoroSession) -> PomodoroSession {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id, session.clone());
        session.clone()
    }

    /// Update a session
    pub fn update_session(&self, id: &Uuid, update: &UpdateSessionRequest) -> Option<PomodoroSession> {
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(id) {
            if let Some(elapsed) = update.elapsed_seconds {
                session.elapsed_seconds = elapsed;
            }
            if let Some(status) = &update.status {
                session.status = status.clone();
                // Set completed_at when completing or cancelling
                if matches!(status, SessionStatus::Completed | SessionStatus::Cancelled) {
                    session.completed_at = Some(Utc::now());
                }
            }
            if let Some(task) = &update.task_description {
                session.task_description = Some(task.clone());
            }
            return Some(session.clone());
        }
        
        None
    }

    /// Get all sessions in a date range
    pub fn get_sessions_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<PomodoroSession> {
        let sessions = self.sessions.read().unwrap();
        
        sessions.values()
            .filter(|s| {
                s.started_at
                    .map(|dt| {
                        let date = dt.date_naive();
                        date >= start && date <= end
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    // =========================================================================
    // Settings
    // =========================================================================

    /// Get settings
    pub fn get_settings(&self) -> PomodoroSettings {
        self.settings.read().unwrap().clone()
    }

    /// Save settings
    pub fn save_settings(&self, new_settings: &PomodoroSettings) -> PomodoroSettings {
        let mut settings = self.settings.write().unwrap();
        *settings = new_settings.clone();
        new_settings.clone()
    }

    // =========================================================================
    // Stats
    // =========================================================================

    /// Get stats for a date range
    pub fn get_stats_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<DailyStats> {
        let sessions = self.get_sessions_range(start, end);
        
        // Group by date
        let mut stats_by_date: HashMap<String, DailyStats> = HashMap::new();
        
        for session in sessions {
            if let Some(started_at) = session.started_at {
                let date = started_at.format("%Y-%m-%d").to_string();
                
                let stats = stats_by_date.entry(date.clone()).or_insert_with(|| DailyStats {
                    date: date.clone(),
                    total_sessions: 0,
                    completed_sessions: 0,
                    total_work_minutes: 0,
                    total_break_minutes: 0,
                });
                
                stats.total_sessions += 1;
                
                if session.status == SessionStatus::Completed {
                    stats.completed_sessions += 1;
                    
                    match session.session_type {
                        SessionType::Work => {
                            stats.total_work_minutes += session.duration_minutes;
                        }
                        SessionType::ShortBreak | SessionType::LongBreak => {
                            stats.total_break_minutes += session.duration_minutes;
                        }
                    }
                }
            }
        }
        
        // Fill in missing dates with zero stats
        let mut current = start;
        while current <= end {
            let date_str = current.format("%Y-%m-%d").to_string();
            stats_by_date.entry(date_str.clone()).or_insert_with(|| DailyStats {
                date: date_str,
                total_sessions: 0,
                completed_sessions: 0,
                total_work_minutes: 0,
                total_break_minutes: 0,
            });
            current += chrono::Duration::days(1);
        }
        
        let mut result: Vec<_> = stats_by_date.into_values().collect();
        result.sort_by(|a, b| a.date.cmp(&b.date));
        result
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
