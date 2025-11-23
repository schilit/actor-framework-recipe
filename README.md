# Actor Framework Recipe 🦀

> **A production-ready, type-safe Actor Model implementation in Rust.**

📚 **[View Full Documentation](https://schilit.github.io/actor-framework-recipe/)**

This recipe demonstrates how to build a robust actor system using Tokio, leveraging Rust's type system to eliminate boilerplate and runtime errors. It is designed as a learning resource for engineers moving from "making it work" to "making it scalable and maintainable."

## 🏗 Architecture

[View Architecture Dependency Graph](./architecture.md)

The system is built on three core pillars: **Type Safety**, **Separation of Concerns**, and **Developer Experience**.

### Why ROA + Actor Model?

This framework combines **Resource-Oriented Architecture (ROA)** with the **Actor Model** to provide:
- Standard CRUD operations on well-defined resources (ROA)
- Isolated state with message-passing concurrency (Actor Model)
- Independent scaling and maintainability

📖 **[Read the complete architectural rationale](https://schilit.github.io/actor-framework-recipe/actor_recipe/framework/index.html#why-roa--actor-model)** in the framework module documentation.

### 1. The Core Abstraction ([`framework`](src/framework/mod.rs))
Instead of writing ad-hoc loops for every actor, we define a generic `ResourceActor<T>`.
-   **`ActorEntity` Trait**: Defines *what* your actor manages (State + Behavior).
-   **`ResourceActor`**: Defines *how* it runs (Runtime with async context injection).
-   **`ResourceClient`**: Defines *how* you talk to it (Interface).

**Why?** This separates the *business logic* (your entity) from the *plumbing* (channels, message loops, error handling).

📖 **[View framework documentation](https://schilit.github.io/actor-framework-recipe/actor_recipe/framework/index.html)** - Architecture overview, context injection, concurrency model

### 2. The Orchestrator ([`lifecycle`](src/lifecycle/mod.rs))
Actors don't exist in a vacuum. The `OrderSystem` acts as the "dependency injection container" and lifecycle manager.
-   It spins up all actors (`User`, `Product`, `Order`).
-   It wires them together via **context injection** (passing clients to `run()`).
-   It handles graceful shutdown.

📖 **[View lifecycle documentation](https://schilit.github.io/actor-framework-recipe/actor_recipe/lifecycle/index.html)** - Orchestration patterns, dependency injection, graceful shutdown

### 3. The Clients ([`clients`](src/clients/mod.rs))
We don't expose raw message passing to the rest of the app. Instead, we wrap `ResourceClient` in domain-specific clients (e.g., `UserClient`).
-   **Type Safety**: Each client provides strongly-typed methods for its domain
-   **Error Mapping**: We use **type-safe errors** (`UserError`, `ProductError`) instead of strings, enabling pattern matching and preserving error context.

📖 **[View client documentation](https://schilit.github.io/actor-framework-recipe/actor_recipe/clients/index.html)** - Wrapper pattern, type-safe errors, orchestration examples

---

## 🚀 Core Concepts

### Generics: The Power of `T`
You'll see `ResourceActor<T: ActorEntity>` everywhere. This means "I can be an actor for *anything*, as long as it behaves like an ActorEntity."
-   **Benefit**: We wrote the message processing loop **once**, and it works for Users, Products, and Orders.
-   **Trade-off**: The code looks more complex initially, but it saves thousands of lines of duplicate code in the long run.

📖 **[View framework module](https://schilit.github.io/actor-framework-recipe/actor_recipe/framework/index.html)** for detailed documentation

### Mocking: Testing without Pain
Testing actors can be hard because they are asynchronous. We solved this in `src/framework/mock.rs`.
-   **`MockClient`**: Fluent expectation builder for readable tests
-   **`create_mock_client`**: Legacy helper for manual mocking
-   **`expect_...` helpers**: Allow you to intercept requests in your test and return fake responses.
-   **See**: `src/integration_tests.rs` for real examples.

📖 **[View testing guide](https://schilit.github.io/actor-framework-recipe/actor_recipe/framework/mock/index.html)** - Complete testing patterns and examples

---

## 📂 Project Structure

```text
src/
├── framework/           # 🧠 The Brain: Generic Actor & Client implementation
│   ├── core.rs          #    - ResourceActor, ActorEntity trait, message types
│   └── mock.rs          #    - Testing utilities and mocks
├── lifecycle/           # 🎼 The Conductor: System orchestration & lifecycle
│   ├── order_system.rs  #    - Actor wiring and dependency injection
│   └── tracing.rs       #    - Observability setup
├── main.rs              # 🏁 Entry Point: Runs the demo application
├── clients/             # 🔌 The Plugs: Type-safe wrappers for actors
│   ├── actor_client.rs  #    - ActorClient trait (common interface)
│   ├── user_client.rs   #    - UserClient implementation
│   ├── product_client.rs#    - ProductClient implementation
│   └── order_client.rs  #    - OrderClient implementation
├── model/               # 📦 The Data: Pure data structures (User, Product, Order)
├── user_actor/          # 👤 User Domain Logic
│   ├── entity.rs        #    - ActorEntity implementation for User
│   ├── error.rs         #    - UserError type (type-safe errors)
│   └── mod.rs           #    - Module exports and factory function
├── product_actor/       # 📦 Product Domain Logic
│   ├── entity.rs        #    - ActorEntity implementation for Product
│   ├── error.rs         #    - ProductError type
│   ├── actions.rs       #    - Custom actions (CheckStock, ReserveStock)
│   └── mod.rs           #    - Module exports and factory function
├── order_actor/         # 🛒 Order Domain Logic
│   ├── entity.rs        #    - ActorEntity implementation with validation
│   ├── error.rs         #    - OrderError type (with #[from] conversions)
│   └── mod.rs           #    - Module exports and factory function
└── integration_tests.rs # ✅ End-to-End Tests
```

## 📚 How-To Guide

**New to the framework?** Check out the [**How-To Guide**](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md) for step-by-step tutorials:

- [**How to Add a New Actor**](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md#how-to-add-a-new-actor) - Walk through creating a `User` actor from scratch
- [**How to Add Custom Actions**](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md#how-to-add-a-custom-action) - Learn by example with `Product`'s stock management
- [**How to Write Tests**](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md#how-to-write-tests) - Master the mock framework with real examples
- [**How to Handle Errors**](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md#how-to-handle-errors) - Type-safe error handling patterns
- [**Understanding Lifecycle Hooks**](https://github.com/schilit/actor-framework-recipe/blob/main/HOWTO.md#understanding-lifecycle-hooks) - When to use `on_create` and `on_delete`

## 🛠 Usage

### Run the Demo
```bash
# Run with info logs
RUST_LOG=info cargo run

# Run with debug logs to see the actor internals
RUST_LOG=debug cargo run

# Run with trace logs (very verbose)
RUST_LOG=trace cargo run

# Filter to specific modules
RUST_LOG=actor_recipe::framework=debug cargo run
```

### Run Tests
```bash
cargo test
```

---

## 🔍 Observability & Tracing

The framework uses the `tracing` crate for structured logging with hierarchical spans. The system traces actor lifecycle events, entity operations, request flows, and errors with detailed context.

### Quick Start

```bash
# Compact logs (default)
RUST_LOG=info cargo run

# Show full payloads with debug logs
RUST_LOG=debug cargo run

# Filter to specific modules
RUST_LOG=actor_recipe::framework=debug cargo run
```

📖 **[View complete tracing documentation](https://schilit.github.io/actor-framework-recipe/actor_recipe/lifecycle/tracing/index.html)** - Detailed examples, workflow traces, and best practices

---

## 👩‍💻 Architecture Notes

1.  **Type-Safe Error Handling**: Each actor defines its own error type (e.g., `UserError`, `ProductError`) that implements `std::error::Error`. This enables pattern matching on specific error types and preserves error context throughout the system. The `#[from]` attribute provides automatic error conversion for actors with dependencies.
2.  **Async Context Injection**: Dependencies are injected at runtime via the `run()` method, not at construction time. This "Late Binding" pattern solves circular dependencies and enables flexible actor wiring.
3.  **Concurrency**: Each `ResourceActor` runs in its own Tokio task. They process messages sequentially (no locks needed for internal state!), but multiple actors run in parallel.
4.  **Observability**: We use `tracing` everywhere with structured logging. The framework automatically creates spans for each operation, providing hierarchical context that's essential for debugging distributed systems.

---

*Built with ❤️ for the Rust community.*
