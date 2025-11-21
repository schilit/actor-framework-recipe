//! Generic actor framework for resource management.
//!
//! This module provides the core building blocks for creating type-safe actor systems
//! that manage domain entities with CRUD operations and custom actions.
//!
//! # Main Components
//!
//! - [`Entity`] - Trait that domain types implement to be managed by actors
//! - [`ResourceActor`] - Generic actor that manages entities
//! - [`ResourceClient`] - Type-safe client for communicating with actors
//! - [`FrameworkError`] - Common error types
//!
//! # Testing
//!
//! See [`mock`] module for utilities to test clients without spawning full actors.

pub mod core;
pub mod mock;

// Re-export core types for convenience
pub use core::*;
