use crate::business::{
    error::PasswordEncoderError,
    services::{PasswordEncoder, PasswordEncoderResult},
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand_core::OsRng;

#[derive(Default)]
pub struct Argon2PasswordEncoder;

impl Argon2PasswordEncoder {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordEncoder for Argon2PasswordEncoder {
    fn encode(&self, password: &str) -> PasswordEncoderResult<String> {
        let salt = SaltString::generate(&mut OsRng);

        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|error| PasswordEncoderError::new(error.to_string()))
    }
}
