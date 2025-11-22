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
use crate::framework::ActorEntity;
use crate::model::{User, UserCreate, UserUpdate};

impl ActorEntity for User {
    type Id = String;
    type CreateParams = UserCreate;
    type UpdateParams = UserUpdate;
    type Action = ();           // No custom actions yet
    type ActionResult = ();

    fn from_create_params(id: String, params: UserCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            name: params.name,
            email: params.email,
        })
    }

    fn on_update(&mut self, update: UserUpdate) -> Result<(), String> {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(email) = update.email {
            self.email = email;
        }
        Ok(())
    }

    fn handle_action(&mut self, _action: ()) -> Result<(), String> {
        Ok(()) // No actions yet
    }
}
```

**Key Points**:
- `from_create_params`: Constructs the entity from the DTO
- `on_update`: Applies updates to the entity's fields
- `handle_action`: Handles custom domain logic (we'll add this later)

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

### Step 4: Wire It Into the System

In `src/lifecycle/order_system.rs`:

```rust
use crate::user_actor::entity::User;
use crate::clients::UserClient;
use crate::framework::ResourceActor;

pub struct OrderSystem {
    pub user_client: UserClient,
    // ... other clients
    user_actor_handle: JoinHandle<()>,
}

impl OrderSystem {
    pub fn new() -> Self {
        // Create the actor and client
        let (user_actor, user_client_inner) = ResourceActor::new(100, || {
            format!("user_{}", Uuid::new_v4())
        });

        // Spawn the actor
        let user_actor_handle = tokio::spawn(user_actor.run());

        // Wrap in domain client
        let user_client = UserClient::new(user_client_inner);

        Self {
            user_client,
            user_actor_handle,
        }
    }

    pub async fn shutdown(self) -> Result<(), String> {
        drop(self.user_client); // Close the channel
        self.user_actor_handle.await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
```

**That's it!** You now have a fully functional `User` actor.

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

    fn handle_action(&mut self, action: ProductAction) -> Result<ProductActionResult, String> {
        match action {
            ProductAction::CheckStock => {
                Ok(ProductActionResult::CheckStock(self.quantity))
            }
            ProductAction::ReserveStock(quantity) => {
                if self.quantity >= quantity {
                    self.quantity -= quantity;
                    Ok(ProductActionResult::ReserveStock(()))
                } else {
                    Err(format!(
                        "Insufficient stock: requested {}, available {}",
                        quantity, self.quantity
                    ))
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

### `on_create(&mut self) -> Result<(), Self::Error>`

Called **after** the entity is created but **before** it's stored.

**Use cases**:
- Logging: "User account created"
- Notifications: Send welcome email
- Validation: Final checks before persistence

**Example**:
```rust
impl ActorEntity for User {
    fn on_create(&mut self) -> Result<(), String> {
        tracing::info!(user_id = %self.id, "User account created");
        // Could send to an event bus, audit log, etc.
        Ok(())
    }
}
```

### `on_delete(&self) -> Result<(), Self::Error>`

Called **before** the entity is removed from the store.

**Use cases**:
- Cleanup: Release external resources
- Archiving: Save to long-term storage
- Notifications: Alert other systems
- Validation: Prevent deletion if constraints exist

**Example**:
```rust
impl ActorEntity for Product {
    fn on_delete(&self) -> Result<(), String> {
        if self.quantity > 0 {
            return Err(format!(
                "Cannot delete product {} with {} items in stock",
                self.id, self.quantity
            ));
        }
        tracing::warn!(product_id = %self.id, "Product deleted");
        Ok(())
    }
}
```

**Current Status**: The framework supports these hooks, but they're not currently used in the example entities. They're provided as extension points for your domain logic.

---

## How to Write Tests

### Unit Test: Testing an Actor in Isolation

Use the `MockClient` for fluent, readable tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::mock::MockClient;

    #[tokio::test]
    async fn test_promote_to_admin() {
        let mut mock = MockClient::<User>::new();

        // Set up expectations
        mock.expect_action("user_1".to_string())
            .return_ok(UserActionResult::PromoteToAdmin(true));

        let client = UserClient::new(mock.client());

        // Execute
        let promoted = client.promote_to_admin("user_1".to_string()).await.unwrap();

        // Verify
        assert!(promoted);
        mock.verify(); // Ensures all expectations were met
    }
}
```

### Integration Test: Testing Actor Coordination

In `tests/order_actor_test.rs`:

```rust
use actor_recipe::clients::{OrderClient, UserClient, ProductClient};
use actor_recipe::model::{Order, User, Product};
use actor_recipe::framework::mock::MockClient;

#[tokio::test]
async fn test_order_creation_flow() {
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();
    let mut order_mock = MockClient::<Order>::new();

    // Define expectations
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "test@example.com")));

    product_mock.expect_get("product_1".to_string())
        .return_ok(Some(Product::new("product_1", "Widget", 10.0, 100)));

    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    order_mock.expect_create()
        .return_ok("order_1".to_string());

    // Wire up clients
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());
    let order_client = OrderClient::new(order_mock.client(), user_client, product_client);

    // Execute
    let order = Order::new("order_1", "user_1", "product_1", 5, 50.0);
    let result = order_client.create_order(order).await;

    // Verify
    assert_eq!(result, Ok("order_1".to_string()));
    user_mock.verify();
    product_mock.verify();
    order_mock.verify();
}
```

**Key Points**:
- Mock clients let you test coordination without spinning up real actors
- Fluent API makes tests readable
- `verify()` ensures all expected interactions happened

---

## How to Handle Errors

### Define Domain-Specific Errors

In `src/user_actor/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}

impl From<String> for UserError {
    fn from(s: String) -> Self {
        UserError::ActorCommunicationError(s)
    }
}
```

### Return Errors from Entity Methods

```rust
impl ActorEntity for User {
    fn from_create_params(id: String, params: UserCreate) -> Result<Self, String> {
        if !params.email.contains('@') {
            return Err(format!("Invalid email: {}", params.email));
        }
        Ok(Self {
            id,
            name: params.name,
            email: params.email,
        })
    }
}
```

### Map Errors in the Client

```rust
impl UserClient {
    pub async fn create_user(&self, user: User) -> Result<String, UserError> {
        let payload = UserCreate {
            name: user.name,
            email: user.email,
        };
        self.inner.create(payload).await
            .map_err(|e| match e {
                FrameworkError::Custom(msg) if msg.contains("Invalid email") => {
                    UserError::InvalidEmail(msg)
                }
                FrameworkError::NotFound(id) => UserError::NotFound(id),
                _ => UserError::ActorCommunicationError(e.to_string()),
            })
    }
}
```

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
