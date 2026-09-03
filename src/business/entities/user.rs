pub mod user_builder;
pub mod user_gateway;
pub mod user_presenter;

#[cfg(test)]
mod user_test;

pub use user_builder::UserBuilder;

use std::time::{Duration, SystemTime};
use uuid::Uuid;

const RESET_PASSWORD_REQUEST_COOLDOWN: Duration = Duration::from_secs(2 * 60 * 60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
    password: String,
    last_reset_password_request: Option<SystemTime>,
    reset_password_code: Option<String>,
}

impl User {
    pub(crate) fn email(&self) -> &str {
        &self.email
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }

    pub(crate) fn reset_password_code(&self) -> Option<&str> {
        self.reset_password_code.as_deref()
    }

    pub(crate) fn is_reset_password_request_cooldown_expired(&self) -> bool {
        self.reset_password_request_remaining_cooldown_minutes()
            .is_none()
    }

    pub(crate) fn reset_password_request_remaining_cooldown_minutes(&self) -> Option<u64> {
        let last_reset_password_request = self.last_reset_password_request?;
        let elapsed = SystemTime::now()
            .duration_since(last_reset_password_request)
            .unwrap_or(Duration::ZERO);

        if elapsed >= RESET_PASSWORD_REQUEST_COOLDOWN {
            return None;
        }

        let remaining_cooldown = RESET_PASSWORD_REQUEST_COOLDOWN - elapsed;
        let seconds =
            remaining_cooldown.as_secs() + u64::from(remaining_cooldown.subsec_nanos() > 0);

        Some(seconds.div_ceil(60).max(1))
    }

    pub(crate) fn define_reset_password_code(&mut self, code: String) {
        self.reset_password_code = Some(code);
        self.last_reset_password_request = Some(SystemTime::now());
    }

    pub(crate) fn reset_password(&mut self, password: String) {
        self.password = password;
        self.reset_password_code = None;
    }

    pub(crate) fn update_personal_information(
        &mut self,
        username: String,
        email: String,
        password: Option<String>,
    ) {
        self.username = username;
        self.email = email;

        if let Some(password) = password {
            self.password = password;
        }
    }
}
