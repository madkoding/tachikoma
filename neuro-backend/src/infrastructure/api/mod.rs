//! =============================================================================
//! API Module - REST API Implementation
//! =============================================================================
//! Contains all HTTP handlers and routes using Axum framework.
//! Exposes the application layer services via REST endpoints.
//! =============================================================================

pub mod routes;
pub mod handlers;
pub mod dto;
pub mod middleware;

pub use routes::create_router;
