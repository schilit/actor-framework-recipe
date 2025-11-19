//! Type-safe wrappers around [`ResourceClient`](crate::actor_framework::ResourceClient).

#[macro_use]
mod macros;
pub mod user_client;
pub mod product_client;
pub mod order_client;

pub use user_client::*;
pub use product_client::*;
pub use order_client::*;
