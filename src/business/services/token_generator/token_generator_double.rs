#![cfg(test)]

use super::{TokenGenerator, TokenGeneratorResult};
use crate::business::entities::user::User;
use std::sync::Mutex;

pub(crate) struct TokenGeneratorDouble {
    token: String,
    received_usernames: Mutex<Vec<String>>,
}

impl TokenGeneratorDouble {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            received_usernames: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn received_usernames(&self) -> Vec<String> {
        self.received_usernames
            .lock()
            .expect("received usernames mutex should not be poisoned")
            .clone()
    }
}

impl TokenGenerator for TokenGeneratorDouble {
    fn generate_for(&self, user: &User) -> TokenGeneratorResult<String> {
        self.received_usernames
            .lock()
            .expect("received usernames mutex should not be poisoned")
            .push(user.username().to_owned());

        Ok(self.token.clone())
    }
}
