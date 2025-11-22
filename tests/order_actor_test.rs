use actor_recipe::clients::{OrderClient, UserClient, ProductClient};
use actor_recipe::model::{Order, User, Product};
use actor_recipe::framework::mock::MockClient;
use actor_recipe::product_actor::ProductActionResult;

#[tokio::test]
async fn test_order_creation_flow() {
    // 1. Setup Mocks
    let mut user_mock = MockClient::<User>::new();
    let mut product_mock = MockClient::<Product>::new();
    let mut order_mock = MockClient::<Order>::new();

    // 2. Define Expectations
    user_mock.expect_get("user_1".to_string())
        .return_ok(Some(User::new("user_1", "test@example.com")));

    product_mock.expect_get("product_1".to_string())
        .return_ok(Some(Product::new("product_1", "Test Product", 20.0, 100)));

    product_mock.expect_action("product_1".to_string())
        .return_ok(ProductActionResult::ReserveStock(()));

    order_mock.expect_create()
        .return_ok("order_1".to_string());

    // 3. Execute
    let user_client = UserClient::new(user_mock.client());
    let product_client = ProductClient::new(product_mock.client());
    let order_client = OrderClient::new(order_mock.client(), user_client, product_client);

    let order = Order::new("order_1", "user_1", "product_1", 5, 100.0);
    let result = order_client.create_order(order).await;

    // 4. Verify
    assert_eq!(result, Ok("order_1".to_string()));
    user_mock.verify();
    product_mock.verify();
    order_mock.verify();
}
