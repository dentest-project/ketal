use crate::business::entities::organization_user::{
    OrganizationUser,
    organization_user_gateway::{GatewayFuture, OrganizationUserGateway},
};
use crate::infrastructure::transaction::sqlx_transaction_manager::current_connection;
use sqlx::types::Json;

#[derive(Clone, Default)]
pub struct SqlxOrganizationUserGateway;

impl SqlxOrganizationUserGateway {
    pub fn new() -> Self {
        Self
    }
}

impl OrganizationUserGateway for SqlxOrganizationUserGateway {
    fn save<'a>(&'a self, organization_user: &'a OrganizationUser) -> GatewayFuture<'a, ()> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            sqlx::query(
                r#"
                INSERT INTO organization_user (organization_id, user_id, permissions)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(organization_user.organization.id)
            .bind(organization_user.user.id)
            .bind(Json(&organization_user.permissions))
            .execute(&mut **connection)
            .await?;

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
            gateway.save(&membership).await,
            Err(OrganizationUserGatewayError::Unexpected(_))
        ));
    }
}
