//! Order-specific resource logic and entity implementation.

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
