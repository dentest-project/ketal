use super::Project;
use crate::business::usecases::outputs::{
    path::PathListItemOutput,
    project::{ProjectListItemOutput, ProjectListOutput},
};

impl From<&Project> for ProjectListItemOutput {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id.to_string(),
            title: project.title.clone(),
            slug: project.slug.clone(),
            visibility: project.visibility,
            root_path: project.root_path.as_ref().map(PathListItemOutput::from),
        }
    }
}

impl From<&[Project]> for ProjectListOutput {
    fn from(projects: &[Project]) -> Self {
        Self(projects.iter().map(ProjectListItemOutput::from).collect())
    }
}
