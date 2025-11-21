use thiserror::Error;

/// Errors that can occur during order operations.
#[derive(Debug, Clone, Error, PartialEq)]
#[allow(dead_code)]
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
