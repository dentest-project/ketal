mod jwt_token_generator;
#[cfg(test)]
pub mod token_generator_double;

use crate::business::{entities::user::User, error::TokenGeneratorError};

pub use jwt_token_generator::JwtTokenGenerator;
#[cfg(test)]
pub(crate) use token_generator_double::TokenGeneratorDouble;

pub type TokenGeneratorResult<T> = Result<T, TokenGeneratorError>;

pub trait TokenGenerator: Send + Sync {
    fn generate_for(&self, user: &User) -> TokenGeneratorResult<String>;
}
