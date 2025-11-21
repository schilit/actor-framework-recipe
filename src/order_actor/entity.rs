use crate::actor_framework::Entity;
use crate::domain::{Order, OrderCreate};

impl Entity for Order {
    type Id = String;
    type CreatePayload = OrderCreate;
    type Patch = (); // No updates for now
    type Action = (); // No custom actions for now
    type ActionResult = ();

    // fn id(&self) -> &String { &self.id }

    /// Creates a new Order from creation parameters.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the order
    /// * `params` - Order creation parameters containing user_id, product_id, quantity, and total
    ///
    /// # Notes
    /// The order is initialized with status "Created".
    fn from_create_params(id: String, params: OrderCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            user_id: params.user_id,
            product_id: params.product_id,
            quantity: params.quantity,
            total: params.total,
            status: "Created".to_string(),
        })
    }

    /// Updates the order.
    ///
    /// Currently, no updates are supported for orders.
    fn on_update(&mut self, _patch: ()) -> Result<(), String> {
        Ok(())
    }

    /// Handles order-specific actions.
    ///
    /// Currently, no custom actions are defined for orders.
    fn handle_action(&mut self, _action: ()) -> Result<(), String> {
        Ok(())
    }
}
