pub mod project_gateway;
pub mod project_presenter;

#[cfg(test)]
mod project_test_stub;

#[cfg(test)]
pub use project_test_stub::ProjectBuilder;

use super::{organization::Organization, path::Path};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectVisibility {
    Public,
    Internal,
    Private,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    id: Uuid,
    title: String,
    slug: String,
    visibility: ProjectVisibility,
    root_path: Option<Path>,
    organization: Option<Organization>,
}
