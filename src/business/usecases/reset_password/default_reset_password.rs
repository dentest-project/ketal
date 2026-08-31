use super::ResetPassword;
use crate::business::{
    entities::user::user_gateway::SqlxUserGateway,
    services::password_encoder::Argon2PasswordEncoder,
};
use std::sync::Arc;

impl Default for ResetPassword {
    fn default() -> Self {
        Self {
            user_gateway: Arc::new(SqlxUserGateway::new()),
            password_encoder: Arc::new(Argon2PasswordEncoder::new()),
        }
    }
}
