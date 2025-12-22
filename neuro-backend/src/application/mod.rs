//! =============================================================================
//! Application Layer - Use Cases and Services
//! =============================================================================
//! This module contains the application services that orchestrate domain
//! operations. These services implement use cases by coordinating between
//! domain entities and infrastructure adapters.
//! 
//! # Architecture Role
//! 
//! The application layer sits between the infrastructure (HTTP handlers, DB)
//! and the domain layer. It:
//! 
//! 1. Receives requests from infrastructure (API handlers)
//! 2. Coordinates domain operations
//! 3. Uses ports to interact with external services
//! 4. Returns results to infrastructure
//! =============================================================================

pub mod services;
