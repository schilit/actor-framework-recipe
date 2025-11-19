use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::{Debug, Display};
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;

// =============================================================================
// 1. THE ABSTRACTION (Traits with Hooks, DTOs, and Actions)
// =============================================================================

/// Trait that any domain entity must implement to be managed by ResourceActor
pub trait Entity: Clone + Send + Sync + 'static {
    type Id: Eq + Hash + Clone + Send + Sync + Display + Debug;
    type CreatePayload: Send + Sync + Debug;
    type Patch: Send + Sync + Debug;
    
    // --- New: Custom Actions ---
    type Action: Send + Sync + Debug;
    type ActionResult: Send + Sync + Debug;

    /// Get the ID of the entity
    fn id(&self) -> &Self::Id;
    
    /// Construct the full Entity from the ID and Payload
    fn from_create(id: Self::Id, payload: Self::CreatePayload) -> Result<Self, String>;

    // --- Lifecycle Hooks ---

    fn on_create(&mut self) -> Result<(), String> { Ok(()) }
    fn on_update(&mut self, patch: Self::Patch) -> Result<(), String>;
    fn on_delete(&self) -> Result<(), String> { Ok(()) }

    // --- Action Handler ---
    
    /// Handle a custom domain-specific action
    fn handle_action(&mut self, action: Self::Action) -> Result<Self::ActionResult, String>;
}

// =============================================================================
// 2. THE GENERIC MESSAGES
// =============================================================================

pub type Response<T> = oneshot::Sender<Result<T, String>>;

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
        let client = ResourceClient { sender };
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
                                let _ = respond_to.send(Err(e));
                                continue;
                            }
                            self.store.insert(id.clone(), item);
                            let _ = respond_to.send(Ok(id));
                        }
                        Err(e) => { let _ = respond_to.send(Err(e)); }
                    }
                }
                ResourceRequest::Get { id, respond_to } => {
                    let item = self.store.get(&id).cloned();
                    let _ = respond_to.send(Ok(item));
                }
                ResourceRequest::Update { id, patch, respond_to } => {
                    if let Some(item) = self.store.get_mut(&id) {
                        if let Err(e) = item.on_update(patch) {
                            let _ = respond_to.send(Err(e));
                            continue;
                        }
                        let _ = respond_to.send(Ok(item.clone()));
                    } else {
                        let _ = respond_to.send(Err(format!("Item not found: {}", id)));
                    }
                }
                ResourceRequest::Delete { id, respond_to } => {
                    if let Some(item) = self.store.get(&id) {
                        if let Err(e) = item.on_delete() {
                            let _ = respond_to.send(Err(e));
                            continue;
                        }
                        self.store.remove(&id);
                        let _ = respond_to.send(Ok(()));
                    } else {
                        let _ = respond_to.send(Err(format!("Item not found: {}", id)));
                    }
                }
                ResourceRequest::Action { id, action, respond_to } => {
                    if let Some(item) = self.store.get_mut(&id) {
                        let result = item.handle_action(action);
                        let _ = respond_to.send(result);
                    } else {
                         let _ = respond_to.send(Err(format!("Item not found: {}", id)));
                    }
                }
            }
        }
    }
}

// =============================================================================
// 4. THE GENERIC CLIENT
// =============================================================================

#[derive(Clone)]
pub struct ResourceClient<T: Entity> {
    sender: mpsc::Sender<ResourceRequest<T>>,
}

impl<T: Entity> ResourceClient<T> {
    pub async fn create(&self, payload: T::CreatePayload) -> Result<T::Id, String> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Create { payload, respond_to })
            .await.map_err(|_| "Actor closed".to_string())?;
        response.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn get(&self, id: T::Id) -> Result<Option<T>, String> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Get { id, respond_to })
            .await.map_err(|_| "Actor closed".to_string())?;
        response.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn update(&self, id: T::Id, patch: T::Patch) -> Result<T, String> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Update { id, patch, respond_to })
            .await.map_err(|_| "Actor closed".to_string())?;
        response.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn delete(&self, id: T::Id) -> Result<(), String> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Delete { id, respond_to })
            .await.map_err(|_| "Actor closed".to_string())?;
        response.await.map_err(|_| "Actor dropped".to_string())?
    }

    pub async fn perform_action(&self, id: T::Id, action: T::Action) -> Result<T::ActionResult, String> {
        let (respond_to, response) = oneshot::channel();
        self.sender.send(ResourceRequest::Action { id, action, respond_to })
            .await.map_err(|_| "Actor closed".to_string())?;
        response.await.map_err(|_| "Actor dropped".to_string())?
    }
}

// =============================================================================
// 5. EXAMPLE USAGE (Test)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
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
        Rename(String),
    }

    impl Entity for SimpleUser {
        type Id = String;
        type CreatePayload = SimpleUserCreate;
        type Patch = SimpleUserPatch;
        type Action = UserAction;
        type ActionResult = bool;

        fn id(&self) -> &String { &self.id }

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
    }
}
