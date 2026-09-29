use crate::business::{
    entities::organization::{
        Organization,
        organization_gateway::{GatewayFuture, OrganizationGateway},
    },
    error::{GatewayError, OrganizationAlreadyExistsError},
};
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryOrganizationGateway {
    organizations: Mutex<HashMap<Uuid, Organization>>,
}

impl OrganizationGateway for InMemoryOrganizationGateway {
    fn find_one_by_id(&self, id: Uuid) -> GatewayFuture<'_, Option<Organization>> {
        Box::pin(async move {
            Ok(self
                .organizations
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?
                .get(&id)
                .cloned())
        })
    }

    fn find_one_by_name<'a>(&'a self, name: &'a str) -> GatewayFuture<'a, Option<Organization>> {
        Box::pin(async move {
            let name = name.to_lowercase();
            Ok(self
                .organizations
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?
                .values()
                .find(|organization| organization.name.to_lowercase() == name)
                .cloned())
        })
    }

    fn save<'a>(&'a self, organization: &'a Organization) -> GatewayFuture<'a, ()> {
        Box::pin(async move {
            let mut organizations = self
                .organizations
                .lock()
                .map_err(|error| GatewayError::new(error.to_string()))?;

            if organizations
                .values()
                .any(|existing| existing.slug == organization.slug)
            {
                return Err(OrganizationAlreadyExistsError.into());
            }

            organizations.insert(organization.id, organization.clone());
            Ok(())
        })
    }
}
