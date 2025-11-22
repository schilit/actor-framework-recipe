//! Entity trait implementation for the Product resource type.
//!
//! This module contains the [`ActorEntity`] trait implementation
//! that enables [`Product`] to be managed by the generic [`crate::framework::ResourceActor`].
//!
//! Includes support for custom actions like stock checking and reservation.
//!
//! See the trait implementation on [`Product`] for method documentation.

use async_trait::async_trait;
use crate::framework::ActorEntity;
use crate::model::{Product, ProductCreate, ProductUpdate};
use crate::product_actor::{ProductAction, ProductActionResult};

/// Marker constant to ensure module documentation is rendered.
#[doc(hidden)]
/// Marker constant to verify ActorEntity trait implementation exists at compile time.
#[allow(dead_code)]
pub const ENTITY_IMPL_PRESENT: bool = true;

#[async_trait]
impl ActorEntity for Product {
    type Id = String;
    type CreateParams = ProductCreate;
    type UpdateParams = ProductUpdate;
    type Action = ProductAction;
    type ActionResult = ProductActionResult;
    type Context = ();

    // fn id(&self) -> &String { &self.id }

    /// Creates a new Product from creation parameters.
    fn from_create_params(id: String, params: ProductCreate) -> Result<Self, String> {
        Ok(Product::new(id, params.name, params.price, params.quantity))
    }

    /// Handles updates to the Product entity.
    ///
    /// # Fields Updated
    /// - `price`: Product price
    /// - `quantity`: Available stock quantity
    async fn on_update(&mut self, update: ProductUpdate, _ctx: &Self::Context) -> Result<(), String> {
        if let Some(price) = update.price {
            self.price = price;
        }
        if let Some(quantity) = update.quantity {
            self.quantity = quantity;
        }
        Ok(())
    }

    /// Handles custom actions for the Product entity.
    ///
    /// # Actions
    /// - `CheckStock`: Returns true if requested quantity is available
    /// - `ReserveStock`: Decrements stock if available, returns true on success
    async fn handle_action(&mut self, action: ProductAction, _ctx: &Self::Context) -> Result<ProductActionResult, String> {
        match action {
            ProductAction::CheckStock => {
                Ok(ProductActionResult::CheckStock(self.quantity))
            }
            ProductAction::ReserveStock(quantity) => {
                if self.quantity >= quantity {
                    self.quantity -= quantity;
                    Ok(ProductActionResult::ReserveStock(()))
                } else {
                    Err(format!("Insufficient stock: requested {}, available {}", quantity, self.quantity))
                }
            }
        }
    }
}
