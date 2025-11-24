//! # User Actor
//!
//! This module implements the User resource actor, managing user entities with CRUD operations.
//!
//! ## Overview
//!
//! The User actor is the simplest example in the system, demonstrating the basic actor pattern
//! without dependencies or custom actions. It manages user registration and profile information.
//!
//! ## Structure
//!
//! - [`entity`] - [`ActorEntity`](actor_framework::ActorEntity) implementation for [`User`]
//! - [`error`] - [`UserError`] type for type-safe error handling
//! - [`new()`] - Factory function that creates the actor and client
//!
//! ## Usage
//!
//! ```rust
//! use actor_sample::user_actor;
//! use actor_sample::model::UserCreate;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create actor and client
//!     let (actor, client) = user_actor::new();
//!
//!     // Start the actor (no dependencies, so context is ())
//!     tokio::spawn(actor.run(()));
//!
//!     // Use the client
//!     let params = UserCreate {
//!         name: "Alice".to_string(),
//!         email: "alice@example.com".to_string(),
//!     };
//!     let id = client.create_user(params).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Key Features
//!
//! - **No dependencies**: User actor has no context dependencies (Context = ())
//! - **Sequential ID generation**: Uses atomic counter for deterministic IDs
//! - **Type-safe errors**: All operations return `Result<T, UserError>`

pub mod entity;
pub mod error;

pub use error::*;

use crate::clients::UserClient;
use crate::model::User;
use actor_framework::ResourceActor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Creates a new User actor and its client.
pub fn new() -> (ResourceActor<User>, UserClient) {
    let user_id_counter = Arc::new(AtomicU64::new(1));
    let next_user_id = move || {
        let id = user_id_counter.fetch_add(1, Ordering::SeqCst);
        format!("user_{}", id)
    };

    let (actor, generic_client) = ResourceActor::new(32, next_user_id);
    let client = UserClient::new(generic_client);

    (actor, client)
}
