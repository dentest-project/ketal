use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("organization not found")]
pub struct OrganizationNotFoundError;

impl Error for OrganizationNotFoundError {
    fn code(&self) -> i64 {
        20_011
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("OrganizationNotFound")
    }
}
