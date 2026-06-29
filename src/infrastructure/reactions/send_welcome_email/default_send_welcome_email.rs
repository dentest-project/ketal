use super::SendWelcomeEmail;
use crate::infrastructure::communication::email::SesSender;
use std::sync::Arc;

impl Default for SendWelcomeEmail {
    fn default() -> Self {
        Self::new(Arc::new(SesSender::new()))
    }
}
