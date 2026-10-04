use super::{Project, ProjectVisibility};
use crate::business::{
    EntityBuilder,
    entities::{organization::Organization, path::Path},
};
use std::cmp::Ordering;
use uuid::Uuid;

pub struct ProjectBuilder {
    entity: Project,
}

impl ProjectBuilder {
    pub fn with_title(mut self, title: String) -> Self {
        self.entity.slug = slug::slugify(&title);
        self.entity.title = title;
        self
    }

    pub fn with_organization(mut self, organization: &Organization) -> Self {
        self.entity.organization = Some(organization.clone());
        self
    }

    pub fn with_visibility(mut self, visibility: ProjectVisibility) -> Self {
        self.entity.visibility = visibility;
        self
    }

    pub fn with_root_path(mut self, root_path: &Path) -> Self {
        self.entity.root_path = Some(root_path.clone());
        self
    }
}

impl EntityBuilder<Project> for ProjectBuilder {
    fn init() -> Self {
        Self {
            entity: Project {
                id: Uuid::new_v4(),
                title: String::new(),
                slug: String::new(),
                visibility: ProjectVisibility::Private,
                root_path: None,
                organization: None,
            },
        }
    }

    fn build(self) -> Project {
        self.entity
    }
}

impl Ord for Project {
    fn cmp(&self, other: &Self) -> Ordering {
        self.title
            .to_lowercase()
            .cmp(&other.title.to_lowercase())
            .then_with(|| self.id.cmp(&other.id))
            .then_with(|| self.title.cmp(&other.title))
            .then_with(|| self.slug.cmp(&other.slug))
            .then_with(|| (self.visibility as u8).cmp(&(other.visibility as u8)))
            .then_with(|| self.root_path.cmp(&other.root_path))
            .then_with(|| self.organization.cmp(&other.organization))
    }
}

impl PartialOrd for Project {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
