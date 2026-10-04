mod default_list_my_projects;

#[cfg(test)]
mod list_my_projects_test;

use crate::{
    business::{
        RequestContext,
        entities::{
            project::{
                Project,
                project_gateway::{ProjectGateway, ProjectGatewayError},
            },
            user::User,
        },
        error::{GatewayError, UnexpectedError, use_case_error},
        usecases::outputs::project::ProjectListOutput,
    },
    infrastructure::guards::AuthenticatedUserGuard,
};
use jsonrpc_usecase::{UseCase, current_context};
use macros::Transactional;
use std::sync::Arc;

pub struct ListMyProjects {
    project_gateway: Arc<dyn ProjectGateway>,
}

use_case_error! {
    pub enum ListMyProjectsError {
        Gateway(GatewayError),
        Unexpected(UnexpectedError),
    }
}

impl From<ProjectGatewayError> for ListMyProjectsError {
    fn from(error: ProjectGatewayError) -> Self {
        match error {
            ProjectGatewayError::Gateway(error) => error.into(),
            ProjectGatewayError::Unexpected(error) => error.into(),
        }
    }
}

#[Transactional]
#[UseCase(guards = [AuthenticatedUserGuard])]
impl ListMyProjects {
    async fn execute(&self, _input: ()) -> Result<ProjectListOutput, ListMyProjectsError> {
        let context = current_context::<RequestContext>().ok_or(UnexpectedError)?;
        let authenticated_user = context.user.as_ref().ok_or(UnexpectedError)?;

        let projects = self.retrieve_projects(authenticated_user).await?;

        Ok(projects.as_slice().into())
    }

    async fn retrieve_projects(
        &self,
        authenticated_user: &User,
    ) -> Result<Vec<Project>, ListMyProjectsError> {
        Ok(self
            .project_gateway
            .find_by_user_without_organization(authenticated_user)
            .await?)
    }
}
