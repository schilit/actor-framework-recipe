/// Represents a customer order.
///
/// # Actor Framework
/// This struct implements the [`ActorEntity`](actor_framework::ActorEntity) trait,
/// allowing it to be managed by a [`ResourceActor`](actor_framework::ResourceActor).
///
/// See [`impl ActorEntity for Order`](#impl-ActorEntity-for-Order) for details on:
/// - Creation parameters ([`OrderCreate`])
use crate::model::{ProductId, UserId};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Type-safe identifier for Orders.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub u32);

impl From<u32> for OrderId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "order_{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    #[allow(dead_code)]
    pub id: OrderId,
    pub user_id: UserId,
    pub product_id: ProductId,
    pub quantity: u32,
    pub total: f64,
    #[allow(dead_code)]
    pub status: String,
}

/// Payload for creating a new order.
#[derive(Debug, Clone)]
pub struct OrderCreate {
    pub user_id: UserId,
    pub product_id: ProductId,
    pub quantity: u32,
    pub total: f64,
}

impl Order {
    /// Creates a new Order instance.
    ///
    /// # Arguments
    /// * `id` - Unique identifier (typically set by the actor system)
    /// * `user_id` - ID of the user placing the order
    /// * `product_id` - ID of the product being ordered
    /// * `quantity` - Quantity ordered
    /// * `total` - Total price for the order
    ///
    /// # Notes
    /// The order is initialized with status "Created".
    /// This constructor is kept for backward compatibility.
    pub fn new(
        id: OrderId,
        user_id: UserId,
        product_id: ProductId,
        quantity: u32,
        total: f64,
    ) -> Self {
        Self {
            id,
            user_id,
            product_id,
            quantity,
            total,
            status: "Created".to_string(),
        }
    }
}
