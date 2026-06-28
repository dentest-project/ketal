use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserListItemOutput {
    pub id: String,
    pub username: String,
}
