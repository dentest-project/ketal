use super::{ListMyOrganizations, ListMyOrganizationsError};
use crate::business::{
    EntityBuilder,
    entities::{
        organization::OrganizationBuilder,
        organization_user::{
            OrganizationUser,
            organization_user_gateway::{
                InMemoryOrganizationUserGateway, OrganizationUserGatewayError,
            },
        },
        user::UserBuilder,
    },
    error::{GatewayError, UnexpectedError, UserAlreadyPartOfOrganizationError},
    usecases::outputs::organization::{OrganizationListItemOutput, OrganizationListOutput},
};
use jsonrpc_usecase::UseCaseExecutionError;
use serde_json::json;
use std::{error::Error, sync::Arc};

#[tokio::test]
async fn lists_only_the_users_organizations_regardless_of_permissions() -> Result<(), Box<dyn Error>>
{
    let use_case = use_case();
    let user = UserBuilder::init().build();
    let other_user = UserBuilder::init().build();
    let member_organization = OrganizationBuilder::init()
        .with_name("Dental Team".to_owned())
        .build();
    let admin_organization = OrganizationBuilder::init()
        .with_name("Admin Team".to_owned())
        .build();
    let other_organization = OrganizationBuilder::init()
        .with_name("Other Team".to_owned())
        .build();

    let mut admin_membership = OrganizationUser::new(&admin_organization, &user);
    admin_membership.make_admin();
    for membership in [
        OrganizationUser::new(&member_organization, &user),
        admin_membership,
        OrganizationUser::new(&member_organization, &other_user),
        OrganizationUser::new(&other_organization, &other_user),
    ] {
        use_case.organization_user_gateway.save(&membership).await?;
    }

    let organizations = use_case.retrieve_organizations(&user).await?;
    let mut output = OrganizationListOutput::from(organizations.as_slice());
    output.0.sort_by(|left, right| left.name.cmp(&right.name));

    assert_eq!(
        output,
        OrganizationListOutput(vec![
            OrganizationListItemOutput::from(&admin_organization),
            OrganizationListItemOutput::from(&member_organization),
        ])
    );
    assert_eq!(
        serde_json::to_value(&output)?,
        json!([
            {
                "id": OrganizationListItemOutput::from(&admin_organization).id,
                "name": "Admin Team",
                "slug": "admin-team"
            },
            {
                "id": OrganizationListItemOutput::from(&member_organization).id,
                "name": "Dental Team",
                "slug": "dental-team"
            }
        ])
    );
    Ok(())
}

#[tokio::test]
async fn returns_an_empty_list_when_the_user_has_no_memberships() -> Result<(), Box<dyn Error>> {
    let use_case = use_case();
    let user = UserBuilder::init().build();
    let other_user = UserBuilder::init().build();
    let organization = OrganizationBuilder::init().build();
    use_case
        .organization_user_gateway
        .save(&OrganizationUser::new(&organization, &other_user))
        .await?;

    let organizations = use_case.retrieve_organizations(&user).await?;

    assert_eq!(
        serde_json::to_value(OrganizationListOutput::from(organizations.as_slice()))?,
        json!([])
    );
    Ok(())
}

#[tokio::test]
async fn returns_unexpected_error_without_a_request_context() {
    let result = use_case().execute(()).await;

    assert!(matches!(
        result,
        Err(UseCaseExecutionError::Execution(
            ListMyOrganizationsError::Unexpected(_)
        ))
    ));
}

#[test]
fn preserves_gateway_failures_and_maps_invalid_read_errors_to_unexpected() {
    let error = ListMyOrganizationsError::from(OrganizationUserGatewayError::Gateway(
        GatewayError::new("membership lookup failed"),
    ));
    assert!(matches!(error, ListMyOrganizationsError::Gateway(_)));
    assert_eq!(error.to_string(), "membership lookup failed");
    assert_eq!(jsonrpc_usecase::Error::code(&error), 20_002);
    assert_eq!(jsonrpc_usecase::Error::message(&error), "GatewayError");

    for error in [
        OrganizationUserGatewayError::Unexpected(UnexpectedError),
        OrganizationUserGatewayError::UserAlreadyPartOfOrganization(
            UserAlreadyPartOfOrganizationError,
        ),
    ] {
        let error = ListMyOrganizationsError::from(error);
        assert!(matches!(error, ListMyOrganizationsError::Unexpected(_)));
        assert_eq!(jsonrpc_usecase::Error::code(&error), 20_009);
        assert_eq!(jsonrpc_usecase::Error::message(&error), "UnexpectedError");
    }
}

fn use_case() -> ListMyOrganizations {
    ListMyOrganizations {
        organization_user_gateway: Arc::new(InMemoryOrganizationUserGateway::default()),
    }
}
