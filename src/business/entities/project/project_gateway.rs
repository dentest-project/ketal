#[cfg(test)]
pub mod in_memory_project_gateway;
pub mod sqlx_project_gateway;

#[cfg(test)]
pub use in_memory_project_gateway::InMemoryProjectGateway;
pub use sqlx_project_gateway::SqlxProjectGateway;

use super::Project;
use crate::business::{
    entities::{path::path_gateway::PathGatewayError, user::User},
    error::{GatewayError, UnexpectedError},
};
use std::{future::Future, pin::Pin};

#[derive(Debug, thiserror::Error)]
pub enum ProjectGatewayError {
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

impl From<sqlx::Error> for ProjectGatewayError {
    fn from(error: sqlx::Error) -> Self {
        Self::Gateway(error.into())
    }
}

impl From<PathGatewayError> for ProjectGatewayError {
    fn from(error: PathGatewayError) -> Self {
        match error {
            PathGatewayError::Gateway(error) => error.into(),
            PathGatewayError::Unexpected(error) => error.into(),
        }
    }
}

pub type GatewayFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProjectGatewayError>> + Send + 'a>>;

pub trait ProjectGateway: Send + Sync {
    fn find_by_user_without_organization<'a>(
        &'a self,
        user: &'a User,
    ) -> GatewayFuture<'a, Vec<Project>>;
}
