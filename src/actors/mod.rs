use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument};
use crate::domain::{User, Product, Order};
use crate::messages::{UserRequest, ProductRequest, OrderRequest, ServiceResponse};
use crate::error::{UserError, ProductError, OrderError};
use crate::clients::{UserClient, ProductClient, OrderClient};

// =============================================================================
// USER SERVICE
// =============================================================================

// UserService replaced by ResourceActor<User>

// ProductService replaced by ResourceActor<Product>

// =============================================================================
// ORDER SERVICE
// =============================================================================

pub struct OrderService {
    receiver: mpsc::Receiver<OrderRequest>,
    user_client: UserClient,
    product_client: ProductClient,
    orders: HashMap<String, Order>,
}

impl OrderService {
    pub fn new(buffer_size: usize, user_client: UserClient, product_client: ProductClient) -> (Self, OrderClient) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let service = Self {
            receiver,
            user_client,
            product_client,
            orders: HashMap::new(),
        };
        let client = OrderClient::new(sender);
        (service, client)
    }

    #[instrument(name = "order_service", skip(self))]
    pub async fn run(mut self) {
        info!("OrderService starting");
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                OrderRequest::CreateOrder { order, respond_to } => {
                    self.handle_create_order(order, respond_to).await;
                }
                OrderRequest::GetOrder { id, respond_to } => {
                    self.handle_get_order(id, respond_to);
                }
                OrderRequest::Shutdown => {
                    info!("OrderService shutting down");
                    break;
                }
            }
        }
        info!("OrderService stopped");
    }

    #[instrument(fields(order_id = %order.id), skip(self, order, respond_to))]
    async fn handle_create_order(&mut self, order: Order, respond_to: ServiceResponse<String, OrderError>) {
        info!("Processing create_order request");

        // Step 1: Validate user
        match self.user_client.get_user(order.user_id.clone()).await {
            Ok(Some(user)) => info!(user_name = %user.name, "User validation successful"),
            Ok(None) => {
                error!("User not found");
                let _ = respond_to.send(Err(OrderError::InvalidUser(order.user_id.clone())));
                return;
            }
            Err(e) => {
                error!(error = %e, "User validation failed");
                let _ = respond_to.send(Err(OrderError::InvalidUser(format!("User validation failed: {}", e))));
                return;
            }
        }

        // Step 2: Validate product
        match self.product_client.get_product(order.product_id.clone()).await {
            Ok(Some(product)) => info!(product_name = %product.name, "Product validation successful"),
            Ok(None) => {
                error!("Product not found");
                let _ = respond_to.send(Err(OrderError::InvalidProduct(order.product_id.clone())));
                return;
            }
            Err(e) => {
                error!(error = %e, "Product validation failed");
                let _ = respond_to.send(Err(OrderError::InvalidProduct(format!("Product validation failed: {}", e))));
                return;
            }
        }

        // Step 3: Reserve stock
        if let Err(e) = self.product_client.reserve_stock(order.product_id.clone(), order.quantity).await {
            error!(error = %e, "Stock reservation failed");
            let _ = respond_to.send(Err(OrderError::InsufficientStock(format!("Stock reservation failed: {}", e))));
            return;
        }

        info!("Stock reserved successfully");

        // Step 4: Create order
        self.orders.insert(order.id.clone(), order.clone());
        info!("Order created successfully");
        let _ = respond_to.send(Ok(order.id));
    }

    #[instrument(fields(order_id = %id), skip(self, respond_to))]
    fn handle_get_order(&self, id: String, respond_to: ServiceResponse<Option<Order>, OrderError>) {
        debug!("Processing get_order request");
        let order = self.orders.get(&id).cloned();
        match &order {
            Some(order) => info!(total = %order.total, "Order found"),
            None => debug!("Order not found"),
        }
        let _ = respond_to.send(Ok(order));
    }
}
