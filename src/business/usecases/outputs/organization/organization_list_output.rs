use super::OrganizationListItemOutput;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct OrganizationListOutput(pub Vec<OrganizationListItemOutput>);
