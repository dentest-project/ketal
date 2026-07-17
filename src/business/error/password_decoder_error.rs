use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError, PartialEq, Eq, Serialize)]
#[error("{failure_reason}")]
pub struct PasswordDecoderError {
    failure_reason: String,
}

impl PasswordDecoderError {
    pub fn new(failure_reason: impl Into<String>) -> Self {
        Self {
            failure_reason: failure_reason.into(),
        }
    }
}

impl Error for PasswordDecoderError {
    fn code(&self) -> i64 {
        20_006
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("PasswordDecoderError")
    }
}
