//! # Actor Recipe for Rust
//!
//! A recipe for building actor systems with minimal boilerplate and good observability.
//!
//! ## Terminology Note
//!
//! This recipe uses terminology that differs from traditional actor system literature:
//!
//! - **Service** (e.g., `UserService`) = **Actor** in traditional terminology
//! - **Client** (e.g., `UserClient`) = **Actor Reference/Handle** in traditional terminology
//!
//! We use "Service" because these components provide business services, and "Client"
//! because they provide a client interface for calling those services. This naming
//! aligns better with common Rust patterns.
//!
//! ## Ingredients
//!
//! - **Foundation** - Start with the basic building blocks
//!     - **Domain types** - Clean business entities separate from actor infrastructure → [`User`], [`Product`], [`Order`]
//!     - **Message enums** - Typed actor communication with oneshot response channels → [`UserRequest`], [`ProductRequest`]
//! - **Core Patterns** - The essential implementation tools  
//!     - **Client method macro** - Generate boilerplate-free client methods with automatic tracing (see UserClient method generation below)
//!     - **Instrumented handlers** - Actor methods with `#[instrument]` for observability → [`UserService::handle_get_user`]
//!     - **Generated clients** - Thin wrappers around message channels with macro-generated methods → [`UserClient`]
//! - **Architecture** - How to structure your actor system
//!     - **Sub-actors** - Domain-specific actors for separation of concerns → [`UserService`], [`ProductService`]
//!     - **Root actors** - Orchestration actors that coordinate multiple sub-actors → [`OrderService`]
//!     - **Handler patterns** - Sync, async, background, and delegation strategies → [`UserService::handle_send_welcome_email_background`]
//! - **System Concerns** - Putting it all together
//!     - **System coordinator** - Lifecycle management and dependency injection → [`OrderSystem`]
//!     - **Tracing setup** - Centralized observability configuration → [`setup_tracing`]
//!
//! ## Instructions
//!
//! 1. **Define your domain types** - Create clean structs for your business entities
//! 2. **Create message enums** - Define typed messages for each actor with oneshot response channels
//! 3. **Implement actors** - Write handler methods with `#[instrument]` for observability
//! 4. **Generate clients** - Use `client_method!` macro to eliminate boilerplate
//! 5. **Coordinate with root actors** - Create orchestration actors for complex workflows
//! 6. **Set up the system** - Use `OrderSystem` to manage startup and shutdown
//! 7. **Configure tracing** - Call `setup_tracing()` to enable structured logging
//!
//! ## What You Get
//!
//! - **80% less boilerplate** - Macro-generated client methods with automatic error handling  
//! - **Professional observability** - Request correlation across actors with timing
//! - **Clean architecture** - Domain-specific actors with clear separation of concerns  
//! - **Production-ready** - Error handling, graceful shutdown, and scaling patterns  
//!
//! ## Example Usage
//!
//! ```rust
//! // Create the entire actor system
//! let system = OrderSystem::new();
//!
//! // Create a user (flows to UserService)
//! let user_id = system.user_client.create_user(user).await?;
//!
//! // Create an order (orchestrates UserService + ProductService)
//! let order_id = system.order_client.create_order(order).await?;
//!
//! // Shutdown gracefully
//! system.shutdown().await?;
//! ```

//! ## Expected Tracing Output
//!
//! ```text
//! INFO order_system: Starting order system
//! INFO user_service: UserService starting
//! INFO product_service: ProductService starting  
//! INFO order_service: OrderService starting
//! INFO user_creation: Creating test user
//! DEBUG create_user{}: Sending request
//! DEBUG handle_create_user{user_name="Alice" user_email="alice@example.com"}: Processing create_user request
//! INFO handle_create_user{user_name="Alice" user_email="alice@example.com"}: User created successfully user_id="user_1"
//! INFO order_processing: Processing order through actor system
//! DEBUG create_order{}: Sending request
//! INFO handle_create_order{order_id="order_1" user_id="user_1" product_id="p1" quantity="5"}: Processing create_order request
//! DEBUG get_user{}: Sending request
//! DEBUG handle_get_user{user_id="user_1"}: Processing get_user request
//! INFO handle_get_user{user_id="user_1"}: User found user_name="Alice"
//! INFO handle_create_order{order_id="order_1"}: User validation successful user_name="Alice"
//! ```

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, instrument, warn, Instrument};

// =============================================================================
// CLIENT METHOD MACRO
// =============================================================================

/// Generate client methods with oneshot channel boilerplate and automatic tracing.
/// Client methods convert domain errors to String for API simplicity.
macro_rules! client_method {
    ($client:ty => fn $method:ident($($param:ident: $param_type:ty),*) -> $return_type:ty as $request:ident::$variant:ident) => {
        impl $client {
            #[instrument(skip(self))]
            pub async fn $method(&self, $($param: $param_type),*) -> std::result::Result<$return_type, String> {
                debug!("Sending request");
                let (respond_to, response) = oneshot::channel();
                self.sender.send($request::$variant {
                    $($param,)*
                    respond_to,
                }).await.map_err(|e| e.to_string())?;

                response.await.map_err(|e| e.to_string()).and_then(|result| result.map_err(|e| e.to_string()))
            }
        }
    };
}

// =============================================================================
// DOMAIN TYPES
// =============================================================================

/// Business domain entities. Pure data structures with no actor-specific concerns.

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: String::new(), // ID will be set by the service
            name: name.into(),
            email: email.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

impl Product {
    pub fn new(id: impl Into<String>, name: impl Into<String>, price: f64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            price,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub quantity: u32,
    pub total: f64,
}

impl Order {
    pub fn new(
        id: impl Into<String>,
        user_id: impl Into<String>,
        product_id: impl Into<String>,
        quantity: u32,
        total: f64,
    ) -> Self {
        Self {
            id: id.into(),
            user_id: user_id.into(),
            product_id: product_id.into(),
            quantity,
            total,
        }
    }
}

// =============================================================================
// MESSAGE ENUMS
// =============================================================================

/// Typed message enums for actor communication. Each variant includes parameters
/// and a oneshot channel for responses.

#[derive(Debug)]
pub enum UserRequest {
    GetUser {
        id: String,
        respond_to: ServiceResponse<Option<User>, UserError>,
    },
    CreateUser {
        user: User,
        respond_to: ServiceResponse<String, UserError>,
    },
    UpdateUser {
        id: String,
        user: User,
        respond_to: ServiceResponse<(), UserError>,
    },
    ListUsers {
        respond_to: ServiceResponse<Vec<User>, UserError>,
    },
    Shutdown,
    #[cfg(test)]
    GetUserCount {
        respond_to: ServiceResponse<usize, UserError>,
    },
}

#[derive(Debug)]
pub enum ProductRequest {
    GetProduct {
        id: String,
        respond_to: ServiceResponse<Option<Product>, ProductError>,
    },
    CheckStock {
        id: String,
        respond_to: ServiceResponse<u32, ProductError>,
    },
    ReserveStock {
        id: String,
        quantity: u32,
        respond_to: ServiceResponse<(), ProductError>,
    },
    Shutdown,
}

#[derive(Debug)]
pub enum OrderRequest {
    CreateOrder {
        order: Order,
        respond_to: ServiceResponse<String, OrderError>,
    },
    GetOrder {
        id: String,
        respond_to: ServiceResponse<Option<Order>, OrderError>,
    },
    Shutdown,
}

// =============================================================================
// USER SERVICE (SUB-ACTOR)
// =============================================================================

/// User-specific error types
#[derive(Debug, Clone)]
pub enum UserError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(String),
    DatabaseError(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::NotFound(id) => write!(f, "User not found: {}", id),
            UserError::AlreadyExists(id) => write!(f, "User already exists: {}", id),
            UserError::ValidationError(msg) => write!(f, "User validation error: {}", msg),
            UserError::DatabaseError(msg) => write!(f, "User database error: {}", msg),
        }
    }
}

impl std::error::Error for UserError {}

/// Generic type aliases for service communication
pub type ServiceResult<T, E> = std::result::Result<T, E>;
pub type ServiceResponse<T, E> = oneshot::Sender<ServiceResult<T, E>>;

/// Macro for clean error response handling
macro_rules! send_error {
    ($respond_to:expr, $error:expr) => {{
        let _ = $respond_to.send(Err($error));
        return;
    }};
}

/// User management actor with instrumented handlers. Demonstrates domain-specific
/// sub-actor pattern with automatic tracing.
/// <!-- anchor: user-service -->
pub struct UserService {
    receiver: mpsc::Receiver<UserRequest>,
    users: HashMap<String, User>,
    next_id: u64,
}

impl UserService {
    pub fn new(buffer_size: usize) -> (Self, UserClient) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let service = Self {
            receiver,
            users: HashMap::new(),
            next_id: 1,
        };
        let client = UserClient::new(sender);
        (service, client)
    }

    /// Main actor loop with tracing
    ///
    /// **Pattern:** The run loop is instrumented at the top level and delegates
    /// to specific handlers for each message type. This provides clean separation
    /// and makes it easy to add new message types.
    #[instrument(name = "user_service", skip(self))]
    pub async fn run(mut self) {
        info!("UserService starting");

        while let Some(msg) = self.receiver.recv().await {
            match msg {
                UserRequest::GetUser { id, respond_to } => {
                    self.handle_get_user(id, respond_to);
                }
                UserRequest::CreateUser { user, respond_to } => {
                    self.handle_create_user(user, respond_to).await;
                }
                UserRequest::UpdateUser {
                    id,
                    user,
                    respond_to,
                } => {
                    self.handle_update_user(id, user, respond_to).await;
                }
                UserRequest::ListUsers { respond_to } => {
                    self.handle_list_users(respond_to);
                }
                UserRequest::Shutdown => {
                    info!("UserService shutting down");
                    break;
                }
                #[cfg(test)]
                UserRequest::GetUserCount { respond_to } => {
                    let _ = respond_to.send(Ok(self.users.len()));
                }
            }
        }

        info!("UserService stopped");
    }

    /// **Sync Handler Example** - Fast, in-memory operation
    ///
    /// Use sync handlers for:
    /// - In-memory data lookups
    /// - Simple computations
    /// - Operations that don't need `await`
    ///
    /// **Tracing:** Fields extract key business data, `skip` excludes large/sensitive data
    #[instrument(fields(user_id = %id), skip(self, respond_to))]
    fn handle_get_user(&self, id: String, respond_to: ServiceResponse<Option<User>, UserError>) {
        debug!("Processing get_user request");

        let user = self.users.get(&id).cloned();

        match &user {
            Some(user) => info!(user_name = %user.name, "User found"),
            None => debug!("User not found"),
        }

        let _ = respond_to.send(Ok(user));
    }

    /// **Async Handler Example** - I/O operation with validation
    ///
    /// Use async handlers for:
    /// - Database operations
    /// - External API calls
    /// - Complex validation logic
    /// - Any operation that needs `await`
    ///
    /// **Security:** Skip the full `user` object but log specific safe fields
    #[instrument(fields(user_name = %user.name, user_email = %user.email), skip(self, user, respond_to))]
    async fn handle_create_user(
        &mut self,
        user: User,
        respond_to: ServiceResponse<String, UserError>,
    ) {
        debug!("Processing create_user request");

        let result = if user.email.is_empty() {
            error!("Validation failed: empty email");
            Err(UserError::ValidationError("Email required".to_string()))
        } else {
            let id = format!("user_{}", self.next_id);
            self.next_id += 1;
            self.users.insert(id.clone(), user);

            info!(user_id = %id, "User created successfully");
            Ok(id)
        };

        let _ = respond_to.send(result);
    }

    /// **Update Handler** - Modification operation with validation
    #[instrument(fields(user_id = %id, user_name = %user.name), skip(self, user, respond_to))]
    async fn handle_update_user(
        &mut self,
        id: String,
        user: User,
        respond_to: ServiceResponse<(), UserError>,
    ) {
        debug!("Processing update_user request");

        let result = if self.users.contains_key(&id) {
            self.users.insert(id.clone(), user);
            info!("User updated successfully");
            Ok(())
        } else {
            error!("User not found for update");
            Err(UserError::NotFound(id))
        };

        let _ = respond_to.send(result);
    }

    /// **Collection Handler** - Returns multiple items
    #[instrument(skip(self, respond_to))]
    fn handle_list_users(&self, respond_to: ServiceResponse<Vec<User>, UserError>) {
        debug!("Processing list_users request");

        let users: Vec<User> = self.users.values().cloned().collect();
        info!(user_count = users.len(), "Listed users");

        let _ = respond_to.send(Ok(users));
    }
}

// =============================================================================
// USER CLIENT
// =============================================================================

/// Client for UserService with macro-generated methods. Thin wrapper around
/// message channels with automatic error handling.

#[derive(Clone)]
pub struct UserClient {
    sender: mpsc::Sender<UserRequest>,
}

impl UserClient {
    pub fn new(sender: mpsc::Sender<UserRequest>) -> Self {
        Self { sender }
    }

    /// Manual methods for special cases (no response needed)
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), String> {
        debug!("Sending shutdown request");
        self.sender
            .send(UserRequest::Shutdown)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// Generate client methods with automatic tracing
client_method!(UserClient => fn get_user(id: String) -> Option<User> as UserRequest::GetUser);
client_method!(UserClient => fn create_user(user: User) -> String as UserRequest::CreateUser);
client_method!(UserClient => fn update_user(id: String, user: User) -> () as UserRequest::UpdateUser);
client_method!(UserClient => fn list_users() -> Vec<User> as UserRequest::ListUsers);

// Test-only method for internal state inspection
// **Pattern:** Use #[cfg(test)] messages to extract actor internal state for testing
#[cfg(test)]
client_method!(UserClient => fn get_user_count() -> usize as UserRequest::GetUserCount);

// =============================================================================
// INGREDIENT 6: PRODUCT SERVICE (SECOND SUB-ACTOR)
// =============================================================================

/// Product-specific error types
#[derive(Debug, Clone)]
pub enum ProductError {
    NotFound(String),
    InsufficientStock { requested: u32, available: u32 },
    InvalidQuantity(u32),
    DatabaseError(String),
}

impl std::fmt::Display for ProductError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductError::NotFound(id) => write!(f, "Product not found: {}", id),
            ProductError::InsufficientStock {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Insufficient stock: requested {}, available {}",
                    requested, available
                )
            }
            ProductError::InvalidQuantity(qty) => write!(f, "Invalid quantity: {}", qty),
            ProductError::DatabaseError(msg) => write!(f, "Product database error: {}", msg),
        }
    }
}

impl std::error::Error for ProductError {}

/// ## Ingredient 6: Additional Sub-Actors
///
/// **Pattern:** Each domain gets its own actor following the same structure.
/// This demonstrates how to scale the pattern to multiple domains.
///
/// **Benefits:**
/// - **Domain isolation** - Each actor is responsible for one business domain
/// - **Independent scaling** - Actors can be scaled independently
/// - **Clear boundaries** - Easy to understand what each actor does
/// - **Parallel development** - Different teams can work on different actors
/// <!-- anchor: product-service -->
pub struct ProductService {
    receiver: mpsc::Receiver<ProductRequest>,
    products: HashMap<String, Product>,
    stock: HashMap<String, u32>,
}

impl ProductService {
    pub fn new(buffer_size: usize) -> (Self, ProductClient) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let service = Self {
            receiver,
            products: HashMap::new(),
            stock: HashMap::new(),
        };
        let client = ProductClient::new(sender);
        (service, client)
    }

    #[instrument(name = "product_service", skip(self))]
    pub async fn run(mut self) {
        info!("ProductService starting");

        while let Some(msg) = self.receiver.recv().await {
            match msg {
                ProductRequest::GetProduct { id, respond_to } => {
                    self.handle_get_product(id, respond_to);
                }
                ProductRequest::CheckStock { id, respond_to } => {
                    self.handle_check_stock(id, respond_to);
                }
                ProductRequest::ReserveStock {
                    id,
                    quantity,
                    respond_to,
                } => {
                    self.handle_reserve_stock(id, quantity, respond_to).await;
                }
                ProductRequest::Shutdown => {
                    info!("ProductService shutting down");
                    break;
                }
            }
        }

        info!("ProductService stopped");
    }

    #[instrument(fields(product_id = %id), skip(self, respond_to))]
    fn handle_get_product(
        &self,
        id: String,
        respond_to: ServiceResponse<Option<Product>, ProductError>,
    ) {
        debug!("Processing get_product request");

        let product = self.products.get(&id).cloned();

        match &product {
            Some(product) => {
                info!(product_name = %product.name, price = %product.price, "Product found")
            }
            None => debug!("Product not found"),
        }

        let _ = respond_to.send(Ok(product));
    }

    #[instrument(fields(product_id = %id), skip(self, respond_to))]
    fn handle_check_stock(&self, id: String, respond_to: ServiceResponse<u32, ProductError>) {
        debug!("Processing check_stock request");

        let stock = self.stock.get(&id).copied().unwrap_or(0);
        info!(stock_level = stock, "Stock checked");

        let _ = respond_to.send(Ok(stock));
    }

    #[instrument(fields(product_id = %id, quantity = %quantity), skip(self, respond_to))]
    async fn handle_reserve_stock(
        &mut self,
        id: String,
        quantity: u32,
        respond_to: ServiceResponse<(), ProductError>,
    ) {
        debug!("Processing reserve_stock request");

        let result = match self.stock.get_mut(&id) {
            Some(current_stock) => {
                if *current_stock >= quantity {
                    *current_stock -= quantity;
                    info!(
                        remaining_stock = *current_stock,
                        "Stock reserved successfully"
                    );
                    Ok(())
                } else {
                    error!(
                        available = *current_stock,
                        requested = quantity,
                        "Insufficient stock"
                    );
                    Err(ProductError::InsufficientStock {
                        requested: quantity,
                        available: *current_stock,
                    })
                }
            }
            None => {
                error!("Product not found");
                Err(ProductError::NotFound(id))
            }
        };

        let _ = respond_to.send(result);
    }
}

#[derive(Clone)]
pub struct ProductClient {
    sender: mpsc::Sender<ProductRequest>,
}

impl ProductClient {
    pub fn new(sender: mpsc::Sender<ProductRequest>) -> Self {
        Self { sender }
    }

    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), String> {
        debug!("Sending shutdown request");
        self.sender
            .send(ProductRequest::Shutdown)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// Generate product client methods
client_method!(ProductClient => fn get_product(id: String) -> Option<Product> as ProductRequest::GetProduct);
client_method!(ProductClient => fn check_stock(id: String) -> u32 as ProductRequest::CheckStock);
client_method!(ProductClient => fn reserve_stock(id: String, quantity: u32) -> () as ProductRequest::ReserveStock);

// =============================================================================
// INGREDIENT 7: ROOT ACTOR (ORCHESTRATOR)
// =============================================================================

/// Order-specific error types
#[derive(Debug, Clone)]
pub enum OrderError {
    NotFound(String),
    InvalidProduct(String),
    InvalidUser(String),
    InsufficientStock(String),
    ValidationError(String),
    DatabaseError(String),
}

impl std::fmt::Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::NotFound(id) => write!(f, "Order not found: {}", id),
            OrderError::InvalidProduct(id) => write!(f, "Invalid product: {}", id),
            OrderError::InvalidUser(id) => write!(f, "Invalid user: {}", id),
            OrderError::InsufficientStock(msg) => write!(f, "Insufficient stock: {}", msg),
            OrderError::ValidationError(msg) => write!(f, "Order validation error: {}", msg),
            OrderError::DatabaseError(msg) => write!(f, "Order database error: {}", msg),
        }
    }
}

impl std::error::Error for OrderError {}

/// ## Ingredient 7: Root Actor for Orchestration
///
/// **Pattern:** Root actors coordinate multiple sub-actors to implement complex
/// business workflows. They don't store domain data themselves - instead they
/// delegate to specialized sub-actors.
///
/// **Why This Works:**
/// - **Separation of concerns** - Root actor focuses on orchestration
/// - **Composition** - Complex operations built from simpler sub-operations
/// - **Error handling** - Centralized error handling and rollback logic
/// - **Tracing** - Full request flow visible across multiple actors
///
/// **Delegation Pattern:**
/// 1. Validate inputs using sub-actors
/// 2. Perform business logic steps in sequence
/// 3. Handle errors and rollbacks appropriately
/// 4. Return results to caller
pub struct OrderService {
    receiver: mpsc::Receiver<OrderRequest>,
    user_client: UserClient,
    product_client: ProductClient,
    orders: HashMap<String, Order>,
}

impl OrderService {
    pub fn new(
        buffer_size: usize,
        user_client: UserClient,
        product_client: ProductClient,
    ) -> (Self, OrderClient) {
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

    /// **Orchestration Handler Example** - Coordinates multiple sub-actors
    ///
    /// This demonstrates the core orchestration pattern:
    /// 1. **Validate user** via UserService
    /// 2. **Validate product** via ProductService  
    /// 3. **Reserve stock** via ProductService
    /// 4. **Create order** locally
    ///
    /// **Error Handling:** Each step can fail, and errors are propagated appropriately.
    /// In a production system, you might add compensation logic (rollbacks).
    ///
    /// **Tracing:** The full workflow is traced across multiple actors, making
    /// debugging complex flows much easier.
    #[instrument(
        fields(
            order_id = %order.id,
            user_id = %order.user_id,
            product_id = %order.product_id,
            quantity = %order.quantity,
            total = %order.total
        ),
        skip(self, order, respond_to)
    )]
    async fn handle_create_order(
        &mut self,
        order: Order,
        respond_to: ServiceResponse<String, OrderError>,
    ) {
        info!("Processing create_order request");

        // Step 1: Validate user via UserService
        let user_result = self.user_client.get_user(order.user_id.clone()).await;

        let _user = match user_result {
            Ok(Some(user)) => {
                info!(user_name = %user.name, "User validation successful");
                user
            }
            Ok(None) => {
                error!("User not found");
                send_error!(respond_to, OrderError::InvalidUser(order.user_id.clone()));
            }
            Err(e) => {
                error!(error = %e, "User validation failed");
                send_error!(
                    respond_to,
                    OrderError::InvalidUser(format!("User validation failed: {}", e))
                );
            }
        };

        // Step 2: Validate product via ProductService
        let product_result = self
            .product_client
            .get_product(order.product_id.clone())
            .await;

        let _product = match product_result {
            Ok(Some(product)) => {
                info!(product_name = %product.name, price = %product.price, "Product validation successful");
                product
            }
            Ok(None) => {
                error!("Product not found");
                send_error!(
                    respond_to,
                    OrderError::InvalidProduct(order.product_id.clone())
                );
            }
            Err(e) => {
                error!(error = %e, "Product validation failed");
                send_error!(
                    respond_to,
                    OrderError::InvalidProduct(format!("Product validation failed: {}", e))
                );
            }
        };

        // Step 3: Reserve stock via ProductService
        let reserve_result = self
            .product_client
            .reserve_stock(order.product_id.clone(), order.quantity)
            .await;

        if let Err(e) = reserve_result {
            error!(error = %e, "Stock reservation failed");
            send_error!(
                respond_to,
                OrderError::InsufficientStock(format!("Stock reservation failed: {}", e))
            );
        }

        info!("Stock reserved successfully");

        // Step 4: Create order (local operation)
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

#[derive(Clone)]
pub struct OrderClient {
    sender: mpsc::Sender<OrderRequest>,
}

impl OrderClient {
    pub fn new(sender: mpsc::Sender<OrderRequest>) -> Self {
        Self { sender }
    }

    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), String> {
        debug!("Sending shutdown request");
        self.sender
            .send(OrderRequest::Shutdown)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// Generate order client methods
client_method!(OrderClient => fn create_order(order: Order) -> String as OrderRequest::CreateOrder);
client_method!(OrderClient => fn get_order(id: String) -> Option<Order> as OrderRequest::GetOrder);

// =============================================================================
// INGREDIENT 8: SYSTEM COORDINATOR
// =============================================================================

/// ## Ingredient 8: System Coordinator
///
/// **Pattern:** The coordinator manages the lifecycle of the entire actor system.
/// It handles startup, dependency injection, and graceful shutdown.
///
/// **Responsibilities:**
/// - **Start sub-actors first** - Ensure dependencies are available
/// - **Inject dependencies** - Pass sub-actor clients to root actors
/// - **Manage handles** - Track all spawned tasks for proper cleanup
/// - **Graceful shutdown** - Shutdown in dependency order and wait for completion
///
/// **Benefits:**
/// - **Single point of control** - Easy to manage the entire system
/// - **Proper initialization order** - Dependencies started before dependents
/// - **Clean shutdown** - No zombie processes or resource leaks
/// - **Error handling** - Centralized error handling for system-wide issues
pub struct OrderSystem {
    pub order_client: OrderClient,
    pub user_client: UserClient,
    pub product_client: ProductClient,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl Default for OrderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderSystem {
    /// Create and start the entire actor system
    ///
    /// **Startup Order:**
    /// 1. Start sub-actors (UserService, ProductService)
    /// 2. Start root actors (OrderService) with sub-actor clients
    /// 3. Return coordinator with all clients for external use
    #[instrument(name = "order_system")]
    pub fn new() -> Self {
        let mut handles = Vec::new();

        info!("Starting order system");

        // Start sub-actors first (no dependencies)
        let (user_service, user_client) = UserService::new(100);
        handles.push(tokio::spawn(user_service.run()));

        let (product_service, product_client) = ProductService::new(100);
        handles.push(tokio::spawn(product_service.run()));

        // Start root actor with sub-actor clients (dependency injection)
        let (order_service, order_client) =
            OrderService::new(100, user_client.clone(), product_client.clone());
        handles.push(tokio::spawn(order_service.run()));

        info!("Order system started successfully");

        Self {
            order_client,
            user_client,
            product_client,
            handles,
        }
    }

    /// Gracefully shutdown the entire actor system
    ///
    /// **Shutdown Order:**
    /// 1. Shutdown root actors first (they depend on sub-actors)
    /// 2. Shutdown sub-actors  
    /// 3. Wait for all tasks to complete
    ///
    /// **Error Handling:** Log errors but continue shutdown to prevent hangs
    #[instrument(skip(self))]
    pub async fn shutdown(self) -> Result<(), String> {
        info!("Shutting down order system");

        // Shutdown in dependency order (root actors first)
        let _ = self.order_client.shutdown().await;
        let _ = self.user_client.shutdown().await;
        let _ = self.product_client.shutdown().await;

        // Wait for all services to finish
        for handle in self.handles {
            if let Err(e) = handle.await {
                error!(error = ?e, "Service shutdown error");
            }
        }

        info!("Order system shutdown complete");
        Ok(())
    }
}

// =============================================================================
// INGREDIENT 9: TRACING SETUP
// =============================================================================

/// ## Ingredient 9: Production-Ready Tracing Setup
///
/// **Pattern:** Configure tracing once at application startup for the entire process.
/// All actors and spans automatically use this configuration.
///
/// **Key Features:**
/// - **Environment-based filtering** - Use `RUST_LOG` env var to control verbosity
/// - **Built-in timing** - See how long each operation takes
/// - **Structured output** - Easy to parse and search logs
/// - **Compact format** - Readable but not verbose
///
/// **Usage:**
/// ```bash
/// RUST_LOG=debug cargo run    # Show debug logs
/// RUST_LOG=info cargo run     # Show info logs only  
/// RUST_LOG=warn cargo run     # Show warnings and errors only
///
/// # For per-module logging, organize services into separate modules:
/// # RUST_LOG=my_app::user_service=debug,my_app::order_service=info cargo run
/// ```
fn setup_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .compact()
        .init();
}

// =============================================================================
// INGREDIENT 10: HANDLER PATTERNS
// =============================================================================

/// ## Ingredient 10: Advanced Handler Patterns
///
/// Beyond basic request-response, actors often need to handle different types of operations:
///
/// ### Sync vs Async Handlers
///
/// **Sync Handlers** (fast, in-memory):
/// ```rust
/// fn handle_get_user_sync(&self, id: String, respond_to: oneshot::Sender<...>) {
///     let result = self.users.get(&id).cloned(); // No await!
///     let _ = respond_to.send(Ok(result));
/// }
/// ```
///
/// **Async Handlers** (I/O, validation):
/// ```rust
/// async fn handle_create_user_async(&mut self, user: User, respond_to: oneshot::Sender<...>) {
///     // Async email validation
///     validate_email_externally(&user.email).await?;
///     let id = self.create_user_internal(user);
///     let _ = respond_to.send(Ok(id));
/// }
/// ```
///
/// ### Background Operations
///
/// **Return Immediately, Work Continues:**
/// ```rust
/// fn handle_send_email_background(&self, user_id: String, respond_to: oneshot::Sender<...>) {
///     // Return success immediately
///     let _ = respond_to.send(Ok(()));
///     
///     // Spawn background work
///     tokio::spawn(async move {
///         send_welcome_email(user_id).await;
///     });
/// }
/// ```
///
/// **Return Job ID, Work Continues:**
/// ```rust
/// fn handle_generate_report(&self, user_id: String, respond_to: oneshot::Sender<...>) {
///     let job_id = generate_job_id();
///     let _ = respond_to.send(Ok(job_id.clone()));
///     
///     tokio::spawn(async move {
///         let report = generate_report(user_id).await;
///         save_report(job_id, report).await;
///     });
/// }
/// ```
///
/// ### When to Use Each Pattern
///
/// - **Sync**: Fast lookups, in-memory operations, simple computations
/// - **Async**: Database calls, external APIs, file I/O, complex validation
/// - **Background**: Email sending, report generation, cleanup tasks, analytics
///
/// Example of a background operation that returns immediately and continues work
impl UserService {
    /// **Background Handler Example** - Task owns the response channel
    ///
    /// This pattern shows how the spawned task can take ownership of respond_to
    /// and send the response after the work completes.
    #[instrument(fields(user_id = %user_id), skip(self, respond_to))]
    pub async fn handle_send_welcome_email_background(
        &self,
        user_id: String,
        respond_to: ServiceResponse<(), UserError>,
    ) {
        debug!("Processing send_welcome_email request");

        // Spawn background task - it takes ownership of respond_to
        tokio::spawn(async move {
            info!(user_id = %user_id, "Starting background email send");

            // Simulate slow email sending
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Simulate email service call
            let success = true; // In real code, this would be an actual email API call

            let result = if success {
                info!(user_id = %user_id, "Welcome email sent successfully");
                Ok(())
            } else {
                error!(user_id = %user_id, "Failed to send welcome email");
                Err(UserError::DatabaseError("Email service failed".to_string()))
            };

            // Task responds when work is actually done
            let _ = respond_to.send(result);
        });
    }

    /// **Alternative Background Pattern** - Return job ID immediately
    ///
    /// Shows another way: return a job ID immediately, do work in background.
    /// Caller can use the job ID to check status later.
    #[instrument(fields(user_id = %user_id), skip(self, respond_to))]
    pub async fn handle_generate_report_background(
        &self,
        user_id: String,
        respond_to: ServiceResponse<String, UserError>,
    ) {
        debug!("Processing generate_report request");

        // Generate a job ID and return it immediately
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let job_id = format!("job_{}_{}", user_id, timestamp);

        info!(job_id = %job_id, "Report generation started");
        let _ = respond_to.send(Ok(job_id.clone()));

        // Spawn background task for the actual report generation
        let user_data = self.users.get(&user_id).cloned();

        tokio::spawn(async move {
            info!(job_id = %job_id, "Starting background report generation");

            // Simulate slow report generation
            tokio::time::sleep(Duration::from_millis(2000)).await;

            match user_data {
                Some(user) => {
                    info!(
                        job_id = %job_id,
                        user_name = %user.name,
                        "Report generated successfully"
                    );
                    // In real code, you would save the report somewhere
                    // and maybe notify the user that it's ready
                }
                None => {
                    error!(job_id = %job_id, "Cannot generate report: user not found");
                }
            }
        });
    }
}

/// Example of concurrent monitoring for performance and blocking detection
///
/// **Pattern:** Use a background task to periodically check system health
/// This can be used for automated alerting or circuit breaker patterns.
///
/// **Blocking Detection:** Uses timeouts to detect when requests hang on the server:
/// - Normal response: < 100ms (debug log)
/// - Slow response: 100-500ms (warning - potential overload)
/// - Timeout: > 500ms (error - likely blocked/hanging)
pub async fn performance_monitor(user_client: UserClient, interval: Duration) {
    let mut interval_timer = tokio::time::interval(interval);

    loop {
        interval_timer.tick().await;
        check_health(&user_client).await;
    }
}

async fn check_health(user_client: &UserClient) {
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(500);

    match tokio::time::timeout(timeout, user_client.get_user("health_check".to_string())).await {
        Ok(Ok(_)) => log_response_time(start.elapsed()),
        Ok(Err(e)) => {
            error!(error = %e, duration_ms = start.elapsed().as_millis(), "Health check failed")
        }
        Err(_) => error!(
            timeout_ms = timeout.as_millis(),
            "Health check timed out - server may be blocked/overloaded"
        ),
    }
}

fn log_response_time(duration: Duration) {
    let duration_ms = duration.as_millis();
    if duration > Duration::from_millis(100) {
        warn!(
            duration_ms,
            "Health check slow but completed - potential server overload"
        );
    } else {
        debug!(duration_ms, "Health check completed normally");
    }
}

// =============================================================================
// USAGE EXAMPLE AND DEMO
// =============================================================================

/// ## Complete Usage Example
///
/// This example demonstrates all the patterns working together:
/// - System startup and coordination
/// - Cross-actor request flows
/// - Error handling and tracing
/// - Graceful shutdown

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

    // Demonstrate additional operations
    let users = system.user_client.list_users().await?;
    info!(user_count = users.len(), "Retrieved user list");

    // Shutdown system gracefully
    system.shutdown().await?;

    info!("Application completed successfully");
    Ok(())
}

// =============================================================================
// RECIPE SUMMARY
// =============================================================================

/// This recipe provides a solid foundation for building production actor systems in Rust!
///
/// ## To Run This Example
///
/// ```bash
/// # Basic run
/// cargo run
///
/// # With debug logging
/// RUST_LOG=debug cargo run
///
/// # With warning level only  
/// RUST_LOG=warn cargo run
///
/// # Generate documentation
/// cargo doc --open
/// ```
#[cfg(test)]
mod tests {
    use super::*;

    /// Demonstrates test-only messages for extracting internal actor state
    #[tokio::test]
    async fn test_user_service_internal_state() -> Result<(), Box<dyn std::error::Error>> {
        // Start just the UserService for testing
        let (user_service, user_client) = UserService::new(10);
        let _handle = tokio::spawn(user_service.run());

        // Initially should have 0 users
        let count = user_client.get_user_count().await?;
        assert_eq!(count, 0);

        // Create a user
        let user = User::new("Test User", "test@example.com");
        let _user_id = user_client.create_user(user).await?;

        // Now should have 1 user
        let count = user_client.get_user_count().await?;
        assert_eq!(count, 1);

        // Shutdown
        user_client.shutdown().await?;
        Ok(())
    }
}
