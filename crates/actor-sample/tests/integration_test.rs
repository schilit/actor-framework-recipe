use actor_sample::clients::actor_client::ActorClient;
use actor_sample::lifecycle::OrderSystem;
use actor_sample::model::{Order, OrderCreate, Product, ProductCreate, User, UserCreate};

/// Full end-to-end integration test with all real actors.
/// This tests the entire system working together.
#[tokio::test]
async fn test_full_order_system_integration() {
    // Create the full system with all real actors
    let system = OrderSystem::new();

    // Create a user
    let user_params = UserCreate {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let user_id = system
        .user_client
        .create_user(user_params)
        .await
        .expect("Failed to create user");

    // Verify user was created
    let retrieved_user = system
        .user_client
        .get(user_id.clone())
        .await
        .expect("Failed to get user")
        .expect("User not found");
    assert_eq!(retrieved_user.name, "Alice");
    assert_eq!(retrieved_user.email, "alice@example.com");

    // Create a product with stock
    let product_params = ProductCreate {
        name: "Super Widget".to_string(),
        price: 25.50,
        quantity: 100,
    };
    let product_id = system
        .product_client
        .create_product(product_params)
        .await
        .expect("Failed to create product");

    // Verify initial stock level
    let initial_stock = system
        .product_client
        .check_stock(product_id.clone())
        .await
        .expect("Failed to check stock");
    assert_eq!(initial_stock, 100);

    // Create an order (should reserve stock)
    let order_params = OrderCreate {
        user_id: user_id.clone(),
        product_id: product_id.clone(),
        quantity: 5,
        total: 127.50,
    };
    let order_id = system
        .order_client
        .create_order(order_params)
        .await
        .expect("Failed to create order");

    // Verify order was created with correct details
    let retrieved_order = system
        .order_client
        .get(order_id.clone())
        .await
        .expect("Failed to get order")
        .expect("Order not found");
    assert_eq!(retrieved_order.user_id, user_id);
    assert_eq!(retrieved_order.product_id, product_id);
    assert_eq!(retrieved_order.quantity, 5);
    assert_eq!(retrieved_order.total, 127.50);

    // Verify stock was decremented
    let final_stock = system
        .product_client
        .check_stock(product_id.clone())
        .await
        .expect("Failed to check stock");
    assert_eq!(
        final_stock, 95,
        "Stock should be decremented by order quantity"
    );

    // Test insufficient stock scenario
    let large_order_params = OrderCreate {
        user_id: user_id.clone(),
        product_id: product_id.clone(),
        quantity: 200,
        total: 5100.0,
    };
    let result = system.order_client.create_order(large_order_params).await;
    assert!(result.is_err(), "Should fail when stock is insufficient");

    // Verify stock wasn't changed after failed order
    let stock_after_failure = system
        .product_client
        .check_stock(product_id.clone())
        .await
        .expect("Failed to check stock");
    assert_eq!(
        stock_after_failure, 95,
        "Stock should not change on failed order"
    );

    // Graceful shutdown
    system.shutdown().await.expect("Failed to shutdown system");
}

/// Test concurrent order creation to verify actor isolation.
#[tokio::test]
async fn test_concurrent_orders() {
    let system = OrderSystem::new();

    // Create a user
    let user_params = UserCreate {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    let user_id = system.user_client.create_user(user_params).await.unwrap();

    // Create a product with limited stock
    let product_params = ProductCreate {
        name: "Limited Widget".to_string(),
        price: 10.0,
        quantity: 20,
    };
    let product_id = system
        .product_client
        .create_product(product_params)
        .await
        .unwrap();

    // Create multiple orders concurrently
    let mut handles = vec![];
    for _i in 0..10 {
        let order_client = system.order_client.clone();
        let uid = user_id.clone();
        let pid = product_id.clone();

        let handle = tokio::spawn(async move {
            let order_params = OrderCreate {
                user_id: uid,
                product_id: pid,
                quantity: 2,
                total: 20.0,
            };
            order_client.create_order(order_params).await
        });
        handles.push(handle);
    }

    // Wait for all orders to complete
    let mut successful = 0;
    let mut failed = 0;
    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successful += 1,
            Err(_) => failed += 1,
        }
    }

    // Exactly 10 orders should succeed (20 stock / 2 per order)
    // The rest should fail due to insufficient stock
    assert_eq!(successful, 10, "Expected exactly 10 successful orders");
    assert_eq!(failed, 0, "Expected no failures with sufficient stock");

    // Verify final stock is zero
    let final_stock = system.product_client.check_stock(product_id).await.unwrap();
    assert_eq!(final_stock, 0, "All stock should be consumed");

    system.shutdown().await.unwrap();
}
