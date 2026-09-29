use super::OrganizationUser;
use crate::business::usecases::outputs::organization_user::OrganizationUserDetailedOutput;

impl From<&OrganizationUser> for OrganizationUserDetailedOutput {
    fn from(organization_user: &OrganizationUser) -> Self {
        Self {
            organization: (&organization_user.organization).into(),
            user: (&organization_user.user).into(),
            permissions: organization_user.permissions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OrganizationUserDetailedOutput;
    use crate::business::{
        EntityBuilder,
        entities::{
            organization::OrganizationBuilder,
            organization_user::{OrganizationPermission, OrganizationUser},
            user::UserBuilder,
        },
    };
    use serde_json::json;

    #[test]
    fn presents_detailed_organization_and_user_without_private_user_fields()
    -> Result<(), serde_json::Error> {
        let organization = OrganizationBuilder::init()
            .with_name("Dental Team".to_owned())
            .build();
        let user = UserBuilder::init()
            .with_username("member".to_owned())
            .with_email("member@example.com".to_owned())
            .with_password("private-password".to_owned())
            .build();
        let membership = OrganizationUser::new(&organization, &user);

        let output = OrganizationUserDetailedOutput::from(&membership);

        assert_eq!(
            serde_json::to_value(output)?,
            json!({
                "organization": {
                    "id": organization.id.to_string(),
                    "name": "Dental Team",
                    "slug": "dental-team",
                },
                "user": {
                    "id": user.id.to_string(),
                    "username": "member",
                    "email": "member@example.com",
                },
                "permissions": [],
            })
        );
        Ok(())
    }

    #[test]
    fn preserves_and_serializes_membership_permissions() -> Result<(), serde_json::Error> {
        let organization = OrganizationBuilder::init().build();
        let user = UserBuilder::init().build();
        let mut membership = OrganizationUser::new(&organization, &user);
        membership.permissions = vec![
            OrganizationPermission::Admin,
            OrganizationPermission::ProjectCreate,
            OrganizationPermission::ProjectWrite,
            OrganizationPermission::Read,
        ];

        let output = OrganizationUserDetailedOutput::from(&membership);

        assert_eq!(output.permissions, membership.permissions);
        assert_eq!(
            serde_json::to_value(output)?["permissions"],
            json!(["admin", "project_create", "project_write", "read"])
        );
        Ok(())
    }
}
