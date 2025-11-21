use thiserror::Error;

/// Errors that can occur during user operations.
#[derive(Debug, Clone, Error, PartialEq)]
#[allow(dead_code)]
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
