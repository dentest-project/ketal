use super::AddUserToOrganization;
use crate::business::entities::{
    organization::organization_gateway::SqlxOrganizationGateway,
    organization_user::organization_user_gateway::SqlxOrganizationUserGateway,
    user::user_gateway::SqlxUserGateway,
};
use std::sync::Arc;

impl Default for AddUserToOrganization {
    fn default() -> Self {
        Self {
            organization_gateway: Arc::new(SqlxOrganizationGateway::new()),
            organization_user_gateway: Arc::new(SqlxOrganizationUserGateway::new()),
            user_gateway: Arc::new(SqlxUserGateway::new()),
        }
    }
}
