//! # Order Actor
//!
//! This module implements the Order resource actor with cross-actor coordination and validation.
//!
//! ## Overview
//!
//! The Order actor demonstrates the most complex pattern: an actor with **context dependencies**.
//! It coordinates with User and Product actors to validate orders and reserve inventory during
//! order creation.
//!
//! ## Structure
//!
//! - [`entity`] - [`ActorEntity`](crate::framework::ActorEntity) implementation for [`Order`]
//! - [`error`] - [`OrderError`] type with automatic error conversion from dependencies
//! - [`new()`] - Factory function that creates the actor and client
//!
//! ## Message Flow: create_order
//!
//! The following diagram shows how a `create_order` call flows through the system,
//! demonstrating actor-to-actor communication and validation:
//!
//! <div align="center">
//! <img src="https://raw.githubusercontent.com/schilit/actor-framework-recipe/main/docs/images/create_order_sequence.png" alt="create_order sequence diagram" width="600"/>
//! </div>
//!
//! **Key Points:**
//! - All communication is asynchronous via message passing
//! - Each actor processes messages sequentially (no locks needed)
//! - Validation happens in `Order::on_create()` before the order is stored
//! - If any step fails, the entire operation fails atomically
//!
//! ## Context Dependencies
//!
//! The Order actor requires User and Product clients in its context:
//!
//! ```rust
//! use actor_recipe::order_actor;
//! use actor_recipe::framework::mock::MockClient;
//! use actor_recipe::clients::{UserClient, ProductClient};
//! use actor_recipe::model::{User, Product};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create mocks for dependencies
//!     let user_mock = MockClient::<User>::new();
//!     let product_mock = MockClient::<Product>::new();
//!     
//!     let user_client = UserClient::new(user_mock.client());
//!     let product_client = ProductClient::new(product_mock.client());
//!
//!     // Create actor and client
//!     let (actor, client) = order_actor::new();
//!
//!     // Start with dependencies injected
//!     tokio::spawn(actor.run((user_client, product_client)));
//! }
//! ```
//!
//! ## Lifecycle Hooks
//!
//! The Order actor uses the `on_create` hook to perform validation and coordination:
//!
//! 1. **Validate user exists** - Queries User actor
//! 2. **Reserve product stock** - Calls Product actor's `reserve_stock` action
//! 3. **Create order** - Only if validation succeeds
//!
//! This ensures orders are always valid and inventory is properly reserved.
//!
//! ## Error Handling
//!
//! The Order actor demonstrates automatic error conversion with `#[from]`:
//!
//! ```rust
//! use thiserror::Error;
//! use actor_recipe::user_actor::UserError;
//! use actor_recipe::product_actor::ProductError;
//!
//! #[derive(Debug, Error)]
//! pub enum OrderError {
//!     #[error("User service error: {0}")]
//!     UserService(#[from] UserError),  // Auto-converts UserError
//!     
//!     #[error("Product service error: {0}")]
//!     ProductService(#[from] ProductError),  // Auto-converts ProductError
//! }
//! ```
//!
//! This allows seamless error propagation from dependency actors.
//!
//! ## Key Features
//!
//! - **Context injection**: Depends on `(UserClient, ProductClient)`
//! - **Cross-actor coordination**: Validates and reserves across multiple actors
//! - **Automatic error conversion**: Uses `#[from]` for clean error handling
//! - **Lifecycle hooks**: Uses `on_create` for validation logic

pub mod entity;
pub mod error;

pub use error::*;

use crate::clients::OrderClient;
use crate::framework::ResourceActor;
use crate::model::Order;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Creates a new Order actor and its client.
pub fn new() -> (ResourceActor<Order>, OrderClient) {
    let order_id_counter = Arc::new(AtomicU64::new(1));
    let next_order_id = move || {
        let id = order_id_counter.fetch_add(1, Ordering::SeqCst);
        format!("order_{}", id)
    };

    let (actor, generic_client) = ResourceActor::new(32, next_order_id);
    let client = OrderClient::new(generic_client);

    (actor, client)
}
