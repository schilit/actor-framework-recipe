//! Entity trait implementation for the User resource type.
//!
//! This module contains the [`ActorEntity`] trait implementation
//! that enables [`User`] to be managed by the generic [`crate::framework::ResourceActor`].
//!
//! See the trait implementation on [`User`] for method documentation.

use crate::framework::ActorEntity;
use crate::model::{User, UserCreate, UserUpdate};
use crate::user_actor::UserError;
use async_trait::async_trait;

#[derive(Debug)]
pub enum UserAction {
    // No custom actions for now
}

/// Marker constant to ensure module documentation is rendered.
#[doc(hidden)]
/// Marker constant to verify ActorEntity trait implementation exists at compile time.
/// This is used by the framework to ensure proper trait implementation.
#[allow(dead_code)]
pub const ENTITY_IMPL_PRESENT: bool = true;

#[async_trait]
impl ActorEntity for User {
    type Id = String;
    type Create = UserCreate;
    type Update = UserUpdate;
    type Action = UserAction;
    type ActionResult = ();
    type Context = ();
    type Error = UserError;

    // fn id(&self) -> &String { &self.id }

    /// Creates a new User from creation parameters.
    fn from_create_params(id: String, params: UserCreate) -> Result<Self, Self::Error> {
        Ok(User {
            id,
            name: params.name,
            email: params.email,
        })
    }

    /// Handles updates to the User entity.
    ///
    /// # Fields Updated
    /// - `name`: User's display name
    /// - `email`: User's email address
    async fn on_update(
        &mut self,
        update: UserUpdate,
        _ctx: &Self::Context,
    ) -> Result<(), Self::Error> {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(email) = update.email {
            self.email = email;
        }
        Ok(())
    }

    async fn handle_action(
        &mut self,
        _action: UserAction,
        _ctx: &Self::Context,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
