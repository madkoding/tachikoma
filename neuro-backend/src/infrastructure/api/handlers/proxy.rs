//! =============================================================================
//! API Gateway Proxy Handler
//! =============================================================================
//! Proxies requests to microservices (checklists, etc.)
//! =============================================================================

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, error};

use crate::AppState;

/// Proxy requests to the checklists microservice
/// Handles: /api/checklists/*
pub async fn proxy_checklists(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    let checklists_url = &state.microservices_config.checklists_url;
    
    // Build the target URL
    let path = request.uri().path();
    let query = request.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("{}{}{}", checklists_url, path, query);
    
    debug!(target = %target_url, "Proxying request to checklists service");

    // Get method and headers
    let method = request.method().clone();
    let headers = request.headers().clone();
    
    // Get body
    let body_bytes = match axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(error = %e, "Failed to read request body");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create HTTP client request
    let client = reqwest::Client::new();
    let mut req_builder = client.request(method, &target_url);

    // Copy headers (except host)
    for (name, value) in headers.iter() {
        if name != "host" {
            if let Ok(value_str) = value.to_str() {
                req_builder = req_builder.header(name.as_str(), value_str);
            }
        }
    }

    // Add body
    if !body_bytes.is_empty() {
        req_builder = req_builder.body(body_bytes.to_vec());
    }

    // Send request
    let response = match req_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!(error = %e, "Failed to proxy request to checklists service");
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    // Build response
    let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::OK);
    let mut builder = Response::builder().status(status);

    // Copy response headers
    for (name, value) in response.headers() {
        builder = builder.header(name.as_str(), value.as_bytes());
    }

    // Get response body
    let body_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(error = %e, "Failed to read response body");
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    let body = Body::from(body_bytes.to_vec());
    
    builder.body(body).map_err(|e| {
        error!(error = %e, "Failed to build response");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
