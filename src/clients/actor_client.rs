use crate::framework::{ActorEntity, FrameworkError, ResourceClient};
use async_trait::async_trait;

/// Trait for resource-specific clients to inherit standard CRUD operations.
///
/// This trait reduces boilerplate by providing default implementations for
/// common operations like `get` and `delete`.
#[async_trait]
pub trait ActorClient<T: ActorEntity>: Send + Sync {
    /// The resource-specific error type.
    type Error: From<String> + Send + Sync;

    /// Access the inner generic ResourceClient.
    fn inner(&self) -> &ResourceClient<T>;

    /// Map framework errors to the specific resource error type.
    fn map_error(e: FrameworkError) -> Self::Error;

    /// Fetch an entity by ID.
    #[tracing::instrument(skip(self))]
    async fn get(&self, id: T::Id) -> Result<Option<T>, Self::Error> {
        tracing::debug!("Sending request");
        self.inner().get(id).await.map_err(Self::map_error)
    }

    /// Delete an entity by ID.
    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: T::Id) -> Result<(), Self::Error> {
        tracing::debug!("Sending request");
        self.inner().delete(id).await.map_err(Self::map_error)
    }
}
