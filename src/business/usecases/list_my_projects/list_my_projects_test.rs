use super::{ListMyProjects, ListMyProjectsError};
use crate::business::{
    EntityBuilder,
    entities::{
        organization::OrganizationBuilder,
        project::{
            Project, ProjectBuilder,
            project_gateway::{GatewayFuture, InMemoryProjectGateway, ProjectGateway},
        },
        user::{User, UserBuilder},
    },
    error::GatewayError,
    usecases::outputs::project::{ProjectListItemOutput, ProjectListOutput},
};
use jsonrpc_usecase::UseCaseExecutionError;
use serde_json::json;
use std::{error::Error, sync::Arc};

#[tokio::test]
async fn lists_only_the_users_projects_without_an_organization_in_alphabetical_order()
-> Result<(), Box<dyn Error>> {
    let user = UserBuilder::init().build();
    let other_user = UserBuilder::init().build();
    let organization = OrganizationBuilder::init().build();
    let first_project = ProjectBuilder::init()
        .with_title("alpha Project".to_owned())
        .build();
    let second_project = ProjectBuilder::init()
        .with_title("Bravo Project".to_owned())
        .build();
    let organization_project = ProjectBuilder::init()
        .with_title("Organization Project".to_owned())
        .with_organization(&organization)
        .build();
    let other_project = ProjectBuilder::init()
        .with_title("Other Project".to_owned())
        .build();
    let use_case = use_case(vec![
        (second_project.clone(), user.clone()),
        (organization_project, user.clone()),
        (first_project.clone(), other_user.clone()),
        (other_project, other_user),
        (first_project.clone(), user.clone()),
    ]);

    let projects = use_case.retrieve_projects(&user).await?;
    let output = ProjectListOutput::from(projects.as_slice());

    assert_eq!(
        projects,
        vec![first_project.clone(), second_project.clone()]
    );
    assert_eq!(
        serde_json::to_value(&output)?,
        json!([
            {
                "id": ProjectListItemOutput::from(&first_project).id,
                "title": "alpha Project",
                "slug": "alpha-project"
            },
            {
                "id": ProjectListItemOutput::from(&second_project).id,
                "title": "Bravo Project",
                "slug": "bravo-project"
            }
        ])
    );
    Ok(())
}

#[tokio::test]
async fn returns_an_empty_list_without_memberships_in_projects_outside_organizations()
-> Result<(), Box<dyn Error>> {
    let user = UserBuilder::init().build();
    let other_user = UserBuilder::init().build();
    let organization = OrganizationBuilder::init().build();
    let organization_project = ProjectBuilder::init()
        .with_organization(&organization)
        .build();

    for memberships in [
        vec![],
        vec![(ProjectBuilder::init().build(), other_user)],
        vec![(organization_project, user.clone())],
    ] {
        let use_case = use_case(memberships);
        let projects = use_case.retrieve_projects(&user).await?;
        let output = ProjectListOutput::from(projects.as_slice());

        assert!(projects.is_empty());
        assert_eq!(serde_json::to_value(output)?, json!([]));
    }
    Ok(())
}

#[tokio::test]
async fn returns_unexpected_error_without_a_request_context() {
    let result = use_case(vec![]).execute(()).await;

    assert!(matches!(
        result,
        Err(UseCaseExecutionError::Execution(
            ListMyProjectsError::Unexpected(_)
        ))
    ));
}

#[tokio::test]
async fn propagates_gateway_failures() {
    let use_case = ListMyProjects {
        project_gateway: Arc::new(FailingProjectGateway),
    };
    let user = UserBuilder::init().build();

    let result = use_case.retrieve_projects(&user).await;

    assert!(matches!(result, Err(ListMyProjectsError::Gateway(_))));
    if let Err(error) = result {
        assert_eq!(error.to_string(), "project lookup failed");
        assert_eq!(jsonrpc_usecase::Error::code(&error), 20_002);
        assert_eq!(jsonrpc_usecase::Error::message(&error), "GatewayError");
    }
}

fn use_case(memberships: Vec<(Project, User)>) -> ListMyProjects {
    ListMyProjects {
        project_gateway: Arc::new(InMemoryProjectGateway::new(memberships)),
    }
}

struct FailingProjectGateway;

impl ProjectGateway for FailingProjectGateway {
    fn find_by_user_without_organization<'a>(
        &'a self,
        _user: &'a User,
    ) -> GatewayFuture<'a, Vec<Project>> {
        Box::pin(async { Err(GatewayError::new("project lookup failed").into()) })
    }
}
