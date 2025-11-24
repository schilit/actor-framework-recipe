/// Represents a product in the inventory.
///
/// # Actor Framework
/// This struct implements the [`ActorEntity`](actor_framework::ActorEntity) trait,
/// allowing it to be managed by a [`ResourceActor`](actor_framework::ResourceActor).
///
/// See [`impl ActorEntity for Product`](#impl-ActorEntity-for-Product) for details on:
/// - Creation parameters ([`ProductCreate`](crate::model::ProductCreate))
/// - Update parameters ([`ProductUpdate`](crate::model::ProductUpdate))
/// - Custom actions ([`ProductAction`](crate::product_actor::actions::ProductAction))
use serde::{Deserialize, Serialize};

use std::fmt::Display;

/// Type-safe identifier for Products.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductId(pub u32);

impl From<u32> for ProductId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl Display for ProductId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "product_{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Product {
    #[allow(dead_code)]
    pub id: ProductId,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

impl Product {
    /// Creates a new Product instance.
    ///
    /// # Arguments
    /// * `id` - Unique identifier (typically set by the actor system)
    /// * `name` - Product name
    /// * `price` - Product price
    /// * `quantity` - Available stock quantity
    pub fn new(id: ProductId, name: impl Into<String>, price: f64, quantity: u32) -> Self {
        Self {
            id,
            name: name.into(),
            price,
            quantity,
        }
    }
}

/// DTOs for Product creation and updates.
#[derive(Debug, Clone)]
pub struct ProductCreate {
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

// DTOs for Product updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductUpdate {
    pub price: Option<f64>,
    pub quantity: Option<u32>,
}
