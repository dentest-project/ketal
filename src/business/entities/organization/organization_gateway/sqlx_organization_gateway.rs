use crate::business::{
    entities::organization::{
        Organization,
        organization_gateway::{GatewayFuture, OrganizationGateway},
    },
    error::OrganizationAlreadyExistsError,
};
use crate::infrastructure::transaction::sqlx_transaction_manager::current_connection;
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct SqlxOrganizationGateway;

impl SqlxOrganizationGateway {
    pub fn new() -> Self {
        Self
    }
}

impl OrganizationGateway for SqlxOrganizationGateway {
    fn find_one_by_id(&self, id: Uuid) -> GatewayFuture<'_, Option<Organization>> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let row = sqlx::query("SELECT id, name, slug FROM organization WHERE id = $1")
                .bind(id)
                .fetch_optional(&mut **connection)
                .await?;

            row.map(|row| {
                Ok(Organization {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    slug: row.try_get("slug")?,
                })
            })
            .transpose()
        })
    }

    fn find_one_by_name<'a>(&'a self, name: &'a str) -> GatewayFuture<'a, Option<Organization>> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let row = sqlx::query(
                "SELECT id, name, slug FROM organization WHERE LOWER(name) = LOWER($1) LIMIT 1",
            )
            .bind(name)
            .fetch_optional(&mut **connection)
            .await?;

            row.map(|row| {
                Ok(Organization {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    slug: row.try_get("slug")?,
                })
            })
            .transpose()
        })
    }

    fn save<'a>(&'a self, organization: &'a Organization) -> GatewayFuture<'a, ()> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let result = sqlx::query(
                r#"
                INSERT INTO organization (id, name, slug)
                VALUES ($1, $2, $3)
                ON CONFLICT (slug) DO NOTHING
                "#,
            )
            .bind(organization.id)
            .bind(&organization.name)
            .bind(&organization.slug)
            .execute(&mut **connection)
            .await?;

            if result.rows_affected() == 0 {
                return Err(OrganizationAlreadyExistsError.into());
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SqlxOrganizationGateway;
    use crate::business::{
        EntityBuilder,
        entities::organization::{
            OrganizationBuilder,
            organization_gateway::{OrganizationGateway, OrganizationGatewayError},
        },
    };

    #[tokio::test]
    async fn requires_an_active_transaction() {
        let gateway = SqlxOrganizationGateway::new();
        let organization = OrganizationBuilder::init()
            .with_name("Dental Team".to_owned())
            .build();

        assert!(matches!(
            gateway.find_one_by_id(organization.id).await,
            Err(OrganizationGatewayError::Unexpected(_))
        ));
        assert!(matches!(
            gateway.find_one_by_name("Dental Team").await,
            Err(OrganizationGatewayError::Unexpected(_))
        ));
        assert!(matches!(
            gateway.save(&organization).await,
            Err(OrganizationGatewayError::Unexpected(_))
        ));
    }
}
