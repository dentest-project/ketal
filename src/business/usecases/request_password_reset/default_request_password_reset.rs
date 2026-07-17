use super::RequestPasswordReset;
use crate::business::{
    entities::user::user_gateway::SqlxUserGateway,
    services::reset_password_code_generator::UuidResetPasswordCodeGenerator,
};
use std::sync::Arc;

impl Default for RequestPasswordReset {
    fn default() -> Self {
        Self {
            user_gateway: Arc::new(SqlxUserGateway::new()),
            reset_password_code_generator: Arc::new(UuidResetPasswordCodeGenerator::new()),
        }
    }
}
