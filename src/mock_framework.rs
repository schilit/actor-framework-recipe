//! # Mock Framework
//!
//! Utilities for testing clients in isolation.
//!
//! Use [`create_mock_client`] to get a client and a receiver.
//! Then use helpers like [`expect_create`] or [`expect_action`] to assert behavior.

use crate::actor_framework::{Entity, ResourceClient, ResourceRequest, FrameworkError};
use tokio::sync::mpsc;

/// Creates a mock client and a receiver for asserting requests.
///
/// # Testing Strategy
/// In unit/integration tests, we don't want to spin up a full `ResourceActor` if we are just
/// testing the *Client* logic (e.g., `OrderClient`).
///
/// Instead, we create a "Mock Client". This client sends messages to a channel we control (`receiver`).
/// We can then inspect the messages arriving on that channel and assert they are correct.
/// This allows us to simulate the Actor's behavior (success, failure, delays) deterministically.
pub fn create_mock_client<T: Entity>(buffer_size: usize) -> (ResourceClient<T>, mpsc::Receiver<ResourceRequest<T>>) {
    let (sender, receiver) = mpsc::channel(buffer_size);
    (ResourceClient::new(sender), receiver)
}

/// Helper to verify that the next message is a Create request
pub async fn expect_create<T: Entity>(receiver: &mut mpsc::Receiver<ResourceRequest<T>>) -> Option<(T::CreateParams, tokio::sync::oneshot::Sender<Result<T::Id, FrameworkError>>)> {
    match receiver.recv().await {
        Some(ResourceRequest::Create { params, respond_to }) => Some((params, respond_to)),
        _ => None,
    }
}

/// Helper to verify that the next message is a Get request
pub async fn expect_get<T: Entity>(receiver: &mut mpsc::Receiver<ResourceRequest<T>>) -> Option<(T::Id, tokio::sync::oneshot::Sender<Result<Option<T>, FrameworkError>>)> {
    match receiver.recv().await {
        Some(ResourceRequest::Get { id, respond_to }) => Some((id, respond_to)),
        _ => None,
    }
}

/// Helper to verify that the next message is an Action request
pub async fn expect_action<T: Entity>(receiver: &mut mpsc::Receiver<ResourceRequest<T>>) -> Option<(T::Id, T::Action, tokio::sync::oneshot::Sender<Result<T::ActionResult, FrameworkError>>)> {
    match receiver.recv().await {
        Some(ResourceRequest::Action { id, action, respond_to }) => Some((id, action, respond_to)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{User, UserCreate};

    #[tokio::test]
    async fn test_mock_client() {
        let (client, mut receiver) = create_mock_client::<User>(10);

        // Test Create
        let create_task = tokio::spawn(async move {
            let user = UserCreate { name: "Test".to_string(), email: "test@example.com".to_string() };
            client.create(user).await
        });

        let (payload, responder) = expect_create(&mut receiver).await.expect("Expected Create request");
        assert_eq!(payload.name, "Test");
        responder.send(Ok("user_1".to_string())).unwrap();

        let result = create_task.await.unwrap();
        assert_eq!(result, Ok("user_1".to_string()));
    }
}
