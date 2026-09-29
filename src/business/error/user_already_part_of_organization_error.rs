use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("user already part of organization")]
pub struct UserAlreadyPartOfOrganizationError;

impl Error for UserAlreadyPartOfOrganizationError {
    fn code(&self) -> i64 {
        20_012
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("UserAlreadyPartOfOrganization")
    }
}
