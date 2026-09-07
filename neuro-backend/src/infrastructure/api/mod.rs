//! =============================================================================
//! API Module - REST API Implementation
//! =============================================================================
//! Contains all HTTP handlers and routes using Axum framework.
//! Exposes the application layer services via REST endpoints.
//! =============================================================================

pub mod dto;
pub mod events;
pub mod handlers;
pub mod middleware;
pub mod routes;

pub use events::EventBroadcaster;
pub use routes::create_router;
pub use routes::ApiDoc;
