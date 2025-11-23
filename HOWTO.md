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

The actor framework supports four distinct testing patterns, each with different trade-offs:

### Pattern 0: Client Logic Test (Pure Mock)

**When to use**: Testing complex orchestration logic in your *Client* (e.g., `OrderClient`) without spinning up any actors.

**What you test**: The client's decision making, error handling, and coordination sequence.

**Example**: Testing `OrderClient` validation logic

```rust
#[tokio::test]
async fn test_order_client_orchestration() {
    // 1. Setup Mocks (No real actors!)
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();
    let mut order_mock = MockClient::<Order>::new();

    // 2. Define Expectations
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "test@example.com")));

    product_mock.expect_get("product_1".to_string())
        .return_ok(Some(Product::new("product_1", "Widget", 10.0, 100)));

    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    order_mock.expect_create()
        .return_ok("order_1".to_string());

    // 3. Create Client with Mocks
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());
    
    // Inject mocks into the OrderClient
    let order_client = OrderClient::new(
        order_mock.client(), // Mock Order actor
        user_client,
        product_client
    );

    // 4. Test OrderClient::create_order
    // 
    // This method contains the orchestration logic we are testing.
    // Internally, `create_order` calls:
    // 1. `user_client.get()` to validate the user
    // 2. `product_client.reserve_stock()` to check/update stock
    // 3. `order_actor.create()` to finally create the order
    let order = Order::new("order_1", "user_1", "product_1", 5, 50.0);
    let result = order_client.create_order(order).await;

    // 5. Verify
    assert_eq!(result, Ok("order_1".to_string()));
    user_mock.verify();
    product_mock.verify();
    order_mock.verify();
}
```

**Pros**:
- ⚡⚡⚡ Fastest (no async tasks spawned)
- ✅ Deterministic
- ✅ Great for testing **Client handling of** edge cases (e.g., "How does OrderClient react when User is not found?")

**Cons**:
- ❌ Doesn't test *any* actor logic
- ❌ Mocks can drift from reality

---

### Pattern 1: Single Actor Test (Fast, Isolated)

**When to use**: Testing a single actor's logic in isolation.

**What you test**: The actor's state management, lifecycle hooks, and action handling.

**Example**: Testing the Product actor's stock management

```rust
#[tokio::test]
async fn test_product_stock_management() {
    // Create a single Product actor using the factory
    let (product_actor, product_client) = crate::product_actor::new();
    
    // Spawn the actor
    let actor_handle = tokio::spawn(product_actor.run());
    
    // Test: Create a product
    let product = Product::new("", "Widget", 10.0, 100);
    let product_id = product_client.create_product(product).await.unwrap();
    
    // Test: Check initial stock
    let stock = product_client.check_stock(product_id.clone()).await.unwrap();
    assert_eq!(stock, 100);
    
    // Test: Reserve stock
    product_client.reserve_stock(product_id.clone(), 30).await.unwrap();
    
    // Test: Verify stock was decremented
    let stock = product_client.check_stock(product_id.clone()).await.unwrap();
    assert_eq!(stock, 70);
    
    // Test: Insufficient stock should fail
    let result = product_client.reserve_stock(product_id.clone(), 100).await;
    assert!(result.is_err());
    
    // Cleanup
    drop(product_client);
    actor_handle.await.unwrap();
}
```

**Pros**:
- ✅ Fast (no system setup)
- ✅ Isolated (no dependencies)
- ✅ Easy to debug

**Cons**:
- ❌ Doesn't test actor coordination
- ❌ Doesn't test dependency injection

---

### Pattern 2: Actor with Mocked Dependencies (Sweet Spot)

**When to use**: Testing an actor that depends on other actors, but you want to isolate the actor under test.

**What you test**: The actor's coordination logic, how it calls dependencies, error handling.

**Example**: Testing Order actor with mocked User and Product actors

```rust
#[tokio::test]
async fn test_order_actor_with_mocked_dependencies() {
    // Setup mock dependencies
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();

    // Define expectations for the dependencies
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "alice@example.com")));

    product_mock.expect_get("product_1".to_string())
        .return_ok(Some(Product::new("product_1", "Widget", 25.0, 50)));

    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    // Create clients from mocks
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());

    // Create REAL Order actor using factory function
    // We inject the clients derived from mocks!
    let (order_actor, order_client) = crate::order_actor::new(
        user_client.clone(), 
        product_client.clone()
    );

    // Execute: This will run through the REAL Order actor
    let order = Order::new("", "user_1", "product_1", 3, 75.0);
    let result = order_client.create_order(order).await;

    // Verify the order was created
    assert!(result.is_ok());

    // Verify mocks were called correctly
    user_mock.verify();
    product_mock.verify();

    // Cleanup
    drop(order_client);
    actor_handle.await.unwrap();
}
```

**Pros**:
- ✅ Tests real actor logic
- ✅ Isolates dependencies (fast, deterministic)
- ✅ Tests coordination without full system
- ✅ Easy to simulate error conditions

**Cons**:
- ❌ Doesn't test real dependency behavior
- ❌ Requires mock setup

---

### Pattern 3: Full System Integration Test (Comprehensive)

**When to use**: Testing the entire system working together, end-to-end flows, concurrency.

**What you test**: Real actor coordination, actual state changes, race conditions, system behavior.

**Example**: Full order creation flow

```rust
#[tokio::test]
async fn test_full_order_system_integration() {
    // Create the full system with all real actors
    let system = OrderSystem::new();

    // Create a user
    let user = User::new("Alice", "alice@example.com");
    let user_id = system.user_client.create_user(user).await.unwrap();

    // Create a product with stock
    let product = Product::new("", "Super Widget", 25.50, 100);
    let product_id = system.product_client.create_product(product).await.unwrap();

    // Verify initial stock level
    let initial_stock = system.product_client.check_stock(product_id.clone()).await.unwrap();
    assert_eq!(initial_stock, 100);

    // Create an order (should reserve stock)
    let order = Order::new("", user_id.clone(), product_id.clone(), 5, 127.50);
    let order_id = system.order_client.create_order(order).await.unwrap();

    // Verify stock was actually decremented
    let final_stock = system.product_client.check_stock(product_id.clone()).await.unwrap();
    assert_eq!(final_stock, 95, "Stock should be decremented by order quantity");

    // Test insufficient stock scenario
    let large_order = Order::new("", user_id.clone(), product_id.clone(), 200, 5100.0);
    let result = system.order_client.create_order(large_order).await;
    assert!(result.is_err(), "Should fail when stock is insufficient");

    // Graceful shutdown
    system.shutdown().await.unwrap();
}
```

**Pros**:
- ✅ Tests real system behavior
- ✅ Catches integration bugs
- ✅ Tests concurrency and race conditions
- ✅ High confidence in correctness

**Cons**:
- ❌ Slower (full system startup)
- ❌ Harder to debug failures
- ❌ More complex setup

---

### Testing Pattern Comparison

| Pattern | Speed | Isolation | Coverage | Use Case |
|---------|-------|-----------|----------|----------|
| **Pure Mock** | ⚡⚡⚡⚡ Instant | ✅ Full | Client logic only | Testing API handlers & orchestration |
| **Single Actor** | ⚡⚡⚡ Fast | ✅ Full | Actor logic only | Unit testing actor behavior |
| **Actor + Mocks** | ⚡⚡ Medium | ✅ Good | Actor + coordination | Testing actor interactions |
| **Full System** | ⚡ Slow | ❌ None | Everything | End-to-end validation |

---

### Advanced: Test-Only Actions

Sometimes you need to inspect internal actor state for testing. Use a Cargo **feature flag** instead of `#[cfg(test)]` so it works with integration tests.

**Step 1: Add feature to `Cargo.toml`**:
```toml
[features]
testing = []  # Enable test-only functionality
```

**Step 2: Guard test-only actions with the feature**:
```rust
#[derive(Debug, Clone)]
pub enum ProductAction {
    CheckStock,
    ReserveStock(u32),
    
    #[cfg(feature = "testing")]
    GetInternalState, // Test-only action
}

#[derive(Debug, Clone)]
pub enum ProductActionResult {
    CheckStock(u32),
    ReserveStock(()),
    
    #[cfg(feature = "testing")]
    GetInternalState {
        quantity: u32,
        reserved: u32,
        pending_orders: usize,
    },
}

impl ActorEntity for Product {
    async fn handle_action(&mut self, action: ProductAction, _ctx: &Self::Context) 
        -> Result<ProductActionResult, String> 
    {
        match action {
            ProductAction::CheckStock => {
                Ok(ProductActionResult::CheckStock(self.quantity))
            }
            ProductAction::ReserveStock(quantity) => {
                // ... normal logic
            }
            #[cfg(feature = "testing")]
            ProductAction::GetInternalState => {
                Ok(ProductActionResult::GetInternalState {
                    quantity: self.quantity,
                    reserved: self.reserved,
                    pending_orders: self.pending_orders.len(),
                })
            }
        }
    }
}
```

**Step 3: Run tests with the feature enabled**:
```bash
cargo test --features testing
```

**Why use a feature instead of `#[cfg(test)]`?**
- `#[cfg(test)]` only works for unit tests in the same crate
- Integration tests in `tests/` directory can't access `#[cfg(test)]` code
- Features work everywhere: unit tests, integration tests, and even production (if needed)

**Use case**: When you need to verify internal state that isn't exposed through normal APIs.

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
