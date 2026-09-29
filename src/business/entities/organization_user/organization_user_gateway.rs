#[cfg(test)]
pub mod in_memory_organization_user_gateway;
pub mod sqlx_organization_user_gateway;

#[cfg(test)]
pub use in_memory_organization_user_gateway::InMemoryOrganizationUserGateway;
pub use sqlx_organization_user_gateway::SqlxOrganizationUserGateway;

use super::OrganizationUser;
use crate::business::{
    entities::{organization::Organization, user::User},
    error::{GatewayError, UnexpectedError, UserAlreadyPartOfOrganizationError},
};
use std::{future::Future, pin::Pin};

#[derive(Debug, thiserror::Error)]
pub enum OrganizationUserGatewayError {
    #[error(transparent)]
    UserAlreadyPartOfOrganization(#[from] UserAlreadyPartOfOrganizationError),
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

impl From<sqlx::Error> for OrganizationUserGatewayError {
    fn from(error: sqlx::Error) -> Self {
        Self::Gateway(error.into())
    }
}

pub type GatewayFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, OrganizationUserGatewayError>> + Send + 'a>>;

pub trait OrganizationUserGateway: Send + Sync {
    fn find_one_by_organization_and_user<'a>(
        &'a self,
        organization: &'a Organization,
        user: &'a User,
    ) -> GatewayFuture<'a, Option<OrganizationUser>>;
    fn save<'a>(&'a self, organization_user: &'a OrganizationUser) -> GatewayFuture<'a, ()>;
}
