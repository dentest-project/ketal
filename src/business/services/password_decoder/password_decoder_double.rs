#![cfg(test)]

use super::{PasswordDecoder, PasswordDecoderResult};
use std::sync::Mutex;

pub(crate) struct PasswordDecoderDouble {
    password_matches: bool,
    received_matches: Mutex<Vec<(String, String)>>,
}

impl PasswordDecoderDouble {
    pub(crate) fn matching(password_matches: bool) -> Self {
        Self {
            password_matches,
            received_matches: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn received_matches(&self) -> Vec<(String, String)> {
        self.received_matches
            .lock()
            .expect("received matches mutex should not be poisoned")
            .clone()
    }
}

impl PasswordDecoder for PasswordDecoderDouble {
    fn matches(&self, password: &str, encoded_password: &str) -> PasswordDecoderResult<bool> {
        self.received_matches
            .lock()
            .expect("received matches mutex should not be poisoned")
            .push((password.to_owned(), encoded_password.to_owned()));

        Ok(self.password_matches)
    }
}
