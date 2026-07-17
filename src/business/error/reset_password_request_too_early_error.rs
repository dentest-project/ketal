use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError, PartialEq, Eq, Serialize)]
#[error("reset password request too early")]
pub struct ResetPasswordRequestTooEarlyError {
    remaining_minutes: u64,
}

impl ResetPasswordRequestTooEarlyError {
    pub fn new(remaining_minutes: u64) -> Self {
        Self { remaining_minutes }
    }

    pub fn remaining_minutes(&self) -> u64 {
        self.remaining_minutes
    }
}

impl Error for ResetPasswordRequestTooEarlyError {
    fn code(&self) -> i64 {
        20_008
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("ResetPasswordRequestTooEarly")
    }
}
