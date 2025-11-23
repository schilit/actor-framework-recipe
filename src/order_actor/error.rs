//! Error types for the Order actor.

use thiserror::Error;
use crate::user_actor::UserError;
use crate::product_actor::ProductError;

/// Errors that can occur during order operations.
///
/// This error type is used both by the client (OrderClient) and internally
/// by the Order entity's lifecycle hooks.
#[derive(Debug, Error)]
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

    /// Error from User service (entity-level)
    #[error("User service error: {0}")]
    UserService(#[from] UserError),

    /// Error from Product service (entity-level)
    #[error("Product service error: {0}")]
    ProductService(#[from] ProductError),

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
