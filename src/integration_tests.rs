#[cfg(test)]
mod tests {
    use crate::clients::{OrderClient, UserClient, ProductClient};
    use crate::domain::{Order, User, Product};
    use crate::mock_framework::{create_mock_client, expect_get, expect_action};
    use crate::product_actor::{ProductAction, ProductActionResult};

    #[tokio::test]
    async fn test_order_creation_flow() {
        // 1. Setup Mocks
        let (user_client_inner, mut user_rx) = create_mock_client::<User>(10);
        let (product_client_inner, mut product_rx) = create_mock_client::<Product>(10);
        let (order_client_inner, mut order_rx) = create_mock_client::<Order>(10);

        let user_client = UserClient::new(user_client_inner);
        let product_client = ProductClient::new(product_client_inner);
        let order_client = OrderClient::new(order_client_inner, user_client, product_client);

        // 2. Execute Order Creation in background
        let order_task = tokio::spawn(async move {
            let order = Order::new("order_1", "user_1", "product_1", 5, 100.0);
            order_client.create_order(order).await
        });

        // 3. Verify Interactions

        // Expect User Get
        let (user_id, responder) = expect_get(&mut user_rx).await.expect("Expected User Get");
        assert_eq!(user_id, "user_1");
        let user = User::new("user_1", "test@example.com");
        responder.send(Ok(Some(user))).unwrap();

        // Expect Product Get
        let (product_id, responder) = expect_get(&mut product_rx).await.expect("Expected Product Get");
        assert_eq!(product_id, "product_1");
        let product = Product::new("product_1", "Test Product", 20.0, 100);
        responder.send(Ok(Some(product))).unwrap();

        // Expect Stock Reservation (Action)
        let (product_id, action, responder) = expect_action(&mut product_rx).await.expect("Expected Product Action");
        assert_eq!(product_id, "product_1");
        match action {
            ProductAction::ReserveStock(qty) => assert_eq!(qty, 5),
            _ => panic!("Unexpected action: {:?}", action),
        }
        responder.send(Ok(ProductActionResult::Reserved)).unwrap();

        // Expect Order Create
        use crate::mock_framework::expect_create;
        let (payload, responder) = expect_create(&mut order_rx).await.expect("Expected Order Create");
        assert_eq!(payload.user_id, "user_1");
        assert_eq!(payload.product_id, "product_1");
        assert_eq!(payload.quantity, 5);
        responder.send(Ok("order_1".to_string())).unwrap();

        // 4. Verify Result
        let result = order_task.await.unwrap();
        assert_eq!(result, Ok("order_1".to_string()));
    }
}
