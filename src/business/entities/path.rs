pub mod path_gateway;

#[cfg(test)]
mod path_test_stub;

#[cfg(test)]
pub use path_test_stub::PathBuilder;

use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub(super) id: Uuid,
    path: String,
    slug: String,
    summary: String,
    parent: Option<Box<Path>>,
}
