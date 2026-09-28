use super::CreateOrganization;
use crate::business::entities::{
    organization::organization_gateway::SqlxOrganizationGateway,
    organization_user::organization_user_gateway::SqlxOrganizationUserGateway,
};
use std::sync::Arc;

impl Default for CreateOrganization {
    fn default() -> Self {
        Self {
            organization_gateway: Arc::new(SqlxOrganizationGateway::new()),
            organization_user_gateway: Arc::new(SqlxOrganizationUserGateway::new()),
        }
    }
}
