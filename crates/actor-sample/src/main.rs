//! # Actor Framework Recipe
//!
//! A reference implementation of a type-safe, generic Actor Framework in Rust.
//!
//! ## 🚀 Core Components
//!
//! - **[framework]**: The heart of the system. Contains the generic [`ResourceActor`](actor_framework::ResourceActor) and [`Entity`](actor_framework::ActorEntity) trait.
//! - **[model]**: Pure data structures ([`User`], [`Product`], [`Order`]) that implement the `Entity` trait.
//! - **[clients]**: Type-safe wrappers (e.g., [`UserClient`](clients::UserClient)) that hide the complexity of message passing.
//! - **[lifecycle]**: Orchestration layer that manages the lifecycle of actors.
//!
//! ## 📚 Quick Start
//!
//! The application entry point is in [`main`], which demonstrates:
//! 1.  Setting up the [`OrderSystem`].
//! 2.  Creating a [`User`] and [`Product`].
//! 3.  Placing [`Order`].
//!
//! ## 🧪 Testing
//!
//! See [`actor_framework::mock`] for utilities to test clients without spawning full actors.

use actor_framework::tracing::setup_tracing;
use actor_sample::lifecycle::OrderSystem;
use actor_sample::model::{OrderCreate, ProductCreate, UserCreate};
use tracing::{error, info, Instrument};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Setup tracing once for the entire application
    setup_tracing();

    info!("Starting application with complete order system");

    // Create the entire order system (starts all services)
    let system = OrderSystem::new();

    // Create test user
    let user_params = UserCreate {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let span = tracing::info_span!("user_creation");
    let user_id = async {
        info!("Creating test user");
        system
            .user_client
            .create_user(user_params)
            .await
            .map_err(|e| e.to_string())
    }
    .instrument(span)
    .await?;

    info!(user_id = %user_id, "User created successfully");

    // Create test product
    let product_params = ProductCreate {
        name: "Test Product".to_string(),
        price: 100.0,
        quantity: 10,
    };
    let product_id = async {
        info!("Creating test product");
        system
            .product_client
            .create_product(product_params)
            .await
            .map_err(|e| e.to_string())
    }
    .await?;

    info!(product_id = %product_id, "Product created successfully");

    // Create test order - this will flow through multiple actors
    let order_params = OrderCreate {
        user_id: user_id.clone(),
        product_id: product_id.clone(),
        quantity: 5,
        total: 500.0,
    };

    let span = tracing::info_span!("order_processing");
    let order_result = async {
        info!("Processing order through order system");
        system.order_client.create_order(order_params).await
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
