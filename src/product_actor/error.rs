use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq)]
#[allow(dead_code)]
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
