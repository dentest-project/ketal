use super::User;
use crate::business::EntityBuilder;
use uuid::Uuid;

pub struct UserBuilder {
    entity: User,
}

impl UserBuilder {
    pub fn with_username(mut self, username: String) -> Self {
        self.entity.username = username;
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.entity.email = email;
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.entity.password = password;
        self
    }
}

impl EntityBuilder<User> for UserBuilder {
    fn init() -> Self {
        Self {
            entity: User {
                id: Uuid::new_v4(),
                username: String::new(),
                email: String::new(),
                password: String::new(),
                last_reset_password_request: None,
                reset_password_code: None,
            },
        }
    }

    fn build(self) -> User {
        self.entity
    }
}
