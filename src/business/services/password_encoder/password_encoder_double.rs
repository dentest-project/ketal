#![cfg(test)]

use super::{PasswordEncoder, PasswordEncoderResult};
use std::sync::Mutex;

pub(crate) struct PasswordEncoderDouble {
    encoded_password: String,
    received_password: Mutex<Option<String>>,
}

impl PasswordEncoderDouble {
    pub(crate) fn encoding(encoded_password: impl Into<String>) -> Self {
        Self {
            encoded_password: encoded_password.into(),
            received_password: Mutex::new(None),
        }
    }

    pub(crate) fn received_password(&self) -> Option<String> {
        self.received_password
            .lock()
            .expect("received password mutex should not be poisoned")
            .clone()
    }
}

impl PasswordEncoder for PasswordEncoderDouble {
    fn encode(&self, password: &str) -> PasswordEncoderResult<String> {
        self.received_password
            .lock()
            .expect("received password mutex should not be poisoned")
            .replace(password.to_owned());

        Ok(self.encoded_password.clone())
    }
}
