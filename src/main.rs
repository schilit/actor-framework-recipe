//! # Actor Framework Recipe
//!
//! A reference implementation of a type-safe, generic Actor Framework in Rust.
//!
//! ## 🚀 Core Components
//!
//! - **[actor_framework]**: The heart of the system. Contains the generic [`ResourceActor`](actor_framework::ResourceActor) and [`Entity`](actor_framework::Entity) trait.
//! - **[domain]**: Pure data structures ([`User`](domain::User), [`Product`](domain::Product), [`Order`](domain::Order)) that implement the `Entity` trait.
//! - **[clients]**: Type-safe wrappers (e.g., [`UserClient`](clients::UserClient)) that hide the complexity of message passing.
//! - **[app_system]**: Orchestration layer that manages the lifecycle of actors.
//!
//! ## 📚 Quick Start
//!
//! The application entry point is in [`main`](main), which demonstrates:
//! 1.  Setting up the [`OrderSystem`](app_system::OrderSystem).
//! 2.  Creating a [`User`](domain::User) and [`Product`](domain::Product).
//! 3.  Placing an [`Order`](domain::Order).
//!
//! ## 🧪 Testing
//!
//! See [`mock_framework`] for utilities to test clients without spawning full actors.

mod domain;
mod clients;

mod app_system;

#[cfg(test)]
mod mock_framework;
#[cfg(test)]
mod integration_tests;

mod actor_framework;
mod user_actor;
mod product_actor;



use tracing::{error, info, Instrument};
use crate::domain::{User, Order, Product};
use crate::app_system::{OrderSystem, setup_tracing};

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

    // Create test product
    let product = Product::new("temp_id", "Test Product", 100.0, 10);
    let product_id = async {
        info!("Creating test product");
        system.product_client.create_product(product).await
            .map_err(|e| e.to_string())
    }.await?;

    info!(product_id = %product_id, "Product created successfully");

    // Create test order - this will flow through multiple actors
    // Note: The ID passed to Order::new is ignored by the system during creation, 
    // as the system generates a new ID.
    let order = Order::new("temp_order_id", user_id, product_id, 5, 50.0);

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
            error!(error = %e, "Order processing failed")
        }
    }

    // Shutdown system gracefully
    system.shutdown().await?;

    info!("Application completed successfully");
    Ok(())
}
