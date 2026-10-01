use super::ListMyOrganizations;
use crate::business::entities::organization_user::organization_user_gateway::SqlxOrganizationUserGateway;
use std::sync::Arc;

impl Default for ListMyOrganizations {
    fn default() -> Self {
        Self {
            organization_user_gateway: Arc::new(SqlxOrganizationUserGateway::new()),
        }
    }
}
