pub mod user_builder;
pub mod user_gateway;
pub mod user_presenter;

#[cfg(test)]
mod user_test;

pub use user_builder::UserBuilder;

use std::time::SystemTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
    password: String,
    last_reset_password_request: Option<SystemTime>,
    reset_password_code: Option<String>,
}
