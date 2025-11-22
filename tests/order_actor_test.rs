use actor_recipe::clients::{OrderClient, UserClient, ProductClient, actor_client::ActorClient};
use actor_recipe::model::{Order, User, Product};
use actor_recipe::framework::{ResourceActor, mock::MockClient};
use actor_recipe::product_actor::ProductActionResult;

/// Integration test: Real Order actor with mocked User and Product dependencies.
/// This tests the Order actor's validation logic (on_create) while isolating it from User/Product actors.
/// 
/// Pattern 2: Actor + Mocks
/// - Real Order actor (tests actor logic in on_create)
/// - Mocked User and Product clients (isolates dependencies)
#[tokio::test]
async fn test_order_actor_with_mocked_dependencies() {
    // Setup mock dependencies
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();

    // Define expectations for the dependencies
    // Order::on_create will call user_client.get() and product_client.reserve_stock()
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "alice@example.com")));

    // reserve_stock() internally calls perform_action()
    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    // Create clients from mocks
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());

    // Create REAL Order actor using factory function (no dependencies)
    let (order_actor, order_client) = actor_recipe::order_actor::new();

    // Spawn the real actor with injected context
    let actor_handle = tokio::spawn(order_actor.run((
        user_client.clone(), 
        product_client.clone()
    )));

    // Execute: This will run through the REAL Order actor
    // The validation happens in Order::on_create
    let order = Order::new("", "user_1", "product_1", 3, 75.0);
    let result = order_client.create_order(order).await;

    // Verify the order was created
    assert!(result.is_ok(), "Order creation failed: {:?}", result.err());
    let order_id = result.unwrap();

    // Verify we can retrieve the order from the real actor
    let retrieved_order = order_client.get(order_id.clone()).await.unwrap();
    assert!(retrieved_order.is_some());
    let order = retrieved_order.unwrap();
    assert_eq!(order.user_id, "user_1");
    assert_eq!(order.product_id, "product_1");
    assert_eq!(order.quantity, 3);

    // Verify mocks were called correctly (by Order::on_create)
    user_mock.verify();
    product_mock.verify();

    // Cleanup
    drop(order_client);
    actor_handle.await.unwrap();
}
