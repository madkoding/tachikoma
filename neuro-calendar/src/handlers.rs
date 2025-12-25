use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

// =============================================================================
// Event Handlers
// =============================================================================

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<EventsResponse>, StatusCode> {
    debug!("Listing events with query: {:?}", query);
    
    let calendar = state.calendar_state.read().await;
    
    let mut events: Vec<CalendarEvent> = calendar.events.values().cloned().collect();
    
    // Filter by date range
    if let Some(start) = query.start {
        events.retain(|e| e.start_time >= start);
    }
    if let Some(end) = query.end {
        events.retain(|e| e.start_time <= end);
    }
    
    // Filter by event type
    if let Some(event_type) = query.event_type {
        events.retain(|e| {
            let type_str = serde_json::to_string(&e.event_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            type_str == event_type
        });
    }
    
    // Sort by start time
    events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    
    let total = events.len();
    
    Ok(Json(EventsResponse { events, total }))
}

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    debug!("Getting event: {}", id);
    
    let calendar = state.calendar_state.read().await;
    
    calendar
        .events
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateEventRequest>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    debug!("Creating event: {}", request.title);
    
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    let event = CalendarEvent {
        id,
        title: request.title,
        description: request.description,
        start_time: request.start_time,
        end_time: request.end_time,
        all_day: request.all_day.unwrap_or(false),
        event_type: request.event_type.unwrap_or(EventType::Event),
        color: request.color,
        location: request.location,
        recurrence: request.recurrence.unwrap_or(RecurrenceType::None),
        reminder_minutes: request.reminder_minutes,
        completed: false,
        created_at: now,
        updated_at: now,
    };
    
    // Create reminder if specified
    if let Some(minutes) = request.reminder_minutes {
        let remind_at = event.start_time - chrono::Duration::minutes(minutes as i64);
        let reminder = Reminder {
            id: Uuid::new_v4(),
            event_id: id,
            remind_at,
            dismissed: false,
            created_at: now,
        };
        
        let mut calendar = state.calendar_state.write().await;
        calendar.reminders.insert(reminder.id, reminder);
        calendar.events.insert(id, event.clone());
    } else {
        let mut calendar = state.calendar_state.write().await;
        calendar.events.insert(id, event.clone());
    }
    
    Ok(Json(event))
}

pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateEventRequest>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    debug!("Updating event: {}", id);
    
    let mut calendar = state.calendar_state.write().await;
    
    let event = calendar
        .events
        .get_mut(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if let Some(title) = request.title {
        event.title = title;
    }
    if let Some(description) = request.description {
        event.description = Some(description);
    }
    if let Some(start_time) = request.start_time {
        event.start_time = start_time;
    }
    if let Some(end_time) = request.end_time {
        event.end_time = Some(end_time);
    }
    if let Some(all_day) = request.all_day {
        event.all_day = all_day;
    }
    if let Some(event_type) = request.event_type {
        event.event_type = event_type;
    }
    if let Some(color) = request.color {
        event.color = Some(color);
    }
    if let Some(location) = request.location {
        event.location = Some(location);
    }
    if let Some(recurrence) = request.recurrence {
        event.recurrence = recurrence;
    }
    if let Some(reminder_minutes) = request.reminder_minutes {
        event.reminder_minutes = Some(reminder_minutes);
    }
    if let Some(completed) = request.completed {
        event.completed = completed;
    }
    
    event.updated_at = Utc::now();
    
    Ok(Json(event.clone()))
}

pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Deleting event: {}", id);
    
    let mut calendar = state.calendar_state.write().await;
    
    // Remove associated reminders
    calendar.reminders.retain(|_, r| r.event_id != id);
    
    if calendar.events.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Reminder Handlers
// =============================================================================

pub async fn list_pending_reminders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RemindersResponse>, StatusCode> {
    debug!("Listing pending reminders");
    
    let calendar = state.calendar_state.read().await;
    let now = Utc::now();
    
    let mut reminders: Vec<ReminderWithEvent> = calendar
        .reminders
        .values()
        .filter(|r| !r.dismissed && r.remind_at <= now)
        .map(|r| ReminderWithEvent {
            reminder: r.clone(),
            event: calendar.events.get(&r.event_id).cloned(),
        })
        .collect();
    
    reminders.sort_by(|a, b| a.reminder.remind_at.cmp(&b.reminder.remind_at));
    
    let total = reminders.len();
    
    Ok(Json(RemindersResponse { reminders, total }))
}

pub async fn dismiss_reminder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("Dismissing reminder: {}", id);
    
    let mut calendar = state.calendar_state.write().await;
    
    if let Some(reminder) = calendar.reminders.get_mut(&id) {
        reminder.dismissed = true;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Today's Events
// =============================================================================

pub async fn get_today_events(
    State(state): State<Arc<AppState>>,
) -> Result<Json<EventsResponse>, StatusCode> {
    debug!("Getting today's events");
    
    let calendar = state.calendar_state.read().await;
    let now = Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
    
    let mut events: Vec<CalendarEvent> = calendar
        .events
        .values()
        .filter(|e| {
            let event_date = e.start_time.date_naive();
            event_date >= today_start.date() && event_date <= today_end.date()
        })
        .cloned()
        .collect();
    
    events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    
    let total = events.len();
    
    Ok(Json(EventsResponse { events, total }))
}

// =============================================================================
// Health Check
// =============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "neuro-calendar",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
