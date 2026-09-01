use jsonrpc_usecase::Error;
use serde::Serialize;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct UnexpectedError;

impl Display for UnexpectedError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("unexpected error")
    }
}

impl std::error::Error for UnexpectedError {}

impl Error for UnexpectedError {
    fn code(&self) -> i64 {
        20_009
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Borrowed("UnexpectedError")
    }
}
