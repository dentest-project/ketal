use super::SendResetPasswordEmail;
use crate::infrastructure::communication::email::SesSender;
use std::sync::Arc;

impl Default for SendResetPasswordEmail {
    fn default() -> Self {
        Self::new(Arc::new(SesSender::new()))
    }
}
