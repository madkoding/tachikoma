//! =============================================================================
//! Infrastructure Services Module
//! =============================================================================
//! Contains concrete implementations of external service adapters.
//! These implement the port traits defined in the domain layer.
//! =============================================================================

pub mod ollama;
pub mod searxng;
pub mod command_executor;
// pub mod voice_engine;  // Local ONNX implementation (deprecated)
pub mod voice_engine_http;  // HTTP client for Docker voice service

pub use ollama::OllamaClient;
pub use searxng::SearxngClient;
pub use command_executor::SafeCommandExecutor;
pub use voice_engine_http::{VoiceEngine, VoiceConfig, VoiceSynthesisRequest};
