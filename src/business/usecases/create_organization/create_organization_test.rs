use super::{CreateOrganization, CreateOrganizationError, CreateOrganizationInput};
use crate::business::{
    EntityBuilder,
    entities::{
        organization::{OrganizationBuilder, organization_gateway::InMemoryOrganizationGateway},
        organization_user::{
            OrganizationUser, organization_user_gateway::InMemoryOrganizationUserGateway,
        },
        user::UserBuilder,
    },
    usecases::outputs::organization::OrganizationDetailedOutput,
};
use jsonrpc_usecase::{UseCaseExecutionError, UseCaseInput};
use std::{error::Error, sync::Arc};

#[tokio::test]
async fn returns_unexpected_error_without_an_authenticated_context() {
    let result = use_case().execute(input("Dental Team")).await;

    assert!(matches!(
        result,
        Err(UseCaseExecutionError::Execution(
            CreateOrganizationError::Unexpected(_)
        ))
    ));
}

#[tokio::test]
async fn creates_and_saves_the_organization() -> Result<(), Box<dyn Error>> {
    let use_case = use_case();
    let input = input("  Dental Team  ").into_processed()?;

    let organization = use_case.create_organization(input).await?;
    let saved_organization = use_case
        .organization_gateway
        .find_one_by_name("Dental Team")
        .await?;
    let output = OrganizationDetailedOutput::from(&organization);

    assert_eq!(saved_organization, Some(organization));
    assert_eq!(output.name, "Dental Team");
    assert_eq!(output.slug, "dental-team");

    Ok(())
}

#[tokio::test]
async fn makes_the_authenticated_user_an_organization_admin() -> Result<(), Box<dyn Error>> {
    let organization_user_gateway = Arc::new(InMemoryOrganizationUserGateway::default());
    let use_case = CreateOrganization {
        organization_user_gateway: organization_user_gateway.clone(),
        ..use_case()
    };
    let organization = OrganizationBuilder::init()
        .with_name("Dental Team".to_owned())
        .build();
    let user = UserBuilder::init().build();

    use_case
        .make_authenticated_user_admin(&organization, &user)
        .await?;

    let mut expected_membership = OrganizationUser::new(&organization, &user);
    expected_membership.make_admin();
    assert_eq!(
        organization_user_gateway.find_one_by_organization_and_user(&organization, &user)?,
        Some(expected_membership)
    );

    Ok(())
}

#[tokio::test]
async fn rejects_duplicate_names_and_slugs() -> Result<(), Box<dyn Error>> {
    let use_case = use_case();
    use_case
        .ensure_organization_does_not_exist("Dental Team")
        .await?;
    let original = use_case.create_organization(input("Dental Team")).await?;

    for name in ["Dental Team", "dental team", "  DENTAL TEAM  "] {
        let input = input(name).into_processed()?;
        let result = use_case
            .ensure_organization_does_not_exist(&input.name)
            .await;

        assert!(matches!(
            result,
            Err(CreateOrganizationError::OrganizationAlreadyExists(_))
        ));
        if let Err(error) = result {
            assert_eq!(jsonrpc_usecase::Error::code(&error), 20_010);
            assert_eq!(
                jsonrpc_usecase::Error::message(&error),
                "OrganizationAlreadyExists"
            );
        }
    }

    let result = use_case.create_organization(input("Dental-Team")).await;
    assert!(matches!(
        result,
        Err(CreateOrganizationError::OrganizationAlreadyExists(_))
    ));

    assert_eq!(
        use_case
            .organization_gateway
            .find_one_by_name("Dental Team")
            .await?,
        Some(original)
    );

    Ok(())
}

#[tokio::test]
async fn rejects_invalid_input_before_starting_creation() -> Result<(), Box<dyn Error>> {
    let use_case = use_case();

    let result = use_case.execute(input("   ")).await;

    assert!(matches!(
        result,
        Err(UseCaseExecutionError::InvalidInput(_))
    ));
    assert!(
        use_case
            .organization_gateway
            .find_one_by_name("")
            .await?
            .is_none()
    );

    Ok(())
}

fn use_case() -> CreateOrganization {
    CreateOrganization {
        organization_gateway: Arc::new(InMemoryOrganizationGateway::default()),
        organization_user_gateway: Arc::new(InMemoryOrganizationUserGateway::default()),
    }
}

fn input(name: &str) -> CreateOrganizationInput {
    CreateOrganizationInput {
        name: name.to_owned(),
    }
}
