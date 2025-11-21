//! Error types for the User actor.

use thiserror::Error;

/// Errors that can occur during user operations.
#[derive(Debug, Clone, Error, PartialEq)]
#[allow(dead_code)]
pub enum UserError {
    /// The requested user was not found.
    #[error("User not found: {0}")]
    NotFound(String),

    /// A user with the same unique identifier (e.g., email) already exists.
    #[error("User already exists: {0}")]
    AlreadyExists(String),

    /// The user data provided is invalid.
    #[error("User validation error: {0}")]
    ValidationError(String),

    /// An underlying database error occurred.
    #[error("User database error: {0}")]
    DatabaseError(String),

    /// An error occurred while communicating with the actor system.
    #[error("Actor communication error: {0}")]
    ActorCommunicationError(String),
}

impl From<String> for UserError {
    fn from(msg: String) -> Self {
        UserError::ActorCommunicationError(msg)
    }
}
