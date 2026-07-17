use crate::business::{
    error::PasswordDecoderError,
    services::{PasswordDecoder, PasswordDecoderResult},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::Error as PasswordHashError};

#[derive(Default)]
pub struct Argon2PasswordDecoder;

impl Argon2PasswordDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordDecoder for Argon2PasswordDecoder {
    fn matches(&self, password: &str, encoded_password: &str) -> PasswordDecoderResult<bool> {
        let hash = PasswordHash::new(encoded_password)
            .map_err(|error| PasswordDecoderError::new(error.to_string()))?;

        match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(PasswordHashError::Password) => Ok(false),
            Err(error) => Err(PasswordDecoderError::new(error.to_string())),
        }
    }
}
