pub mod in_memory_user_gateway;
pub mod sqlx_user_gateway;

pub use in_memory_user_gateway::InMemoryUserGateway;
pub use sqlx_user_gateway::SqlxUserGateway;

use super::User;
use crate::business::error::GatewayError;
use std::{future::Future, pin::Pin};

pub type GatewayResult<T> = Result<T, GatewayError>;
pub type GatewayFuture<'a, T> = Pin<Box<dyn Future<Output = GatewayResult<T>> + Send + 'a>>;

pub trait UserGateway: Send + Sync {
    fn save<'a>(&'a self, user: &'a User) -> GatewayFuture<'a, ()>;
    fn find_one_by_email_or_username<'a>(
        &'a self,
        email: &'a str,
        username: &'a str,
    ) -> GatewayFuture<'a, Option<User>>;
}
