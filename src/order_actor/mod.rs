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
//! - [`entity`] - [`ActorEntity`](crate::framework::ActorEntity) implementation for [`Order`](crate::model::Order)
//! - [`error`] - [`OrderError`] type with automatic error conversion from dependencies
//! - [`new()`] - Factory function that creates the actor and client
//!
//! ## Context Dependencies
//!
//! The Order actor requires User and Product clients in its context:
//!
//! ```rust,ignore
//! use crate::order_actor;
//!
//! // Create actor and client
//! let (actor, client) = order_actor::new();
//!
//! // Start with dependencies injected
//! tokio::spawn(actor.run((user_client.clone(), product_client.clone())));
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
//! ```rust,ignore
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
