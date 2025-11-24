//! # Generic Actor Server
//!
//! This module defines the generic actor that manages entities.

use crate::client::ResourceClient;
use crate::entity::ActorEntity;
use crate::error::FrameworkError;
use crate::message::ResourceRequest;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

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
/// ## ResourceActor
///
/// The `ResourceActor<T>` struct is the *server* side of the framework. It owns the in‑memory store for a given entity type `T: ActorEntity` and processes all incoming `ResourceRequest<T>` messages sequentially. Each actor runs in its own Tokio task, guaranteeing exclusive access to its state without any locking.
///
/// * **Concurrency model** – each actor processes one message at a time, eliminating data races.
/// * **Context injection** – a user‑provided `Context` is passed to every lifecycle hook, enabling dependency injection.
/// * **Uniform API** – works with any entity that implements `ActorEntity`, providing a generic CRUD + Action implementation.
///
pub struct ResourceActor<T: ActorEntity> {
    receiver: mpsc::Receiver<ResourceRequest<T>>,
    store: HashMap<T::Id, T>,
    next_id: u32,
}

impl<T: ActorEntity> ResourceActor<T> {
    pub fn new(buffer_size: usize) -> (Self, ResourceClient<T>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let actor = Self {
            receiver,
            store: HashMap::new(),
            next_id: 1,
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
                    let id = T::Id::from(self.next_id);
                    self.next_id += 1;

                    match T::from_create_params(id.clone(), params) {
                        Ok(mut item) => {
                            // Await the async hook
                            if let Err(e) = item.on_create(&context).await {
                                warn!(entity_type, error = %e, "on_create failed");
                                let _ =
                                    respond_to.send(Err(FrameworkError::EntityError(Box::new(e))));
                                continue;
                            }
                            self.store.insert(id.clone(), item);
                            info!(entity_type, %id, size = self.store.len(), "Created");
                            let _ = respond_to.send(Ok(id));
                        }
                        Err(e) => {
                            warn!(entity_type, error = %e, "Create failed");
                            let _ = respond_to.send(Err(FrameworkError::EntityError(Box::new(e))));
                        }
                    }
                }
                ResourceRequest::Get { id, respond_to } => {
                    let item = self.store.get(&id).cloned();
                    let found = item.is_some();
                    debug!(entity_type, %id, found, "Get");
                    let _ = respond_to.send(Ok(item));
                }
                ResourceRequest::Update {
                    id,
                    update,
                    respond_to,
                } => {
                    debug!(entity_type, %id, ?update, "Update");
                    if let Some(item) = self.store.get_mut(&id) {
                        // Await the async hook
                        if let Err(e) = item.on_update(update, &context).await {
                            warn!(entity_type, %id, error = %e, "Update failed");
                            let _ = respond_to.send(Err(FrameworkError::EntityError(Box::new(e))));
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
                            let _ = respond_to.send(Err(FrameworkError::EntityError(Box::new(e))));
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
                ResourceRequest::Action {
                    id,
                    action,
                    respond_to,
                } => {
                    debug!(entity_type, %id, ?action, "Action");
                    if let Some(item) = self.store.get_mut(&id) {
                        // Await the async hook
                        let result = item
                            .handle_action(action, &context)
                            .await
                            .map_err(|e| FrameworkError::EntityError(Box::new(e)));
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
