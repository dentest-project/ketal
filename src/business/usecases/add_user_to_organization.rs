mod add_user_to_organization_input;
mod default_add_user_to_organization;

#[cfg(test)]
mod add_user_to_organization_test;

use crate::{
    business::{
        RequestContext,
        entities::{
            organization::{
                Organization,
                organization_gateway::{OrganizationGateway, OrganizationGatewayError},
            },
            organization_user::{
                OrganizationUser,
                organization_user_gateway::{
                    OrganizationUserGateway, OrganizationUserGatewayError,
                },
            },
            user::{User, user_gateway::UserGateway},
        },
        error::{
            GatewayError, OrganizationNotFoundError, UnexpectedError,
            UserAlreadyPartOfOrganizationError, UserNotAllowedToAdministrateOrganizationError,
            UserNotFoundError, use_case_error,
        },
        usecases::outputs::organization_user::OrganizationUserDetailedOutput,
    },
    infrastructure::guards::AuthenticatedUserGuard,
};
use jsonrpc_usecase::{UseCase, current_context};
use macros::Transactional;
use std::sync::Arc;
use uuid::Uuid;

pub use add_user_to_organization_input::AddUserToOrganizationInput;

pub struct AddUserToOrganization {
    organization_gateway: Arc<dyn OrganizationGateway>,
    organization_user_gateway: Arc<dyn OrganizationUserGateway>,
    user_gateway: Arc<dyn UserGateway>,
}

use_case_error! {
    pub enum AddUserToOrganizationError {
        UserNotAllowedToAdministrateOrganization(UserNotAllowedToAdministrateOrganizationError),
        UserAlreadyPartOfOrganization(UserAlreadyPartOfOrganizationError),
        OrganizationNotFound(OrganizationNotFoundError),
        UserNotFound(UserNotFoundError),
        Gateway(GatewayError),
        Unexpected(UnexpectedError),
    }
}

impl From<OrganizationGatewayError> for AddUserToOrganizationError {
    fn from(error: OrganizationGatewayError) -> Self {
        match error {
            OrganizationGatewayError::OrganizationAlreadyExists(_) => UnexpectedError.into(),
            OrganizationGatewayError::Gateway(error) => error.into(),
            OrganizationGatewayError::Unexpected(error) => error.into(),
        }
    }
}

impl From<OrganizationUserGatewayError> for AddUserToOrganizationError {
    fn from(error: OrganizationUserGatewayError) -> Self {
        match error {
            OrganizationUserGatewayError::UserAlreadyPartOfOrganization(error) => error.into(),
            OrganizationUserGatewayError::Gateway(error) => error.into(),
            OrganizationUserGatewayError::Unexpected(error) => error.into(),
        }
    }
}

#[Transactional]
#[UseCase(guards = [AuthenticatedUserGuard])]
impl AddUserToOrganization {
    async fn execute(
        &self,
        input: AddUserToOrganizationInput,
    ) -> Result<OrganizationUserDetailedOutput, AddUserToOrganizationError> {
        let context = current_context::<RequestContext>().ok_or(UnexpectedError)?;
        let authenticated_user = context.user.as_ref().ok_or(UnexpectedError)?;

        let organization = self.retrieve_organization(input.organization_id).await?;
        self.ensure_authenticated_user_is_allowed_to_administrate_organization(
            &organization,
            authenticated_user,
        )
        .await?;
        let user_to_add = self.retrieve_user_to_add(input.user_id).await?;
        let organization_user = self
            .add_user_to_organization(&organization, &user_to_add)
            .await?;

        Ok((&organization_user).into())
    }

    async fn retrieve_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Organization, AddUserToOrganizationError> {
        self.organization_gateway
            .find_one_by_id(organization_id)
            .await?
            .ok_or_else(|| OrganizationNotFoundError.into())
    }

    async fn ensure_authenticated_user_is_allowed_to_administrate_organization(
        &self,
        organization: &Organization,
        authenticated_user: &User,
    ) -> Result<(), AddUserToOrganizationError> {
        let membership = self
            .organization_user_gateway
            .find_one_by_organization_and_user(organization, authenticated_user)
            .await?
            .ok_or(UserNotAllowedToAdministrateOrganizationError)?;
        membership.ensure_can_administrate()?;

        Ok(())
    }

    async fn retrieve_user_to_add(
        &self,
        user_id: Uuid,
    ) -> Result<User, AddUserToOrganizationError> {
        self.user_gateway
            .find_one_by_id(user_id)
            .await?
            .ok_or_else(|| UserNotFoundError.into())
    }

    async fn add_user_to_organization(
        &self,
        organization: &Organization,
        user_to_add: &User,
    ) -> Result<OrganizationUser, AddUserToOrganizationError> {
        let membership = OrganizationUser::new(organization, user_to_add);
        self.organization_user_gateway.save(&membership).await?;

        Ok(membership)
    }
}
