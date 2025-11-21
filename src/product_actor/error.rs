//! Error types for the Product actor.

use thiserror::Error;

/// Errors that can occur during product operations.
#[derive(Debug, Clone, Error, PartialEq)]
#[allow(dead_code)]
pub enum ProductError {
    /// The requested product was not found.
    #[error("Product not found: {0}")]
    NotFound(String),

    /// The requested quantity exceeds the available stock.
    #[error("Insufficient stock: requested {requested}, available {available}")]
    InsufficientStock { requested: u32, available: u32 },

    /// The provided quantity is invalid (e.g., zero or negative).
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(u32),

    /// An underlying database error occurred.
    #[error("Product database error: {0}")]
    DatabaseError(String),

    /// An error occurred while communicating with the actor system.
    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}

impl From<String> for ProductError {
    fn from(msg: String) -> Self {
        ProductError::ActorCommunicationError(msg)
    }
}
