//! =============================================================================
//! Infrastructure Services Module
//! =============================================================================
//! Contains concrete implementations of external service adapters.
//! These implement the port traits defined in the domain layer.
//! =============================================================================

pub mod command_executor;
pub mod ollama;
pub mod openai;
pub mod searxng;
// pub mod voice_engine;  // Local ONNX implementation (deprecated)
pub mod voice_engine_http; // HTTP client for Docker voice service

pub use command_executor::SafeCommandExecutor;
pub use ollama::OllamaClient;
pub use openai::OpenAiClient;
pub use searxng::SearxngClient;
pub use voice_engine_http::{VoiceConfig, VoiceEngine, VoiceSynthesisRequest};
