use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProjectListItemOutput {
    pub id: String,
    pub title: String,
    pub slug: String,
}
