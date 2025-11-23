# How-To Guide: Building with the Actor Framework

This guide walks you through common tasks when building systems with this actor framework.

## Table of Contents

1. [How to Add a New Actor](#how-to-add-a-new-actor)
2. [How to Add a Custom Action](#how-to-add-a-custom-action)
3. [How to Write Tests](#how-to-write-tests)
4. [How to Handle Errors](#how-to-handle-errors)

---

## How to Add a New Actor

Let's walk through adding a new `User` actor to the system. This involves 4 steps:

### Step 1: Define the Resource Model

Create your data structure in `src/model/user.rs`:

```rust
/// Represents a registered user in the system.
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: String::new(), // Will be set by the actor
            name: name.into(),
            email: email.into(),
        }
    }
}

/// DTO for creating a new user
#[derive(Debug, Clone)]
pub struct UserCreate {
    pub name: String,
    pub email: String,
}

/// DTO for updating a user
#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
}
```

**Key Points**:
- The main struct (`User`) holds the state
- `UserCreate` is the payload for creation (no ID needed)
- `UserUpdate` uses `Option<T>` for partial updates

### Step 2: Implement the `ActorEntity` Trait

Create `src/user_actor/entity.rs`:

```rust
use async_trait::async_trait;
use crate::framework::ActorEntity;
use crate::model::{User, UserCreate, UserUpdate};
use crate::user_actor::UserError;

#[async_trait]
impl ActorEntity for User {
    type Id = String;
    type Create = UserCreate;
    type Update = UserUpdate;
    type Action = ();           // No custom actions yet
    type ActionResult = ();
    type Context = ();          // No dependencies
    type Error = UserError;     // Type-safe errors

    fn from_create_params(id: String, params: UserCreate) -> Result<Self, Self::Error> {
        Ok(Self {
            id,
            name: params.name,
            email: params.email,
        })
    }

    async fn on_update(&mut self, update: UserUpdate, _ctx: &Self::Context) -> Result<(), Self::Error> {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(email) = update.email {
            self.email = email;
        }
        Ok(())
    }

    async fn handle_action(&mut self, _action: (), _ctx: &Self::Context) -> Result<(), Self::Error> {
        Ok(()) // No actions yet
    }
}
```

**Key Points**:
- `#[async_trait]`: Required for async methods in traits
- `type Context`: Dependencies injected at runtime (use `()` if none)
- `type Error`: Your custom error type (enables type-safe error handling)
- `from_create_params`: Constructs the entity from the DTO
- `on_update`: Applies updates to the entity's fields (async)
- `handle_action`: Handles custom domain logic (async)

### Step 3: Create a Domain-Specific Client

Create `src/clients/user_client.rs`:

```rust
use crate::model::{User, UserCreate, UserUpdate};
use crate::user_actor::UserError;
use crate::framework::{ResourceClient, FrameworkError};
use crate::clients::actor_client::ActorClient;
use async_trait::async_trait;

#[derive(Clone)]
pub struct UserClient {
    inner: ResourceClient<User>,
}

impl UserClient {
    pub fn new(inner: ResourceClient<User>) -> Self {
        Self { inner }
    }

    pub async fn create_user(&self, user: User) -> Result<String, UserError> {
        let payload = UserCreate {
            name: user.name,
            email: user.email,
        };
        self.inner.create(payload).await
            .map_err(|e| UserError::ActorCommunicationError(e.to_string()))
    }

    pub async fn update_user(&self, id: String, update: UserUpdate) -> Result<User, UserError> {
        self.inner.update(id, update).await
            .map_err(|e| UserError::ActorCommunicationError(e.to_string()))
    }
}

#[async_trait]
impl ActorClient<User> for UserClient {
    type Error = UserError;

    fn inner(&self) -> &ResourceClient<User> {
        &self.inner
    }

    fn map_error(e: FrameworkError) -> Self::Error {
        UserError::ActorCommunicationError(e.to_string())
    }
}
```

**Key Points**:
- Wraps the generic `ResourceClient<User>`
- Provides domain-specific methods (`create_user`, `update_user`)
- Implements `ActorClient` trait to inherit `get()` and `delete()`

### Step 4: Create a Factory Function (Optional but Recommended)

To make your actor easy to instantiate, add a factory function in `src/user_actor/mod.rs`:

```rust
use crate::framework::ResourceActor;
use crate::clients::UserClient;
use crate::model::User;
use uuid::Uuid;

/// Creates a new User actor and its client.
pub fn new() -> (ResourceActor<User>, UserClient) {
    // 1. Create the generic actor and client
    let (actor, generic_client) = ResourceActor::new(100, || {
        format!("user_{}", Uuid::new_v4())
    });

    // 2. Wrap the generic client in our domain client
    let client = UserClient::new(generic_client);

    (actor, client)
}
```

### Step 5: Wire It Into the System

In `src/lifecycle/order_system.rs`, usage becomes much cleaner with the factory pattern.

**Important**: Actors now use **Context Injection** via `run()`. Dependencies are passed when starting the actor, not when creating it.

```rust
use crate::user_actor;

impl OrderSystem {
    pub fn new() -> Self {
        // 1. Create Actor & Client using the factory
        let (user_actor, user_client) = user_actor::new();

        // 2. Spawn the actor with context (User has no dependencies, so pass ())
        let user_actor_handle = tokio::spawn(user_actor.run(()));

        Self {
            user_client,
            handles: vec![user_actor_handle],
        }
    }
}
```

**Key Points**:
- `user_actor::new()` creates the actor and client (no dependencies)
- `user_actor.run(())` starts the actor with an empty context (User has no dependencies)
- For actors with dependencies, you pass them to `run()` (see Advanced Patterns below)

**That's it!** You now have a fully functional `User` actor with a clean initialization API.

---

## How to Add a Custom Action

Actions are for domain-specific operations that don't fit CRUD. Let's look at how `Product` implements stock management actions.

### Step 1: Define the Action Enum

In `src/product_actor/actions.rs`:

```rust
/// Custom actions for Product entities
#[derive(Debug, Clone)]
pub enum ProductAction {
    /// Checks the current stock level without modifying it
    CheckStock,
    /// Reserves a specified amount of stock
    ReserveStock(u32),
}

/// Results from ProductActions - variants match 1:1 with ProductAction
#[derive(Debug, Clone)]
pub enum ProductActionResult {
    /// Result from CheckStock action - returns the current stock level
    CheckStock(u32),
    /// Result from ReserveStock action - returns unit on success
    ReserveStock(()),
}
```

**Why separate enums?**
- `ProductAction`: What you want to do (input)
- `ProductActionResult`: What happened (output) - type-safe results!

### Step 2: Implement in the Entity

In `src/product_actor/entity.rs`:

```rust
use super::actions::{ProductAction, ProductActionResult};

impl ActorEntity for Product {
    type Action = ProductAction;
    type ActionResult = ProductActionResult;
    type Error = ProductError;

    async fn handle_action(&mut self, action: ProductAction, _ctx: &Self::Context) -> Result<ProductActionResult, Self::Error> {
        match action {
            ProductAction::CheckStock => {
                Ok(ProductActionResult::CheckStock(self.quantity))
            }
            ProductAction::ReserveStock(quantity) => {
                if self.quantity >= quantity {
                    self.quantity -= quantity;
                    Ok(ProductActionResult::ReserveStock(()))
                } else {
                    Err(ProductError::InsufficientStock { 
                        requested: quantity, 
                        available: self.quantity 
                    })
                }
            }
        }
    }
}
```

**Key Points**:
- Actions can mutate state (`self.quantity -= quantity`)
- Actions can fail with domain-specific errors
- Return type-safe results (not just `bool`)

### Step 3: Add Client Methods

In `src/clients/product_client.rs`:

```rust
use crate::product_actor::{ProductAction, ProductActionResult};

impl ProductClient {
    pub async fn check_stock(&self, id: String) -> Result<u32, ProductError> {
        match self.inner.perform_action(id, ProductAction::CheckStock).await {
            Ok(ProductActionResult::CheckStock(level)) => Ok(level),
            Ok(_) => unreachable!("CheckStock action must return CheckStock result"),
            Err(e) => Err(ProductError::ActorCommunicationError(e.to_string())),
        }
    }

    pub async fn reserve_stock(&self, id: String, quantity: u32) -> Result<(), ProductError> {
        match self.inner.perform_action(id, ProductAction::ReserveStock(quantity)).await {
            Ok(ProductActionResult::ReserveStock(())) => Ok(()),
            Ok(_) => unreachable!("ReserveStock must return ReserveStock result"),
            Err(e) => Err(ProductError::ActorCommunicationError(e.to_string())),
        }
    }
}
```

**Why this pattern?**
- The client unwraps the `ProductActionResult` enum
- Callers get a simple `Result<u32, ProductError>` or `Result<(), ProductError>`
- Type safety ensures we can't mix up action results
- The `unreachable!()` catches programmer errors at runtime

### When to Use Actions vs. Updates

| Use Case | Use |
|----------|-----|
| Simple field changes | `Update` (e.g., change price) |
| Complex business logic | `Action` (e.g., reserve stock with validation) |
| Coordination with other actors | `Action` (e.g., check availability before ordering) |
| Operations that can fail | `Action` (e.g., insufficient stock) |
| Read-only queries on state | `Action` (e.g., check stock level) |

---

## Understanding Lifecycle Hooks

The `ActorEntity` trait provides optional lifecycle hooks:

### `on_create(&mut self, ctx: &Self::Context) -> Result<(), Self::Error>`

Called **after** the entity is created but **before** it's stored.

**Use cases**:
- Logging: "User account created"
- Notifications: Send welcome email
- Validation: Final checks before persistence

**Example**:
```rust
impl ActorEntity for User {
    async fn on_create(&mut self, _ctx: &Self::Context) -> Result<(), Self::Error> {
        tracing::info!(user_id = %self.id, "User account created");
        // Could send to an event bus, audit log, etc.
        Ok(())
    }
}
```

### `on_delete(&self, ctx: &Self::Context) -> Result<(), Self::Error>`

Called **before** the entity is removed from the store.

**Use cases**:
- Cleanup: Release external resources
- Archiving: Save to long-term storage
- Notifications: Alert other systems
- Validation: Prevent deletion if constraints exist

**Example**:
```rust
impl ActorEntity for User {
    async fn on_delete(&self, _ctx: &Self::Context) -> Result<(), Self::Error> {
        tracing::warn!(user_id = %self.id, "User account deleted");
        Ok(())
    }
}
```

**Current Status**: The framework supports these hooks, but they're not currently used in the example entities. They're provided as extension points for your domain logic.

---

## How to Write Tests
 
Comprehensive testing documentation has been moved to the [`framework::mock`](crate::framework::mock) module.
 
Please run `cargo doc --open` or view `src/framework/mock.rs` to see the guide on:
- **Pattern 0**: Client Logic Test (Pure Mock)
- **Pattern 1**: Single Actor Test
- **Pattern 2**: Actor with Mocked Dependencies
- **Pattern 3**: Full System Integration Test
- **Advanced**: Test-Only Actions with Feature Flags

---

## How to Handle Errors

The framework uses **type-safe error handling** via associated error types. Each entity defines its own error type that implements `std::error::Error`.

### Step 1: Define Your Error Type

Each actor has an error type in `src/*_actor/error.rs`. This error type serves **both** the client and the entity:

```rust
//! src/user_actor/error.rs
use thiserror::Error;

/// Errors for User operations (used by both client and entity)
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),  // Entity-level validation

    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),  // Framework errors
}

impl From<String> for UserError {
    fn from(s: String) -> Self {
        UserError::ActorCommunicationError(s)
    }
}
```

**Key Points**:
- **No `Clone` or `PartialEq`** - These prevent implementing `std::error::Error`
- **Dual purpose** - Used by both entity methods and client methods
- **Structured variants** - Enable pattern matching on specific error types

### Step 2: Use `type Error` in Your Entity

```rust
use crate::user_actor::UserError;

#[async_trait]
impl ActorEntity for User {
    type Id = String;
    type Create = UserCreate;
    type Update = UserUpdate;
    type Error = UserError;  // ← Type-safe errors!
    // ... other types ...

    fn from_create_params(id: String, params: UserCreate) -> Result<Self, Self::Error> {
        // Validate email format
        if !params.email.contains('@') {
            return Err(UserError::InvalidEmail(params.email));
        }
        Ok(Self { id, name: params.name, email: params.email })
    }

    async fn on_update(&mut self, update: UserUpdate, _ctx: &Self::Context) 
        -> Result<(), Self::Error> 
    {
        if let Some(email) = update.email {
            if !email.contains('@') {
                return Err(UserError::InvalidEmail(email));
            }
            self.email = email;
        }
        Ok(())
    }
}
```

### Step 3: Error Propagation with `#[from]`

For entities that depend on other services (like `Order`), use `#[from]` for automatic error conversion:

```rust
//! src/order_actor/error.rs
use thiserror::Error;
use crate::user_actor::UserError;
use crate::product_actor::ProductError;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("User {0} not found")]
    InvalidUser(String),

    #[error("Product {0} not found")]
    InvalidProduct(String),

    // Automatic conversion from UserError
    #[error("User service error: {0}")]
    UserService(#[from] UserError),

    // Automatic conversion from ProductError
    #[error("Product service error: {0}")]
    ProductService(#[from] ProductError),

    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}
```

**Usage in Order entity**:

```rust
async fn on_create(&mut self, (user_client, product_client): &Self::Context) 
    -> Result<(), Self::Error> 
{
    // 1. Validate User - errors auto-convert via #[from]
    let user = user_client.get(self.user_id.clone()).await?;
    
    if user.is_none() {
        return Err(OrderError::InvalidUser(self.user_id.clone()));
    }

    // 2. Reserve Stock - ProductError auto-converts to OrderError::ProductService
    product_client.reserve_stock(
        self.product_id.clone(), 
        self.quantity
    ).await?;  // ← No .map_err() needed!

    Ok(())
}
```

### Step 4: Pattern Match on Errors

Clients can now pattern match on specific error types:

```rust
match order_client.create_order(order).await {
    Ok(order_id) => println!("Order created: {}", order_id),
    Err(OrderError::InvalidUser(user_id)) => {
        println!("User {} doesn't exist", user_id);
    }
    Err(OrderError::ProductService(ProductError::InsufficientStock { requested, available })) => {
        println!("Not enough stock: need {}, have {}", requested, available);
    }
    Err(e) => println!("Other error: {}", e),
}
```

### Benefits of Type-Safe Errors

1. **No `.to_string()` loss** - Error context is preserved
2. **Pattern matching** - Handle specific errors differently
3. **Automatic conversion** - `#[from]` eliminates boilerplate
4. **Better error messages** - Structured data in error variants
5. **Compile-time safety** - Catch error handling bugs early

---


**Best Practices**:
- Entity methods return `Result<T, String>` (simple)
- Clients map to domain-specific errors (`UserError`)
- Callers get rich, typed errors they can match on

---

## Common Patterns

### Pattern: Actor Coordination

When one actor needs data from another:

```rust
impl OrderClient {
    pub async fn create_order(&self, order: Order) -> Result<String, OrderError> {
        // 1. Validate user exists
        let user = self.user_client.get(order.user_id.clone()).await?
            .ok_or_else(|| OrderError::UserNotFound(order.user_id.clone()))?;

        // 2. Validate product and reserve stock
        let product = self.product_client.get(order.product_id.clone()).await?
            .ok_or_else(|| OrderError::ProductNotFound(order.product_id.clone()))?;

        self.product_client.reserve_stock(order.product_id.clone(), order.quantity).await?;

        // 3. Create the order
        let payload = OrderCreate {
            user_id: order.user_id,
            product_id: order.product_id,
            quantity: order.quantity,
            total_price: order.total_price,
        };
        self.inner.create(payload).await
            .map_err(|e| OrderError::ActorCommunicationError(e.to_string()))
    }
}
```

**Key Points**:
- Clients orchestrate coordination
- Each actor stays isolated
- Failures are handled gracefully (e.g., stock reservation can fail)

### Pattern: ID Generation

Use closures for flexible ID generation:

```rust
// UUID-based
let (actor, client) = ResourceActor::new(100, || {
    format!("user_{}", Uuid::new_v4())
});

// Sequential
let counter = Arc::new(AtomicU64::new(1));
let (actor, client) = ResourceActor::new(100, move || {
    let id = counter.fetch_add(1, Ordering::SeqCst);
    format!("user_{}", id)
});
```

---

## Next Steps

- **Read the Code**: Start with `src/user_actor/` for a simple example
- **Run the Tests**: `cargo test` to see everything in action
- **Experiment**: Try adding a new actor or action
- **Ask Questions**: Open an issue on GitHub if you get stuck!
