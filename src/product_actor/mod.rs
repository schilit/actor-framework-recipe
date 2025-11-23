//! Product-specific resource logic, including stock management actions.

mod actions;
pub mod entity;
pub mod error;

pub use actions::*;
pub use error::*;

use crate::clients::ProductClient;
use crate::framework::ResourceActor;
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
