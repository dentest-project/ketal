#[cfg(test)]
pub mod in_memory_organization_gateway;
pub mod sqlx_organization_gateway;

#[cfg(test)]
pub use in_memory_organization_gateway::InMemoryOrganizationGateway;
pub use sqlx_organization_gateway::SqlxOrganizationGateway;

use super::Organization;
use crate::business::error::{GatewayError, OrganizationAlreadyExistsError, UnexpectedError};
use std::{future::Future, pin::Pin};

#[derive(Debug, thiserror::Error)]
pub enum OrganizationGatewayError {
    #[error(transparent)]
    OrganizationAlreadyExists(#[from] OrganizationAlreadyExistsError),
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

impl From<sqlx::Error> for OrganizationGatewayError {
    fn from(error: sqlx::Error) -> Self {
        Self::Gateway(error.into())
    }
}

pub type GatewayResult<T> = Result<T, OrganizationGatewayError>;
pub type GatewayFuture<'a, T> = Pin<Box<dyn Future<Output = GatewayResult<T>> + Send + 'a>>;

pub trait OrganizationGateway: Send + Sync {
    fn find_one_by_name<'a>(&'a self, name: &'a str) -> GatewayFuture<'a, Option<Organization>>;
    fn save<'a>(&'a self, organization: &'a Organization) -> GatewayFuture<'a, ()>;
}
