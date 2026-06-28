use super::User;
use crate::business::usecases::outputs::user::{UserDetailedOutput, UserListItemOutput};

impl Into<UserDetailedOutput> for &User {
    fn into(self) -> UserDetailedOutput {
        UserDetailedOutput {
            id: self.id.to_string(),
            username: self.username.clone(),
            email: self.email.clone(),
        }
    }
}

impl Into<UserListItemOutput> for &User {
    fn into(self) -> UserListItemOutput {
        UserListItemOutput {
            id: self.id.to_string(),
            username: self.username.clone(),
        }
    }
}
