# Actor Framework Recipe

> **A Recipe for Resource-oriented Actors in Rust.**

This project demonstrates a pattern for building type-safe actor systems using Tokio. It's organized as a **Cargo Workspace** with two crates:

- **`actor-framework`** - Reusable actor framework library
- **`actor-sample`** - Example application demonstrating the framework

## Documentation

- **[Framework API Docs](target/doc/actor_framework/index.html)** - Run `cargo doc --open`
- **[Example App Docs](target/doc/actor_sample/index.html)** - Application documentation

## Project Structure

```text
crates/
├── actor-framework/     # Reusable Framework Library
│   ├── entity.rs        #   - ActorEntity trait
│   ├── actor.rs         #   - ResourceActor implementation
│   ├── client.rs        #   - ResourceClient implementation
│   ├── message.rs       #   - Message types
│   ├── error.rs         #   - Framework errors
│   └── mock.rs          #   - Testing utilities
│
└── actor-sample/        # Example Application
    ├── model/           #   - Domain models (User, Product, Order)
    ├── *_actor/         #   - Actor implementations
    ├── clients/         #   - Type-safe client wrappers
    ├── lifecycle/       #   - System orchestration
    └── main.rs          #   - Demo application
```

## Quick Start

### Running the Example

```bash
# Run with logging enabled
RUST_LOG=info cargo run -p actor-sample

# Or from workspace root
RUST_LOG=info cargo run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run framework tests only
cargo test -p actor-framework

# Run with output
cargo test -- --nocapture
```

### Viewing Documentation

```bash
# Generate and open docs for all crates
cargo doc --no-deps --open

# Framework docs only
cargo doc -p actor-framework --open
```
