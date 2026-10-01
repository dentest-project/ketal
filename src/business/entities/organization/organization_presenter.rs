use super::Organization;
use crate::business::usecases::outputs::organization::{
    OrganizationDetailedOutput, OrganizationListItemOutput, OrganizationListOutput,
};

impl From<&Organization> for OrganizationDetailedOutput {
    fn from(organization: &Organization) -> Self {
        Self {
            id: organization.id.to_string(),
            name: organization.name.clone(),
            slug: organization.slug.clone(),
        }
    }
}

impl From<&Organization> for OrganizationListItemOutput {
    fn from(organization: &Organization) -> Self {
        Self {
            id: organization.id.to_string(),
            name: organization.name.clone(),
            slug: organization.slug.clone(),
        }
    }
}

impl From<&[Organization]> for OrganizationListOutput {
    fn from(organizations: &[Organization]) -> Self {
        Self(
            organizations
                .iter()
                .map(OrganizationListItemOutput::from)
                .collect(),
        )
    }
}
