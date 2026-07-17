use super::SendResetPasswordRequestEmail;
use crate::{
    business::entities::user::user_gateway::SqlxUserGateway,
    infrastructure::{communication::email::SesSender, config::allowed_origin},
};
use std::sync::Arc;

impl Default for SendResetPasswordRequestEmail {
    fn default() -> Self {
        Self::new(
            Arc::new(SqlxUserGateway::new()),
            Arc::new(SesSender::new()),
            allowed_origin(),
        )
    }
}
