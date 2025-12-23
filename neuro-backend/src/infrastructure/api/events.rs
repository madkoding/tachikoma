//! =============================================================================
//! Server-Sent Events (SSE) for Real-time Graph Updates
//! =============================================================================
//! Provides real-time streaming of memory events to connected clients.
//! This allows the admin UI to receive new memories as they are created
//! without polling the entire graph.
//! =============================================================================

use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// =============================================================================
/// Memory Event Types
/// =============================================================================
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum MemoryEvent {
    /// A new memory was created
    Created(MemoryEventData),
    /// A memory was updated
    Updated(MemoryEventData),
    /// A memory was deleted
    Deleted { id: String },
    /// A new relation was created
    RelationCreated(RelationEventData),
    /// Heartbeat to keep connection alive
    Heartbeat,
}

/// Minimal data for memory events - enough for the graph UI to render
#[derive(Debug, Clone, Serialize)]
pub struct MemoryEventData {
    pub id: String,
    pub content: String,
    pub memory_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelationEventData {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
}

/// =============================================================================
/// Event Broadcaster
/// =============================================================================
/// Manages broadcasting memory events to all connected SSE clients.
/// =============================================================================
#[derive(Debug, Clone)]
pub struct EventBroadcaster {
    sender: broadcast::Sender<MemoryEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Broadcast a memory event to all connected clients
    pub fn broadcast(&self, event: MemoryEvent) {
        // Ignore errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Create a new subscriber stream for SSE
    pub fn subscribe(&self) -> BroadcastStream<MemoryEvent> {
        BroadcastStream::new(self.sender.subscribe())
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new(100) // Default capacity of 100 events
    }
}

/// =============================================================================
/// SSE Handler
/// =============================================================================
/// Creates an SSE stream for a client to receive memory events.
/// =============================================================================
pub fn create_event_stream(
    broadcaster: Arc<EventBroadcaster>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = broadcaster.subscribe();
    
    // Map broadcast events to SSE events
    let event_stream = stream
        .filter_map(|result| {
            match result {
                Ok(event) => {
                    let json = serde_json::to_string(&event).ok()?;
                    Some(Ok(Event::default().data(json)))
                }
                Err(_) => None, // Skip lagged events
            }
        });

    // Add periodic heartbeat to keep connection alive
    let heartbeat = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(Duration::from_secs(30))
    )
    .map(|_| Ok(Event::default().comment("heartbeat")));

    // Merge event stream with heartbeat
    let merged = futures::stream::select(event_stream, heartbeat);

    Sse::new(merged).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
