//! =============================================================================
//! Domain Ports Module
//! =============================================================================
//! Contains abstract interfaces (traits) that define how the domain
//! communicates with external services. These are the "ports" in
//! hexagonal architecture.
//! 
//! # Ports vs Adapters
//! 
//! - **Ports** (this module): Abstract interfaces defined by the domain
//! - **Adapters** (infrastructure): Concrete implementations of ports
//! 
//! This separation allows the domain to remain independent of infrastructure
//! concerns like databases, HTTP clients, or external APIs.
//! =============================================================================

pub mod checklist_repository;
pub mod command_executor;
pub mod llm_provider;
pub mod memory_repository;
pub mod music_repository;
pub mod search_provider;
