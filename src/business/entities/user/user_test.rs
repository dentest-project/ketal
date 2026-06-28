use super::User;
use uuid::Uuid;

#[cfg(test)]
impl User {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }
}
