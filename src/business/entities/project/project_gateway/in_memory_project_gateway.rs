use crate::business::entities::{
    project::{
        Project,
        project_gateway::{GatewayFuture, ProjectGateway},
    },
    user::User,
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryProjectGateway {
    projects: HashMap<(Uuid, Uuid), Project>,
}

impl InMemoryProjectGateway {
    pub fn new(memberships: Vec<(Project, User)>) -> Self {
        Self {
            projects: memberships
                .into_iter()
                .map(|(project, user)| ((project.id, user.id), project))
                .collect(),
        }
    }
}

impl ProjectGateway for InMemoryProjectGateway {
    fn find_by_user_without_organization<'a>(
        &'a self,
        user: &'a User,
    ) -> GatewayFuture<'a, Vec<Project>> {
        Box::pin(async move {
            let mut projects: Vec<_> = self
                .projects
                .iter()
                .filter(|((_, user_id), project)| {
                    *user_id == user.id && project.organization.is_none()
                })
                .map(|(_, project)| project.clone())
                .collect();
            projects.sort();

            Ok(projects)
        })
    }
}
