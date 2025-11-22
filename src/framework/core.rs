//! # Core Actor Framework
//!
//! This module defines the generic building blocks for the actor system.
//!
//! ## Key Types
//!
//! - [`ActorEntity`]: The trait that all resource types must implement.
//! - [`ResourceActor`]: The generic actor that manages entities.
//! - [`ResourceClient`]: The generic client for communicating with actors.
//! - [`FrameworkError`]: Common errors (e.g., ActorClosed, NotFound).

use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::{Debug, Display};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, warn};


// =============================================================================
// 1. THE ABSTRACTION (Traits with Hooks, DTOs, and Actions)
// =============================================================================

/// Trait that any resource entity must implement to be managed by ResourceActor.
///
/// # Architecture Note
/// Why do we need this trait?
/// By defining a contract (`ActorEntity`) that all our resource types (User, Product, Order)
/// must satisfy, we can write the `ResourceActor` logic *once* and reuse it everywhere.
/// This is "Polymorphism" in action.
///
/// We use "Associated Types" (type Id, type CreateParams, etc.) to enforce type safety.
/// A `User` entity requires a `UserCreate` payload, and you can't accidentally send it
/// a `ProductCreate` payload. The compiler prevents this class of bugs entirely.
///
/// # Provided Methods (Hooks)
/// This trait includes **Provided Methods** (methods with default implementations) for lifecycle hooks:
/// - [`ActorEntity::on_create`]
/// - [`ActorEntity::on_delete`]
///
/// You do **not** need to implement these methods unless you want to customize behavior.
/// The default implementation does nothing (`Ok(())`).
use async_trait::async_trait;

/// Trait that any resource entity must implement to be managed by ResourceActor.
///
/// # Architecture Note
/// By defining a contract (`ActorEntity`) that all our resource types (User, Product, Order)
/// must satisfy, we can write the `ResourceActor` logic *once* and reuse it everywhere.
///
/// # Async & Context
/// This trait is `#[async_trait]` to allow asynchronous operations in hooks (e.g., calling other actors).
/// It also defines a `Context` type, which is injected into every hook. This allows "Late Binding"
/// of dependencies (passing clients to `run()` instead of `new()`).
#[async_trait]
pub trait ActorEntity: Clone + Send + Sync + 'static {
    /// The unique identifier for this entity (e.g., String, Uuid, u64).
    type Id: Eq + Hash + Clone + Send + Sync + Display + Debug;
    
    /// The data required to create a new instance (DTO - Data Transfer Object).
    type CreateParams: Send + Sync + Debug;
    
    /// The data required to update an existing instance.
    type UpdateParams: Send + Sync + Debug;
    
    /// Enum representing resource-specific operations (e.g., `ReserveStock`).
    type Action: Send + Sync + Debug;
    
    /// The result type returned by custom actions.
    type ActionResult: Send + Sync + Debug;

    /// The runtime context (dependencies) injected into the actor.
    /// Use `()` if no dependencies are needed.
    type Context: Send + Sync;

    /// Construct the full Entity from the ID and Payload.
    /// This is called synchronously before `on_create`.
    fn from_create_params(id: Self::Id, params: Self::CreateParams) -> Result<Self, String>;

    // --- Lifecycle Hooks (Async) ---

    /// Called immediately after the entity is created and initialized.
    /// Use this hook to perform validation or side effects (e.g., checking other actors).
    async fn on_create(&mut self, _ctx: &Self::Context) -> Result<(), String> { Ok(()) }

    /// Called when an update request is received.
    async fn on_update(&mut self, update: Self::UpdateParams, _ctx: &Self::Context) -> Result<(), String>;

    /// Called immediately before the entity is removed from the system.
    async fn on_delete(&self, _ctx: &Self::Context) -> Result<(), String> { Ok(()) }

    // --- Action Handler (Async) ---
    
    /// Handle a custom resource-specific action.
    async fn handle_action(&mut self, action: Self::Action, _ctx: &Self::Context) -> Result<Self::ActionResult, String>;
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
///
/// # Resource-Oriented Architecture
/// This enum implements a **Resource-Oriented** design pattern where each actor manages a specific
/// type of resource (the [`ActorEntity`]). Instead of defining ad-hoc messages for every operation,
/// we standardize around a set of lifecycle operations that apply to almost any persistent resource.
///
/// # The CRUD Pattern
/// The variants of this enum map directly to standard **CRUD** (Create, Read, Update, Delete) operations,
/// plus a custom `Action` variant for resource-specific logic that doesn't fit the CRUD model.
///
/// - **Create**: Lifecycle start. Uses [`ActorEntity::CreateParams`] to initialize a new resource.
/// - **Get (Read)**: Retrieval. Fetches the current state of the resource by ID.
/// - **Update**: State mutation. Uses [`ActorEntity::UpdateParams`] to modify an existing resource.
/// - **Delete**: Lifecycle end. Removes the resource.
/// - **Action**: Extensibility. Executes a custom [`ActorEntity::Action`].
///
/// # Entity Interaction
/// This type is generic over `T: ActorEntity`. It uses the associated types defined in the [`ActorEntity`] trait
/// (like `CreateParams`, `UpdateParams`, `Action`) to ensure type safety for every operation.
/// This guarantees that you can't send a "User Create" payload to a "Product" actor.
#[derive(Debug)]
pub enum ResourceRequest<T: ActorEntity> {
    Create {
        params: T::CreateParams,
        respond_to: Response<T::Id>,
    },
    Get {
        id: T::Id,
        respond_to: Response<Option<T>>,
    },
    Update {
        id: T::Id,
        update: T::UpdateParams,
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
pub struct ResourceActor<T: ActorEntity> {
    receiver: mpsc::Receiver<ResourceRequest<T>>,
    store: HashMap<T::Id, T>,
    next_id_fn: Box<dyn Fn() -> T::Id + Send + Sync>,
}

impl<T: ActorEntity> ResourceActor<T> {
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

    /// Runs the actor's event loop, processing messages until the channel closes.
    ///
    /// # Context Injection
    /// The `context` argument is injected into every entity hook. This allows entities
    /// to access external dependencies (like other clients) that were created *after*
    /// the actor was instantiated but *before* the loop started.
    pub async fn run(mut self, context: T::Context) {
        // Extract just the type name (e.g., "User" instead of "actor_recipe::model::user::User")
        let entity_type = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("Unknown");
        info!(entity_type, "Actor started");
        
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                ResourceRequest::Create { params, respond_to } => {
                    debug!(entity_type, ?params, "Create");
                    let id = (self.next_id_fn)();
                    
                    match T::from_create_params(id.clone(), params) {
                        Ok(mut item) => {
                            // Await the async hook
                            if let Err(e) = item.on_create(&context).await {
                                warn!(entity_type, error = %e, "on_create failed");
                                let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                                continue;
                            }
                            self.store.insert(id.clone(), item);
                            info!(entity_type, %id, size = self.store.len(), "Created");
                            let _ = respond_to.send(Ok(id));
                        }
                        Err(e) => {
                            warn!(entity_type, error = %e, "Create failed");
                            let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                        }
                    }
                }
                ResourceRequest::Get { id, respond_to } => {
                    let item = self.store.get(&id).cloned();
                    let found = item.is_some();
                    debug!(entity_type, %id, found, "Get");
                    let _ = respond_to.send(Ok(item));
                }
                ResourceRequest::Update { id, update, respond_to } => {
                    debug!(entity_type, %id, ?update, "Update");
                    if let Some(item) = self.store.get_mut(&id) {
                        // Await the async hook
                        if let Err(e) = item.on_update(update, &context).await {
                            warn!(entity_type, %id, error = %e, "Update failed");
                            let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                            continue;
                        }
                        info!(entity_type, %id, "Updated");
                        let _ = respond_to.send(Ok(item.clone()));
                    } else {
                        warn!(entity_type, %id, "Not found");
                        let _ = respond_to.send(Err(FrameworkError::NotFound(id.to_string())));
                    }
                }
                ResourceRequest::Delete { id, respond_to } => {
                    debug!(entity_type, %id, "Delete");
                    if let Some(item) = self.store.get(&id) {
                        // Await the async hook
                        if let Err(e) = item.on_delete(&context).await {
                            warn!(entity_type, %id, error = %e, "on_delete failed");
                            let _ = respond_to.send(Err(FrameworkError::Custom(e)));
                            continue;
                        }
                        self.store.remove(&id);
                        info!(entity_type, %id, size = self.store.len(), "Deleted");
                        let _ = respond_to.send(Ok(()));
                    } else {
                        warn!(entity_type, %id, "Not found");
                        let _ = respond_to.send(Err(FrameworkError::NotFound(id.to_string())));
                    }
                }
                ResourceRequest::Action { id, action, respond_to } => {
                    debug!(entity_type, %id, ?action, "Action");
                    if let Some(item) = self.store.get_mut(&id) {
                        // Await the async hook
                        let result = item.handle_action(action, &context).await
                            .map_err(FrameworkError::Custom);
                        match &result {
                            Ok(_) => info!(entity_type, %id, "Action ok"),
                            Err(e) => warn!(entity_type, %id, error = %e, "Action failed"),
                        }
                        let _ = respond_to.send(result);
                    } else {
                        warn!(entity_type, %id, "Not found");
                        let _ = respond_to.send(Err(FrameworkError::NotFound(id.to_string())));
                    }
                }
            }
        }
        
        info!(entity_type, size = self.store.len(), "Shutdown");
    }
}

// =============================================================================
// 4. THE GENERIC CLIENT
// =============================================================================

/// A type-safe client for interacting with a `ResourceActor`.
#[derive(Clone)]
pub struct ResourceClient<T: ActorEntity> {
    sender: mpsc::Sender<ResourceRequest<T>>,
}

impl<T: ActorEntity> ResourceClient<T> {
    pub fn new(sender: mpsc::Sender<ResourceRequest<T>>) -> Self {
        Self { sender }
    }

    pub async fn create(&self, params: T::CreateParams) -> Result<T::Id, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Create { params, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }

    pub async fn get(&self, id: T::Id) -> Result<Option<T>, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Get { id, respond_to })
            .await.map_err(|_| FrameworkError::ActorClosed)?;
        response.await.map_err(|_| FrameworkError::ActorDropped)?
    }

    pub async fn update(&self, id: T::Id, update: T::UpdateParams) -> Result<T, FrameworkError> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Update { id, update, respond_to })
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
    struct SimpleUserUpdate {
        name: Option<String>,
    }

    // Custom Actions
    #[derive(Debug)]
    enum UserAction {
        PromoteToAdmin,
        #[allow(dead_code)]
        Rename(String),
    }

    #[async_trait]
    impl ActorEntity for SimpleUser {
        type Id = String;
        type CreateParams = SimpleUserCreate;
        type UpdateParams = SimpleUserUpdate;
        type Action = UserAction;
        type ActionResult = bool;
        type Context = ();

        // fn id(&self) -> &String { &self.id }

        fn from_create_params(id: String, params: SimpleUserCreate) -> Result<Self, String> {
            Ok(Self {
                id,
                name: params.name,
                is_admin: false,
                created_at: 100,
            })
        }

        async fn on_update(&mut self, update: SimpleUserUpdate, _ctx: &Self::Context) -> Result<(), String> {
            if let Some(name) = update.name {
                self.name = name;
            }
            Ok(())
        }

        async fn handle_action(&mut self, action: UserAction, _ctx: &Self::Context) -> Result<bool, String> {
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
        tokio::spawn(actor.run(()));

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
        let update = SimpleUserUpdate { name: Some("Bob".into()) };
        let updated_user = client.update(id.clone(), update).await.unwrap();
        assert_eq!(updated_user.name, "Bob");

        // 5. Delete
        client.delete(id.clone()).await.unwrap();
        let deleted_user = client.get(id.clone()).await.unwrap();
        assert!(deleted_user.is_none());
    }
}
