use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Event Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Event,
    Task,
    Reminder,
    Birthday,
    Holiday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecurrenceType {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub event_type: EventType,
    pub color: Option<String>,
    pub location: Option<String>,
    pub recurrence: RecurrenceType,
    pub reminder_minutes: Option<i32>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub all_day: Option<bool>,
    pub event_type: Option<EventType>,
    pub color: Option<String>,
    pub location: Option<String>,
    pub recurrence: Option<RecurrenceType>,
    pub reminder_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub all_day: Option<bool>,
    pub event_type: Option<EventType>,
    pub color: Option<String>,
    pub location: Option<String>,
    pub recurrence: Option<RecurrenceType>,
    pub reminder_minutes: Option<i32>,
    pub completed: Option<bool>,
}

// =============================================================================
// Reminder Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: Uuid,
    pub event_id: Uuid,
    pub remind_at: DateTime<Utc>,
    pub dismissed: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Query Parameters
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub event_type: Option<String>,
}

// =============================================================================
// Calendar State (In-Memory Storage)
// =============================================================================

#[derive(Debug, Default)]
pub struct CalendarState {
    pub events: HashMap<Uuid, CalendarEvent>,
    pub reminders: HashMap<Uuid, Reminder>,
}

impl CalendarState {
    pub fn new() -> Self {
        Self::default()
    }
}

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<CalendarEvent>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct RemindersResponse {
    pub reminders: Vec<ReminderWithEvent>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReminderWithEvent {
    pub reminder: Reminder,
    pub event: Option<CalendarEvent>,
}

// =============================================================================
// Predefined Colors
// =============================================================================

pub const EVENT_COLORS: &[(&str, &str)] = &[
    ("blue", "#3b82f6"),
    ("green", "#22c55e"),
    ("red", "#ef4444"),
    ("yellow", "#eab308"),
    ("purple", "#a855f7"),
    ("pink", "#ec4899"),
    ("orange", "#f97316"),
    ("teal", "#14b8a6"),
];
