use crate::actor_framework::Entity;
use crate::domain::{Product, ProductCreate, ProductPatch};
use super::actions::{ProductAction, ProductActionResult};

impl Entity for Product {
    type Id = String;
    type CreatePayload = ProductCreate;
    type Patch = ProductPatch;
    type Action = ProductAction;
    type ActionResult = ProductActionResult;

    // fn id(&self) -> &String { &self.id }

    fn from_create_params(id: String, params: ProductCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            name: params.name,
            price: params.price,
            quantity: params.quantity,
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
