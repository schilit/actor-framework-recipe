//! Entity trait implementation for the Order resource type.
//!
//! This module contains the [`ActorEntity`] trait implementation
//! that enables [`Order`] to be managed by the generic [`crate::framework::ResourceActor`].
//!
//! See the trait implementation on [`Order`] for method documentation.

use crate::clients::{actor_client::ActorClient, ProductClient, UserClient};
use crate::framework::ActorEntity;
use crate::model::{Order, OrderCreate};
use crate::order_actor::OrderError;
use async_trait::async_trait;

/// Marker constant to ensure module documentation is rendered.
#[doc(hidden)]
/// Marker constant to verify ActorEntity trait implementation exists at compile time.
#[allow(dead_code)]
pub const ENTITY_IMPL_PRESENT: bool = true;

#[async_trait]
impl ActorEntity for Order {
    type Id = String;
    type CreateParams = OrderCreate;
    type UpdateParams = (); // No updates for now
    type Action = (); // No custom actions for now
    type ActionResult = ();
    type Context = (UserClient, ProductClient);
    type Error = OrderError;

    // fn id(&self) -> &String { &self.id }

    /// Creates a new Order from creation parameters.
    fn from_create_params(id: Self::Id, params: Self::CreateParams) -> Result<Self, Self::Error> {
        Ok(Self::new(
            id,
            params.user_id,
            params.product_id,
            params.quantity,
            params.total,
        ))
    }

    /// Validates the order by checking User existence and reserving Product stock.
    async fn on_create(
        &mut self,
        (user_client, product_client): &Self::Context,
    ) -> Result<(), Self::Error> {
        // 1. Validate User
        let user = user_client.get(self.user_id.clone()).await?;

        if user.is_none() {
            return Err(OrderError::InvalidUser(self.user_id.clone()));
        }

        // 2. Reserve Stock - errors automatically convert via #[from]
        product_client
            .reserve_stock(self.product_id.clone(), self.quantity)
            .await?;

        Ok(())
    }

    async fn handle_action(
        &mut self,
        _action: Self::Action,
        _ctx: &Self::Context,
    ) -> Result<Self::ActionResult, Self::Error> {
        Ok(())
    }

    async fn on_update(
        &mut self,
        _update: Self::UpdateParams,
        _ctx: &Self::Context,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_delete(&self, _ctx: &Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }
}
