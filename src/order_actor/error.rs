//! Error types for the Order actor.

use thiserror::Error;

/// Errors that can occur during order operations.
#[derive(Debug, Clone, Error, PartialEq)]
#[allow(dead_code)]
pub enum OrderError {
    /// The requested order was not found.
    #[error("Order not found: {0}")]
    NotFound(String),

    /// The product specified in the order is invalid or does not exist.
    #[error("Invalid product: {0}")]
    InvalidProduct(String),

    /// The user specified in the order is invalid or does not exist.
    #[error("Invalid user: {0}")]
    InvalidUser(String),

    /// There is insufficient stock to fulfill the order.
    #[error("Insufficient stock: {0}")]
    InsufficientStock(String),

    /// The order data provided is invalid.
    #[error("Order validation error: {0}")]
    ValidationError(String),

    /// An underlying database error occurred.
    #[error("Order database error: {0}")]
    DatabaseError(String),

    /// An error occurred while communicating with the actor system.
    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}

impl From<String> for OrderError {
    fn from(msg: String) -> Self {
        OrderError::ActorCommunicationError(msg)
    }
}
