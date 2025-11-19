//! # Core Actor Framework
//!
//! This module defines the generic building blocks for the actor system.
//!
//! ## Key Types
//!
//! - [`Entity`]: The trait that all domain objects must implement.
//! - [`ResourceActor`]: The generic actor that manages entities.
//! - [`ResourceClient`]: The generic client for communicating with actors.
//! - [`FrameworkError`]: Common errors (e.g., ActorClosed, NotFound).

use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::{Debug, Display};
use tokio::sync::{mpsc, oneshot};


// =============================================================================
// 1. THE ABSTRACTION (Traits with Hooks, DTOs, and Actions)
// =============================================================================

/// Trait that any domain entity must implement to be managed by ResourceActor.
///
/// # Architecture Note
/// Why do we need this trait?
/// By defining a contract (`Entity`) that all our domain objects (User, Product, Order)
/// must satisfy, we can write the `ResourceActor` logic *once* and reuse it everywhere.
/// This is "Polymorphism" in action.
///
/// We use "Associated Types" (type Id, type CreatePayload, etc.) to enforce type safety.
/// A `User` entity requires a `UserCreate` payload, and you can't accidentally send it
/// a `ProductCreate` payload. The compiler prevents this class of bugs entirely.
pub trait Entity: Clone + Send + Sync + 'static {
    /// The unique identifier for this entity (e.g., String, Uuid, u64).
    type Id: Eq + Hash + Clone + Send + Sync + Display + Debug;
    
    /// The data required to create a new instance (DTO - Data Transfer Object).
    type CreatePayload: Send + Sync + Debug;
    
    /// The data required to update an existing instance.
    type Patch: Send + Sync + Debug;
    
    // --- New: Custom Actions ---
    /// Enum representing domain-specific operations (e.g., `ReserveStock`).
    type Action: Send + Sync + Debug;
    
    /// The result type returned by custom actions.
    type ActionResult: Send + Sync + Debug;

    /// Construct the full Entity from the ID and Payload.
    /// This is called by the actor when it receives a `Create` request.
    fn from_create(id: Self::Id, payload: Self::CreatePayload) -> Result<Self, String>;

    // --- Lifecycle Hooks ---
    // These allow the entity to execute logic during lifecycle events.
    // Default implementations do nothing (Ok(())), but can be overridden.

    fn on_create(&mut self) -> Result<(), String> { Ok(()) }
    fn on_update(&mut self, patch: Self::Patch) -> Result<(), String>;
    fn on_delete(&self) -> Result<(), String> { Ok(()) }

    // --- Action Handler ---
    
    /// Handle a custom domain-specific action.
    /// This is where the "business logic" for complex operations lives.
    fn handle_action(&mut self, action: Self::Action) -> Result<Self::ActionResult, String>;
}

// =============================================================================
// 2. THE GENERIC MESSAGES
// =============================================================================

// =============================================================================
// 2. THE GENERIC MESSAGES & ERRORS
// =============================================================================

/// Errors that can occur within the actor framework itself.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FrameworkError {
    #[error("Actor closed")]
    ActorClosed,
    #[error("Actor dropped response channel")]
    ActorDropped,
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Type alias for the one-shot response channel used by actors.
pub type Response<T> = oneshot::Sender<Result<T, FrameworkError>>;

/// Internal message type sent to the actor to request operations.
#[derive(Debug)]
pub enum ResourceRequest<T: Entity> {
    Create {
        payload: T::CreatePayload,
        respond_to: Response<T::Id>,
    },
    Get {
        id: T::Id,
        respond_to: Response<Option<T>>,
    },
    Update {
        id: T::Id,
        patch: T::Patch,
        respond_to: Response<T>,
    },
    #[allow(dead_code)]
    Delete {
        id: T::Id,
        respond_to: Response<()>,
    },
    Action {
        id: T::Id,
        action: T::Action,
        respond_to: Response<T::ActionResult>,
    }
}

// =============================================================================
// 3. THE GENERIC ACTOR SERVER
// =============================================================================

/// The generic actor that manages a collection of entities.
///
/// # Architecture Note
/// This struct is the "Server" half of the actor. It owns the state (`store`) and
/// the receiver end of the channel.
///
/// **Concurrency Model**:
/// Even though we might have 1000 `ResourceActor` instances running, each one
/// processes its own messages *sequentially* in a loop. This means we don't need
/// `Mutex` or `RwLock` for the `store`! The "Actor Model" gives us safety through
/// exclusive ownership of state within the task.
pub struct ResourceActor<T: Entity> {
    receiver: mpsc::Receiver<ResourceRequest<T>>,
    store: HashMap<T::Id, T>,
    next_id_fn: Box<dyn Fn() -> T::Id + Send + Sync>,
}

impl<T: Entity> ResourceActor<T> {
    pub fn new(
        buffer_size: usize, 
        next_id_fn: impl Fn() -> T::Id + Send + Sync + 'static
    ) -> (Self, ResourceClient<T>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let actor = Self {
            receiver,
            store: HashMap::new(),
            next_id_fn: Box::new(next_id_fn),
        };
        let client = ResourceClient::new(sender);
        (actor, client)
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                ResourceRequest::Create { payload, respond_to } => {
                    let id = (self.next_id_fn)();
                    match T::from_create(id.clone(), payload) {
                        Ok(mut item) => {
                            if let Err(e) = item.on_create() {
                                let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                                continue;
                            }
                            self.store.insert(id.clone(), item);
                            let _ = respond_to.send(Ok(id));
                        }
                        Err(e) => { let _ = respond_to.send(Err(FrameworkError::Custom(e))); }
                    }
                }
                ResourceRequest::Get { id, respond_to } => {
                    let item = self.store.get(&id).cloned();
                    let _ = respond_to.send(Ok(item));
                }
                ResourceRequest::Update { id, patch, respond_to } => {
                    if let Some(item) = self.store.get_mut(&id) {
                        if let Err(e) = item.on_update(patch) {
                            let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                            continue;
                        }
                        let _ = respond_to.send(Ok(item.clone()));
                    } else {
                        let _ = respond_to.send(Err(FrameworkError::NotFound(id.to_string())));
                    }
                }
                ResourceRequest::Delete { id, respond_to } => {
                    if let Some(item) = self.store.get(&id) {
                        if let Err(e) = item.on_delete() {
                            let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                            continue;
                        }
                        self.store.remove(&id);
                        let _ = respond_to.send(Ok(()));
                    } else {
                        let _ = respond_to.send(Err(FrameworkError::NotFound(id.to_string())));
                    }
                }
                ResourceRequest::Action { id, action, respond_to } => {
                    if let Some(item) = self.store.get_mut(&id) {
                        let result = item.handle_action(action)
                            .map_err(FrameworkError::Custom);
                        let _ = respond_to.send(result);
                    } else {
                         let _ = respond_to.send(Err(FrameworkError::NotFound(id.to_string())));
                    }
                }
            }
        }
    }
}

// =============================================================================
// 4. THE GENERIC CLIENT
// =============================================================================

/// A type-safe client for interacting with a `ResourceActor`.
#[derive(Clone)]
pub struct ResourceClient<T: Entity> {
    sender: mpsc::Sender<ResourceRequest<T>>,
}

impl<T: Entity> ResourceClient<T> {
    pub fn new(sender: mpsc::Sender<ResourceRequest<T>>) -> Self {
        Self { sender }
    }

    pub async fn create(&self, payload: T::CreatePayload) -> Result<T::Id, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Create { payload, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }

    pub async fn get(&self, id: T::Id) -> Result<Option<T>, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Get { id, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }

    pub async fn update(&self, id: T::Id, patch: T::Patch) -> Result<T, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Update { id, patch, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }

    #[allow(dead_code)]
    pub async fn delete(&self, id: T::Id) -> Result<(), FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Delete { id, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }

    pub async fn perform_action(&self, id: T::Id, action: T::Action) -> Result<T::ActionResult, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Action { id, action, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }
}

// =============================================================================
// 5. EXAMPLE USAGE (Test)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    // --- Domain Definition ---

    #[derive(Clone, Debug, PartialEq)]
    struct SimpleUser {
        id: String,
        name: String,
        is_admin: bool,
        created_at: u64,
    }

    #[derive(Debug)]
    struct SimpleUserCreate {
        name: String,
    }

    #[derive(Debug)]
    struct SimpleUserPatch {
        name: Option<String>,
    }

    // Custom Actions
    #[derive(Debug)]
    enum UserAction {
        PromoteToAdmin,
        #[allow(dead_code)]
        Rename(String),
    }

    impl Entity for SimpleUser {
        type Id = String;
        type CreatePayload = SimpleUserCreate;
        type Patch = SimpleUserPatch;
        type Action = UserAction;
        type ActionResult = bool;

        // fn id(&self) -> &String { &self.id }

        fn from_create(id: String, payload: SimpleUserCreate) -> Result<Self, String> {
            Ok(Self {
                id,
                name: payload.name,
                is_admin: false,
                created_at: 100,
            })
        }

        fn on_update(&mut self, patch: SimpleUserPatch) -> Result<(), String> {
            if let Some(name) = patch.name {
                self.name = name;
            }
            Ok(())
        }

        fn handle_action(&mut self, action: UserAction) -> Result<bool, String> {
            match action {
                UserAction::PromoteToAdmin => {
                    if self.is_admin {
                        Ok(false)
                    } else {
                        self.is_admin = true;
                        Ok(true)
                    }
                }
                UserAction::Rename(new_name) => {
                    self.name = new_name;
                    Ok(true)
                }
            }
        }
    }

    // --- Test ---

    #[tokio::test]
    async fn test_resource_actor_with_actions() {
        // ID Generator
        let counter = Arc::new(AtomicU64::new(1));
        let next_id = move || {
            let id = counter.fetch_add(1, Ordering::SeqCst);
            format!("user_{}", id)
        };

        // Start Actor
        let (actor, client) = ResourceActor::new(10, next_id);
        tokio::spawn(actor.run());

        // 1. Create
        let payload = SimpleUserCreate { name: "Alice".into() };
        let id: String = client.create(payload).await.unwrap();

        // 2. Perform Action: Promote
        let changed: bool = client.perform_action(id.clone(), UserAction::PromoteToAdmin).await.unwrap();
        assert!(changed);

        // Verify state
        let user: SimpleUser = client.get(id.clone()).await.unwrap().unwrap();
        assert!(user.is_admin);

        // 3. Perform Action: Promote again (should return false)
        let changed_again: bool = client.perform_action(id.clone(), UserAction::PromoteToAdmin).await.unwrap();
        assert!(!changed_again);

        // 4. Update
        let patch = SimpleUserPatch { name: Some("Bob".into()) };
        let updated_user = client.update(id.clone(), patch).await.unwrap();
        assert_eq!(updated_user.name, "Bob");

        // 5. Delete
        client.delete(id.clone()).await.unwrap();
        let deleted_user = client.get(id.clone()).await.unwrap();
        assert!(deleted_user.is_none());
    }
}
