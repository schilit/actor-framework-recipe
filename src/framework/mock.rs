//! # Mock Framework
//!
//! Utilities for testing clients in isolation.
//!
//! Use [`create_mock_client`] to get a client and a receiver.
//! Then use helpers like [`expect_create`] or [`expect_action`] to assert behavior.

use crate::framework::{ActorEntity, ResourceClient, ResourceRequest, FrameworkError};
use tokio::sync::mpsc;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// =============================================================================
// EXPECTATION BUILDER API
// =============================================================================

/// Represents an expected request to the mock client.
///
/// This enum is used internally by `MockClient` to track what requests
/// are expected and what responses should be returned.
#[allow(dead_code)] // Future features: Update, Delete, Action expectations
enum Expectation<T: ActorEntity> {
    Get {
        id: T::Id,
        response: Result<Option<T>, FrameworkError>,
    },
    Create {
        response: Result<T::Id, FrameworkError>,
    },
    Update {
        id: T::Id,
        response: Result<T, FrameworkError>,
    },
    Delete {
        id: T::Id,
        response: Result<(), FrameworkError>,
    },
    Action {
        id: T::Id,
        response: Result<T::ActionResult, FrameworkError>,
    },
}

/// A mock client with expectation tracking for fluent testing.
///
/// # Example
/// ```ignore
/// let mut mock = MockClient::<User>::new();
/// mock.expect_get("user_1".to_string()).return_ok(Some(user));
/// mock.expect_create().return_ok("user_2".to_string());
///
/// let client = mock.client();
/// // Use client in tests...
/// mock.verify(); // Ensures all expectations were met
/// ```
pub struct MockClient<T: ActorEntity> {
    client: ResourceClient<T>,
    expectations: Arc<Mutex<VecDeque<Expectation<T>>>>,
    _handle: tokio::task::JoinHandle<()>,
}

impl<T: ActorEntity + Send + 'static> MockClient<T>
where
    T::Id: Send,
    T::CreateParams: Send,
    T::UpdateParams: Send,
    T::Action: Send,
    T::ActionResult: Send,
{
    /// Creates a new mock client with no expectations.
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::channel::<ResourceRequest<T>>(100);
        let expectations = Arc::new(Mutex::new(VecDeque::new()));
        let expectations_clone = expectations.clone();

        // Spawn background task to handle requests
        let handle = tokio::spawn(async move {
            while let Some(request) = receiver.recv().await {
                let mut exps = expectations_clone.lock().unwrap();
                let expectation = exps.pop_front();
                drop(exps); // Release lock before async operations

                match (request, expectation) {
                    (ResourceRequest::Get { id: _, respond_to }, Some(Expectation::Get { id: _, response })) => {
                        let _ = respond_to.send(response);
                    }
                    (ResourceRequest::Create { params: _, respond_to }, Some(Expectation::Create { response })) => {
                        let _ = respond_to.send(response);
                    }
                    (ResourceRequest::Update { id: _, update: _, respond_to }, Some(Expectation::Update { id: _, response })) => {
                        let _ = respond_to.send(response);
                    }
                    (ResourceRequest::Delete { id: _, respond_to }, Some(Expectation::Delete { id: _, response })) => {
                        let _ = respond_to.send(response);
                    }
                    (ResourceRequest::Action { id: _, action: _, respond_to }, Some(Expectation::Action { id: _, response })) => {
                        let _ = respond_to.send(response);
                    }
                    _ => {
                        panic!("Unexpected request or expectation mismatch");
                    }
                }
            }
        });

        Self {
            client: ResourceClient::new(sender),
            expectations,
            _handle: handle,
        }
    }

    /// Returns the client for use in tests.
    pub fn client(&self) -> ResourceClient<T> {
        self.client.clone()
    }

    /// Expects a `get` operation.
    pub fn expect_get(&mut self, id: T::Id) -> GetExpectationBuilder<T> {
        GetExpectationBuilder {
            id,
            expectations: self.expectations.clone(),
        }
    }

    /// Expects a `create` operation.
    pub fn expect_create(&mut self) -> CreateExpectationBuilder<T> {
        CreateExpectationBuilder {
            expectations: self.expectations.clone(),
        }
    }

    /// Expects an `action` operation.
    pub fn expect_action(&mut self, id: T::Id) -> ActionExpectationBuilder<T> {
        ActionExpectationBuilder {
            id,
            expectations: self.expectations.clone(),
        }
    }

    /// Verifies that all expectations were met.
    pub fn verify(&self) {
        let exps = self.expectations.lock().unwrap();
        if !exps.is_empty() {
            panic!("Not all expectations were met. {} remaining", exps.len());
        }
    }
}

/// Builder for `get` expectations.
pub struct GetExpectationBuilder<T: ActorEntity> {
    id: T::Id,
    expectations: Arc<Mutex<VecDeque<Expectation<T>>>>,
}

impl<T: ActorEntity> GetExpectationBuilder<T> {
    /// Sets the expectation to return a successful result.
    pub fn return_ok(self, value: Option<T>) {
        let mut exps = self.expectations.lock().unwrap();
        exps.push_back(Expectation::Get {
            id: self.id,
            response: Ok(value),
        });
    }

    /// Sets the expectation to return an error.
    pub fn return_err(self, error: FrameworkError) {
        let mut exps = self.expectations.lock().unwrap();
        exps.push_back(Expectation::Get {
            id: self.id,
            response: Err(error),
        });
    }
}

/// Builder for `create` expectations.
pub struct CreateExpectationBuilder<T: ActorEntity> {
    expectations: Arc<Mutex<VecDeque<Expectation<T>>>>,
}

impl<T: ActorEntity> CreateExpectationBuilder<T> {
    /// Sets the expectation to return a successful result.
    pub fn return_ok(self, id: T::Id) {
        let mut exps = self.expectations.lock().unwrap();
        exps.push_back(Expectation::Create {
            response: Ok(id),
        });
    }

    /// Sets the expectation to return an error.
    pub fn return_err(self, error: FrameworkError) {
        let mut exps = self.expectations.lock().unwrap();
        exps.push_back(Expectation::Create {
            response: Err(error),
        });
    }
}

/// Builder for `action` expectations.
pub struct ActionExpectationBuilder<T: ActorEntity> {
    id: T::Id,
    expectations: Arc<Mutex<VecDeque<Expectation<T>>>>,
}

impl<T: ActorEntity> ActionExpectationBuilder<T> {
    /// Sets the expectation to return a successful result.
    pub fn return_ok(self, result: T::ActionResult) {
        let mut exps = self.expectations.lock().unwrap();
        exps.push_back(Expectation::Action {
            id: self.id,
            response: Ok(result),
        });
    }

    /// Sets the expectation to return an error.
    pub fn return_err(self, error: FrameworkError) {
        let mut exps = self.expectations.lock().unwrap();
        exps.push_back(Expectation::Action {
            id: self.id,
            response: Err(error),
        });
    }
}

// =============================================================================
// LEGACY HELPERS (for backward compatibility)
// =============================================================================

/// Creates a mock client and a receiver for asserting requests.
///
/// # Testing Strategy
/// In unit/integration tests, we don't want to spin up a full `ResourceActor` if we are just
/// testing the *Client* logic (e.g., `OrderClient`).
///
/// Instead, we create a "Mock Client". This client sends messages to a channel we control (`receiver`).
/// We can then inspect the messages arriving on that channel and assert they are correct.
/// This allows us to simulate the Actor's behavior (success, failure, delays) deterministically.
///
/// **Note**: Consider using [`MockClient`] for a more fluent API.
pub fn create_mock_client<T: ActorEntity>(buffer_size: usize) -> (ResourceClient<T>, mpsc::Receiver<ResourceRequest<T>>) {
    let (sender, receiver) = mpsc::channel(buffer_size);
    (ResourceClient::new(sender), receiver)
}

/// Helper to verify that the next message is a Create request
pub async fn expect_create<T: ActorEntity>(receiver: &mut mpsc::Receiver<ResourceRequest<T>>) -> Option<(T::CreateParams, tokio::sync::oneshot::Sender<Result<T::Id, FrameworkError>>)> {
    match receiver.recv().await {
        Some(ResourceRequest::Create { params, respond_to }) => Some((params, respond_to)),
        _ => None,
    }
}

/// Helper to verify that the next message is a Get request
pub async fn expect_get<T: ActorEntity>(receiver: &mut mpsc::Receiver<ResourceRequest<T>>) -> Option<(T::Id, tokio::sync::oneshot::Sender<Result<Option<T>, FrameworkError>>)> {
    match receiver.recv().await {
        Some(ResourceRequest::Get { id, respond_to }) => Some((id, respond_to)),
        _ => None,
    }
}

/// Helper to verify that the next message is an Action request
pub async fn expect_action<T: ActorEntity>(receiver: &mut mpsc::Receiver<ResourceRequest<T>>) -> Option<(T::Id, T::Action, tokio::sync::oneshot::Sender<Result<T::ActionResult, FrameworkError>>)> {
    match receiver.recv().await {
        Some(ResourceRequest::Action { id, action, respond_to }) => Some((id, action, respond_to)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{User, UserCreate};

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

    #[tokio::test]
    async fn test_mock_client_with_expectations() {
        use crate::model::User;

        // Create mock with fluent expectation API
        let mut mock = MockClient::<User>::new();
        
        // Set up expectations
        mock.expect_create().return_ok("user_1".to_string());
        mock.expect_get("user_1".to_string()).return_ok(Some(User::new("user_1", "test@example.com")));
        
        let client = mock.client();

        // Execute operations
        let user = UserCreate { name: "Test".to_string(), email: "test@example.com".to_string() };
        let id = client.create(user).await.unwrap();
        assert_eq!(id, "user_1");

        let fetched = client.get("user_1".to_string()).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().email, "test@example.com");

        // Verify all expectations were met
        mock.verify();
    }
}
