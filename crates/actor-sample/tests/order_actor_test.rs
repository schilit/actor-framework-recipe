use actor_framework::mock::MockClient;
use actor_framework::ActorClient;
use actor_sample::clients::{OrderClient, ProductClient, UserClient};
use actor_sample::model::{Order, OrderCreate, Product, ProductId, User, UserId};
use actor_sample::product_actor::ProductActionResult;

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
    user_mock
        .expect_get(UserId(1))
        .return_ok(Some(User::new("Alice", "alice@example.com")));

    // reserve_stock() internally calls perform_action()
    product_mock
        .expect_action(ProductId(1))
        .return_ok(ProductActionResult::ReserveStock(()));

    // Create clients from mocks
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());

    // Create REAL Order actor using factory function (no dependencies)
    let (order_actor, order_generic_client) = actor_sample::order_actor::new();
    let order_client = OrderClient::new(order_generic_client);

    // Spawn the real actor with injected context
    let actor_handle = tokio::spawn(order_actor.run((user_client.clone(), product_client.clone())));

    // Execute: This will run through the REAL Order actor
    // The validation happens in Order::on_create
    let order_params = OrderCreate {
        user_id: UserId(1),
        product_id: ProductId(1),
        quantity: 3,
        total: 75.0,
    };
    let result = order_client.create_order(order_params).await;

    // Verify the order was created
    assert!(result.is_ok(), "Order creation failed: {:?}", result.err());
    let order_id = result.unwrap();

    // Verify we can retrieve the order from the real actor
    let retrieved_order = order_client.get(order_id.clone()).await.unwrap();
    assert!(retrieved_order.is_some());
    let order = retrieved_order.unwrap();
    assert_eq!(order.user_id, UserId(1));
    assert_eq!(order.product_id, ProductId(1));
    assert_eq!(order.quantity, 3);

    // Verify mocks were called correctly (by Order::on_create)
    user_mock.verify();
    product_mock.verify();

    // Cleanup
    drop(order_client);
    actor_handle.await.unwrap();
}
