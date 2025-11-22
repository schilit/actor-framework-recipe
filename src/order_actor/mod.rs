//! Order-specific resource logic and entity implementation.

pub mod entity;
pub mod error;

pub use error::*;

use crate::framework::ResourceActor;
use crate::clients::{OrderClient, UserClient, ProductClient};
use crate::model::Order;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Creates a new Order actor and its client.
pub fn new(user_client: UserClient, product_client: ProductClient) -> (ResourceActor<Order>, OrderClient) {
    let order_id_counter = Arc::new(AtomicU64::new(1));
    let next_order_id = move || {
        let id = order_id_counter.fetch_add(1, Ordering::SeqCst);
        format!("order_{}", id)
    };

    let (actor, generic_client) = ResourceActor::new(32, next_order_id);
    let client = OrderClient::new(generic_client, user_client, product_client);

    (actor, client)
}
