mod argon2_password_encoder;
#[cfg(test)]
pub mod password_encoder_double;

use crate::business::error::PasswordEncoderError;

pub use argon2_password_encoder::Argon2PasswordEncoder;
#[cfg(test)]
pub(crate) use password_encoder_double::PasswordEncoderDouble;

pub type PasswordEncoderResult<T> = Result<T, PasswordEncoderError>;

pub trait PasswordEncoder: Send + Sync {
    fn encode(&self, password: &str) -> PasswordEncoderResult<String>;
}
