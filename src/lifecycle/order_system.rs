use tracing::{info, error};
use crate::clients::{OrderClient, UserClient, ProductClient};

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
        
        // Create the User actor and its client using the factory function
        let (user_actor, user_client) = crate::user_actor::new();
        
        // Spawn the actor in a background task
        let user_handle = tokio::spawn(user_actor.run());

        // =====================================================================
        // 2. Setup Product Actor
        // =====================================================================
        
        // Create the Product actor and its client using the factory function
        let (product_actor, product_client) = crate::product_actor::new();
        
        // Spawn the actor in a background task
        let product_handle = tokio::spawn(product_actor.run());

        // =====================================================================
        // 3. Setup Order Actor (with dependencies)
        // =====================================================================
        
        // Create the Order actor and its client using the factory function
        // Dependencies are injected here
        let (order_actor, order_client) = crate::order_actor::new(
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
