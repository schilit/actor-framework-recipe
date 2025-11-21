use tracing::{info, error};
use crate::clients::{OrderClient, UserClient, ProductClient};
use crate::framework::ResourceActor;
use crate::domain::{User, Product, Order};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The main runtime orchestrator for the actor-based order management system.
///
/// `OrderSystem` is responsible for:
/// - **Lifecycle Management**: Starting and stopping all actors in the system
/// - **Dependency Wiring**: Connecting actors that depend on each other (e.g., OrderClient needs UserClient)
/// - **Resource Coordination**: Managing shared resources like ID generators
///
/// # Architecture
///
/// The system consists of three main actors:
/// - **User Actor**: Manages user entities (CRUD operations)
/// - **Product Actor**: Manages product entities with stock tracking
/// - **Order Actor**: Manages orders and coordinates with User and Product actors
///
/// # Example
///
/// ```ignore
/// let system = OrderSystem::new();
/// 
/// // Use the clients to interact with actors
/// let user_id = system.user_client.create_user(user_data).await?;
/// let product_id = system.product_client.create_product(product_data).await?;
/// let order_id = system.order_client.create_order(order_data).await?;
///
/// // Gracefully shut down when done
/// system.shutdown().await?;
/// ```
pub struct OrderSystem {
    /// Client for interacting with the Order actor
    pub order_client: OrderClient,
    
    /// Client for interacting with the User actor
    pub user_client: UserClient,
    
    /// Client for interacting with the Product actor
    pub product_client: ProductClient,
    
    /// Task handles for all running actors (used for graceful shutdown)
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl OrderSystem {
    /// Creates and initializes a new `OrderSystem` with all actors running.
    ///
    /// This method:
    /// 1. Creates ID generators for each entity type
    /// 2. Spawns ResourceActors for User, Product, and Order
    /// 3. Wires up dependencies (OrderClient depends on UserClient and ProductClient)
    /// 4. Spawns each actor in its own Tokio task
    ///
    /// # Returns
    ///
    /// A fully initialized `OrderSystem` with all actors running and ready to accept requests.
    pub fn new() -> Self {
        // =====================================================================
        // 1. Setup User Actor
        // =====================================================================
        
        // Create a thread-safe counter for generating unique user IDs
        let user_id_counter = Arc::new(AtomicU64::new(1));
        let next_user_id = move || {
            let id = user_id_counter.fetch_add(1, Ordering::SeqCst);
            format!("user_{}", id)
        };
        
        // Create the User actor and its client
        // - Buffer size of 32 means the actor can queue up to 32 pending requests
        // - The actor runs independently in its own task
        let (user_actor, user_resource_client) = ResourceActor::<User>::new(32, next_user_id);
        let user_client = UserClient::new(user_resource_client);
        
        // Spawn the actor in a background task
        let user_handle = tokio::spawn(user_actor.run());

        // =====================================================================
        // 2. Setup Product Actor
        // =====================================================================
        
        // Create a thread-safe counter for generating unique product IDs
        let product_id_counter = Arc::new(AtomicU64::new(1));
        let next_product_id = move || {
            let id = product_id_counter.fetch_add(1, Ordering::SeqCst);
            format!("product_{}", id)
        };
        
        // Create the Product actor and its client
        let (product_actor, product_resource_client) = ResourceActor::<Product>::new(32, next_product_id);
        let product_client = ProductClient::new(product_resource_client);
        
        // Spawn the actor in a background task
        let product_handle = tokio::spawn(product_actor.run());

        // =====================================================================
        // 3. Setup Order Actor (with dependencies)
        // =====================================================================
        
        // Create a thread-safe counter for generating unique order IDs
        let order_id_counter = Arc::new(AtomicU64::new(1));
        let next_order_id = move || {
            let id = order_id_counter.fetch_add(1, Ordering::SeqCst);
            format!("order_{}", id)
        };

        // Create the Order actor and its client
        let (order_actor, order_resource_client) = ResourceActor::<Order>::new(32, next_order_id);
        
        // Wire up dependencies: OrderClient needs UserClient and ProductClient
        // to validate users and manage product stock when creating orders
        let order_client = OrderClient::new(
            order_resource_client,
            user_client.clone(),
            product_client.clone()
        );
        
        // Spawn the actor in a background task
        let order_handle = tokio::spawn(order_actor.run());

        // =====================================================================
        // Return the fully initialized system
        // =====================================================================
        
        Self {
            order_client,
            user_client,
            product_client,
            // Store handles for graceful shutdown
            handles: vec![user_handle, product_handle, order_handle],
        }
    }

    /// Gracefully shuts down the entire system.
    ///
    /// This method:
    /// 1. Drops all clients, which closes their communication channels
    /// 2. Waits for all actor tasks to complete
    /// 3. Returns an error if any actor task panicked
    ///
    /// # Shutdown Process
    ///
    /// When clients are dropped, the underlying channels are closed. Each `ResourceActor`
    /// detects the closed channel and exits its event loop gracefully.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all actors shut down cleanly
    /// - `Err(String)` if any actor task failed or panicked
    ///
    /// # Example
    ///
    /// ```ignore
    /// let system = OrderSystem::new();
    /// // ... use the system ...
    /// system.shutdown().await?;
    /// ```
    pub async fn shutdown(self) -> Result<(), String> {
        info!("Shutting down system...");
        
        // =====================================================================
        // Step 1: Close all channels by dropping clients
        // =====================================================================
        
        // When we drop the clients, their internal channel senders are dropped.
        // This causes the actors' receivers to return None, signaling shutdown.
        drop(self.order_client);
        drop(self.user_client);
        drop(self.product_client);

        // =====================================================================
        // Step 2: Wait for all actor tasks to complete
        // =====================================================================
        
        for handle in self.handles {
            // Wait for the actor task to finish
            // If the task panicked, this will return an Err
            if let Err(e) = handle.await {
                error!("Actor task failed: {:?}", e);
                return Err(format!("Actor task failed: {:?}", e));
            }
        }
        
        info!("System shutdown complete.");
        Ok(())
    }
}
