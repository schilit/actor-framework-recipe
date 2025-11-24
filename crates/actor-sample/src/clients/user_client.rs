//! # User Client
//!
//! Provides a high‑level API for interacting with the `User` actor.
//! It wraps a `ResourceClient<User>` and exposes domain‑specific methods.
use crate::model::{User, UserCreate, UserId, UserUpdate};
use crate::user_actor::UserError;
use actor_framework::ActorClient;
use actor_framework::{FrameworkError, ResourceClient};
use async_trait::async_trait;
use tracing::{debug, instrument};

/// Client for interacting with the User actor.
#[derive(Clone)]
pub struct UserClient {
    inner: ResourceClient<User>,
}

impl UserClient {
    pub fn new(inner: ResourceClient<User>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl ActorClient<User> for UserClient {
    type Error = UserError;

    fn inner(&self) -> &ResourceClient<User> {
        &self.inner
    }

    fn map_error(e: FrameworkError) -> Self::Error {
        UserError::ActorCommunicationError(e.to_string())
    }
}

impl UserClient {
    // Custom create method as it needs specific payload conversion

    #[instrument(skip(self))]
    pub async fn create_user(&self, params: UserCreate) -> Result<UserId, UserError> {
        debug!("Sending request");
        self.inner
            .create(params)
            .await
            .map_err(|e| UserError::ActorCommunicationError(e.to_string()))
    }

    // New method utilizing the generic update
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn update_user(&self, id: UserId, update: UserUpdate) -> Result<User, UserError> {
        debug!("Sending request");
        self.inner
            .update(id, update)
            .await
            .map_err(|e| UserError::ActorCommunicationError(e.to_string()))
    }
}
