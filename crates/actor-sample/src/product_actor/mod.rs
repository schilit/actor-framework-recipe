//! # Product Actor
//!
//! This module implements the Product resource actor with inventory management and custom actions.
//!
//! ## Overview
//!
//! The Product actor demonstrates how to add custom domain-specific actions beyond CRUD operations.
//! It manages product catalog and inventory, with actions for checking and reserving stock.
//!
//! ## Structure
//!
//! - [`entity`] - [`ActorEntity`](actor_framework::ActorEntity) implementation for [`Product`]
//! - [`error`] - [`ProductError`] type for type-safe error handling
//! - [`actions`] - [`ProductAction`] and [`ProductActionResult`] for stock management
//! - [`new()`] - Factory function that creates the actor and client
//!
//! ## Custom Actions
//!
//! The Product actor showcases the Action pattern for domain-specific operations:
//!
//! ```rust,ignore
//! // Check current stock level (read-only)
//! let stock = product_client.check_stock(product_id).await?;
//!
//! // Reserve stock for an order (mutating, can fail)
//! product_client.reserve_stock(product_id, quantity).await?;
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use actor_sample::product_actor;
//! use actor_sample::model::ProductCreate;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create actor and client
//!     let (actor, client) = product_actor::new();
//!
//!     // Start the actor (no dependencies)
//!     tokio::spawn(actor.run(()));
//!
//!     // Create a product
//!     let params = ProductCreate {
//!         name: "Widget".to_string(),
//!         price: 29.99,
//!         quantity: 100,
//!     };
//!     let id = client.create_product(params).await?;
//!
//!     // Reserve stock
//!     client.reserve_stock(id, 5).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Key Features
//!
//! - **Custom actions**: Stock management via [`ProductAction`]
//! - **Business logic validation**: `reserve_stock` fails if insufficient inventory
//! - **Type-safe results**: Actions return strongly-typed [`ProductActionResult`]

pub mod actions;
pub mod entity;
pub mod error;

pub use actions::*;
pub use error::*;

use crate::clients::ProductClient;
use actor_framework::ResourceActor;
use crate::model::Product;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Creates a new Product actor and its client.
pub fn new() -> (ResourceActor<Product>, ProductClient) {
    let product_id_counter = Arc::new(AtomicU64::new(1));
    let next_product_id = move || {
        let id = product_id_counter.fetch_add(1, Ordering::SeqCst);
        format!("product_{}", id)
    };

    let (actor, generic_client) = ResourceActor::new(32, next_product_id);
    let client = ProductClient::new(generic_client);

    (actor, client)
}
