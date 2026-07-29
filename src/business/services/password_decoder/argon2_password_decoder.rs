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
        if !encoded_password.starts_with("$argon2") {
            return Ok(false);
        }

        let hash = PasswordHash::new(encoded_password)
            .map_err(|error| PasswordDecoderError::new(error.to_string()))?;

        match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(PasswordHashError::Password) => Ok(false),
            Err(error) => Err(PasswordDecoderError::new(error.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Argon2PasswordDecoder;
    use crate::business::services::PasswordDecoder;

    #[test]
    fn returns_false_for_unsupported_hash_format() {
        assert!(
            !Argon2PasswordDecoder::new()
                .matches(
                    "secret123",
                    "$2b$12$hX7K7AqwWYoHfvGgXE2vUe4v84LQdicAsWfuGnXez0P0yLbMivSSq"
                )
                .expect("unsupported hashes should not fail")
        );
    }
}
