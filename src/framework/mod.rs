//! # Actor Framework Core
//!
//! This module provides the foundational building blocks for creating type-safe, concurrent
//! actor systems in Rust. It implements a **Resource-Oriented Architecture (ROA)** pattern
//! on top of the **Actor Model**, providing a clean abstraction for managing stateful entities.
//!
//! ## Why ROA + Actor Model?
//!
//! This framework combines **Resource-Oriented Architecture (ROA)** with the **Actor Model**
//! to create a powerful pattern for building scalable systems.
//!
//! ### Resource-Oriented Architecture (ROA)
//!
//! - Standard CRUD operations (Create, Read, Update, Delete) on well-defined resources
//! - Predictable lifecycle management
//! - Clean, uniform API surface across all resource types
//!
//! ### Actor Model
//!
//! - Isolated state (no shared memory, no locks)
//! - Message-passing concurrency
//! - Sequential processing within each actor eliminates race conditions
//!
//! ### The Synergy
//!
//! - **Separation**: Each resource type (User, Product, Order) gets its own actor with completely isolated state
//! - **Coordination**: When resources need to interact (e.g., Order reserving Product stock), they communicate via **Action messages** instead of direct coupling
//! - **Scalability**: Independent resources can scale independently without coordination overhead
//! - **Maintainability**: Changes to one resource type don't ripple through the system
//!
//! This pattern excels in systems with many loosely-coupled resources that occasionally need
//! to coordinate. The ROA provides structure, while the Actor Model provides safe concurrency.
//!
//! **Further Reading**:
//! - [Actor Model (Wikipedia)](https://en.wikipedia.org/wiki/Actor_model) - Foundational concurrency pattern by Carl Hewitt
//! - [Resource-Oriented Architecture](https://www.ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm) - Roy Fielding's dissertation on REST/ROA principles
//! - [Actors in Rust](https://ryhl.io/blog/actors-with-tokio/) - Practical guide to implementing actors with Tokio
//!
//! ## Architecture Overview
//!
//! The framework separates concerns into three layers:
//!
//! 1. **Entity Layer** ([`ActorEntity`]) - Your business logic and domain models
//! 2. **Runtime Layer** ([`ResourceActor`]) - Message processing and concurrency
//! 3. **Interface Layer** ([`ResourceClient`]) - Type-safe communication
//!
//! This separation means you write your business logic **once** in the entity trait,
//! and the framework handles all the async message passing, error handling, and state management.
//!
//! ## Core Abstractions
//!
//! ### [`ActorEntity`] - The Business Logic
//!
//! Define what your actor manages and how it behaves:
//!
//! ```rust,ignore
//! #[async_trait]
//! impl ActorEntity for User {
//!     type Id = String;
//!     type Create = UserCreate;
//!     type Update = UserUpdate;
//!     type Action = ();
//!     type ActionResult = ();
//!     type Context = ();
//!     type Error = UserError;
//!
//!     fn from_create_params(id: String, params: UserCreate) -> Result<Self, Self::Error> {
//!         Ok(Self { id, name: params.name, email: params.email })
//!     }
//!
//!     async fn on_update(&mut self, update: UserUpdate, _ctx: &Self::Context) 
//!         -> Result<(), Self::Error> 
//!     {
//!         if let Some(name) = update.name { self.name = name; }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### [`ResourceActor`] - The Runtime
//!
//! A generic actor that manages any `T: ActorEntity`. You never need to write message loops
//! or handle channels manually:
//!
//! ```rust,ignore
//! let (actor, client) = ResourceActor::new(100, || format!("user_{}", Uuid::new_v4()));
//! tokio::spawn(actor.run(()));  // Context injection happens here
//! ```
//!
//! ### [`ResourceClient`] - The Interface
//!
//! Type-safe communication with actors. All methods are async and return `Result`:
//!
//! ```rust,ignore
//! let id = client.create(user_params).await?;
//! let user = client.get(id.clone()).await?;
//! client.update(id.clone(), update).await?;
//! client.delete(id).await?;
//! ```
//!
//! ## Context Injection Pattern
//!
//! Dependencies are injected at **runtime** via the `run()` method, not at construction time.
//! This "late binding" pattern solves circular dependencies:
//!
//! ```rust,ignore
//! // 1. Create all actors (no dependencies yet)
//! let (user_actor, user_client) = user_actor::new();
//! let (product_actor, product_client) = product_actor::new();
//! let (order_actor, order_client) = order_actor::new();
//!
//! // 2. Wire dependencies when starting actors
//! tokio::spawn(user_actor.run(()));  // User has no dependencies
//! tokio::spawn(product_actor.run(()));  // Product has no dependencies
//! tokio::spawn(order_actor.run((user_client.clone(), product_client.clone())));
//! ```
//!
//! The `Order` actor receives `(UserClient, ProductClient)` as its context, allowing it to
//! validate users and reserve product stock during order creation.
//!
//! ## Type Safety
//!
//! The framework leverages Rust's type system to eliminate entire classes of runtime errors:
//!
//! - **Compile-time guarantees**: Can't send wrong message types to actors
//! - **Type-safe errors**: Each entity defines its own error type
//! - **No stringly-typed APIs**: IDs, actions, and results are all strongly typed
//!
//! ## Concurrency Model
//!
//! - Each actor runs in its own Tokio task
//! - Messages are processed **sequentially** within an actor (no locks needed!)
//! - Multiple actors run in **parallel** (true concurrency)
//! - No shared mutable state (message passing only)
//!
//! ## Testing
//!
//! See the [`mock`] module for comprehensive testing utilities and patterns.

pub mod core;
pub mod mock;

// Re-export core types for convenience
pub use core::*;
