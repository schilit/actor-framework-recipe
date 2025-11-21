//! Entity trait implementation for the Order domain type.
//!
//! This module contains the [`Entity`] trait implementation
//! that enables [`Order`] to be managed by the generic [`crate::actor_framework::ResourceActor`].
//!
//! See the trait implementation on [`Order`] for method documentation.

use crate::actor_framework::Entity;
use crate::domain::{Order, OrderCreate};

/// Marker constant to ensure module documentation is rendered.
#[doc(hidden)]
pub const ENTITY_IMPL_PRESENT: bool = true;

impl Entity for Order {
    type Id = String;
    type CreateParams = OrderCreate;
    type UpdateParams = (); // No updates for now
    type Action = (); // No custom actions for now
    type ActionResult = ();

    // fn id(&self) -> &String { &self.id }

    /// Creates a new Order from creation parameters.
    fn from_create_params(id: Self::Id, params: Self::CreateParams) -> Result<Self, String> {
        Ok(Self::new(id, params.user_id, params.product_id, params.quantity, params.total))
    }

    fn handle_action(&mut self, _action: Self::Action) -> Result<Self::ActionResult, String> {
        Ok(())
    }

    fn on_update(&mut self, _update: Self::UpdateParams) -> Result<(), String> {
        Ok(())
    }
}
