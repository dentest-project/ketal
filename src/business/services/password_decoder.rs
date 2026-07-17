mod argon2_password_decoder;
#[cfg(test)]
pub mod password_decoder_double;

use crate::business::error::PasswordDecoderError;

pub use argon2_password_decoder::Argon2PasswordDecoder;
#[cfg(test)]
pub(crate) use password_decoder_double::PasswordDecoderDouble;

pub type PasswordDecoderResult<T> = Result<T, PasswordDecoderError>;

pub trait PasswordDecoder: Send + Sync {
    fn matches(&self, password: &str, encoded_password: &str) -> PasswordDecoderResult<bool>;
}
