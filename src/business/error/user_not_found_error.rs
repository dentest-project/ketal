use jsonrpc_usecase::Error;
use serde::Serialize;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct UserNotFoundError;

impl Display for UserNotFoundError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("user not found")
    }
}

impl std::error::Error for UserNotFoundError {}

impl Error for UserNotFoundError {
    fn code(&self) -> i64 {
        20_007
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("UserNotFound")
    }
}
