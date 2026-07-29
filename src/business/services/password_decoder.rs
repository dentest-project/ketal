mod argon2_password_decoder;
mod bcrypt_password_decoder;
mod multiple_password_decoder;
#[cfg(test)]
pub mod password_decoder_double;

use crate::business::error::PasswordDecoderError;

pub use argon2_password_decoder::Argon2PasswordDecoder;
pub use bcrypt_password_decoder::BcryptPasswordDecoder;
pub use multiple_password_decoder::MultiplePasswordDecoder;
#[cfg(test)]
pub(crate) use password_decoder_double::PasswordDecoderDouble;

pub type PasswordDecoderResult<T> = Result<T, PasswordDecoderError>;

pub trait PasswordDecoder: Send + Sync {
    fn matches(&self, password: &str, encoded_password: &str) -> PasswordDecoderResult<bool>;
}
