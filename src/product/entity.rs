use crate::actor_framework::Entity;
use crate::domain::Product;
use super::dtos::{ProductCreate, ProductPatch};
use super::actions::{ProductAction, ProductActionResult};

impl Entity for Product {
    type Id = String;
    type CreatePayload = ProductCreate;
    type Patch = ProductPatch;
    type Action = ProductAction;
    type ActionResult = ProductActionResult;

    fn id(&self) -> &String { &self.id }

    fn from_create(id: String, payload: ProductCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            name: payload.name,
            price: payload.price,
            quantity: payload.quantity,
        })
    }

    fn on_update(&mut self, patch: ProductPatch) -> Result<(), String> {
        if let Some(price) = patch.price {
            self.price = price;
        }
        if let Some(quantity) = patch.quantity {
            self.quantity = quantity;
        }
        Ok(())
    }

    fn handle_action(&mut self, action: ProductAction) -> Result<ProductActionResult, String> {
        match action {
            ProductAction::CheckStock => {
                Ok(ProductActionResult::StockLevel(self.quantity))
            }
            ProductAction::ReserveStock(amount) => {
                if self.quantity >= amount {
                    self.quantity -= amount;
                    Ok(ProductActionResult::Reserved)
                } else {
                    Err(format!("Insufficient stock: {} available, {} requested", self.quantity, amount))
                }
            }
        }
    }
}
