//! Type-safe wrappers around [`ResourceClient`](crate::framework::ResourceClient).

pub mod user_client;
pub mod product_client;
pub mod order_client;
pub mod traits;

pub use user_client::*;
pub use product_client::*;
pub use order_client::*;
pub use traits::*;
