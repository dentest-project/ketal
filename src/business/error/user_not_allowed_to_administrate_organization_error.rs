use jsonrpc_usecase::Error;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("user is not allowed to administrate the organization")]
pub struct UserNotAllowedToAdministrateOrganizationError;

impl Error for UserNotAllowedToAdministrateOrganizationError {
    fn code(&self) -> i64 {
        20_013
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("UserNotAllowedToAdministrateOrganization")
    }
}
