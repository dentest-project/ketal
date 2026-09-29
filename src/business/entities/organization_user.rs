pub mod organization_user_gateway;
pub mod organization_user_presenter;

use super::{organization::Organization, user::User};
use crate::business::error::UserNotAllowedToAdministrateOrganizationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationPermission {
    Admin,
    ProjectCreate,
    ProjectWrite,
    Read,
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

    fn has_permission(&self, permission: OrganizationPermission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn ensure_can_administrate(
        &self,
    ) -> Result<(), UserNotAllowedToAdministrateOrganizationError> {
        if self.has_permission(OrganizationPermission::Admin) {
            Ok(())
        } else {
            Err(UserNotAllowedToAdministrateOrganizationError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OrganizationPermission, OrganizationUser};
    use crate::business::{
        EntityBuilder,
        entities::{organization::OrganizationBuilder, user::UserBuilder},
        error::UserNotAllowedToAdministrateOrganizationError,
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
        assert_eq!(
            membership.ensure_can_administrate(),
            Err(UserNotAllowedToAdministrateOrganizationError)
        );
    }

    #[test]
    fn grants_administrator_permission_without_duplicates() {
        let organization = OrganizationBuilder::init().build();
        let user = UserBuilder::init().build();
        let mut membership = OrganizationUser::new(&organization, &user);

        membership.make_admin();
        membership.make_admin();

        assert_eq!(membership.permissions, vec![OrganizationPermission::Admin]);
        assert_eq!(membership.ensure_can_administrate(), Ok(()));
    }

    #[test]
    fn recognizes_stored_permissions_and_only_grants_admin_access_to_admins()
    -> Result<(), serde_json::Error> {
        let organization = OrganizationBuilder::init().build();
        let user = UserBuilder::init().build();
        let mut membership = OrganizationUser::new(&organization, &user);
        membership.permissions =
            serde_json::from_str(r#"["project_create", "project_write", "read"]"#)?;

        assert_eq!(
            membership.ensure_can_administrate(),
            Err(UserNotAllowedToAdministrateOrganizationError)
        );
        membership.permissions = serde_json::from_str(r#"["read", "admin"]"#)?;
        assert_eq!(membership.ensure_can_administrate(), Ok(()));
        Ok(())
    }
}
