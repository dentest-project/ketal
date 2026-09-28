use super::Organization;
use crate::business::EntityBuilder;
use uuid::Uuid;

pub struct OrganizationBuilder {
    entity: Organization,
}

impl OrganizationBuilder {
    pub fn with_name(mut self, name: String) -> Self {
        let mut slug = slug::slugify(&name);

        if slug.is_empty() {
            // Keep a deterministic URL identifier even for punctuation-only names.
            let encoded_name: String = name
                .to_lowercase()
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            slug = format!("organization-{encoded_name}");
        }

        self.entity.slug = slug.chars().take(255).collect();
        self.entity.name = name;
        self
    }
}

impl EntityBuilder<Organization> for OrganizationBuilder {
    fn init() -> Self {
        Self {
            entity: Organization {
                id: Uuid::new_v4(),
                name: String::new(),
                slug: String::new(),
            },
        }
    }

    fn build(self) -> Organization {
        self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::OrganizationBuilder;
    use crate::business::EntityBuilder;

    #[test]
    fn generates_a_slug_from_an_accented_name() {
        let organization = OrganizationBuilder::init()
            .with_name("Équipe Dentaire".to_owned())
            .build();

        assert_eq!(organization.name, "Équipe Dentaire");
        assert_eq!(organization.slug, "equipe-dentaire");
        assert!(!organization.id.is_nil());
    }

    #[test]
    fn bounds_expanded_slugs_to_the_database_column_length() {
        let organization = OrganizationBuilder::init()
            .with_name("影".repeat(255))
            .build();

        assert!(!organization.slug.is_empty());
        assert!(organization.slug.len() <= 255);
    }

    #[test]
    fn punctuation_only_names_have_distinct_deterministic_slugs() {
        let first = OrganizationBuilder::init()
            .with_name("!!!".to_owned())
            .build();
        let duplicate = OrganizationBuilder::init()
            .with_name("!!!".to_owned())
            .build();
        let other = OrganizationBuilder::init()
            .with_name("???".to_owned())
            .build();

        assert!(!first.slug.is_empty());
        assert_eq!(first.slug, duplicate.slug);
        assert_ne!(first.slug, other.slug);
    }
}
