use crate::business::{
    entities::project::ProjectVisibility, usecases::outputs::path::PathListItemOutput,
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProjectListItemOutput {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub visibility: ProjectVisibility,
    pub root_path: Option<PathListItemOutput>,
}
