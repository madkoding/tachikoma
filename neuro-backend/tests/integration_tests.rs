//! Integration tests for tachikoma-backend API
//! 
//! These tests verify the HTTP API endpoints work correctly.
//! They require a running SurrealDB instance and use in-memory storage.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::{Service, ServiceExt};

// Import from the main crate
use tachikoma_backend::{
    application::services::{ChatService, MemoryService, ModelManager},
    domain::ports::{
        llm_provider::LlmProvider,
        memory_repository::MemoryRepository,
        search_provider::SearchProvider,
    },
    infrastructure::{
        api::create_router,
        config::Config,
        database::{DatabasePool, SurrealDbRepository},
        services::{OllamaClient, SearxngClient},
    },
};

/// Helper to create a test API router
async fn create_test_app() -> Router {
    let config = Config::init().unwrap_or_else(|_| Config {
        port: 3000,
        db_url: "mem://test".to_string(),
        ollama_url: "http://localhost:11434".to_string(),
        searxng_url: "http://localhost:8080".to_string(),
        voice_url: "http://localhost:8100".to_string(),
    });

    let pool = DatabasePool::new(&config.db_url).await.unwrap();
    
    let repo = Arc::new(SurrealDbRepository::new(pool.clone()));
    let llm = Arc::new(OllamaClient::new(&config.ollama_url));
    let search = Arc::new(SearxngClient::new(&config.searxng_url));
    
    let memory_service = Arc::new(MemoryService::new(repo.clone()));
    let chat_service = Arc::new(ChatService::new(repo.clone(), llm.clone(), memory_service.clone()));
    let model_manager = Arc::new(ModelManager::new(llm.clone()));

    create_router(
        config,
        pool,
        repo,
        llm,
        search,
        chat_service,
        memory_service,
        model_manager,
    )
}

/// Helper to make HTTP requests to the test app
async fn make_request<S>(
    app: &mut S,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, String)
where
    S: Service<Request<Body>, Response = Router> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
{
    let req = match body {
        Some(json) => Request::builder()
            .method(method)
            .uri(path)
            .header("Content-Type", "application/json")
            .body(Body::from(json.to_string()))
            .unwrap(),
        None => Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap(),
    };

    let response = app.ready().await.unwrap().call(req).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body).to_string();

    (status, body_str)
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let mut app = create_test_app().await;
    
    let (status, body) = make_request(&mut app, "GET", "/api/health", None).await;
    
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert!(json["timestamp"].is_string());
}

#[tokio::test]
async fn test_ping_endpoint() {
    let mut app = create_test_app().await;
    
    let (status, body) = make_request(&mut app, "GET", "/api/ping", None).await;
    
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert!(json["uptime"].is_number());
}

// ============================================================================
// Memory API Tests
// ============================================================================

#[tokio::test]
async fn test_create_memory() {
    let mut app = create_test_app().await;
    
    let memory_data = json!({
        "content": "Test memory content",
        "memory_type": "note",
        "tags": ["test", "integration"]
    });
    
    let (status, body) = make_request(
        &mut app,
        "POST",
        "/api/memory",
        Some(memory_data),
    )
    .await;
    
    assert_eq!(status, StatusCode::CREATED);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert!(json["id"].is_string());
    assert_eq!(json["content"], "Test memory content");
    assert_eq!(json["memory_type"], "note");
}

#[tokio::test]
async fn test_get_memories() {
    let mut app = create_test_app().await;
    
    // First create a memory
    let memory_data = json!({
        "content": "Memory for list test",
        "memory_type": "note"
    });
    let _ = make_request(&mut app, "POST", "/api/memory", Some(memory_data)).await;
    
    // Then get all memories
    let (status, body) = make_request(&mut app, "GET", "/api/memory", None).await;
    
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert!(json.is_array());
}

#[tokio::test]
async fn test_search_memories() {
    let mut app = create_test_app().await;
    
    // Create a memory with specific content
    let memory_data = json!({
        "content": "This is a searchable test memory",
        "memory_type": "note",
        "tags": ["searchable"]
    });
    let _ = make_request(&mut app, "POST", "/api/memory", Some(memory_data)).await;
    
    // Search for it
    let (status, body) = make_request(
        &mut app,
        "GET",
        "/api/memory/search?q=searchable",
        None,
    )
    .await;
    
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert!(json.is_array());
}

// ============================================================================
// Chat API Tests
// ============================================================================

#[tokio::test]
async fn test_create_chat_message() {
    let mut app = create_test_app().await;
    
    let message_data = json!({
        "content": "Hello, this is a test message",
        "role": "user"
    });
    
    let (status, body) = make_request(
        &mut app,
        "POST",
        "/api/chat",
        Some(message_data),
    )
    .await;
    
    // Note: This might fail if Ollama is not running, so we accept multiple status codes
    if status == StatusCode::OK {
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["id"].is_string());
        assert_eq!(json["role"], "assistant");
    }
    // If service unavailable, that's also OK for integration test
    assert!(status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE);
}

// ============================================================================
// LLM Gateway Tests
// ============================================================================

#[tokio::test]
async fn test_llm_generate() {
    let mut app = create_test_app().await;
    
    let generate_data = json!({
        "prompt": "Say hello",
        "model": "qwen3:0.6b"
    });
    
    let (status, body) = make_request(
        &mut app,
        "POST",
        "/api/llm/generate",
        Some(generate_data),
    )
    .await;
    
    // This requires Ollama to be running
    if status == StatusCode::OK {
        let json: Value = serde_json::from_str(&body).unwrap();
        assert!(json["response"].is_string());
    }
    // Accept service unavailable if Ollama is not running
    assert!(status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE);
}

// ============================================================================
// System Info Tests
// ============================================================================

#[tokio::test]
async fn test_system_info() {
    let mut app = create_test_app().await;
    
    let (status, body) = make_request(&mut app, "GET", "/api/system/info", None).await;
    
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_str(&body).unwrap();
    assert!(json["platform"].is_string());
    assert!(json["arch"].is_string());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_404_on_unknown_route() {
    let mut app = create_test_app().await;
    
    let (status, _) = make_request(&mut app, "GET", "/api/unknown/route", None).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_json_returns_400() {
    let mut app = create_test_app().await;
    
    let (status, _) = make_request(
        &mut app,
        "POST",
        "/api/memory",
        Some(json!("invalid json")),
    )
    .await;
    
    // Should return bad request for invalid memory data
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY);
}
