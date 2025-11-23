#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128.png")]
#![doc(html_favicon_url = "https://www.rust-lang.org/favicon.ico")]
//! # Actor Framework Recipe
//!
//! > **A Recipe for Resource-oriented Actors in Rust.**
//!
//! This crate demonstrates a pattern for building clean, type-safe actor systems using Tokio.
//! It combines **Resource-Oriented Architecture (ROA)** with the **Actor Model** to provide
//! isolated state management with standard CRUD operations.
//!
//! ## 🏗️ Design Philosophy
//!
//! ### Why ROA + Actor Model?
//!
//! This framework combines two powerful concepts:
//! - **Resource-Oriented Architecture (ROA)**: Standard CRUD operations on well-defined resources.
//! - **Actor Model**: Isolated state with message-passing concurrency.
//!
//! This combination provides:
//! - **Independent Scaling**: Each actor runs in its own task.
//! - **Maintainability**: Clear separation of concerns.
//! - **Type Safety**: Compile-time guarantees for all operations.
//!
//! ## 🚀 Core Concepts
//!
//! ### Generics: The Power of `T`
//! You'll see `ResourceActor<T: ActorEntity>` everywhere. This means "I can be an actor for *anything*, as long as it behaves like an ActorEntity."
//! -   **Benefit**: We wrote the message processing loop **once**, and it works for Users, Products, and Orders.
//! -   **Trade-off**: The code looks more complex initially, but it saves thousands of lines of duplicate code in the long run.
//!
//! ### Mocking: Testing without Pain
//! Testing actors can be hard because they are asynchronous. We solved this with `MockClient`.
//! See the [`framework::mock`] module for a complete guide.
//!
//! ## 👩‍💻 Architecture Notes
//!
//! ### 1. Type-Safe Error Handling
//! Each actor defines its own error type (e.g., `UserError`, `ProductError`) that implements `std::error::Error`.
//! This enables pattern matching on specific error types and preserves error context throughout the system.
//! The `#[from]` attribute provides automatic error conversion for actors with dependencies.
//!
//! ### 2. Async Context Injection
//! Dependencies are injected at runtime via the `run()` method, not at construction time.
//! This "Late Binding" pattern solves circular dependencies and enables flexible actor wiring.
//!
//! ### 3. Concurrency Model
//! Each `ResourceActor` runs in its own Tokio task. They process messages sequentially (no locks needed for internal state!),
//! but multiple actors run in parallel.
//!
//! ### 4. Observability
//! We use `tracing` everywhere with structured logging. The framework automatically creates spans for each operation,
//! providing hierarchical context that's essential for debugging distributed systems.
//! See the [`lifecycle::tracing`] module for details.
//!
//! ## 🗺️ Module Tour
//!
//! The codebase is organized into four main layers. Here is your map:
//!
//! ### 1. The Engine ([`framework`])
//! This is the core of the system. It defines the generic `ResourceActor<T>` that powers
//! everything.
//! - **Role**: Separates the *business logic* (your entity) from the *plumbing* (channels, message loops, error handling).
//! - **Key items**: [`ActorEntity`](framework::ActorEntity), [`ResourceActor`](framework::ResourceActor).
//!
//! ### 2. The Orchestrator ([`lifecycle`])
//! Actors don't exist in a vacuum. The lifecycle module handles this.
//! - **Role**: Acts as the "dependency injection container" that spins up actors and wires them together.
//! - **Key items**: [`OrderSystem`](lifecycle::OrderSystem), [`shutdown`](lifecycle::OrderSystem::shutdown).
//!
//! ### 3. The Interface ([`clients`])
//! We don't expose raw message passing to the rest of the app.
//! - **Role**: Wraps the generic `ResourceClient` in domain-specific clients to provide type safety and hide implementation details.
//! - **Key items**: [`UserClient`](clients::UserClient), [`OrderClient`](clients::OrderClient).
//!
//! ### 4. The Implementation ([`user_actor`], [`product_actor`], [`order_actor`])
//! These are the actual domain actors built using the recipe.
//! - **Role**: Concrete implementations of the `ActorEntity` trait.
//!
//! ## 🚀 Quick Start
//!
//! If you are new here, start with the **[How-To Guide](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md)**.
//!
//! ### Running the Demo
//!
//! ```bash
//! # Run with info logs
//! RUST_LOG=info cargo run
//! ```
//!
//! ### Running Tests
//!
//! ```bash
//! cargo test
//! ```

pub mod clients;
pub mod framework;
pub mod lifecycle;
pub mod model;
pub mod order_actor;
pub mod product_actor;
pub mod user_actor;
