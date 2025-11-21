# Actor Framework Recipe 🦀

> **A production-ready, type-safe Actor Model implementation in Rust.**

This recipe demonstrates how to build a robust actor system using Tokio, leveraging Rust's type system to eliminate boilerplate and runtime errors. It is designed as a learning resource for engineers moving from "making it work" to "making it scalable and maintainable."

## 🏗 Architecture

[View Architecture Dependency Graph](architecture.md)

The system is built on three core pillars: **Type Safety**, **Separation of Concerns**, and **Developer Experience**.

### 1. The Core Abstraction (`src/framework/`)
Instead of writing ad-hoc loops for every actor, we define a generic `ResourceActor<T>`.
-   **`Entity` Trait**: Defines *what* your actor manages (State).
-   **`ResourceActor`**: Defines *how* it runs (Runtime).
-   **`ResourceClient`**: Defines *how* you talk to it (Interface).

**Why?** This separates the *business logic* (your entity) from the *plumbing* (channels, message loops, error handling).

### 2. The Orchestrator (`src/lifecycle/`)
Actors don't exist in a vacuum. The `OrderSystem` acts as the "dependency injection container" and lifecycle manager.
-   It spins up all actors (`User`, `Product`, `Order`).
-   It wires them together (passing `UserClient` to `OrderClient`).
-   It handles graceful shutdown.

### 3. The Clients (`src/clients/`)
We don't expose raw message passing to the rest of the app. Instead, we wrap `ResourceClient` in domain-specific clients (e.g., `UserClient`).
-   **Type Safety**: Each client provides strongly-typed methods for its domain
-   **Error Mapping**: We map generic framework errors to domain errors (`UserError`), so callers know exactly what went wrong.

---

## 🚀 Core Concepts

### Generics: The Power of `T`
You'll see `ResourceActor<T: Entity>` everywhere. This means "I can be an actor for *anything*, as long as it behaves like an Entity."
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
│   ├── core.rs          #    - ResourceActor, Entity trait, message types
│   └── mock.rs          #    - Testing utilities and mocks
├── lifecycle/           # 🎼 The Conductor: System orchestration & lifecycle
│   ├── order_system.rs  #    - Actor wiring and dependency injection
│   └── tracing.rs       #    - Observability setup
├── main.rs              # 🏁 Entry Point: Runs the demo application
├── clients/             # 🔌 The Plugs: Type-safe wrappers for actors
│   ├── traits.rs        #    - DomainClient trait
│   └── ...
├── domain/              # 📦 The Data: Pure data structures (User, Product, Order)
├── user_actor/          # 👤 User Domain Logic
├── product_actor/       # 📦 Product Domain Logic
├── order_actor/         # 🛒 Order Domain Logic
└── integration_tests.rs # ✅ End-to-End Tests
```

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

The tracing output shows the complete order creation workflow:

1. **User Validation** → Actor Get request → User found
2. **Product Validation** → Actor Get request → Product found
3. **Stock Reservation** → Actor Action request → Stock reserved
4. **Order Creation** → Actor Create request → Order created

Each step is traced with structured fields (`id`, `store_size`, `found`, etc.) that can be filtered and analyzed in production logging systems.

---

## 👩‍💻 Architecture Notes

1.  **Error Handling**: Notice `FrameworkError` vs `UserError`. We distinguish between "The actor system broke" (Framework) and "The user doesn't exist" (Domain). This is crucial for reliable systems.
2.  **Concurrency**: Each `ResourceActor` runs in its own Tokio task. They process messages sequentially (no locks needed for internal state!), but multiple actors run in parallel.
3.  **Observability**: We use `tracing` everywhere with structured logging. The framework automatically creates spans for each operation, providing hierarchical context that's essential for debugging distributed systems.

---

*Built with ❤️ for the Rust community.*
