use super::Login;
use crate::business::{
    entities::user::user_gateway::SqlxUserGateway,
    services::{password_decoder::Argon2PasswordDecoder, token_generator::JwtTokenGenerator},
};
use std::sync::Arc;

impl Default for Login {
    fn default() -> Self {
        Self {
            user_gateway: Arc::new(SqlxUserGateway::new()),
            password_decoder: Arc::new(Argon2PasswordDecoder::new()),
            token_generator: Arc::new(JwtTokenGenerator::new()),
        }
    }
}
