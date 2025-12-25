//! =============================================================================
//! API Gateway Proxy Handler
//! =============================================================================
//! Proxies requests to microservices (checklists, music, etc.)
//! =============================================================================

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use futures_util::StreamExt;
use std::sync::Arc;
use tracing::{debug, error};

use crate::AppState;

/// Generic proxy function to forward requests to a microservice
async fn proxy_to_service(
    service_url: &str,
    service_name: &str,
    request: Request,
    stream_response: bool,
) -> Result<Response, StatusCode> {
    // Build the target URL
    // Note: The path from request.uri() doesn't include /api prefix since it's stripped by nest()
    // We need to add /api back for the microservice
    let path = request.uri().path();
    let query = request.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("{}/api{}{}", service_url, path, query);
    
    debug!(target = %target_url, service = %service_name, "Proxying request");

    // Get method and headers
    let method = request.method().clone();
    let headers = request.headers().clone();
    
    // Get body
    let body_bytes = match axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(error = %e, service = %service_name, "Failed to read request body");
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
            error!(error = %e, service = %service_name, "Failed to proxy request");
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

    // For streaming responses (audio/video), use streaming body
    if stream_response {
        let stream = response.bytes_stream().map(|result| {
            result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
        let body = Body::from_stream(stream);
        
        builder.body(body).map_err(|e| {
            error!(error = %e, service = %service_name, "Failed to build streaming response");
            StatusCode::INTERNAL_SERVER_ERROR
        })
    } else {
        // For regular responses, buffer the full body
        let body_bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!(error = %e, service = %service_name, "Failed to read response body");
                return Err(StatusCode::BAD_GATEWAY);
            }
        };

        let body = Body::from(body_bytes.to_vec());
        
        builder.body(body).map_err(|e| {
            error!(error = %e, service = %service_name, "Failed to build response");
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }
}

/// Proxy requests to the checklists microservice
/// Handles: /api/checklists/*
pub async fn proxy_checklists(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_to_service(
        &state.microservices_config.checklists_url,
        "checklists",
        request,
        false, // No streaming needed for checklists
    ).await
}

/// Proxy requests to the music microservice
/// Handles: /api/music/*
pub async fn proxy_music(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    debug!("🎵 Proxying music request: {}", path);
    
    // Enable streaming for audio stream endpoints
    let is_stream = path.contains("/stream/");
    
    proxy_to_service(
        &state.microservices_config.music_url,
        "music",
        request,
        is_stream,
    ).await
}

/// Proxy requests to the memory microservice
/// Handles: /api/memories/*
pub async fn proxy_memory(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    debug!("🧠 Proxying memory request: {}", path);
    
    // Enable streaming for SSE events endpoint
    let is_stream = path.contains("/events");
    
    proxy_to_service(
        &state.microservices_config.memory_url,
        "memory",
        request,
        is_stream,
    ).await
}

/// Proxy requests to the chat microservice
/// Handles: /api/chat/*
pub async fn proxy_chat(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    debug!("💬 Proxying chat request: {}", path);
    
    // Enable streaming for SSE chat stream endpoint
    let is_stream = path.contains("/stream");
    
    proxy_to_service(
        &state.microservices_config.chat_url,
        "chat",
        request,
        is_stream,
    ).await
}

/// Proxy requests to the agent microservice
/// Handles: /api/agent/*
pub async fn proxy_agent(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("🤖 Proxying agent request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.agent_url,
        "agent",
        request,
        false, // No streaming for agent
    ).await
}

/// Proxy requests to the pomodoro microservice
/// Handles: /api/pomodoro/*
pub async fn proxy_pomodoro(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("🍅 Proxying pomodoro request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.pomodoro_url,
        "pomodoro",
        request,
        false, // No streaming for pomodoro
    ).await
}

/// Proxy requests to the kanban microservice
/// Handles: /api/kanban/*
pub async fn proxy_kanban(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("🗂️ Proxying kanban request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.kanban_url,
        "kanban",
        request,
        false, // No streaming for kanban
    ).await
}

/// Proxy requests to the note microservice
/// Handles: /api/notes/*
pub async fn proxy_note(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("📝 Proxying note request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.note_url,
        "note",
        request,
        false, // No streaming for notes
    ).await
}

/// Proxy requests to the docs microservice
/// Handles: /api/docs/*
pub async fn proxy_docs(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("📄 Proxying docs request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.docs_url,
        "docs",
        request,
        false, // No streaming for docs
    ).await
}

/// Proxy requests to the calendar microservice
/// Handles: /api/calendar/*
pub async fn proxy_calendar(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("🗓️ Proxying calendar request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.calendar_url,
        "calendar",
        request,
        false, // No streaming for calendar
    ).await
}

/// Proxy requests to the image microservice
/// Handles: /api/images/*
pub async fn proxy_image(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("🖼️ Proxying image request: {}", request.uri().path());
    
    proxy_to_service(
        &state.microservices_config.image_url,
        "image",
        request,
        false, // No streaming for images
    ).await
}
