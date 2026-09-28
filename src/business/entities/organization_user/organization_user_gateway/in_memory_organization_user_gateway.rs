use crate::business::{
    entities::{
        organization::Organization,
        organization_user::{
            OrganizationUser,
            organization_user_gateway::{GatewayFuture, OrganizationUserGateway},
        },
        user::User,
    },
    error::GatewayError,
};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

pub(crate) type OrganizationUsers = HashMap<(Uuid, Uuid), OrganizationUser>;

#[derive(Default)]
pub struct InMemoryOrganizationUserGateway {
    organization_users: Mutex<OrganizationUsers>,
}

impl InMemoryOrganizationUserGateway {
    pub(crate) fn find_one_by_organization_and_user(
        &self,
        organization: &Organization,
        user: &User,
    ) -> Result<Option<OrganizationUser>, GatewayError> {
        Ok(self
            .organization_users
            .lock()
            .map_err(|error| GatewayError::new(error.to_string()))?
            .get(&(organization.id, user.id))
            .cloned())
    }
}

impl OrganizationUserGateway for InMemoryOrganizationUserGateway {
    fn save<'a>(&'a self, organization_user: &'a OrganizationUser) -> GatewayFuture<'a, ()> {
        Box::pin(async move {
            let mut organization_users = self
                .organization_users
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?;
            let key = (organization_user.organization.id, organization_user.user.id);

            if organization_users.contains_key(&key) {
                return Err(GatewayError::new("organization membership already exists").into());
            }

            organization_users.insert(key, organization_user.clone());
            Ok(())
        })
    }
}
