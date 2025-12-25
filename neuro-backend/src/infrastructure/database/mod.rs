//! =============================================================================
//! Database Module - SurrealDB Implementation
//! =============================================================================
//! Contains the SurrealDB adapter implementing the MemoryRepository port.
//! =============================================================================

mod pool;
mod surreal_checklist_repository;
mod surreal_music_repository;
mod surreal_repository;

pub use pool::DatabasePool;
pub use surreal_checklist_repository::SurrealChecklistRepository;
pub use surreal_music_repository::SurrealMusicRepository;
pub use surreal_repository::SurrealDbRepository;
