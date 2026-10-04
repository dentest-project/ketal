pub mod sqlx_path_gateway;

use crate::business::error::{GatewayError, UnexpectedError};

#[derive(Debug, thiserror::Error)]
pub enum PathGatewayError {
    #[error(transparent)]
    Gateway(#[from] GatewayError),
    #[error(transparent)]
    Unexpected(#[from] UnexpectedError),
}

impl From<sqlx::Error> for PathGatewayError {
    fn from(error: sqlx::Error) -> Self {
        Self::Gateway(error.into())
    }
}
