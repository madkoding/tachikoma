//! =============================================================================
//! API Handlers Module
//! =============================================================================
//! Contains HTTP handlers organized by domain.
//! =============================================================================

pub mod agent;
pub mod chat;
pub mod checklist;
pub mod graph;
pub mod kanban;
pub mod llm;
pub mod memory;
pub mod music;
pub mod proxy;
pub mod system;
pub mod voice;

pub use agent::*;
pub use chat::*;
pub use checklist::*;
pub use graph::*;
pub use kanban::*;
pub use llm::*;
pub use memory::*;
pub use music::*;
pub use proxy::*;
pub use system::*;
pub use voice::*;
