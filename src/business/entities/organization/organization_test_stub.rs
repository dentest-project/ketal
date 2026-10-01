use super::Organization;
use std::cmp::Ordering;

impl Ord for Organization {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .to_lowercase()
            .cmp(&other.name.to_lowercase())
            .then_with(|| self.id.cmp(&other.id))
            .then_with(|| self.name.cmp(&other.name))
            .then_with(|| self.slug.cmp(&other.slug))
    }
}

impl PartialOrd for Organization {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
