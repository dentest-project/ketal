use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserDetailedOutput {
    pub id: String,
    pub username: String,
    pub email: String,
}
