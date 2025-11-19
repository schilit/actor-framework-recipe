use tokio::sync::oneshot;
use crate::domain::{User, Product, Order};
use crate::error::{UserError, ProductError, OrderError};

/// Generic type aliases for service communication
pub type ServiceResult<T, E> = std::result::Result<T, E>;
pub type ServiceResponse<T, E> = oneshot::Sender<ServiceResult<T, E>>;

/// Typed message enums for actor communication. Each variant includes parameters
/// and a oneshot channel for responses.

#[derive(Debug)]
pub enum UserRequest {
    GetUser {
        id: String,
        respond_to: ServiceResponse<Option<User>, UserError>,
    },
    CreateUser {
        user: User,
        respond_to: ServiceResponse<String, UserError>,
    },
    UpdateUser {
        id: String,
        user: User,
        respond_to: ServiceResponse<(), UserError>,
    },
    ListUsers {
        respond_to: ServiceResponse<Vec<User>, UserError>,
    },
    Shutdown,
    #[cfg(test)]
    GetUserCount {
        respond_to: ServiceResponse<usize, UserError>,
    },
}

#[derive(Debug)]
pub enum ProductRequest {
    GetProduct {
        id: String,
        respond_to: ServiceResponse<Option<Product>, ProductError>,
    },
    CheckStock {
        id: String,
        respond_to: ServiceResponse<u32, ProductError>,
    },
    ReserveStock {
        id: String,
        quantity: u32,
        respond_to: ServiceResponse<(), ProductError>,
    },
    Shutdown,
}

#[derive(Debug)]
pub enum OrderRequest {
    CreateOrder {
        order: Order,
        respond_to: ServiceResponse<String, OrderError>,
    },
    GetOrder {
        id: String,
        respond_to: ServiceResponse<Option<Order>, OrderError>,
    },
    Shutdown,
}
