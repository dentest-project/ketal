pub mod organization_user_gateway;

use super::{organization::Organization, user::User};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationPermission {
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganizationUser {
    organization: Organization,
    user: User,
    permissions: Vec<OrganizationPermission>,
}

impl OrganizationUser {
    pub fn new(organization: &Organization, user: &User) -> Self {
        Self {
            organization: organization.clone(),
            user: user.clone(),
            permissions: Vec::new(),
        }
    }

    pub fn make_admin(&mut self) {
        if !self.permissions.contains(&OrganizationPermission::Admin) {
            self.permissions.push(OrganizationPermission::Admin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OrganizationPermission, OrganizationUser};
    use crate::business::{
        EntityBuilder,
        entities::{organization::OrganizationBuilder, user::UserBuilder},
    };

    #[test]
    fn new_membership_has_no_permissions_and_references_its_organization_and_user() {
        let organization = OrganizationBuilder::init()
            .with_name("Dental Team".to_owned())
            .build();
        let user = UserBuilder::init().build();
        let membership = OrganizationUser::new(&organization, &user);

        assert_eq!(membership.organization, organization);
        assert_eq!(membership.user, user);
        assert!(membership.permissions.is_empty());
    }

    #[test]
    fn grants_administrator_permission_without_duplicates() {
        let organization = OrganizationBuilder::init().build();
        let user = UserBuilder::init().build();
        let mut membership = OrganizationUser::new(&organization, &user);

        membership.make_admin();
        membership.make_admin();

        assert_eq!(membership.permissions, vec![OrganizationPermission::Admin]);
    }
}
