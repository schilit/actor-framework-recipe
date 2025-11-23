//! Pure data structures (DTOs) implementing the [`ActorEntity`](crate::framework::ActorEntity) trait.

pub mod order;
pub mod product;
pub mod user;

pub use order::*;
pub use product::*;
pub use user::*;
