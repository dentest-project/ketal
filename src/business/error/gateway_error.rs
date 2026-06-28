use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError, PartialEq, Eq, Serialize)]
#[error("{failure_reason}")]
pub struct GatewayError {
    failure_reason: String,
}

impl GatewayError {
    pub fn new(failure_reason: impl Into<String>) -> Self {
        Self {
            failure_reason: failure_reason.into(),
        }
    }
}

impl Error for GatewayError {
    fn code(&self) -> i64 {
        20_002
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("GatewayError")
    }
}

impl From<sqlx::Error> for GatewayError {
    fn from(error: sqlx::Error) -> Self {
        Self::new(error.to_string())
    }
}
