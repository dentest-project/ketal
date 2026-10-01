use crate::business::{
    entities::{
        organization::{
            Organization, organization_gateway::sqlx_organization_gateway::organization_from_row,
        },
        organization_user::{
            OrganizationPermission, OrganizationUser,
            organization_user_gateway::{GatewayFuture, OrganizationUserGateway},
        },
        user::User,
    },
    error::UserAlreadyPartOfOrganizationError,
};
use crate::infrastructure::transaction::sqlx_transaction_manager::current_connection;
use sqlx::{Row, types::Json};

#[derive(Clone, Default)]
pub struct SqlxOrganizationUserGateway;

impl SqlxOrganizationUserGateway {
    pub fn new() -> Self {
        Self
    }
}

impl OrganizationUserGateway for SqlxOrganizationUserGateway {
    fn find_by_user<'a>(&'a self, user: &'a User) -> GatewayFuture<'a, Vec<Organization>> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let rows = sqlx::query(
                r#"
                SELECT organization.id, organization.name, organization.slug
                FROM organization
                INNER JOIN organization_user ON organization_user.organization_id = organization.id
                WHERE organization_user.user_id = $1
                ORDER BY LOWER(organization.name), organization.id
                "#,
            )
            .bind(user.id)
            .fetch_all(&mut **connection)
            .await?;

            rows.iter()
                .map(organization_from_row)
                .collect::<Result<Vec<_>, _>>()
                .map_err(Into::into)
        })
    }

    fn find_one_by_organization_and_user<'a>(
        &'a self,
        organization: &'a Organization,
        user: &'a User,
    ) -> GatewayFuture<'a, Option<OrganizationUser>> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let row = sqlx::query(
                "SELECT permissions FROM organization_user WHERE organization_id = $1 AND user_id = $2",
            )
            .bind(organization.id)
            .bind(user.id)
            .fetch_optional(&mut **connection)
            .await?;

            row.map(|row| {
                let permissions: Json<Vec<OrganizationPermission>> = row.try_get("permissions")?;
                Ok(OrganizationUser {
                    organization: organization.clone(),
                    user: user.clone(),
                    permissions: permissions.0,
                })
            })
            .transpose()
        })
    }

    fn save<'a>(&'a self, organization_user: &'a OrganizationUser) -> GatewayFuture<'a, ()> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let result = sqlx::query(
                r#"
                INSERT INTO organization_user (organization_id, user_id, permissions)
                VALUES ($1, $2, $3)
                ON CONFLICT (organization_id, user_id) DO NOTHING
                "#,
            )
            .bind(organization_user.organization.id)
            .bind(organization_user.user.id)
            .bind(Json(&organization_user.permissions))
            .execute(&mut **connection)
            .await?;

            if result.rows_affected() == 0 {
                return Err(UserAlreadyPartOfOrganizationError.into());
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SqlxOrganizationUserGateway;
    use crate::business::{
        EntityBuilder,
        entities::{
            organization::OrganizationBuilder,
            organization_user::{
                OrganizationUser,
                organization_user_gateway::{
                    OrganizationUserGateway, OrganizationUserGatewayError,
                },
            },
            user::UserBuilder,
        },
    };

    #[tokio::test]
    async fn requires_an_active_transaction() {
        let gateway = SqlxOrganizationUserGateway::new();
        let organization = OrganizationBuilder::init().build();
        let user = UserBuilder::init().build();
        let membership = OrganizationUser::new(&organization, &user);

        assert!(matches!(
            gateway.find_by_user(&user).await,
            Err(OrganizationUserGatewayError::Unexpected(_))
        ));
        assert!(matches!(
            gateway
                .find_one_by_organization_and_user(&organization, &user)
                .await,
            Err(OrganizationUserGatewayError::Unexpected(_))
        ));
        assert!(matches!(
            gateway.save(&membership).await,
            Err(OrganizationUserGatewayError::Unexpected(_))
        ));
    }
}
