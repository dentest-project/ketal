use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("organization already exists")]
pub struct OrganizationAlreadyExistsError;

impl Error for OrganizationAlreadyExistsError {
    fn code(&self) -> i64 {
        20_010
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("OrganizationAlreadyExists")
    }
}
