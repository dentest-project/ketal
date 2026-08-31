mod register_mail;
mod reset_password_mail;
mod reset_password_request_mail;
pub mod sender;

pub use Mail as Email;
pub use register_mail::RegisterMail;
pub use reset_password_mail::ResetPasswordMail;
pub use reset_password_request_mail::ResetPasswordRequestMail;
pub use sender::{Sender, SenderError, SenderFuture, SenderResult, SesSender};

pub trait Mail: Send + Sync {
    fn subject(&self) -> String;

    fn plain(&self) -> String;
}
