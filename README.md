# Actor Framework Recipe 🦀

> **A production-ready, type-safe Actor Model implementation in Rust.**

This recipe demonstrates how to build a robust actor system using Tokio, leveraging Rust's type system to eliminate boilerplate and runtime errors. It is designed as a learning resource for engineers moving from "making it work" to "making it scalable and maintainable."

## 🏗 Architecture

[View Architecture Dependency Graph](architecture.md)

The system is built on three core pillars: **Type Safety**, **Separation of Concerns**, and **Developer Experience**.

### 1. The Core Abstraction (`src/actor_framework.rs`)
Instead of writing ad-hoc loops for every actor, we define a generic `ResourceActor<T>`.
-   **`Entity` Trait**: Defines *what* your actor manages (State).
-   **`ResourceActor`**: Defines *how* it runs (Runtime).
-   **`ResourceClient`**: Defines *how* you talk to it (Interface).

**Why?** This separates the *business logic* (your entity) from the *plumbing* (channels, message loops, error handling).

### 2. The Orchestrator (`src/app_system/`)
Actors don't exist in a vacuum. The `OrderSystem` acts as the "dependency injection container" and lifecycle manager.
-   It spins up all actors (`User`, `Product`, `Order`).
-   It wires them together (passing `UserClient` to `OrderClient`).
-   It handles graceful shutdown.

### 3. The Clients (`src/clients/`)
We don't expose raw message passing to the rest of the app. Instead, we wrap `ResourceClient` in domain-specific clients (e.g., `UserClient`).
-   **Macros**: We use `impl_basic_client!` to generate standard CRUD methods, keeping code DRY.
-   **Typed Errors**: We map generic framework errors to domain errors (`UserError`), so callers know exactly what went wrong.

---

## 🚀 Core Concepts

### Generics: The Power of `T`
You'll see `ResourceActor<T: Entity>` everywhere. This means "I can be an actor for *anything*, as long as it behaves like an Entity."
-   **Benefit**: We wrote the message processing loop **once**, and it works for Users, Products, and Orders.
-   **Trade-off**: The code looks more complex initially, but it saves thousands of lines of duplicate code in the long run.

### Macros: Reducing Boilerplate
Check out `src/clients/macros.rs`. We use `impl_basic_client!` to automatically write `get_user`, `delete_user`, etc.
-   **How it works**: It takes the type (`User`) and the error (`UserError`) and generates the implementation at compile time.
-   **Pro Tip**: Use `cargo expand` to see what the macro generates!

### Mocking: Testing without Pain
Testing actors can be hard because they are asynchronous. We solved this in `src/mock_framework.rs`.
-   **`create_mock_client`**: Gives you a real `ResourceClient` but connected to a test channel, not a real actor.
-   **`expect_...` helpers**: Allow you to intercept requests in your test and return fake responses.
-   **See**: `src/integration_tests.rs` for a real example.

---

## 📂 Project Structure

```text
src/
├── actor_framework.rs   # 🧠 The Brain: Generic Actor & Client implementation
├── mock_framework.rs    # 🧪 The Lab: Utilities for mocking actors in tests
├── main.rs              # 🏁 Entry Point: Runs the demo application
├── clients/             # 🔌 The Plugs: Type-safe wrappers for actors
│   ├── macros.rs        #    - Macros to generate client code
│   └── ...
├── domain/              # 📦 The Data: Pure data structures (User, Product, Order)
├── app_system/          # 🎼 The Conductor: System orchestration & shutdown
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
```

### Run Tests
```bash
cargo test
```

---

## 👩‍💻 Architecture Notes

1.  **Error Handling**: Notice `FrameworkError` vs `UserError`. We distinguish between "The actor system broke" (Framework) and "The user doesn't exist" (Domain). This is crucial for reliable systems.
2.  **Concurrency**: Each `ResourceActor` runs in its own Tokio task. They process messages sequentially (no locks needed for internal state!), but multiple actors run in parallel.
3.  **Observability**: We use `tracing` everywhere. In a distributed system, logs without correlation IDs are useless. This framework passes context automatically.

---

*Built with ❤️ for the Rust community.*
