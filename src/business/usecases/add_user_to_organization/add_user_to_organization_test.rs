use super::{AddUserToOrganization, AddUserToOrganizationError, AddUserToOrganizationInput};
use crate::business::{
    EntityBuilder,
    entities::{
        organization::{
            Organization, OrganizationBuilder, organization_gateway::InMemoryOrganizationGateway,
        },
        organization_user::{
            OrganizationUser, organization_user_gateway::InMemoryOrganizationUserGateway,
        },
        user::{User, UserBuilder, user_gateway::InMemoryUserGateway},
    },
    usecases::outputs::{
        organization::OrganizationDetailedOutput,
        organization_user::OrganizationUserDetailedOutput,
        user::{UserDetailedOutput, UserListItemOutput},
    },
};
use jsonrpc_usecase::UseCaseExecutionError;
use std::{error::Error, sync::Arc};
use uuid::Uuid;

#[tokio::test]
async fn admin_adds_user_without_any_permissions() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;

    let organization = fixture
        .use_case
        .retrieve_organization(fixture.organization_id)
        .await?;
    fixture
        .use_case
        .ensure_authenticated_user_is_allowed_to_administrate_organization(
            &organization,
            &fixture.admin,
        )
        .await?;
    let user_to_add = fixture
        .use_case
        .retrieve_user_to_add(fixture.user_id)
        .await?;
    let membership = fixture
        .use_case
        .add_user_to_organization(&organization, &user_to_add)
        .await?;
    let output = OrganizationUserDetailedOutput::from(&membership);

    assert_eq!(organization, fixture.organization);
    assert_eq!(user_to_add, fixture.user);
    assert_eq!(
        fixture.membership().await?,
        Some(OrganizationUser::new(&fixture.organization, &fixture.user))
    );
    assert_eq!(
        output.organization,
        OrganizationDetailedOutput::from(&fixture.organization)
    );
    let expected_user: UserDetailedOutput = (&fixture.user).into();
    assert_eq!(output.user, expected_user);
    assert!(output.permissions.is_empty());
    Ok(())
}

#[tokio::test]
async fn denies_members_without_admin_permission() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let membership = OrganizationUser::new(&fixture.organization, &fixture.user);
    fixture
        .use_case
        .organization_user_gateway
        .save(&membership)
        .await?;

    let result = fixture
        .use_case
        .ensure_authenticated_user_is_allowed_to_administrate_organization(
            &fixture.organization,
            &fixture.user,
        )
        .await;

    assert_administration_denied(result);
    assert_eq!(fixture.membership().await?, Some(membership));
    Ok(())
}

#[tokio::test]
async fn denies_nonmembers_and_admins_of_other_organizations() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let outsider = UserBuilder::init().build();

    let result = fixture
        .use_case
        .ensure_authenticated_user_is_allowed_to_administrate_organization(
            &fixture.organization,
            &outsider,
        )
        .await;
    assert_administration_denied(result);

    let other_organization = OrganizationBuilder::init()
        .with_name("Other team".to_owned())
        .build();
    fixture
        .use_case
        .organization_gateway
        .save(&other_organization)
        .await?;
    let mut other_membership = OrganizationUser::new(&other_organization, &outsider);
    other_membership.make_admin();
    fixture
        .use_case
        .organization_user_gateway
        .save(&other_membership)
        .await?;

    let result = fixture
        .use_case
        .ensure_authenticated_user_is_allowed_to_administrate_organization(
            &fixture.organization,
            &outsider,
        )
        .await;
    assert_administration_denied(result);
    assert!(fixture.membership().await?.is_none());
    Ok(())
}

#[tokio::test]
async fn returns_organization_not_found() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let result = fixture.use_case.retrieve_organization(Uuid::new_v4()).await;

    assert!(matches!(
        result,
        Err(AddUserToOrganizationError::OrganizationNotFound(_))
    ));
    if let Err(error) = result {
        assert_rpc_error(&error, 20_011, "OrganizationNotFound");
    }
    assert!(fixture.membership().await?.is_none());
    Ok(())
}

#[tokio::test]
async fn returns_user_not_found() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let result = fixture.use_case.retrieve_user_to_add(Uuid::new_v4()).await;

    assert!(matches!(
        result,
        Err(AddUserToOrganizationError::UserNotFound(_))
    ));
    if let Err(error) = result {
        assert_rpc_error(&error, 20_007, "UserNotFound");
    }
    assert!(fixture.membership().await?.is_none());
    Ok(())
}

#[tokio::test]
async fn rejects_existing_members_and_preserves_their_permissions() -> Result<(), Box<dyn Error>> {
    for is_admin in [false, true] {
        let fixture = Fixture::new().await?;
        let mut membership = OrganizationUser::new(&fixture.organization, &fixture.user);
        if is_admin {
            membership.make_admin();
        }
        fixture
            .use_case
            .organization_user_gateway
            .save(&membership)
            .await?;

        let result = fixture
            .use_case
            .add_user_to_organization(&fixture.organization, &fixture.user)
            .await;

        assert!(matches!(
            result,
            Err(AddUserToOrganizationError::UserAlreadyPartOfOrganization(_))
        ));
        if let Err(error) = result {
            assert_rpc_error(&error, 20_012, "UserAlreadyPartOfOrganization");
        }
        assert_eq!(fixture.membership().await?, Some(membership));
    }
    Ok(())
}

#[tokio::test]
async fn admin_cannot_add_themselves_again() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let result = fixture
        .use_case
        .add_user_to_organization(&fixture.organization, &fixture.admin)
        .await;

    assert!(matches!(
        result,
        Err(AddUserToOrganizationError::UserAlreadyPartOfOrganization(_))
    ));
    let mut expected = OrganizationUser::new(&fixture.organization, &fixture.admin);
    expected.make_admin();
    assert_eq!(
        fixture
            .use_case
            .organization_user_gateway
            .find_one_by_organization_and_user(&fixture.organization, &fixture.admin)
            .await?,
        Some(expected)
    );
    Ok(())
}

#[tokio::test]
async fn repeated_additions_only_create_one_membership() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;
    let (first, second) = tokio::join!(
        fixture
            .use_case
            .add_user_to_organization(&fixture.organization, &fixture.user),
        fixture
            .use_case
            .add_user_to_organization(&fixture.organization, &fixture.user),
    );

    assert!(matches!(
        (first, second),
        (
            Ok(_),
            Err(AddUserToOrganizationError::UserAlreadyPartOfOrganization(_))
        ) | (
            Err(AddUserToOrganizationError::UserAlreadyPartOfOrganization(_)),
            Ok(_)
        )
    ));
    assert_eq!(
        fixture.membership().await?,
        Some(OrganizationUser::new(&fixture.organization, &fixture.user))
    );
    Ok(())
}

#[tokio::test]
async fn returns_unexpected_error_without_a_request_context() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new().await?;

    let result = fixture.use_case.execute(fixture.input()).await;

    assert!(matches!(
        result,
        Err(UseCaseExecutionError::Execution(
            AddUserToOrganizationError::Unexpected(_)
        ))
    ));
    assert!(fixture.membership().await?.is_none());
    Ok(())
}

fn assert_rpc_error(error: &AddUserToOrganizationError, code: i64, message: &str) {
    assert_eq!(jsonrpc_usecase::Error::code(error), code);
    assert_eq!(jsonrpc_usecase::Error::message(error), message);
}

fn assert_administration_denied(result: Result<(), AddUserToOrganizationError>) {
    assert!(matches!(
        result,
        Err(AddUserToOrganizationError::UserNotAllowedToAdministrateOrganization(_))
    ));
    if let Err(error) = result {
        assert_rpc_error(&error, 20_013, "UserNotAllowedToAdministrateOrganization");
        assert_eq!(
            error.to_string(),
            "user is not allowed to administrate the organization"
        );
    }
}

fn user_id(user: &User) -> Result<Uuid, uuid::Error> {
    let output: UserListItemOutput = user.into();
    output.id.parse()
}

struct Fixture {
    use_case: AddUserToOrganization,
    organization: Organization,
    admin: User,
    user: User,
    organization_id: Uuid,
    user_id: Uuid,
}

impl Fixture {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let use_case = AddUserToOrganization {
            organization_gateway: Arc::new(InMemoryOrganizationGateway::default()),
            organization_user_gateway: Arc::new(InMemoryOrganizationUserGateway::default()),
            user_gateway: Arc::new(InMemoryUserGateway::default()),
        };
        let organization = OrganizationBuilder::init()
            .with_name("Dental Team".to_owned())
            .build();
        let admin = UserBuilder::init()
            .with_username("admin".to_owned())
            .build();
        let user = UserBuilder::init()
            .with_username("member".to_owned())
            .build();
        use_case.organization_gateway.save(&organization).await?;
        use_case.user_gateway.save(&admin).await?;
        use_case.user_gateway.save(&user).await?;
        let mut membership = OrganizationUser::new(&organization, &admin);
        membership.make_admin();
        use_case.organization_user_gateway.save(&membership).await?;

        Ok(Self {
            organization_id: OrganizationDetailedOutput::from(&organization).id.parse()?,
            user_id: user_id(&user)?,
            use_case,
            organization,
            admin,
            user,
        })
    }

    fn input(&self) -> AddUserToOrganizationInput {
        AddUserToOrganizationInput {
            organization_id: self.organization_id,
            user_id: self.user_id,
        }
    }

    async fn membership(&self) -> Result<Option<OrganizationUser>, Box<dyn Error>> {
        Ok(self
            .use_case
            .organization_user_gateway
            .find_one_by_organization_and_user(&self.organization, &self.user)
            .await?)
    }
}
