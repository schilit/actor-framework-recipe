use actor_recipe::clients::{OrderClient, UserClient, ProductClient, actor_client::ActorClient};
use actor_recipe::model::{Order, User, Product};
use actor_recipe::framework::{ResourceActor, mock::MockClient};
use actor_recipe::product_actor::ProductActionResult;

/// Unit test: Tests OrderClient coordination logic with all mocked dependencies.
/// This verifies that OrderClient calls the right methods in the right order.
#[tokio::test]
async fn test_order_client_coordination() {
    // Setup mocks for all dependencies
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();
    let mut order_mock = MockClient::<Order>::new();

    // Define expectations
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "test@example.com")));

    product_mock.expect_get("product_1".to_string())
        .return_ok(Some(Product::new("product_1", "Test Product", 20.0, 100)));

    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    order_mock.expect_create()
        .return_ok("order_1".to_string());

    // Create client with mocked dependencies
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());
    let order_client = OrderClient::new(order_mock.client(), user_client, product_client);

    // Execute
    let order = Order::new("order_1", "user_1", "product_1", 5, 100.0);
    let result = order_client.create_order(order).await;

    // Verify
    assert_eq!(result, Ok("order_1".to_string()));
    user_mock.verify();
    product_mock.verify();
    order_mock.verify();
}

/// Integration test: Real Order actor with mocked User and Product dependencies.
/// This tests the Order actor's logic while isolating it from User/Product actors.
#[tokio::test]
async fn test_order_actor_with_mocked_dependencies() {
    // Setup mock dependencies
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();

    // Define expectations for the dependencies
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "alice@example.com")));

    product_mock.expect_get("product_1".to_string())
        .return_ok(Some(Product::new("product_1", "Widget", 25.0, 50)));

    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    // Create REAL Order actor
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    let order_id_counter = Arc::new(AtomicU64::new(1));
    let (order_actor, order_resource_client) = ResourceActor::<Order>::new(32, move || {
        let id = order_id_counter.fetch_add(1, Ordering::SeqCst);
        format!("order_{}", id)
    });

    // Spawn the real actor
    let actor_handle = tokio::spawn(order_actor.run());

    // Create OrderClient with real Order actor but mocked dependencies
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());
    let order_client = OrderClient::new(order_resource_client, user_client, product_client);

    // Execute: This will run through the REAL Order actor
    let order = Order::new("", "user_1", "product_1", 3, 75.0);
    let result = order_client.create_order(order).await;

    // Verify the order was created
    assert!(result.is_ok());
    let order_id = result.unwrap();

    // Verify we can retrieve the order from the real actor
    let retrieved_order = order_client.get(order_id.clone()).await.unwrap();
    assert!(retrieved_order.is_some());
    let order = retrieved_order.unwrap();
    assert_eq!(order.user_id, "user_1");
    assert_eq!(order.product_id, "product_1");
    assert_eq!(order.quantity, 3);

    // Verify mocks were called correctly
    user_mock.verify();
    product_mock.verify();

    // Cleanup
    drop(order_client);
    actor_handle.await.unwrap();
}
