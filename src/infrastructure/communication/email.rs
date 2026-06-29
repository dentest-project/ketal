mod register_mail;
pub mod sender;

pub use Mail as Email;
pub use register_mail::RegisterMail;
pub use sender::{Sender, SenderError, SenderFuture, SenderResult, SesSender};

pub trait Mail: Send + Sync {
    fn subject(&self) -> String;

    fn plain(&self) -> String;
}
