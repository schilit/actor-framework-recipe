# Actor Framework Recipe 🦀

> **A production-ready, type-safe Actor Model implementation in Rust.**

📚 **[View Full Documentation](https://schilit.github.io/actor-framework-recipe/)**

This recipe demonstrates how to build a robust actor system using Tokio, leveraging Rust's type system to eliminate boilerplate and runtime errors. It is designed as a learning resource for engineers moving from "making it work" to "making it scalable and maintainable."

## 🏗 Architecture

[View Architecture Dependency Graph](https://github.com/schilit/actor-framework-recipe/blob/main/architecture.md)

The system is built on three core pillars: **Type Safety**, **Separation of Concerns**, and **Developer Experience**.

### Why ROA + Actor Model?

This framework combines **Resource-Oriented Architecture (ROA)** with the **Actor Model** to create a powerful pattern for building scalable systems:

**Resource-Oriented Architecture (ROA)**:
- Standard CRUD operations (Create, Read, Update, Delete) on well-defined resources
- Predictable lifecycle management
- Clean, uniform API surface across all resource types

**Actor Model**:
- Isolated state (no shared memory, no locks)
- Message-passing concurrency
- Sequential processing within each actor eliminates race conditions

**The Synergy**:
- **Separation**: Each resource type (User, Product, Order) gets its own actor with completely isolated state
- **Coordination**: When resources need to interact (e.g., Order reserving Product stock), they communicate via **Action messages** instead of direct coupling
- **Scalability**: Independent resources can scale independently without coordination overhead
- **Maintainability**: Changes to one resource type don't ripple through the system

This pattern excels in systems with many loosely-coupled resources that occasionally need to coordinate. The ROA provides structure, while the Actor Model provides safe concurrency.

**Further Reading**:
- [Actor Model (Wikipedia)](https://en.wikipedia.org/wiki/Actor_model) - Foundational concurrency pattern by Carl Hewitt
- [Resource-Oriented Architecture](https://www.ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm) - Roy Fielding's dissertation on REST/ROA principles
- [Actors in Rust](https://ryhl.io/blog/actors-with-tokio/) - Practical guide to implementing actors with Tokio

### 1. The Core Abstraction (`src/framework/`)
Instead of writing ad-hoc loops for every actor, we define a generic `ResourceActor<T>`.
-   **`ActorEntity` Trait**: Defines *what* your actor manages (State + Behavior).
-   **`ResourceActor`**: Defines *how* it runs (Runtime with async context injection).
-   **`ResourceClient`**: Defines *how* you talk to it (Interface).

**Why?** This separates the *business logic* (your entity) from the *plumbing* (channels, message loops, error handling).

### 2. The Orchestrator (`src/lifecycle/`)
Actors don't exist in a vacuum. The `OrderSystem` acts as the "dependency injection container" and lifecycle manager.
-   It spins up all actors (`User`, `Product`, `Order`).
-   It wires them together via **context injection** (passing clients to `run()`).
-   It handles graceful shutdown.

### 3. The Clients (`src/clients/`)
We don't expose raw message passing to the rest of the app. Instead, we wrap `ResourceClient` in domain-specific clients (e.g., `UserClient`).
-   **Type Safety**: Each client provides strongly-typed methods for its domain
-   **Error Mapping**: We use **type-safe errors** (`UserError`, `ProductError`) instead of strings, enabling pattern matching and preserving error context.

---

## 🚀 Core Concepts

### Generics: The Power of `T`
You'll see `ResourceActor<T: ActorEntity>` everywhere. This means "I can be an actor for *anything*, as long as it behaves like an ActorEntity."
-   **Benefit**: We wrote the message processing loop **once**, and it works for Users, Products, and Orders.
-   **Trade-off**: The code looks more complex initially, but it saves thousands of lines of duplicate code in the long run.

### Mocking: Testing without Pain
Testing actors can be hard because they are asynchronous. We solved this in `src/framework/mock.rs`.
-   **`MockClient`**: Fluent expectation builder for readable tests
-   **`create_mock_client`**: Legacy helper for manual mocking
-   **`expect_...` helpers**: Allow you to intercept requests in your test and return fake responses.
-   **See**: `src/integration_tests.rs` for real examples.

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

The framework uses the `tracing` crate with a compact format that hides the crate/module prefix (`with_target(false)`). This keeps log lines short while still providing rich structured data.

### What Gets Traced
- **Actor Lifecycle**: Startup, shutdown, and final state
- **Entity Operations**: Create, Get, Update, Delete, and custom Actions
- **Request Flow**: Hierarchical spans showing the complete request path
- **Errors**: Detailed error context with entity IDs and failure reasons

### Debug Flag for Full Payload
When you run the application with `RUST_LOG=debug`, the `create_order` function logs the full `Order` payload **once** at the start of the request:
```rust
debug!(?order, "create_order called");
```
The `?` syntax is a `tracing` macro feature that records the variable using its `Debug` representation as a structured field.

Running with `RUST_LOG=debug` will show:
```
DEBUG create_order called order={...}
INFO order_processing:create_order: Processing create_order request (Client Side)
```
All subsequent logs remain concise, showing only the workflow hierarchy.

### Usage Examples
```bash
# Compact logs (default)
RUST_LOG=info cargo run

# Show full Order payload once
RUST_LOG=debug cargo run

# Very verbose tracing
RUST_LOG=trace cargo run
```

### Workflow Trace Example

The tracing output shows the complete order creation workflow with hierarchical spans. Here's what you'll see when creating an order:

```
INFO Sending create_order to actor
INFO Created user_id="user_1" size=1
INFO Created product_id="product_1" size=1
INFO Action ok product_id="product_1"
INFO Created order_id="order_1" size=1
```

**With `RUST_LOG=debug`**, you'll see the full Order payload and validation flow:

```
DEBUG create_order called order=Order { id: "", user_id: "user_1", product_id: "product_1", quantity: 3, total: 75.0 }
INFO Sending create_order to actor
DEBUG Get user_id="user_1"
INFO Created user_id="user_1" size=1
DEBUG Get product_id="product_1"
INFO Created product_id="product_1" size=1
DEBUG Action product_id="product_1" action=ReserveStock(3)
INFO Action ok product_id="product_1"
DEBUG Create params=OrderCreate { user_id: "user_1", product_id: "product_1", quantity: 3, total: 75.0 }
INFO Created order_id="order_1" size=1
```

**Key Observations**:
1. **User Validation** → `Get user_id="user_1"` → User found in actor
2. **Product Validation** → `Get product_id="product_1"` → Product found
3. **Stock Reservation** → `Action...ReserveStock(3)` → Stock reserved (happens in `Order::on_create`)
4. **Order Creation** → `Create params=OrderCreate{...}` → Order created

Each step is traced with structured fields that can be filtered and analyzed in production logging systems.

---

## 👩‍💻 Architecture Notes

1.  **Type-Safe Error Handling**: Each actor defines its own error type (e.g., `UserError`, `ProductError`) that implements `std::error::Error`. This enables pattern matching on specific error types and preserves error context throughout the system. The `#[from]` attribute provides automatic error conversion for actors with dependencies.
2.  **Async Context Injection**: Dependencies are injected at runtime via the `run()` method, not at construction time. This "Late Binding" pattern solves circular dependencies and enables flexible actor wiring.
3.  **Concurrency**: Each `ResourceActor` runs in its own Tokio task. They process messages sequentially (no locks needed for internal state!), but multiple actors run in parallel.
4.  **Observability**: We use `tracing` everywhere with structured logging. The framework automatically creates spans for each operation, providing hierarchical context that's essential for debugging distributed systems.

---

*Built with ❤️ for the Rust community.*
