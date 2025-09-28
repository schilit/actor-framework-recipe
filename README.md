# Actor Recipe for Rust

[![CI](https://github.com/schilit/actor-recipe/actions/workflows/ci.yml/badge.svg)](https://github.com/schilit/actor-recipe/actions/workflows/ci.yml)
[![Documentation](https://github.com/schilit/actor-recipe/actions/workflows/docs.yml/badge.svg)](https://schilit.github.io/actor-recipe/)

A recipe for building actor systems with minimal boilerplate and good observability.

## Overview

This project demonstrates a complete actor system implementation in Rust, featuring:

- **80% less boilerplate** - Macro-generated client methods with automatic error handling
- **Professional observability** - Request correlation across actors with timing
- **Clean architecture** - Domain-specific actors with clear separation of concerns
- **Type-safe error handling** - Domain-specific error types (UserError, ProductError, OrderError)
- **Test-friendly** - Test-only messages for inspecting internal actor state
- **Production-ready** - Error handling, graceful shutdown, and scaling patterns

## Architecture

The system consists of three main actor types:

### Sub-Actors (Domain-Specific)
- **[UserService](src/actor_recipe.rs#L286)** - Manages user data (create, get, update, list)
- **[ProductService](src/actor_recipe.rs#L519)** - Handles products and inventory (get, check stock, reserve)

### Root Actor (Orchestrator)
- **[OrderService](src/actor_recipe.rs#L709)** - Coordinates user and product services to create orders

### System Coordinator
- **[OrderSystem](src/actor_recipe.rs#L916)** - Manages lifecycle, dependency injection, and graceful shutdown

## Key Features

### Terminology
This implementation uses business-friendly terminology:
- **Service** (e.g., `UserService`) = Actor
- **Client** (e.g., `UserClient`) = Actor Reference/Handle

### Macro-Generated Clients
The [`client_method!`](src/actor_recipe.rs#L97) macro eliminates boilerplate for actor communication:

```rust
// This generates a complete client method with tracing:
client_method!(UserClient => fn get_user(id: String) -> Option<User> as UserRequest::GetUser);

// Equivalent to writing 15+ lines of boilerplate code manually
```

### Comprehensive Tracing
All operations are automatically traced with structured logging:

```
INFO user_creation: Creating test user
DEBUG create_user{}: Sending request
DEBUG handle_create_user{user_name="Alice" user_email="alice@example.com"}: Processing create_user request
INFO handle_create_user{user_name="Alice" user_email="alice@example.com"}: User created successfully user_id="user_1"
```

### Handler Patterns
Multiple patterns for different operation types:
- **[Sync handlers](src/actor_recipe.rs#L354)** - Fast, in-memory operations
- **[Async handlers](src/actor_recipe.rs#L401)** - I/O operations with validation
- **[Background handlers](src/actor_recipe.rs#L1101)** - Task owns response channel
- **[Orchestration handlers](src/actor_recipe.rs#L778)** - Coordinate multiple sub-actors

## Usage

### Running the Example

```bash
# Basic run
cargo run

# With debug logging
RUST_LOG=debug cargo run

# With warning level only
RUST_LOG=warn cargo run
```

### Using in Your Code

```rust
// Create the entire order system
let system = OrderSystem::new();

// Create a user (flows to UserService)
let user = User::new("Alice", "alice@example.com");
let user_id = system.user_client.create_user(user).await?;

let order = Order::new("order_1", user_id, "p1", 5, 50.0);

// Process order (orchestrates UserService + ProductService, fails - no products in demo)
match system.order_client.create_order(order).await {
    Ok(order_id) => println!("Order created: {}", order_id),
    Err(e) => println!("Order failed (expected): {}", e),
}

// Shutdown gracefully
system.shutdown().await?;
```

### Generating Documentation

```bash
# Generate and open documentation
cargo doc --open
```

## Project Structure

```
src/
└── actor_recipe.rs    # Complete implementation with extensive documentation
```

The single file contains:
- **[Domain types](src/actor_recipe.rs#L122)** (User, Product, Order)
- **[Message enums](src/actor_recipe.rs#L190)** for typed communication
- **[Service implementations](src/actor_recipe.rs#L286)** with tracing
- **[Client generation macros](src/actor_recipe.rs#L97)**
- **[System coordination](src/actor_recipe.rs#L916)**
- **[Test-only messages](src/actor_recipe.rs#L471)** for internal state inspection
- **[Usage examples and patterns](src/actor_recipe.rs#L1238)**

## Dependencies

- `tokio` - Async runtime with full features
- `tracing` - Structured logging
- `tracing-subscriber` - Log formatting and filtering

## License

This is a reference implementation and recipe - use it as a foundation for your own actor systems.