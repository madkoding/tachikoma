//! =============================================================================
//! Database Module - SurrealDB Implementation
//! =============================================================================
//! Contains the SurrealDB adapter implementing the MemoryRepository port.
//! =============================================================================

mod pool;
mod surreal_repository;

pub use pool::DatabasePool;
pub use surreal_repository::SurrealDbRepository;
