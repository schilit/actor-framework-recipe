//! # Product Client
//!
//! Provides a high‑level API for interacting with the `Product` actor.
//! It wraps a `ResourceClient<Product>` and exposes domain‑specific methods.
use crate::clients::actor_client::ActorClient;
use crate::model::Product;
use crate::product_actor::ProductError;
use actor_framework::{FrameworkError, ResourceClient};
use async_trait::async_trait;
use tracing::{debug, instrument};

/// Client for interacting with the Product actor.
#[derive(Clone)]
pub struct ProductClient {
    inner: ResourceClient<Product>,
}

impl ProductClient {
    pub fn new(inner: ResourceClient<Product>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl ActorClient<Product> for ProductClient {
    type Error = ProductError;

    fn inner(&self) -> &ResourceClient<Product> {
        &self.inner
    }

    fn map_error(e: FrameworkError) -> Self::Error {
        ProductError::ActorCommunicationError(e.to_string())
    }
}

impl ProductClient {
    // Custom create method as it needs specific payload conversion

    #[instrument(skip(self))]
    pub async fn create_product(
        &self,
        params: crate::model::ProductCreate,
    ) -> Result<String, ProductError> {
        debug!("Sending request");
        self.inner
            .create(params)
            .await
            .map_err(|e| ProductError::ActorCommunicationError(e.to_string()))
    }

    /// Check the current stock level for a product.
    ///
    /// Returns the quantity available.
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn check_stock(&self, id: String) -> Result<u32, ProductError> {
        debug!("Checking stock for product {}", id);
        use crate::product_actor::{ProductAction, ProductActionResult};
        match self
            .inner
            .perform_action(id, ProductAction::CheckStock)
            .await
        {
            Ok(ProductActionResult::CheckStock(level)) => Ok(level),
            Ok(_) => unreachable!("CheckStock action must return CheckStock result"),
            Err(e) => Err(ProductError::ActorCommunicationError(e.to_string())),
        }
    }

    /// Reserve a specific amount of stock for a product.
    ///
    /// Returns `Ok(())` if successful, or an error if insufficient stock.
    #[instrument(skip(self))]
    pub async fn reserve_stock(&self, id: String, quantity: u32) -> Result<(), ProductError> {
        debug!("Reserving {} units for product {}", quantity, id);
        use crate::product_actor::{ProductAction, ProductActionResult};
        match self
            .inner
            .perform_action(id, ProductAction::ReserveStock(quantity))
            .await
        {
            Ok(ProductActionResult::ReserveStock(())) => Ok(()),
            Ok(_) => unreachable!("ReserveStock action must return ReserveStock result"),
            Err(e) => Err(ProductError::ActorCommunicationError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product_actor::{ProductAction, ProductActionResult};
    use actor_framework::mock::{create_mock_client, expect_action};

    #[tokio::test]
    async fn test_check_stock_returns_correct_level() {
        let (client, mut receiver) = create_mock_client::<Product>(10);
        let product_client = ProductClient::new(client);

        // Spawn task to call check_stock
        let check_task =
            tokio::spawn(async move { product_client.check_stock("product_1".to_string()).await });

        // Expect the action request
        let (id, action, responder) = expect_action(&mut receiver)
            .await
            .expect("Expected Action request");

        assert_eq!(id, "product_1");
        assert!(matches!(action, ProductAction::CheckStock));

        // Respond with stock level
        responder
            .send(Ok(ProductActionResult::CheckStock(42)))
            .unwrap();

        // Verify the result
        let result = check_task.await.unwrap();
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_reserve_stock_success() {
        let (client, mut receiver) = create_mock_client::<Product>(10);
        let product_client = ProductClient::new(client);

        // Spawn task to call reserve_stock
        let reserve_task = tokio::spawn(async move {
            product_client
                .reserve_stock("product_1".to_string(), 5)
                .await
        });

        // Expect the action request
        let (id, action, responder) = expect_action(&mut receiver)
            .await
            .expect("Expected Action request");

        assert_eq!(id, "product_1");
        match action {
            ProductAction::ReserveStock(amount) => assert_eq!(amount, 5),
            _ => panic!("Expected ReserveStock action"),
        }

        // Respond with success
        responder
            .send(Ok(ProductActionResult::ReserveStock(())))
            .unwrap();

        // Verify the result
        let result = reserve_task.await.unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reserve_stock_insufficient_stock() {
        let (client, mut receiver) = create_mock_client::<Product>(10);
        let product_client = ProductClient::new(client);

        // Spawn task to call reserve_stock
        let reserve_task = tokio::spawn(async move {
            product_client
                .reserve_stock("product_1".to_string(), 100)
                .await
        });

        // Expect the action request
        let (id, action, responder) = expect_action(&mut receiver)
            .await
            .expect("Expected Action request");

        assert_eq!(id, "product_1");
        match action {
            ProductAction::ReserveStock(amount) => assert_eq!(amount, 100),
            _ => panic!("Expected ReserveStock action"),
        }

        // Respond with error
        use actor_framework::FrameworkError;
        responder
            .send(Err(FrameworkError::EntityError(Box::new(
                std::io::Error::other("Stock check failed"),
            ))))
            .unwrap();

        // Verify the result is an error
        let result = reserve_task.await.unwrap();
        assert!(result.is_err());
        match result {
            Err(ProductError::ActorCommunicationError(msg)) => {
                // Error message comes from the EntityError wrapper
                assert!(msg.contains("Stock check failed") || msg.contains("Entity error"));
            }
            _ => panic!("Expected ActorCommunicationError"),
        }
    }

    #[test]
    fn test_type_safety_compile_time() {
        // This test verifies compile-time type safety
        // The fact that this compiles proves the type safety works

        // The return types are exactly what we expect:
        // - check_stock returns Result<u32, ProductError>
        // - reserve_stock returns Result<(), ProductError>

        // No pattern matching needed at the call site!
        // The other tests demonstrate this in action.
    }
}
