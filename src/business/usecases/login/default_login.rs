use super::Login;
use crate::business::{
    entities::user::user_gateway::SqlxUserGateway,
    services::{
        password_decoder::{Argon2PasswordDecoder, BcryptPasswordDecoder, MultiplePasswordDecoder},
        token_generator::JwtTokenGenerator,
    },
};
use std::sync::Arc;

impl Default for Login {
    fn default() -> Self {
        Self {
            user_gateway: Arc::new(SqlxUserGateway::new()),
            password_decoder: Arc::new(MultiplePasswordDecoder::new(vec![
                Arc::new(Argon2PasswordDecoder::new()),
                Arc::new(BcryptPasswordDecoder::new()),
            ])),
            token_generator: Arc::new(JwtTokenGenerator::new()),
        }
    }
}
