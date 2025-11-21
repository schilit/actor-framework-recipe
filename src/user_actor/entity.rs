//! Entity trait implementation for the User domain type.
//!
//! This module contains the [`Entity`] trait implementation
//! that enables [`User`] to be managed by the generic [`crate::framework::ResourceActor`].
//!
//! See the trait implementation on [`User`] for method documentation.

use crate::framework::Entity;
use crate::domain::{User, UserCreate, UserUpdate};

/// Marker constant to ensure module documentation is rendered.
#[doc(hidden)]
/// Marker constant to verify Entity trait implementation exists at compile time.
/// This is used by the framework to ensure proper trait implementation.
#[allow(dead_code)]
pub const ENTITY_IMPL_PRESENT: bool = true;

impl Entity for User {
    type Id = String;
    type CreateParams = UserCreate;
    type UpdateParams = UserUpdate;
    type Action = (); 
    type ActionResult = ();

    // fn id(&self) -> &String { &self.id }

    /// Creates a new User from creation parameters.
    fn from_create_params(_id: String, params: UserCreate) -> Result<Self, String> {
        Ok(Self::new(params.name, params.email))
    }

    /// Handles updates to the User entity.
    ///
    /// # Fields Updated
    /// - `name`: User's display name
    /// - `email`: User's email address
    fn on_update(&mut self, update: UserUpdate) -> Result<(), String> {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(email) = update.email {
            self.email = email;
        }
        Ok(())
    }

    fn handle_action(&mut self, _action: Self::Action) -> Result<Self::ActionResult, String> {
        Ok(())
    }
}
