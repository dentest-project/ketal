use super::Path;
use crate::business::EntityBuilder;
use std::cmp::Ordering;
use uuid::Uuid;

pub struct PathBuilder {
    entity: Path,
}

impl PathBuilder {
    pub fn with_path(mut self, path: String) -> Self {
        self.entity.slug = slug::slugify(&path);
        self.entity.path = path;
        self
    }

    pub fn with_summary(mut self, summary: String) -> Self {
        self.entity.summary = summary;
        self
    }

    pub fn with_parent(mut self, parent: &Path) -> Self {
        self.entity.parent = Some(Box::new(parent.clone()));
        self
    }
}

impl EntityBuilder<Path> for PathBuilder {
    fn init() -> Self {
        Self {
            entity: Path {
                id: Uuid::new_v4(),
                path: String::new(),
                slug: String::new(),
                summary: String::new(),
                parent: None,
            },
        }
    }

    fn build(self) -> Path {
        self.entity
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.path.cmp(&other.path))
            .then_with(|| self.slug.cmp(&other.slug))
            .then_with(|| self.summary.cmp(&other.summary))
            .then_with(|| self.parent.cmp(&other.parent))
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
