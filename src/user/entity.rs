use crate::actor_framework::Entity;
use crate::domain::{User, UserCreate, UserPatch};

impl Entity for User {
    type Id = String;
    type CreatePayload = UserCreate;
    type Patch = UserPatch;
    type Action = (); 
    type ActionResult = ();

    fn id(&self) -> &String { &self.id }

    fn from_create(id: String, payload: UserCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            name: payload.name,
            email: payload.email,
        })
    }

    fn on_update(&mut self, patch: UserPatch) -> Result<(), String> {
        if let Some(name) = patch.name {
            self.name = name;
        }
        if let Some(email) = patch.email {
            self.email = email;
        }
        Ok(())
    }

    fn handle_action(&mut self, _action: ()) -> Result<(), String> {
        Ok(())
    }
}
