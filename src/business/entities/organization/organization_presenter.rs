use super::Organization;
use crate::business::usecases::outputs::organization::OrganizationDetailedOutput;

impl From<&Organization> for OrganizationDetailedOutput {
    fn from(organization: &Organization) -> Self {
        Self {
            id: organization.id.to_string(),
            name: organization.name.clone(),
            slug: organization.slug.clone(),
        }
    }
}
