use crate::clients::actor_client::ActorClient;
use crate::framework::{FrameworkError, ResourceClient};
use crate::model::{User, UserCreate, UserUpdate};
use crate::user_actor::UserError;
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
    pub async fn create_user(&self, user: User) -> Result<String, UserError> {
        debug!("Sending request");
        // Adapter: Convert legacy User struct to UserCreate payload
        let payload = UserCreate {
            name: user.name,
            email: user.email,
        };
        self.inner
            .create(payload)
            .await
            .map_err(|e| UserError::ActorCommunicationError(e.to_string()))
    }

    // New method utilizing the generic update
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn update_user(&self, id: String, update: UserUpdate) -> Result<User, UserError> {
        debug!("Sending request");
        self.inner
            .update(id, update)
            .await
            .map_err(|e| UserError::ActorCommunicationError(e.to_string()))
    }
}
