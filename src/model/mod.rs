//! Pure data structures (DTOs) implementing the [`ActorEntity`](crate::framework::ActorEntity) trait.

pub mod user;
pub mod product;
pub mod order;

pub use user::*;
pub use product::*;
pub use order::*;
