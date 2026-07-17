use jsonrpc_usecase::Error;
use serde::Serialize;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct InvalidCredentialsError;

impl Display for InvalidCredentialsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid credentials")
    }
}

impl std::error::Error for InvalidCredentialsError {}

impl Error for InvalidCredentialsError {
    fn code(&self) -> i64 {
        20_004
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("InvalidCredentials")
    }
}
