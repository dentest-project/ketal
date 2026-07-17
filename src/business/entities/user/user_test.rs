use super::User;
use uuid::Uuid;

impl User {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}
