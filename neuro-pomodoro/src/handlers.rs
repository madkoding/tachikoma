//! API Handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, Utc};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "neuro-pomodoro",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get current timer state
pub async fn get_timer_state(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TimerState>, StatusCode> {
    debug!("Getting timer state");
    
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let sessions = state.store.get_today_sessions();
    
    let active_session = sessions.iter().find(|s| {
        matches!(s.status, SessionStatus::Working | SessionStatus::ShortBreak | 
                 SessionStatus::LongBreak | SessionStatus::Paused)
    }).cloned();
    
    let completed_pomodoros = sessions.iter()
        .filter(|s| s.session_type == SessionType::Work && s.status == SessionStatus::Completed)
        .count() as u32;
    
    let total_work_minutes = sessions.iter()
        .filter(|s| s.session_type == SessionType::Work && s.status == SessionStatus::Completed)
        .map(|s| s.duration_minutes)
        .sum();
    
    let total_break_minutes = sessions.iter()
        .filter(|s| matches!(s.session_type, SessionType::ShortBreak | SessionType::LongBreak) 
                && s.status == SessionStatus::Completed)
        .map(|s| s.duration_minutes)
        .sum();
    
    let settings = state.store.get_settings();
    
    Ok(Json(TimerState {
        active_session,
        today_stats: DailyStats {
            date: today,
            total_sessions: completed_pomodoros,
            completed_sessions: completed_pomodoros,
            total_work_minutes,
            total_break_minutes,
        },
        settings,
        completed_today: completed_pomodoros,
    }))
}

/// Start a new pomodoro session
pub async fn start_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StartSessionRequest>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Starting new {:?} session", request.session_type);
    
    let settings = state.store.get_settings();
    
    let duration = request.duration_minutes.unwrap_or_else(|| {
        match request.session_type {
            SessionType::Work => settings.work_duration_minutes,
            SessionType::ShortBreak => settings.short_break_minutes,
            SessionType::LongBreak => settings.long_break_minutes,
        }
    });
    
    let session = PomodoroSession {
        id: Uuid::new_v4(),
        session_type: request.session_type.clone(),
        status: match request.session_type {
            SessionType::Work => SessionStatus::Working,
            SessionType::ShortBreak => SessionStatus::ShortBreak,
            SessionType::LongBreak => SessionStatus::LongBreak,
        },
        duration_minutes: duration,
        elapsed_seconds: 0,
        started_at: Some(Utc::now()),
        completed_at: None,
        task_description: request.task_description,
        created_at: Utc::now(),
    };
    
    let created = state.store.create_session(&session);
    Ok(Json(created))
}

/// Update a session (pause, resume, update elapsed time)
pub async fn update_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    debug!("Updating session {}: {:?}", id, request);
    
    match state.store.update_session(&id, &request) {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Complete a session
pub async fn complete_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Completing session {}", id);
    
    let update = UpdateSessionRequest {
        elapsed_seconds: None,
        status: Some(SessionStatus::Completed),
        task_description: None,
    };
    
    match state.store.update_session(&id, &update) {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Cancel a session
pub async fn cancel_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Cancelling session {}", id);
    
    let update = UpdateSessionRequest {
        elapsed_seconds: None,
        status: Some(SessionStatus::Cancelled),
        task_description: None,
    };
    
    match state.store.update_session(&id, &update) {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Pause a session
pub async fn pause_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Pausing session {}", id);
    
    let update = UpdateSessionRequest {
        elapsed_seconds: request.elapsed_seconds,
        status: Some(SessionStatus::Paused),
        task_description: None,
    };
    
    match state.store.update_session(&id, &update) {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Resume a paused session
pub async fn resume_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Resuming session {}", id);
    
    let session = match state.store.get_session(&id) {
        Some(s) => s,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let new_status = match session.session_type {
        SessionType::Work => SessionStatus::Working,
        SessionType::ShortBreak => SessionStatus::ShortBreak,
        SessionType::LongBreak => SessionStatus::LongBreak,
    };
    
    let update = UpdateSessionRequest {
        elapsed_seconds: None,
        status: Some(new_status),
        task_description: None,
    };
    
    match state.store.update_session(&id, &update) {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get user settings
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PomodoroSettings>, StatusCode> {
    debug!("Getting settings");
    Ok(Json(state.store.get_settings()))
}

/// Save user settings
pub async fn save_settings(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<PomodoroSettings>,
) -> Result<Json<PomodoroSettings>, StatusCode> {
    info!("Saving settings");
    let saved = state.store.save_settings(&settings);
    Ok(Json(saved))
}

/// Get today's sessions
pub async fn get_today_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PomodoroSession>>, StatusCode> {
    debug!("Getting today's sessions");
    Ok(Json(state.store.get_today_sessions()))
}

/// Query parameters for stats endpoint
#[derive(Debug, serde::Deserialize)]
pub struct StatsQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub date: Option<String>,
}

/// Get stats for a date range
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<StatsQuery>,
) -> Result<Json<Vec<DailyStats>>, StatusCode> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let start_str = params.start.unwrap_or_else(|| today.clone());
    let end_str = params.end.unwrap_or(today);
    
    debug!("Getting stats from {} to {}", start_str, end_str);
    
    let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let end = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    
    Ok(Json(state.store.get_stats_range(start, end)))
}

/// Get daily stats (for a specific date or today)
pub async fn get_daily_stats(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<StatsQuery>,
) -> Result<Json<DailyStats>, StatusCode> {
    let date_str = params.date.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    
    debug!("Getting daily stats for {}", date_str);
    
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    
    let stats = state.store.get_stats_range(date, date);
    let day_stats = stats.into_iter().next().unwrap_or_else(|| DailyStats {
        date: date_str,
        total_sessions: 0,
        completed_sessions: 0,
        total_work_minutes: 0,
        total_break_minutes: 0,
    });
    
    Ok(Json(day_stats))
}

/// Get weekly stats (last 7 days)
pub async fn get_weekly_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DailyStats>>, StatusCode> {
    let today = Utc::now().date_naive();
    let week_ago = today - chrono::Duration::days(6);
    
    debug!("Getting weekly stats from {} to {}", week_ago, today);
    
    Ok(Json(state.store.get_stats_range(week_ago, today)))
}

// =============================================================================
// Active Session API (for frontend compatibility)
// =============================================================================

/// Get active session (if any)
pub async fn get_active_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<PomodoroSession>>, StatusCode> {
    debug!("Getting active session");
    
    let sessions = state.store.get_today_sessions();
    
    let active_session = sessions.into_iter().find(|s| {
        matches!(s.status, SessionStatus::Working | SessionStatus::ShortBreak | 
                 SessionStatus::LongBreak | SessionStatus::Paused)
    });
    
    Ok(Json(active_session))
}

/// Pause the active session (without needing ID)
pub async fn pause_active_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Pausing active session");
    
    let sessions = state.store.get_today_sessions();
    let active_session = sessions.into_iter().find(|s| {
        matches!(s.status, SessionStatus::Working | SessionStatus::ShortBreak | SessionStatus::LongBreak)
    });
    
    match active_session {
        Some(session) => {
            let update = UpdateSessionRequest {
                elapsed_seconds: None,
                status: Some(SessionStatus::Paused),
                task_description: None,
            };
            
            match state.store.update_session(&session.id, &update) {
                Some(updated) => Ok(Json(updated)),
                None => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Resume the active (paused) session (without needing ID)
pub async fn resume_active_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Resuming active session");
    
    let sessions = state.store.get_today_sessions();
    let paused_session = sessions.into_iter().find(|s| s.status == SessionStatus::Paused);
    
    match paused_session {
        Some(session) => {
            let new_status = match session.session_type {
                SessionType::Work => SessionStatus::Working,
                SessionType::ShortBreak => SessionStatus::ShortBreak,
                SessionType::LongBreak => SessionStatus::LongBreak,
            };
            
            let update = UpdateSessionRequest {
                elapsed_seconds: None,
                status: Some(new_status),
                task_description: None,
            };
            
            match state.store.update_session(&session.id, &update) {
                Some(updated) => Ok(Json(updated)),
                None => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Complete the active session (without needing ID)
pub async fn complete_active_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Completing active session");
    
    let sessions = state.store.get_today_sessions();
    let active_session = sessions.into_iter().find(|s| {
        matches!(s.status, SessionStatus::Working | SessionStatus::ShortBreak | 
                 SessionStatus::LongBreak | SessionStatus::Paused)
    });
    
    match active_session {
        Some(session) => {
            let update = UpdateSessionRequest {
                elapsed_seconds: None,
                status: Some(SessionStatus::Completed),
                task_description: None,
            };
            
            match state.store.update_session(&session.id, &update) {
                Some(updated) => Ok(Json(updated)),
                None => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Cancel the active session (without needing ID)
pub async fn cancel_active_session(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    info!("Cancelling active session");
    
    let sessions = state.store.get_today_sessions();
    let active_session = sessions.into_iter().find(|s| {
        matches!(s.status, SessionStatus::Working | SessionStatus::ShortBreak | 
                 SessionStatus::LongBreak | SessionStatus::Paused)
    });
    
    match active_session {
        Some(session) => {
            let update = UpdateSessionRequest {
                elapsed_seconds: None,
                status: Some(SessionStatus::Cancelled),
                task_description: None,
            };
            
            match state.store.update_session(&session.id, &update) {
                Some(_) => Ok(StatusCode::NO_CONTENT),
                None => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get session history
pub async fn get_session_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HistoryQuery>,
) -> Result<Json<Vec<PomodoroSession>>, StatusCode> {
    let limit = params.limit.unwrap_or(20);
    debug!("Getting session history (limit: {})", limit);
    
    let sessions = state.store.get_today_sessions();
    let history: Vec<_> = sessions.into_iter()
        .filter(|s| matches!(s.status, SessionStatus::Completed | SessionStatus::Cancelled))
        .take(limit)
        .collect();
    
    Ok(Json(history))
}

#[derive(Debug, serde::Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

/// Update settings (PUT method)
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<Json<PomodoroSettings>, StatusCode> {
    info!("Updating settings");
    
    let mut settings = state.store.get_settings();
    
    if let Some(v) = request.work_duration_minutes { settings.work_duration_minutes = v; }
    if let Some(v) = request.short_break_minutes { settings.short_break_minutes = v; }
    if let Some(v) = request.long_break_minutes { settings.long_break_minutes = v; }
    if let Some(v) = request.sessions_until_long_break { settings.sessions_until_long_break = v; }
    if let Some(v) = request.auto_start_breaks { settings.auto_start_breaks = v; }
    if let Some(v) = request.auto_start_work { settings.auto_start_work = v; }
    
    let saved = state.store.save_settings(&settings);
    Ok(Json(saved))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateSettingsRequest {
    pub work_duration_minutes: Option<u32>,
    pub short_break_minutes: Option<u32>,
    pub long_break_minutes: Option<u32>,
    pub sessions_until_long_break: Option<u32>,
    pub auto_start_breaks: Option<bool>,
    pub auto_start_work: Option<bool>,
}
