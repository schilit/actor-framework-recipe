//! Runtime orchestration and lifecycle management.
//!
//! This module contains the infrastructure for managing the application's runtime environment,
//! including:
//!
//! - **Actor lifecycle management**: Starting, wiring, and shutting down actors
//! - **System orchestration**: Coordinating dependencies between actors
//! - **Observability setup**: Initializing tracing and logging
//!
//! # Main Components
//!
//! - [`OrderSystem`] - The primary orchestrator that manages all actors and their dependencies
//! - [`setup_tracing`] - Initializes the tracing/logging infrastructure
//!
//! # Future Additions
//!
//! As the application grows, this module may include:
//! - Configuration management
//! - Health checks
//! - Metrics collection
//! - Graceful shutdown coordination
//! - Actor registry/discovery

pub mod order_system;
pub mod tracing;

pub use order_system::*;
pub use tracing::*;
