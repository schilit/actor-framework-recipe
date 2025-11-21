//! Entity trait implementation for [`User`](crate::domain::User).
//!
//! This module contains the [`Entity`](crate::actor_framework::Entity) trait implementation
//! that enables `User` to be managed by the generic [`ResourceActor`](crate::actor_framework::ResourceActor).
//!
//! See the [trait implementation on `User`](crate::domain::User#impl-Entity-for-User) for method documentation.

use crate::actor_framework::Entity;
use crate::domain::{User, UserCreate, UserUpdate};

impl Entity for User {
    type Id = String;
    type CreateParams = UserCreate;
    type UpdateParams = UserUpdate;
    type Action = (); 
    type ActionResult = ();

    // fn id(&self) -> &String { &self.id }

    /// Creates a new User from creation parameters.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the user
    /// * `params` - User creation parameters containing name and email
    fn from_create_params(id: String, params: UserCreate) -> Result<Self, String> {
        Ok(Self {
            id,
            name: params.name,
            email: params.email,
        })
    }

    /// Updates the user's profile information.
    ///
    /// # Arguments
    /// * `patch` - Contains optional updates for name and/or email
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

    /// Handles user-specific actions.
    ///
    /// Currently, no custom actions are defined for users.
    fn handle_action(&mut self, _action: ()) -> Result<(), String> {
        Ok(())
    }
}
