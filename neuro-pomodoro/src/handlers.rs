//! API Handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info};
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
    
    // Get today's date
    let today = Utc::now().format("%Y-%m-%d").to_string();
    
    // Get sessions for today
    let sessions = state.client.get_today_sessions().await.unwrap_or_default();
    
    // Find active session (not completed or cancelled)
    let active_session = sessions.iter().find(|s| {
        matches!(s.status, SessionStatus::Working | SessionStatus::ShortBreak | 
                 SessionStatus::LongBreak | SessionStatus::Paused)
    }).cloned();
    
    // Calculate today's stats
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
    
    // Get settings
    let settings = state.client.get_settings().await.unwrap_or_default();
    
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
    
    // Get settings for default durations
    let settings = state.client.get_settings().await.unwrap_or_default();
    
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
    
    match state.client.create_session(&session).await {
        Ok(created) => Ok(Json(created)),
        Err(e) => {
            error!("Failed to create session: {}", e);
            // Return the session anyway for client-side tracking
            Ok(Json(session))
        }
    }
}

/// Update a session (pause, resume, update elapsed time)
pub async fn update_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    debug!("Updating session {}: {:?}", id, request);
    
    match state.client.update_session(&id.to_string(), &request).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => {
            error!("Failed to update session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
    
    match state.client.update_session(&id.to_string(), &update).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => {
            error!("Failed to complete session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
    
    match state.client.update_session(&id.to_string(), &update).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => {
            error!("Failed to cancel session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
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
    
    match state.client.update_session(&id.to_string(), &update).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => {
            error!("Failed to pause session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Resume a paused session
pub async fn resume_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PomodoroSession>, StatusCode> {
    info!("Resuming session {}", id);
    
    // Get current session to determine what type it was
    let session = match state.client.get_session(&id.to_string()).await {
        Ok(Some(s)) => s,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get session: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
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
    
    match state.client.update_session(&id.to_string(), &update).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => {
            error!("Failed to resume session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user settings
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PomodoroSettings>, StatusCode> {
    debug!("Getting settings");
    
    let settings = state.client.get_settings().await.unwrap_or_default();
    Ok(Json(settings))
}

/// Save user settings
pub async fn save_settings(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<PomodoroSettings>,
) -> Result<Json<PomodoroSettings>, StatusCode> {
    info!("Saving settings");
    
    match state.client.save_settings(&settings).await {
        Ok(saved) => Ok(Json(saved)),
        Err(e) => {
            error!("Failed to save settings: {}", e);
            // Return the settings anyway
            Ok(Json(settings))
        }
    }
}

/// Get today's sessions
pub async fn get_today_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PomodoroSession>>, StatusCode> {
    debug!("Getting today's sessions");
    
    let sessions = state.client.get_today_sessions().await.unwrap_or_default();
    Ok(Json(sessions))
}

/// Get stats for a date range
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<StatsQuery>,
) -> Result<Json<Vec<DailyStats>>, StatusCode> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let start = params.start.unwrap_or_else(|| today.clone());
    let end = params.end.unwrap_or(today);
    
    debug!("Getting stats from {} to {}", start, end);
    
    match state.client.get_stats_range(&start, &end).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!("Failed to get stats: {}", e);
            Ok(Json(vec![]))
        }
    }
}

/// Query parameters for stats endpoint
#[derive(Debug, serde::Deserialize)]
pub struct StatsQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub date: Option<String>,
}

/// Get daily stats (for a specific date or today)
pub async fn get_daily_stats(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<StatsQuery>,
) -> Result<Json<DailyStats>, StatusCode> {
    let date = params.date.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    
    debug!("Getting daily stats for {}", date);
    
    match state.client.get_stats_range(&date, &date).await {
        Ok(stats) => {
            // Return the stats for the day, or empty stats if not found
            let day_stats = stats.into_iter().next().unwrap_or_else(|| DailyStats {
                date: date.clone(),
                total_sessions: 0,
                completed_sessions: 0,
                total_work_minutes: 0,
                total_break_minutes: 0,
            });
            Ok(Json(day_stats))
        }
        Err(e) => {
            error!("Failed to get daily stats: {}", e);
            Ok(Json(DailyStats {
                date,
                total_sessions: 0,
                completed_sessions: 0,
                total_work_minutes: 0,
                total_break_minutes: 0,
            }))
        }
    }
}

/// Get weekly stats (last 7 days)
pub async fn get_weekly_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DailyStats>>, StatusCode> {
    let today = Utc::now().date_naive();
    let week_ago = today - chrono::Duration::days(6);
    
    let start = week_ago.format("%Y-%m-%d").to_string();
    let end = today.format("%Y-%m-%d").to_string();
    
    debug!("Getting weekly stats from {} to {}", start, end);
    
    match state.client.get_stats_range(&start, &end).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!("Failed to get weekly stats: {}", e);
            Ok(Json(vec![]))
        }
    }
}
