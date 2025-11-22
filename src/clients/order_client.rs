use tracing::{info, instrument, debug};
use crate::model::Order;
use crate::order_actor::OrderError;
use crate::framework::{ResourceClient, FrameworkError};
use async_trait::async_trait;
use crate::clients::actor_client::ActorClient;

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

    #[instrument(skip(self, order))]
    pub async fn create_order(&self, order: Order) -> Result<String, OrderError> {
        debug!(?order, "create_order called");
        info!("Sending create_order to actor");

        // Create order - validation happens in Order::on_create
        let payload = crate::model::OrderCreate {
            user_id: order.user_id,
            product_id: order.product_id,
            quantity: order.quantity,
            total: order.total,
        };

        self.inner.create(payload).await
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
