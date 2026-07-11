//! Data models for Pomodoro service

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pomodoro session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Work session in progress
    Working,
    /// Short break in progress
    ShortBreak,
    /// Long break in progress
    LongBreak,
    /// Session paused
    Paused,
    /// Session completed
    Completed,
    /// Session cancelled
    Cancelled,
}

/// Pomodoro session type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

/// A single Pomodoro session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSession {
    pub id: Uuid,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub duration_minutes: u32,
    pub elapsed_seconds: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub task_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Daily statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub total_sessions: u32,
    pub completed_sessions: u32,
    pub total_work_minutes: u32,
    pub total_break_minutes: u32,
}

/// User settings for Pomodoro timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSettings {
    #[serde(alias = "work_minutes")]
    pub work_duration_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    #[serde(alias = "pomodoros_before_long_break")]
    pub sessions_until_long_break: u32,
    pub auto_start_breaks: bool,
    #[serde(alias = "auto_start_pomodoros")]
    pub auto_start_work: bool,
}

impl Default for PomodoroSettings {
    fn default() -> Self {
        Self {
            work_duration_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_until_long_break: 4,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }
}

/// Request to start a new session
#[derive(Debug, Clone, Deserialize)]
pub struct StartSessionRequest {
    pub session_type: SessionType,
    pub duration_minutes: Option<u32>,
    pub task_description: Option<String>,
}

/// Request to update session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub elapsed_seconds: Option<u32>,
    pub status: Option<SessionStatus>,
    pub task_description: Option<String>,
}

/// Response with current timer state
#[derive(Debug, Clone, Serialize)]
pub struct TimerState {
    pub active_session: Option<PomodoroSession>,
    pub today_stats: DailyStats,
    pub settings: PomodoroSettings,
    pub completed_today: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session() -> PomodoroSession {
        PomodoroSession {
            id: Uuid::new_v4(),
            session_type: SessionType::Work,
            status: SessionStatus::Working,
            duration_minutes: 25,
            elapsed_seconds: 0,
            started_at: Some(Utc::now()),
            completed_at: None,
            task_description: Some("coding".to_string()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn pomodoro_session_serde_roundtrip() {
        let s = sample_session();
        let json = serde_json::to_string(&s).unwrap();
        let back: PomodoroSession = serde_json::from_str(&json).unwrap();
        assert_eq!(s.id, back.id);
        assert_eq!(s.duration_minutes, back.duration_minutes);
    }

    #[test]
    fn session_status_snake_case() {
        let json = serde_json::to_string(&SessionStatus::ShortBreak).unwrap();
        assert_eq!(json, "\"short_break\"");
        let back: SessionStatus = serde_json::from_str("\"long_break\"").unwrap();
        assert_eq!(back, SessionStatus::LongBreak);
    }

    #[test]
    fn session_type_serde_roundtrip() {
        let json = serde_json::to_string(&SessionType::Work).unwrap();
        assert_eq!(json, "\"work\"");
        let back: SessionType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, SessionType::Work);
    }

    #[test]
    fn pomodoro_settings_default() {
        let s = PomodoroSettings::default();
        assert_eq!(s.work_duration_minutes, 25);
        assert_eq!(s.short_break_minutes, 5);
        assert_eq!(s.long_break_minutes, 15);
        assert_eq!(s.sessions_until_long_break, 4);
        assert!(!s.auto_start_breaks);
        assert!(!s.auto_start_work);
    }

    #[test]
    fn pomodoro_settings_alias_deserialize() {
        let json = r#"{"work_minutes":50,"short_break_minutes":10,"long_break_minutes":20,"pomodoros_before_long_break":3,"auto_start_breaks":true,"auto_start_pomodoros":true}"#;
        let s: PomodoroSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.work_duration_minutes, 50);
        assert_eq!(s.sessions_until_long_break, 3);
        assert!(s.auto_start_work);
    }
}
