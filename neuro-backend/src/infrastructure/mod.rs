//! =============================================================================
//! Infrastructure Layer - External Adapters
//! =============================================================================
//! This module contains the infrastructure adapters that implement the domain
//! ports. These are the concrete implementations for databases, external APIs,
//! and other infrastructure concerns.
//! 
//! # Architecture Role
//! 
//! The infrastructure layer provides concrete implementations of the abstract
//! ports defined in the domain layer:
//! 
//! - `SurrealDbRepository` implements `MemoryRepository`
//! - `OllamaClient` implements `LlmProvider`
//! - `SearxngClient` implements `SearchProvider`
//! - `LocalCommandExecutor` implements `CommandExecutor`
//! =============================================================================

pub mod api;
pub mod config;
pub mod database;
pub mod services;


