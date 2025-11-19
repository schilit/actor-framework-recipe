use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),
    #[error("User already exists: {0}")]
    AlreadyExists(String),
    #[error("User validation error: {0}")]
    ValidationError(String),
    #[error("User database error: {0}")]
    DatabaseError(String),
    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}

#[derive(Debug, Clone, Error)]
pub enum ProductError {
    #[error("Product not found: {0}")]
    NotFound(String),
    #[error("Insufficient stock: requested {requested}, available {available}")]
    InsufficientStock { requested: u32, available: u32 },
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(u32),
    #[error("Product database error: {0}")]
    DatabaseError(String),
    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}

#[derive(Debug, Clone, Error)]
pub enum OrderError {
    #[error("Order not found: {0}")]
    NotFound(String),
    #[error("Invalid product: {0}")]
    InvalidProduct(String),
    #[error("Invalid user: {0}")]
    InvalidUser(String),
    #[error("Insufficient stock: {0}")]
    InsufficientStock(String),
    #[error("Order validation error: {0}")]
    ValidationError(String),
    #[error("Order database error: {0}")]
    DatabaseError(String),
    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}
