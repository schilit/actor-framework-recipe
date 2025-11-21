//! Entity trait implementation for [`Product`](crate::domain::Product).
//!
//! This module contains the [`Entity`](crate::actor_framework::Entity) trait implementation
//! that enables `Product` to be managed by the generic [`ResourceActor`](crate::actor_framework::ResourceActor).
//!
//! Includes support for custom actions like stock checking and reservation.
//!
//! See the [trait implementation on `Product`](crate::domain::Product#impl-Entity-for-Product) for method documentation.

use crate::actor_framework::Entity;
use crate::domain::{Product, ProductCreate, ProductUpdate};
use super::actions::{ProductAction, ProductActionResult};

impl Entity for Product {
    type Id = String;
    type CreateParams = ProductCreate;
    type UpdateParams = ProductUpdate;
    type Action = ProductAction;
    type ActionResult = ProductActionResult;

    // fn id(&self) -> &String { &self.id }

    /// Creates a new Product from creation parameters.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the product
    /// * `params` - Product creation parameters containing name, price, and quantity
    fn from_create_params(id: String, params: ProductCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            name: params.name,
            price: params.price,
            quantity: params.quantity,
        })
    }

    /// Updates the product's price and/or quantity.
    ///
    /// # Arguments
    /// * `patch` - Contains optional updates for price and/or quantity
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

    /// Handles product-specific actions.
    ///
    /// # Actions
    /// - `CheckStock`: Returns the current stock level
    /// - `ReserveStock(amount)`: Decrements stock by the specified amount
    ///
    /// # Errors
    /// Returns an error if attempting to reserve more stock than available.
    fn handle_action(&mut self, action: ProductAction) -> Result<ProductActionResult, String> {
        match action {
            ProductAction::CheckStock => {
                Ok(ProductActionResult::CheckStock(self.quantity))
            }
            ProductAction::ReserveStock(amount) => {
                if self.quantity >= amount {
                    self.quantity -= amount;
                    Ok(ProductActionResult::ReserveStock(()))
                } else {
                    Err(format!("Insufficient stock: {} available, {} requested", self.quantity, amount))
                }
            }
        }
    }
}
