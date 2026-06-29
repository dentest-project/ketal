mod ses_sender;

use std::error::Error as StdError;
use std::{future::Future, pin::Pin};

use super::Mail;
pub use ses_sender::SesSender;

pub type SenderResult<T> = Result<T, SenderError>;
pub type SenderFuture<'a> = Pin<Box<dyn Future<Output = SenderResult<()>> + Send + 'a>>;

#[derive(Debug, thiserror::Error)]
pub enum SenderError {
    #[error("invalid email address: {0}")]
    Address(String),
    #[error("invalid email sender configuration: {0}")]
    Configuration(String),
    #[error("failed to build email message: {0}")]
    Message(Box<dyn StdError + Send + Sync>),
    #[error("failed to send email: {0}")]
    Transport(Box<dyn StdError + Send + Sync>),
}

impl SenderError {
    pub fn address(message: impl Into<String>) -> Self {
        Self::Address(message.into())
    }

    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    pub fn message(error: impl StdError + Send + Sync + 'static) -> Self {
        Self::Message(Box::new(error))
    }

    pub fn transport(error: impl StdError + Send + Sync + 'static) -> Self {
        Self::Transport(Box::new(error))
    }
}

pub trait Sender: Send + Sync {
    fn send<'a>(&'a self, mail: &'a dyn Mail, to: String) -> SenderFuture<'a>;
}
