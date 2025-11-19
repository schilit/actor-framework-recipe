use tracing::{error, info, instrument};
use crate::domain::{Order, OrderCreate};
use crate::app_system::OrderError;
use crate::actor_framework::ResourceClient;
use crate::clients::{UserClient, ProductClient};

/// Client for interacting with the Order actor.
///
/// This client handles complex orchestration, validating users and products
/// before creating an order.
#[derive(Clone)]
pub struct OrderClient {
    inner: ResourceClient<Order>,
    user_client: UserClient,
    product_client: ProductClient,
}

impl OrderClient {
    pub fn new(
        inner: ResourceClient<Order>,
        user_client: UserClient,
        product_client: ProductClient
    ) -> Self {
        Self { 
            inner,
            user_client,
            product_client,
        }
    }

    #[instrument(skip(self))]
    pub async fn create_order(&self, order: Order) -> Result<String, OrderError> {
        info!("Processing create_order request (Client Side)");

        // Step 1: Validate user
        match self.user_client.get_user(order.user_id.clone()).await {
            Ok(Some(user)) => info!(user_name = %user.name, "User validation successful"),
            Ok(None) => {
                error!("User not found");
                return Err(OrderError::InvalidUser(order.user_id.clone()));
            }
            Err(e) => {
                error!(error = %e, "User validation failed");
                return Err(OrderError::InvalidUser(format!("User validation failed: {}", e)));
            }
        }

        // Step 2: Validate product
        match self.product_client.get_product(order.product_id.clone()).await {
            Ok(Some(product)) => info!(product_name = %product.name, "Product validation successful"),
            Ok(None) => {
                error!("Product not found");
                return Err(OrderError::InvalidProduct(order.product_id.clone()));
            }
            Err(e) => {
                error!(error = %e, "Product validation failed");
                return Err(OrderError::InvalidProduct(format!("Product validation failed: {}", e)));
            }
        }

        // Step 3: Reserve stock
        if let Err(e) = self.product_client.reserve_stock(order.product_id.clone(), order.quantity).await {
            error!(error = %e, "Stock reservation failed");
            return Err(OrderError::InsufficientStock(format!("Stock reservation failed: {}", e)));
        }

        info!("Stock reserved successfully");

        // Step 4: Create order in ResourceActor
        let payload = OrderCreate {
            user_id: order.user_id,
            product_id: order.product_id,
            quantity: order.quantity,
            total: order.total,
        };

        self.inner.create(payload).await.map_err(|e| OrderError::ActorCommunicationError(e.to_string()))
    }
}

impl_client_methods!(OrderClient, Order, OrderError, order);
