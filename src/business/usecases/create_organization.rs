mod create_organization_input;
mod default_create_organization;

#[cfg(test)]
mod create_organization_test;

use crate::{
    business::{
        EntityBuilder, RequestContext,
        entities::{
            organization::{
                Organization, OrganizationBuilder,
                organization_gateway::{OrganizationGateway, OrganizationGatewayError},
            },
            organization_user::{
                OrganizationUser,
                organization_user_gateway::{
                    OrganizationUserGateway, OrganizationUserGatewayError,
                },
            },
            user::User,
        },
        error::{GatewayError, OrganizationAlreadyExistsError, UnexpectedError, use_case_error},
        usecases::outputs::organization::OrganizationDetailedOutput,
    },
    infrastructure::guards::AuthenticatedUserGuard,
};
use jsonrpc_usecase::{UseCase, current_context};
use macros::Transactional;
use std::sync::Arc;

pub use create_organization_input::CreateOrganizationInput;

pub struct CreateOrganization {
    organization_gateway: Arc<dyn OrganizationGateway>,
    organization_user_gateway: Arc<dyn OrganizationUserGateway>,
}

use_case_error! {
    pub enum CreateOrganizationError {
        OrganizationAlreadyExists(OrganizationAlreadyExistsError),
        Gateway(GatewayError),
        Unexpected(UnexpectedError),
    }
}

impl From<OrganizationGatewayError> for CreateOrganizationError {
    fn from(error: OrganizationGatewayError) -> Self {
        match error {
            OrganizationGatewayError::OrganizationAlreadyExists(error) => error.into(),
            OrganizationGatewayError::Gateway(error) => error.into(),
            OrganizationGatewayError::Unexpected(error) => error.into(),
        }
    }
}

impl From<OrganizationUserGatewayError> for CreateOrganizationError {
    fn from(error: OrganizationUserGatewayError) -> Self {
        match error {
            OrganizationUserGatewayError::UserAlreadyPartOfOrganization(_) => {
                UnexpectedError.into()
            }
            OrganizationUserGatewayError::Gateway(error) => error.into(),
            OrganizationUserGatewayError::Unexpected(error) => error.into(),
        }
    }
}

#[Transactional]
#[UseCase(guards = [AuthenticatedUserGuard])]
impl CreateOrganization {
    async fn execute(
        &self,
        input: CreateOrganizationInput,
    ) -> Result<OrganizationDetailedOutput, CreateOrganizationError> {
        let context = current_context::<RequestContext>().ok_or(UnexpectedError)?;
        let user = context.user.as_ref().ok_or(UnexpectedError)?;

        self.ensure_organization_does_not_exist(&input.name).await?;
        let organization = self.create_organization(input).await?;
        self.make_authenticated_user_admin(&organization, user)
            .await?;

        Ok((&organization).into())
    }

    async fn ensure_organization_does_not_exist(
        &self,
        name: &str,
    ) -> Result<(), CreateOrganizationError> {
        if self
            .organization_gateway
            .find_one_by_name(name)
            .await?
            .is_some()
        {
            return Err(OrganizationAlreadyExistsError.into());
        }

        Ok(())
    }

    async fn create_organization(
        &self,
        input: CreateOrganizationInput,
    ) -> Result<Organization, CreateOrganizationError> {
        let organization = OrganizationBuilder::init().with_name(input.name).build();
        self.organization_gateway.save(&organization).await?;

        Ok(organization)
    }

    async fn make_authenticated_user_admin(
        &self,
        organization: &Organization,
        user: &User,
    ) -> Result<(), CreateOrganizationError> {
        let mut organization_user = OrganizationUser::new(organization, user);
        organization_user.make_admin();

        self.organization_user_gateway
            .save(&organization_user)
            .await?;

        Ok(())
    }
}
