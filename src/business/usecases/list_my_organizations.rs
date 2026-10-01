mod default_list_my_organizations;

#[cfg(test)]
mod list_my_organizations_test;

use crate::{
    business::{
        RequestContext,
        entities::{
            organization::Organization,
            organization_user::organization_user_gateway::{
                OrganizationUserGateway, OrganizationUserGatewayError,
            },
            user::User,
        },
        error::{GatewayError, UnexpectedError, use_case_error},
        usecases::outputs::organization::OrganizationListOutput,
    },
    infrastructure::guards::AuthenticatedUserGuard,
};
use jsonrpc_usecase::{UseCase, current_context};
use macros::Transactional;
use std::sync::Arc;

pub struct ListMyOrganizations {
    organization_user_gateway: Arc<dyn OrganizationUserGateway>,
}

use_case_error! {
    pub enum ListMyOrganizationsError {
        Gateway(GatewayError),
        Unexpected(UnexpectedError),
    }
}

impl From<OrganizationUserGatewayError> for ListMyOrganizationsError {
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
impl ListMyOrganizations {
    async fn execute(
        &self,
        _input: (),
    ) -> Result<OrganizationListOutput, ListMyOrganizationsError> {
        let context = current_context::<RequestContext>().ok_or(UnexpectedError)?;
        let authenticated_user = context.user.as_ref().ok_or(UnexpectedError)?;

        let organizations = self.retrieve_organizations(authenticated_user).await?;

        Ok(organizations.as_slice().into())
    }

    async fn retrieve_organizations(
        &self,
        authenticated_user: &User,
    ) -> Result<Vec<Organization>, ListMyOrganizationsError> {
        Ok(self
            .organization_user_gateway
            .find_by_user(authenticated_user)
            .await?)
    }
}
