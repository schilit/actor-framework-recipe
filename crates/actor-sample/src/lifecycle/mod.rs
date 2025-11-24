//! # System Lifecycle & Orchestration
//!
//! This module manages the runtime lifecycle of actor-based systems, handling the complex
//! coordination of starting, wiring, and shutting down multiple interdependent actors.
//!
//! ## The Orchestration Pattern
//!
//! In actor systems, individual actors are simple, but **wiring them together** is where
//! complexity lives. This module provides the "conductor" that coordinates the entire system.
//!
//! **Key Responsibilities:**
//! 1. **Actor Creation** - Instantiate all actors and their clients
//! 2. **Dependency Injection** - Wire actors together via context injection
//! 3. **Lifecycle Management** - Start actors in the correct order
//! 4. **Graceful Shutdown** - Coordinate clean termination of all actors
//! 5. **Observability Setup** - Initialize tracing and logging infrastructure
//!
//! ## The OrderSystem Pattern
//!
//! The [`OrderSystem`] demonstrates a complete lifecycle orchestrator:
//!
//! ```rust,ignore
//! impl OrderSystem {
//!     pub fn new() -> Self {
//!         // 1. Create actors (no dependencies yet - avoids circular refs)
//!         let (user_actor, user_client) = user_actor::new();
//!         let (product_actor, product_client) = product_actor::new();
//!         let (order_actor, order_client) = order_actor::new();
//!
//!         // 2. Start actors with their dependencies injected
//!         let user_handle = tokio::spawn(user_actor.run(()));
//!         let product_handle = tokio::spawn(product_actor.run(()));
//!         let order_handle = tokio::spawn(
//!             order_actor.run((user_client.clone(), product_client.clone()))
//!         );
//!
//!         Self {
//!             user_client,
//!             product_client,
//!             order_client,
//!             handles: vec![user_handle, product_handle, order_handle],
//!         }
//!     }
//!
//!     pub async fn shutdown(self) {
//!         // Drop clients to signal shutdown, then await all actors
//!         drop(self.user_client);
//!         drop(self.product_client);
//!         drop(self.order_client);
//!         
//!         for handle in self.handles {
//!             let _ = handle.await;
//!         }
//!     }
//! }
//! ```
//!
//! ## Dependency Injection via Context
//!
//! The framework uses **late binding** to solve circular dependency problems:
//!
//! - **Construction time**: Create actors without dependencies
//! - **Runtime**: Inject dependencies via `run(context)`
//!
//! This pattern allows `Order` to depend on `User` and `Product` without creating
//! circular references during construction.
//!
//! Each actor defines its `Context` associated type:
//!
//! ```rust,ignore
//! // No dependencies
//! impl ActorEntity for User {
//!     type Context = ();
//! }
//!
//! // Depends on User and Product clients
//! impl ActorEntity for Order {
//!     type Context = (UserClient, ProductClient);
//! }
//! ```
//!
//! ## Graceful Shutdown
//!
//! The shutdown pattern follows these steps:
//!
//! 1. **Drop all clients** - Closes the sender side of channels
//! 2. **Actors detect closure** - `receiver.recv()` returns `None`
//! 3. **Actors clean up** - Process remaining messages, log final state
//! 4. **Await completion** - Wait for all actor tasks to finish
//!
//! This ensures no messages are lost and all actors terminate cleanly.
//!
//! **With Context Dependencies:** When actors hold clients in their context (e.g., `Order` actor has `UserClient` and
//! `ProductClient`), those clients are clones and won't prevent shutdown as long as the
//! dependency graph is **acyclic**. Each actor shuts down when its own channel closes.
//!
//! **For cyclic dependencies**: Use an explicit `Shutdown` action instead of relying on
//! channel closure. This ensures deterministic shutdown order regardless of dependency structure.
//!
//! ## Observability & Tracing
//!
//! The [`setup_tracing`] function initializes structured logging for the entire system.
//!
//! The framework uses the `tracing` crate with hierarchical spans to trace:
//! - Actor lifecycle events (startup, shutdown)
//! - Entity operations (Create, Get, Update, Delete, Actions)
//! - Request flows with complete context
//! - Errors with detailed entity IDs
//!
//! **Usage:**
//! ```bash
//! RUST_LOG=info cargo run      # Compact logs
//! RUST_LOG=debug cargo run     # Full payloads
//! ```
//!
//! 📖 **For complete tracing documentation**, see the [`tracing`] module with detailed
//! examples, workflow traces, and best practices.
//!
//! ## Future Extensions
//!
//! As systems grow, this module may include:
//!
//! - Configuration management (loading from files/env)
//! - Health checks and readiness probes
//! - Metrics collection and export
//! - Actor registry for dynamic discovery
//! - Hot reload and zero-downtime updates

pub mod order_system;
pub mod tracing;

pub use order_system::*;
pub use tracing::*;
