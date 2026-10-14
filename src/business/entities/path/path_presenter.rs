use super::Path;
use crate::business::usecases::outputs::path::PathListItemOutput;

impl From<&Path> for PathListItemOutput {
    fn from(path: &Path) -> Self {
        Self {
            id: path.id.to_string(),
            name: path.path.clone(),
            slug: path.slug.clone(),
        }
    }
}
