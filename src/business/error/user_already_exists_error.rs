use jsonrpc_usecase::Error;
use serde::Serialize;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct UserAlreadyExistsError;

impl Display for UserAlreadyExistsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("user already exists")
    }
}

impl std::error::Error for UserAlreadyExistsError {}

impl Error for UserAlreadyExistsError {
    fn code(&self) -> i64 {
        20_001
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("UserAlreadyExists")
    }
}
