use crate::business::{
    entities::{
        organization::{
            Organization, organization_gateway::sqlx_organization_gateway::organization_from_row,
        },
        organization_user::{
            OrganizationPermission, OrganizationUser,
            organization_user_gateway::{GatewayFuture, OrganizationUserGateway},
        },
        user::{User, user_gateway::sqlx_user_gateway::user_from_row},
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
                .map(|row| organization_from_row(row, ""))
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
                r#"
                SELECT organization.id AS organization_id,
                       organization.name AS organization_name,
                       organization.slug AS organization_slug,
                       app_user.id, app_user.username, app_user.email, app_user.password,
                       app_user.roles, app_user.last_reset_password_request,
                       app_user.reset_password_code, organization_user.permissions
                FROM organization_user
                INNER JOIN organization ON organization.id = organization_user.organization_id
                INNER JOIN app_user ON app_user.id = organization_user.user_id
                WHERE organization_user.organization_id = $1 AND organization_user.user_id = $2
                "#,
            )
            .bind(organization.id)
            .bind(user.id)
            .fetch_optional(&mut **connection)
            .await?;

            row.map(|row| {
                let permissions: Json<Vec<OrganizationPermission>> = row.try_get("permissions")?;
                Ok(OrganizationUser {
                    organization: organization_from_row(&row, "organization_")?,
                    user: user_from_row(&row)?,
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
    use crate::{
        business::{
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
        },
        infrastructure::transaction::{
            run_in_transaction,
            sqlx_transaction_manager::{SqlxTransactionManager, current_connection},
            with_transaction_manager,
        },
    };
    use std::{error::Error, sync::Arc};

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

    #[tokio::test]
    async fn retrieves_membership_relationships_from_persistence() -> Result<(), Box<dyn Error>> {
        with_transaction_manager(
            Arc::new(SqlxTransactionManager),
            run_in_transaction(async {
                let organization = OrganizationBuilder::init()
                    .with_name("Stored organization".to_owned())
                    .build();
                let user = UserBuilder::init()
                    .with_username("stored-user".to_owned())
                    .with_email("stored-user@example.com".to_owned())
                    .with_password("stored-password".to_owned())
                    .build();
                let mut membership = OrganizationUser::new(&organization, &user);
                membership.make_admin();

                {
                    let connection = current_connection()?;
                    let mut connection = connection.lock().await;
                    for statement in [
                        "CREATE TEMPORARY TABLE organization (LIKE organization) ON COMMIT DROP",
                        "CREATE TEMPORARY TABLE app_user (LIKE app_user) ON COMMIT DROP",
                        "CREATE TEMPORARY TABLE organization_user (LIKE organization_user INCLUDING INDEXES) ON COMMIT DROP",
                    ] {
                        sqlx::query(statement).execute(&mut **connection).await?;
                    }
                    sqlx::query(
                        "INSERT INTO organization (id, name, slug) VALUES ($1, 'Stored organization', 'stored-organization')",
                    )
                    .bind(organization.id)
                    .execute(&mut **connection)
                    .await?;
                    sqlx::query(
                        r#"
                        INSERT INTO app_user (id, username, email, password, roles,
                                              last_reset_password_request, reset_password_code)
                        VALUES ($1, $2, $3, $4, '["ROLE_USER"]', NULL, NULL)
                        "#,
                    )
                    .bind(user.id)
                    .bind(user.username())
                    .bind(user.email())
                    .bind(user.password())
                    .execute(&mut **connection)
                    .await?;
                }

                let gateway = SqlxOrganizationUserGateway::new();
                gateway.save(&membership).await?;
                let mut lookup_organization = OrganizationBuilder::init()
                    .with_name("Stale organization".to_owned())
                    .build();
                lookup_organization.id = organization.id;
                let mut lookup_user = user.clone();
                lookup_user.update_personal_information(
                    "stale-user".to_owned(),
                    "stale-user@example.com".to_owned(),
                    Some("stale-password".to_owned()),
                );

                assert_eq!(
                    gateway
                        .find_one_by_organization_and_user(&lookup_organization, &lookup_user)
                        .await?,
                    Some(membership)
                );
                assert_eq!(gateway.find_by_user(&lookup_user).await?, vec![organization]);
                assert!(
                    gateway
                        .find_one_by_organization_and_user(
                            &lookup_organization,
                            &UserBuilder::init().build(),
                        )
                        .await?
                        .is_none()
                );
                Ok(())
            }),
        )
        .await
    }
}
