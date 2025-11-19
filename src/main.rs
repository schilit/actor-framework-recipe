mod domain;
mod messages;
mod error;
mod clients;
mod actors;
mod system;

#[cfg(test)]
mod mock_test;

mod actor_framework;
mod user;
mod product;



use tracing::{error, info, Instrument};
use crate::domain::{User, Order};
use crate::system::{OrderSystem, setup_tracing};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Setup tracing once for the entire application
    setup_tracing();

    info!("Starting application with complete order system");

    // Create the entire order system (starts all services)
    let system = OrderSystem::new();

    // Create test user
    let user = User::new("Alice", "alice@example.com");

    let span = tracing::info_span!("user_creation");
    let user_id = async {
        info!("Creating test user");
        system.user_client.create_user(user).await
            .map_err(|e| e.to_string())
    }
    .instrument(span)
    .await?;

    info!(user_id = %user_id, "User created successfully");

    // Create test order - this will flow through multiple actors
    let order = Order::new("order_1", user_id, "p1", 5, 50.0);

    let span = tracing::info_span!("order_processing");
    let order_result = async {
        info!("Processing order through order system");
        system.order_client.create_order(order).await
    }
    .instrument(span)
    .await;

    match order_result {
        Ok(order_id) => info!(order_id = %order_id, "Order processed successfully"),
        Err(e) => {
            error!(error = %e, "Order processing failed (expected - no test products in stock)")
        }
    }

    // Shutdown system gracefully
    system.shutdown().await?;

    info!("Application completed successfully");
    Ok(())
}
