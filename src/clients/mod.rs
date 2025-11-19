use tokio::sync::{mpsc, oneshot};
use tracing::{debug, instrument};
use crate::messages::{UserRequest, ProductRequest, OrderRequest, ServiceResponse};
use crate::domain::{User, Product, Order, UserCreate, UserPatch};
use crate::error::{UserError, ProductError, OrderError};
use crate::actor_framework::ResourceClient;

// =============================================================================
// 1. User Client (Refactored to use ResourceClient)
// =============================================================================

#[derive(Clone)]
pub struct UserClient {
    inner: ResourceClient<User>,
}

impl UserClient {
    pub fn new(inner: ResourceClient<User>) -> Self {
        Self { inner }
    }

    #[instrument(skip(self))]
    pub async fn get_user(&self, id: String) -> Result<Option<User>, UserError> {
        debug!("Sending request");
        self.inner.get(id).await.map_err(|e| UserError::ActorCommunicationError(e))
    }

    #[instrument(skip(self))]
    pub async fn create_user(&self, user: User) -> Result<String, UserError> {
        debug!("Sending request");
        // Adapter: Convert legacy User struct to UserCreate payload
        let payload = UserCreate {
            name: user.name,
            email: user.email,
        };
        self.inner.create(payload).await.map_err(|e| UserError::ActorCommunicationError(e))
    }
    
    // New method utilizing the generic update
    #[instrument(skip(self))]
    pub async fn update_user(&self, id: String, patch: UserPatch) -> Result<User, UserError> {
        debug!("Sending request");
        self.inner.update(id, patch).await.map_err(|e| UserError::ActorCommunicationError(e))
    }
}

// =============================================================================
// 2. Product Client (Refactored to use ResourceClient)
// =============================================================================

#[derive(Clone)]
pub struct ProductClient {
    inner: ResourceClient<Product>,
}

impl ProductClient {
    pub fn new(inner: ResourceClient<Product>) -> Self {
        Self { inner }
    }

    #[instrument(skip(self))]
    pub async fn get_product(&self, id: String) -> Result<Option<Product>, ProductError> {
        debug!("Sending request");
        self.inner.get(id).await.map_err(|e| ProductError::ActorCommunicationError(e))
    }

    #[instrument(skip(self))]
    pub async fn check_stock(&self, id: String) -> Result<u32, ProductError> {
        debug!("Sending request");
        use crate::product::{ProductAction, ProductActionResult};
        match self.inner.perform_action(id, ProductAction::CheckStock).await {
            Ok(ProductActionResult::StockLevel(level)) => Ok(level),
            Ok(_) => Err(ProductError::ActorCommunicationError("Unexpected result".to_string())),
            Err(e) => Err(ProductError::ActorCommunicationError(e)),
        }
    }

    #[instrument(skip(self))]
    pub async fn reserve_stock(&self, id: String, quantity: u32) -> Result<(), ProductError> {
        debug!("Sending request");
        use crate::product::{ProductAction, ProductActionResult};
        match self.inner.perform_action(id, ProductAction::ReserveStock(quantity)).await {
            Ok(ProductActionResult::Reserved) => Ok(()),
            Ok(_) => Err(ProductError::ActorCommunicationError("Unexpected result".to_string())),
            Err(e) => Err(ProductError::ActorCommunicationError(e)),
        }
    }
}

// =============================================================================
// Macro for legacy clients (OrderClient still uses this)
// =============================================================================

macro_rules! client_method {
    ($client:ty => fn $method:ident($($param:ident: $param_type:ty),*) -> $return_type:ty as $request:ident::$variant:ident, Error = $error_type:ty) => {
        impl $client {
            #[instrument(skip(self))]
            pub async fn $method(&self, $($param: $param_type),*) -> Result<$return_type, $error_type> {
                debug!("Sending request");
                let (respond_to, response) = oneshot::channel();
                self.sender.send($request::$variant {
                    $($param,)*
                    respond_to,
                }).await.map_err(|_| <$error_type>::ActorCommunicationError("Actor closed".to_string()))?;

                response.await.map_err(|_| <$error_type>::ActorCommunicationError("Actor dropped".to_string()))?
            }
        }
    };
}

// =============================================================================
// 3. Order Client (Legacy Macro)
// =============================================================================

#[derive(Clone)]
pub struct OrderClient {
    sender: mpsc::Sender<OrderRequest>,
}

impl OrderClient {
    pub fn new(sender: mpsc::Sender<OrderRequest>) -> Self {
        Self { sender }
    }
}

client_method!(OrderClient => fn create_order(order: Order) -> String as OrderRequest::CreateOrder, Error = OrderError);
client_method!(OrderClient => fn get_order(id: String) -> Option<Order> as OrderRequest::GetOrder, Error = OrderError);
