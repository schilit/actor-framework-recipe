//! Pure data structures (DTOs) implementing the [`Entity`](crate::framework::Entity) trait.

pub mod user;
pub mod product;
pub mod order;

pub use user::*;
pub use product::*;
pub use order::*;
