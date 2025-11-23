//! User-specific resource logic and entity implementation.

pub mod entity;
pub mod error;

pub use error::*;

use crate::clients::UserClient;
use crate::framework::ResourceActor;
use crate::model::User;
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
