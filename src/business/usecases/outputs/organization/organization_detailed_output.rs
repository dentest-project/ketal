use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OrganizationDetailedOutput {
    pub id: String,
    pub name: String,
    pub slug: String,
}
