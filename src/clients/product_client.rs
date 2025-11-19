use tracing::{debug, instrument};
use crate::domain::Product;
use crate::product_actor::ProductError;
use crate::actor_framework::ResourceClient;

/// Client for interacting with the Product actor.
#[derive(Clone)]
pub struct ProductClient {
    inner: ResourceClient<Product>,
}

impl_basic_client!(ProductClient, Product, ProductError, product);

impl ProductClient {
    // Custom create method as it needs specific payload conversion

    #[instrument(skip(self))]
    pub async fn create_product(&self, product: Product) -> Result<String, ProductError> {
        debug!("Sending request");
        let payload = crate::product_actor::ProductCreate {
            name: product.name,
            price: product.price,
            quantity: product.quantity,
        };
        self.inner.create(payload).await.map_err(|e| ProductError::ActorCommunicationError(e.to_string()))
    }

    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn check_stock(&self, id: String) -> Result<u32, ProductError> {
        debug!("Sending request");
        use crate::product_actor::{ProductAction, ProductActionResult};
        match self.inner.perform_action(id, ProductAction::CheckStock).await {
            Ok(ProductActionResult::StockLevel(level)) => Ok(level),
            Ok(_) => Err(ProductError::ActorCommunicationError("Unexpected result".to_string())),
            Err(e) => Err(ProductError::ActorCommunicationError(e.to_string())),
        }
    }

    #[instrument(skip(self))]
    pub async fn reserve_stock(&self, id: String, quantity: u32) -> Result<(), ProductError> {
        debug!("Sending request");
        use crate::product_actor::{ProductAction, ProductActionResult};
        match self.inner.perform_action(id, ProductAction::ReserveStock(quantity)).await {
            Ok(ProductActionResult::Reserved) => Ok(()),
            Ok(_) => Err(ProductError::ActorCommunicationError("Unexpected result".to_string())),
            Err(e) => Err(ProductError::ActorCommunicationError(e.to_string())),
        }
    }
}
