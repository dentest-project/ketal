use super::PathGatewayError;
use crate::business::{entities::path::Path, error::UnexpectedError};
use sqlx::{PgConnection, Row, postgres::PgRow};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub(crate) async fn find_by_ids(
    connection: &mut PgConnection,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, Path>, PathGatewayError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT id, path, slug, summary, parent_id FROM path WHERE id = ANY($1)
            UNION
            SELECT parent.id, parent.path, parent.slug, parent.summary, parent.parent_id
            FROM path AS parent
            INNER JOIN ancestors ON ancestors.parent_id = parent.id
        )
        SELECT id, path, slug, summary, parent_id FROM ancestors
        "#,
    )
    .bind(ids)
    .fetch_all(connection)
    .await?;
    let rows_by_id = rows
        .iter()
        .map(|row| Ok((row.try_get::<Uuid, _>("id")?, row)))
        .collect::<Result<HashMap<_, _>, sqlx::Error>>()?;

    ids.iter()
        .map(|id| Ok((*id, path_from_rows(*id, &rows_by_id)?)))
        .collect()
}

fn path_from_rows(id: Uuid, rows: &HashMap<Uuid, &PgRow>) -> Result<Path, PathGatewayError> {
    let mut ancestry = Vec::new();
    let mut visited = HashSet::new();
    let mut next_id = Some(id);

    while let Some(id) = next_id {
        if !visited.insert(id) {
            return Err(UnexpectedError.into());
        }
        let row = rows.get(&id).ok_or(UnexpectedError)?;
        ancestry.push(*row);
        next_id = row.try_get("parent_id")?;
    }

    let mut path = None;
    for row in ancestry.into_iter().rev() {
        path = Some(Path {
            id: row.try_get("id")?,
            path: row.try_get("path")?,
            slug: row.try_get("slug")?,
            summary: row.try_get("summary")?,
            parent: path.map(Box::new),
        });
    }
    path.ok_or_else(|| UnexpectedError.into())
}
