use super::ProjectListItemOutput;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ProjectListOutput(pub Vec<ProjectListItemOutput>);
