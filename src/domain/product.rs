/// Represents a product in the inventory.
///
/// # Actor Framework
/// This struct implements the [`Entity`](crate::actor_framework::Entity) trait,
/// allowing it to be managed by a [`ResourceActor`](crate::actor_framework::ResourceActor).
///
/// See [`impl Entity for Product`](#impl-Entity-for-Product) for details on:
/// - Creation parameters ([`ProductCreate`](crate::domain::ProductCreate))
/// - Update parameters ([`ProductUpdate`](crate::domain::ProductUpdate))
/// - Custom actions ([`ProductAction`](crate::product_actor::actions::ProductAction))
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct Product {
    #[allow(dead_code)]
    pub id: String,
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
    pub fn new(id: impl Into<String>, name: impl Into<String>, price: f64, quantity: u32) -> Self {
        Self {
            id: id.into(),
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
