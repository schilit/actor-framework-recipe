# Actor Framework Recipe 🦀

> **A Recipe for Resource-oriented Actors in Rust.**

📚 **[View Full Documentation](https://schilit.github.io/actor-framework-recipe/)**

This recipe demonstrates a pattern for building clean actor systems using Tokio, leveraging Rust's type system to eliminate boilerplate and runtime errors. It is designed as a learning resource for engineers moving from "making it work" to "making it type-safe and maintainable."

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

# Filter to specific modules
RUST_LOG=actor_recipe::framework=debug cargo run
```

### Run Tests
```bash
cargo test
```

---

*Built with ❤️ for the Rust community.*
