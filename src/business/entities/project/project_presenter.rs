use super::Project;
use crate::business::usecases::outputs::project::{ProjectListItemOutput, ProjectListOutput};

impl From<&Project> for ProjectListItemOutput {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id.to_string(),
            title: project.title.clone(),
            slug: project.slug.clone(),
        }
    }
}

impl From<&[Project]> for ProjectListOutput {
    fn from(projects: &[Project]) -> Self {
        Self(projects.iter().map(ProjectListItemOutput::from).collect())
    }
}
