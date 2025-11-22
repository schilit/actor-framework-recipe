//! Generic actor framework for resource management.
//!
//! This module provides the core building blocks for creating type-safe actor systems
//! that manage resource entities with CRUD operations and custom actions.
//!
//! # Main Components
//!
//! - [`ActorEntity`] - Trait that resource types implement to be managed by actors
//! - [`ResourceActor`] - Generic actor that manages entities
//! - [`ResourceClient`] - Typub use core::{ActorEntity, ResourceActor, ResourceClient, ResourceRequest, FrameworkError};`] - Common error types
//!
//! # Testing
//!
//! See [`mock`] module for utilities to test clients without spawning full actors.

pub mod core;
pub mod mock;

// Re-export core types for convenience
pub use core::*;
