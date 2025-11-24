//! # Order Client
//!
//! Provides a high‑level API for interacting with the `Order` actor.
//! It wraps a `ResourceClient<Order>` and handles orchestration logic.
use crate::clients::actor_client::ActorClient;
use actor_framework::{FrameworkError, ResourceClient};
use crate::model::Order;
use crate::order_actor::OrderError;
use async_trait::async_trait;
use tracing::{debug, info, instrument};

/// Client for interacting with the Order actor.
///
/// Orchestration logic (user validation, stock reservation) now happens
/// in the Order actor's `on_create` hook.
#[derive(Clone)]
pub struct OrderClient {
    inner: ResourceClient<Order>,
}

impl OrderClient {
    pub fn new(inner: ResourceClient<Order>) -> Self {
        Self { inner }
    }

    #[instrument(skip(self))]
    pub async fn create_order(&self, params: crate::model::OrderCreate) -> Result<String, OrderError> {
        debug!("create_order called");
        info!("Sending create_order to actor");

        // Create order - validation happens in Order::on_create
        self.inner
            .create(params)
            .await
            .map_err(|e| OrderError::ActorCommunicationError(e.to_string()))
    }
}

#[async_trait]
impl ActorClient<Order> for OrderClient {
    type Error = OrderError;

    fn inner(&self) -> &ResourceClient<Order> {
        &self.inner
    }

    fn map_error(e: FrameworkError) -> Self::Error {
        OrderError::ActorCommunicationError(e.to_string())
    }
}
