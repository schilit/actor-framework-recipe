use serde::{Deserialize, Serialize};

/// Represents a registered user in the system.
///
/// # Actor Framework
/// This struct implements the [`ActorEntity`](actor_framework::ActorEntity) trait,
/// allowing it to be managed by a [`ResourceActor`](actor_framework::ResourceActor).
///
/// See [`impl ActorEntity for User`](#impl-ActorEntity-for-User) for details on:
/// - Creation parameters ([`UserCreate`])
/// - Update parameters ([`UserUpdate`])
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

/// Payload for creating a new user.
#[derive(Debug, Clone)]
pub struct UserCreate {
    pub name: String,
    pub email: String,
}

/// Payload for updating an existing user.
/// DTOs for User updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl User {
    /// Creates a new User instance.
    ///
    /// # Arguments
    /// * `name` - User's display name
    /// * `email` - User's email address
    ///
    /// # Notes
    /// The `id` field is initialized as an empty string and will be set by the actor system.
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            name: name.into(),
            email: email.into(),
        }
    }
}
