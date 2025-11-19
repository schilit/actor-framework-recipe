use tracing::{info, error};
use crate::clients::{OrderClient, UserClient, ProductClient};
use crate::actor_framework::ResourceActor;
use crate::domain::{User, Product, Order};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The main application system that orchestrates all actors.
///
/// Responsible for starting up actors, wiring them together, and handling shutdown.
pub struct OrderSystem {
    pub order_client: OrderClient,
    pub user_client: UserClient,
    pub product_client: ProductClient,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl OrderSystem {
    pub fn new() -> Self {
        // 1. Setup User Service (Refactored to ResourceActor)
        let user_id_counter = Arc::new(AtomicU64::new(1));
        let next_user_id = move || {
            let id = user_id_counter.fetch_add(1, Ordering::SeqCst);
            format!("user_{}", id)
        };
        
        let (user_actor, user_resource_client) = ResourceActor::<User>::new(32, next_user_id);
        let user_client = UserClient::new(user_resource_client);
        let user_handle = tokio::spawn(user_actor.run());

        // 2. Setup Product Service (Refactored to ResourceActor)
        let product_id_counter = Arc::new(AtomicU64::new(1));
        let next_product_id = move || {
            let id = product_id_counter.fetch_add(1, Ordering::SeqCst);
            format!("product_{}", id)
        };
        
        let (product_actor, product_resource_client) = ResourceActor::<Product>::new(32, next_product_id);
        let product_client = ProductClient::new(product_resource_client);
        let product_handle = tokio::spawn(product_actor.run());

        // 3. Setup Order Service (Refactored to ResourceActor)
        let order_id_counter = Arc::new(AtomicU64::new(1));
        let next_order_id = move || {
            let id = order_id_counter.fetch_add(1, Ordering::SeqCst);
            format!("order_{}", id)
        };

        let (order_actor, order_resource_client) = ResourceActor::<Order>::new(32, next_order_id);
        let order_client = OrderClient::new(order_resource_client, user_client.clone(), product_client.clone());
        let order_handle = tokio::spawn(order_actor.run());

        Self {
            order_client,
            user_client,
            product_client,
            handles: vec![user_handle, product_handle, order_handle],
        }
    }

    pub async fn shutdown(self) -> Result<(), String> {
        info!("Shutting down system...");
        // In a real system, we'd send shutdown signals.
        // Here we just drop the clients (which closes channels) and wait for handles.
        // Note: ResourceActor shuts down when channel is closed.
        
        // Drop clients to close channels
        drop(self.order_client);
        drop(self.user_client);
        drop(self.product_client);

        for handle in self.handles {
            if let Err(e) = handle.await {
                error!("Actor task failed: {:?}", e);
                return Err(format!("Actor task failed: {:?}", e));
            }
        }
        
        info!("System shutdown complete.");
        Ok(())
    }
}
