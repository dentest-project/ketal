use crate::{
    business::{
        entities::{
            organization::organization_gateway::sqlx_organization_gateway::organization_from_row,
            path::{Path, path_gateway::sqlx_path_gateway},
            project::{
                Project, ProjectVisibility,
                project_gateway::{GatewayFuture, ProjectGateway, ProjectGatewayError},
            },
            user::User,
        },
        error::UnexpectedError,
    },
    infrastructure::transaction::sqlx_transaction_manager::current_connection,
};
use sqlx::{Row, postgres::PgRow};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct SqlxProjectGateway;

impl SqlxProjectGateway {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectGateway for SqlxProjectGateway {
    fn find_by_user_without_organization<'a>(
        &'a self,
        user: &'a User,
    ) -> GatewayFuture<'a, Vec<Project>> {
        Box::pin(async move {
            let connection = current_connection()?;
            let mut connection = connection.lock().await;
            let rows = sqlx::query(
                r#"
                SELECT project.id, project.title, project.slug,
                       project.visibility::text AS visibility, project.root_path_id,
                       project.organization_id,
                       organization.id AS related_organization_id,
                       organization.name AS related_organization_name,
                       organization.slug AS related_organization_slug
                FROM project
                INNER JOIN project_user ON project_user.project_id = project.id
                LEFT JOIN organization ON organization.id = project.organization_id
                WHERE project_user.user_id = $1 AND project.organization_id IS NULL
                ORDER BY LOWER(project.title), project.id
                "#,
            )
            .bind(user.id)
            .fetch_all(&mut **connection)
            .await?;

            let root_path_ids = rows
                .iter()
                .map(|row| row.try_get::<Option<Uuid>, _>("root_path_id"))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let paths = sqlx_path_gateway::find_by_ids(&mut connection, &root_path_ids).await?;

            rows.iter()
                .map(|row| project_from_row(row, &paths))
                .collect()
        })
    }
}

fn project_from_row(
    row: &PgRow,
    paths: &HashMap<Uuid, Path>,
) -> Result<Project, ProjectGatewayError> {
    let visibility = match row.try_get::<&str, _>("visibility")? {
        "public" => ProjectVisibility::Public,
        "internal" => ProjectVisibility::Internal,
        "private" => ProjectVisibility::Private,
        _ => return Err(UnexpectedError.into()),
    };
    let root_path = row
        .try_get::<Option<Uuid>, _>("root_path_id")?
        .map(|id| paths.get(&id).cloned().ok_or(UnexpectedError))
        .transpose()?;
    let organization = row
        .try_get::<Option<Uuid>, _>("organization_id")?
        .map(|_| organization_from_row(row, "related_organization_"))
        .transpose()?;

    Ok(Project {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        slug: row.try_get("slug")?,
        visibility,
        root_path,
        organization,
    })
}

#[cfg(test)]
mod tests {
    use super::SqlxProjectGateway;
    use crate::{
        business::{
            EntityBuilder,
            entities::{
                organization::OrganizationBuilder,
                path::PathBuilder,
                project::{
                    ProjectBuilder, ProjectVisibility,
                    project_gateway::{ProjectGateway, ProjectGatewayError},
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
        let user = UserBuilder::init().build();

        assert!(matches!(
            SqlxProjectGateway::new()
                .find_by_user_without_organization(&user)
                .await,
            Err(ProjectGatewayError::Unexpected(_))
        ));
    }

    #[tokio::test]
    async fn retrieves_complete_projects_without_an_organization() -> Result<(), Box<dyn Error>> {
        with_transaction_manager(
            Arc::new(SqlxTransactionManager),
            run_in_transaction(async {
                let user = UserBuilder::init().build();
                let other_user = UserBuilder::init().build();
                let organization = OrganizationBuilder::init()
                    .with_name("Project organization".to_owned())
                    .build();
                let parent = PathBuilder::init()
                    .with_path("Parent path".to_owned())
                    .with_summary("Parent summary".to_owned())
                    .build();
                let root_path = PathBuilder::init()
                    .with_path("Root path".to_owned())
                    .with_summary("Root summary".to_owned())
                    .with_parent(&parent)
                    .build();
                let private_project = ProjectBuilder::init()
                    .with_title("alpha Project".to_owned())
                    .with_visibility(ProjectVisibility::Private)
                    .with_root_path(&root_path)
                    .build();
                let public_project = ProjectBuilder::init()
                    .with_title("Bravo Project".to_owned())
                    .with_visibility(ProjectVisibility::Public)
                    .build();
                let organization_project = ProjectBuilder::init()
                    .with_title("Organization project".to_owned())
                    .with_visibility(ProjectVisibility::Internal)
                    .with_organization(&organization)
                    .build();
                let unrelated_project = ProjectBuilder::init()
                    .with_title("Other user's project".to_owned())
                    .with_visibility(ProjectVisibility::Internal)
                    .build();

                {
                    let connection = current_connection()?;
                    let mut connection = connection.lock().await;
                    for statement in [
                        "CREATE TEMPORARY TABLE organization (LIKE organization) ON COMMIT DROP",
                        "CREATE TEMPORARY TABLE path (LIKE path) ON COMMIT DROP",
                        "CREATE TEMPORARY TABLE project (LIKE project) ON COMMIT DROP",
                        "CREATE TEMPORARY TABLE project_user (LIKE project_user) ON COMMIT DROP",
                    ] {
                        sqlx::query(statement).execute(&mut **connection).await?;
                    }
                    sqlx::query(
                        "INSERT INTO organization (id, name, slug) VALUES ($1, 'Project organization', 'project-organization')",
                    )
                    .bind(organization.id)
                    .execute(&mut **connection)
                    .await?;
                    sqlx::query(
                        r#"
                        INSERT INTO path (id, path, slug, summary, parent_id)
                        VALUES ($1, 'Parent path', 'parent-path', 'Parent summary', NULL),
                               ($2, 'Root path', 'root-path', 'Root summary', $1)
                        "#,
                    )
                    .bind(parent.id)
                    .bind(root_path.id)
                    .execute(&mut **connection)
                    .await?;

                    for (project, visibility, owner) in [
                        (&organization_project, "internal", &user),
                        (&public_project, "public", &user),
                        (&private_project, "private", &user),
                        (&unrelated_project, "internal", &other_user),
                    ] {
                        sqlx::query(
                            r#"
                            INSERT INTO project (id, title, slug, visibility, root_path_id, organization_id)
                            VALUES ($1, $2, $3, $4::text::project_visibility, $5, $6)
                            "#,
                        )
                        .bind(project.id)
                        .bind(&project.title)
                        .bind(&project.slug)
                        .bind(visibility)
                        .bind(project.root_path.as_ref().map(|path| path.id))
                        .bind(project.organization.as_ref().map(|organization| organization.id))
                        .execute(&mut **connection)
                        .await?;
                        sqlx::query(
                            "INSERT INTO project_user (project_id, user_id, permissions, token) VALUES ($1, $2, '[]', '')",
                        )
                        .bind(project.id)
                        .bind(owner.id)
                        .execute(&mut **connection)
                        .await?;
                    }
                }

                let gateway = SqlxProjectGateway::new();
                assert_eq!(
                    gateway.find_by_user_without_organization(&user).await?,
                    vec![private_project, public_project]
                );
                assert_eq!(gateway.find_by_user_without_organization(&other_user).await?, vec![unrelated_project]);
                assert!(gateway.find_by_user_without_organization(&UserBuilder::init().build()).await?.is_empty());

                {
                    let connection = current_connection()?;
                    let mut connection = connection.lock().await;
                    sqlx::query("UPDATE path SET parent_id = $1 WHERE id = $2")
                        .bind(root_path.id)
                        .bind(parent.id)
                        .execute(&mut **connection)
                        .await?;
                }
                assert!(matches!(
                    gateway.find_by_user_without_organization(&user).await,
                    Err(ProjectGatewayError::Unexpected(_))
                ));

                Ok(())
            }),
        )
        .await
    }
}
