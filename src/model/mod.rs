//! # Resource Models & Data Transfer Objects
//!
//! This module contains the **pure data structures** that represent the core business entities
//! (resources) in the system. These types are shared across the entire application and form
//! the public contract between actors and their clients.
//!
//! ## Architecture Principles
//!
//! ### 1. Pure Data Structures
//!
//! All types in this module are **plain data** with no business logic:
//!
//! - No methods (except constructors and simple getters)
//! - No dependencies on framework code
//! - Easily serializable (ready for JSON, databases, etc.)
//! - Can be used in any layer of the application
//!
//! ### 2. DTOs vs Entities
//!
//! We distinguish between different types of data structures:
//!
//! **DTO (Data Transfer Object)** - A design pattern for objects that carry data between
//! processes or layers. DTOs have no business logic, only data fields. In this framework,
//! we use DTOs to represent different "views" of an entity for different operations.
//!
//! **Entity** - The full resource with all fields:
//! ```rust,ignore
//! pub struct User {
//!     pub id: String,
//!     pub name: String,
//!     pub email: String,
//! }
//! ```
//!
//! **Create DTO** - Parameters for creating a new resource (no ID):
//! ```rust,ignore
//! pub struct UserCreate {
//!     pub name: String,
//!     pub email: String,
//! }
//! ```
//!
//! **Update DTO** - Partial updates (all fields optional):
//! ```rust,ignore
//! pub struct UserUpdate {
//!     pub name: Option<String>,
//!     pub email: Option<String>,
//! }
//! ```
//!
//! This pattern ensures type safety: you **can't** create a user without a name,
//! but you **can** update just the email without touching the name.
//!
//! ## Resource Models
//!
//! A **resource** is a business entity that the system manages (User, Product, Order).
//! Each resource has its own actor and follows the CRUD pattern.
//!
//! ### [`User`]
//!
//! Represents a registered user in the system. Users are referenced by orders
//! to track who placed each order.
//!
//! ### [`Product`]
//!
//! Represents a product available for purchase. Products track inventory levels
//! and support stock reservation for order fulfillment.
//!
//! ### [`Order`]
//!
//! Represents a customer order. Orders reference both a user (who placed it)
//! and a product (what was ordered), demonstrating actor coordination.
//!
//! ## Design Patterns
//!
//! ### Separation from Actor Logic
//!
//! These models are **separate** from the actor implementations:
//!
//! - **Models** (`src/model/`) - Pure data, no framework dependencies
//! - **Actors** (`src/*_actor/`) - Business logic via [`ActorEntity`](crate::framework::ActorEntity) trait
//!
//! This separation allows:
//! - Models to be used in non-actor contexts (HTTP handlers, CLI, etc.)
//! - Easy serialization without actor-specific concerns
//! - Clear boundaries between data and behavior
//!
//! ### Future: Multi-Crate Support
//!
//! This structure is designed to support splitting into multiple crates:
//!
//! ```text
//! my-resources/     # Pure resource models (this module)
//! my-actors/        # Actor implementations
//! my-framework/     # Generic framework code
//! ```
//!
//! The models have **zero dependencies** on the framework, making them
//! easy to extract into a shared library.

pub mod order;
pub mod product;
pub mod user;

pub use order::*;
pub use product::*;
pub use user::*;
