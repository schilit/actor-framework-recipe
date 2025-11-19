//! Pure data structures (DTOs) implementing the [`Entity`](crate::actor_framework::Entity) trait.

pub mod user;
pub mod product;
pub mod order;

pub use user::*;
pub use product::*;
pub use order::*;
