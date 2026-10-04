use super::ListMyProjects;
use crate::business::entities::project::project_gateway::SqlxProjectGateway;
use std::sync::Arc;

impl Default for ListMyProjects {
    fn default() -> Self {
        Self {
            project_gateway: Arc::new(SqlxProjectGateway::new()),
        }
    }
}
