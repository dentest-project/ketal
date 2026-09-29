use crate::business::{
    entities::organization_user::OrganizationPermission,
    usecases::outputs::{organization::OrganizationDetailedOutput, user::UserDetailedOutput},
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OrganizationUserDetailedOutput {
    pub organization: OrganizationDetailedOutput,
    pub user: UserDetailedOutput,
    pub permissions: Vec<OrganizationPermission>,
}
