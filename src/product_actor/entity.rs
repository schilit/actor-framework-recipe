//! Entity trait implementation for the Product domain type.
//!
//! This module contains the [`Entity`] trait implementation
//! that enables [`Product`] to be managed by the generic [`crate::framework::ResourceActor`].
//!
//! Includes support for custom actions like stock checking and reservation.
//!
//! See the trait implementation on [`Product`] for method documentation.

use crate::framework::Entity;
use crate::model::{Product, ProductCreate, ProductUpdate};
use super::actions::{ProductAction, ProductActionResult};

/// Marker constant to ensure module documentation is rendered.
#[doc(hidden)]
/// Marker constant to verify Entity trait implementation exists at compile time.
#[allow(dead_code)]
pub const ENTITY_IMPL_PRESENT: bool = true;

impl Entity for Product {
    type Id = String;
    type CreateParams = ProductCreate;
    type UpdateParams = ProductUpdate;
    type Action = ProductAction;
    type ActionResult = ProductActionResult;

    // fn id(&self) -> &String { &self.id }

    /// Creates a new Product from creation parameters.
    fn from_create_params(id: String, params: ProductCreate) -> Result<Self, String> {
        Ok(Self::new(id, params.name, params.price, params.quantity))
    }

    /// Handles updates to the Product entity.
    ///
    /// # Fields Updated
    /// - `price`: Product price
    /// - `quantity`: Available stock quantity
    fn on_update(&mut self, update: ProductUpdate) -> Result<(), String> {
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
    fn handle_action(&mut self, action: ProductAction) -> Result<ProductActionResult, String> {
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
