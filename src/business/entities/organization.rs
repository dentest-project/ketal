pub mod organization_builder;
pub mod organization_gateway;
pub mod organization_presenter;

#[cfg(test)]
mod organization_test_stub;

pub use organization_builder::OrganizationBuilder;

use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Organization {
    pub(super) id: Uuid,
    name: String,
    slug: String,
}
