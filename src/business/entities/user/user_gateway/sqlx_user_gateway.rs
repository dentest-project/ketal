use crate::{
    business::entities::user::{
        User,
        user_gateway::{GatewayFuture, GatewayResult, UserGateway},
    },
    database::shared_pg_pool,
};
use serde_json::json;
use sqlx::{
    Row,
    postgres::PgRow,
    types::{
        Json,
        chrono::{DateTime, Utc},
    },
};
use uuid::Uuid;
#[derive(Clone, Default)]
pub struct SqlxUserGateway;

impl SqlxUserGateway {
    pub fn new() -> Self {
        Self
    }
}

impl UserGateway for SqlxUserGateway {
    fn find_one_by_id(&self, id: Uuid) -> GatewayFuture<'_, Option<User>> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT
                    id,
                    username,
                    email,
                    password,
                    last_reset_password_request,
                    reset_password_code
                FROM app_user
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(shared_pg_pool())
            .await?;

            row.map(|row| user_from_row(&row)).transpose()
        })
    }

    fn save<'a>(&'a self, user: &'a User) -> GatewayFuture<'a, ()> {
        let id = user.id;
        let username = user.username.clone();
        let email = user.email.clone();
        let password = user.password.clone();
        let last_reset_password_request =
            user.last_reset_password_request.map(DateTime::<Utc>::from);
        let reset_password_code = user.reset_password_code.clone();

        Box::pin(async move {
            sqlx::query(
                r#"
                INSERT INTO app_user (
                    id,
                    username,
                    email,
                    password,
                    roles,
                    last_reset_password_request,
                    reset_password_code
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE
                SET
                    username = EXCLUDED.username,
                    email = EXCLUDED.email,
                    password = EXCLUDED.password,
                    last_reset_password_request = EXCLUDED.last_reset_password_request,
                    reset_password_code = EXCLUDED.reset_password_code
                "#,
            )
            .bind(id)
            .bind(username)
            .bind(email)
            .bind(password)
            .bind(Json(json!(["ROLE_USER"])))
            .bind(last_reset_password_request)
            .bind(reset_password_code)
            .execute(shared_pg_pool())
            .await?;

            Ok(())
        })
    }

    fn find_one_by_username<'a>(&'a self, username: &'a str) -> GatewayFuture<'a, Option<User>> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT
                    id,
                    username,
                    email,
                    password,
                    last_reset_password_request,
                    reset_password_code
                FROM app_user
                WHERE LOWER(username) = LOWER($1)
                LIMIT 1
                "#,
            )
            .bind(username)
            .fetch_optional(shared_pg_pool())
            .await?;

            row.map(|row| user_from_row(&row)).transpose()
        })
    }

    fn find_one_by_email_or_username<'a>(
        &'a self,
        email: &'a str,
        username: &'a str,
    ) -> GatewayFuture<'a, Option<User>> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT
                    id,
                    username,
                    email,
                    password,
                    last_reset_password_request,
                    reset_password_code
                FROM app_user
                WHERE LOWER(email) = LOWER($1) OR LOWER(username) = LOWER($2)
                LIMIT 1
                "#,
            )
            .bind(email)
            .bind(username)
            .fetch_optional(shared_pg_pool())
            .await?;

            row.map(|row| user_from_row(&row)).transpose()
        })
    }

    fn find_one_by_email_or_username_excluding_user<'a>(
        &'a self,
        email: &'a str,
        username: &'a str,
        excluded_user: &'a User,
    ) -> GatewayFuture<'a, Option<User>> {
        let excluded_user_id = excluded_user.id;

        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT
                    id,
                    username,
                    email,
                    password,
                    last_reset_password_request,
                    reset_password_code
                FROM app_user
                WHERE id != $1
                    AND (LOWER(email) = LOWER($2) OR LOWER(username) = LOWER($3))
                LIMIT 1
                "#,
            )
            .bind(excluded_user_id)
            .bind(email)
            .bind(username)
            .fetch_optional(shared_pg_pool())
            .await?;

            row.map(|row| user_from_row(&row)).transpose()
        })
    }

    fn find_one_by_reset_password_code<'a>(
        &'a self,
        reset_password_code: &'a str,
    ) -> GatewayFuture<'a, Option<User>> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT
                    id,
                    username,
                    email,
                    password,
                    last_reset_password_request,
                    reset_password_code
                FROM app_user
                WHERE reset_password_code = $1
                LIMIT 1
                "#,
            )
            .bind(reset_password_code)
            .fetch_optional(shared_pg_pool())
            .await?;

            row.map(|row| user_from_row(&row)).transpose()
        })
    }
}

fn user_from_row(row: &PgRow) -> GatewayResult<User> {
    Ok(User {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        email: row.try_get("email")?,
        password: row.try_get("password")?,
        last_reset_password_request: row
            .try_get::<Option<DateTime<Utc>>, _>("last_reset_password_request")?
            .map(Into::into),
        reset_password_code: row.try_get("reset_password_code")?,
    })
}

#[cfg(test)]
mod tests {
    use super::SqlxUserGateway;
    use crate::{
        business::{
            EntityBuilder,
            entities::user::{UserBuilder, user_gateway::UserGateway},
        },
        database::shared_pg_pool,
    };
    use sqlx::Row;
    use std::{error::Error, io};
    use uuid::Uuid;

    #[tokio::test]
    async fn saves_and_finds_user_in_app_user() -> Result<(), Box<dyn Error>> {
        let suffix = Uuid::new_v4();
        let mut user = UserBuilder::init()
            .with_username(format!("gateway-test-{suffix}"))
            .with_email(format!("sqlx-user-gateway-test-{suffix}@example.com"))
            .with_password("secret".to_owned())
            .build();
        let reset_password_code = format!("reset-{suffix}");
        user.define_reset_password_code(reset_password_code.clone());

        let gateway = SqlxUserGateway::new();

        gateway.save(&user).await?;

        let row = sqlx::query(
            r#"
            SELECT username, email, password
            FROM app_user
            WHERE id = $1
            "#,
        )
        .bind(user.id)
        .fetch_one(shared_pg_pool())
        .await?;

        assert_eq!(row.try_get::<String, _>("username")?, user.username);
        assert_eq!(row.try_get::<String, _>("email")?, user.email);
        assert_eq!(row.try_get::<String, _>("password")?, user.password);

        let found_user = gateway
            .find_one_by_email_or_username(&user.email.to_uppercase(), "unused-username")
            .await?
            .ok_or_else(|| {
                io::Error::other("saved user should be found by email without matching case")
            })?;

        assert_eq!(found_user.id, user.id);
        assert_eq!(found_user.username, user.username);
        assert_eq!(found_user.email, user.email);
        assert_eq!(found_user.password, user.password);

        assert_eq!(gateway.find_one_by_id(user.id).await?, Some(found_user));
        assert!(gateway.find_one_by_id(Uuid::new_v4()).await?.is_none());

        let found_by_username = gateway
            .find_one_by_username(&user.username.to_uppercase())
            .await?
            .ok_or_else(|| {
                io::Error::other("saved user should be found by username without matching case")
            })?;

        assert_eq!(found_by_username.id, user.id);
        assert_eq!(found_by_username.username, user.username);
        assert_eq!(found_by_username.email, user.email);
        assert_eq!(found_by_username.password, user.password);

        let excluded_user = gateway
            .find_one_by_email_or_username_excluding_user(&user.email, &user.username, &user)
            .await?;
        assert!(excluded_user.is_none());

        let found_by_reset_password_code = gateway
            .find_one_by_reset_password_code(&reset_password_code)
            .await?
            .ok_or_else(|| io::Error::other("saved user should be found by reset password code"))?;

        assert_eq!(found_by_reset_password_code.id, user.id);
        assert_eq!(found_by_reset_password_code.username, user.username);
        assert_eq!(found_by_reset_password_code.email, user.email);
        assert_eq!(found_by_reset_password_code.password, user.password);

        sqlx::query("DELETE FROM app_user WHERE id = $1")
            .bind(user.id)
            .execute(shared_pg_pool())
            .await?;

        Ok(())
    }
}
