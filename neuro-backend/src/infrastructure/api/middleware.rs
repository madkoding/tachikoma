//! =============================================================================
//! API Middleware
//! =============================================================================
//! Contains middleware for request/response processing.
//! =============================================================================

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{info, Span};
use std::time::Instant;

/// =============================================================================
/// Request logging middleware
/// =============================================================================
/// Logs all incoming requests with timing information.
/// =============================================================================
pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    let span = tracing::info_span!(
        "request",
        method = %method,
        uri = %uri,
    );
    let _guard = span.enter();

    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();

    info!(
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

/// =============================================================================
/// Request ID middleware
/// =============================================================================
/// Adds a unique request ID to each request for tracing.
/// =============================================================================
pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    
    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    let mut response = next.run(request).await;
    
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    response
}
