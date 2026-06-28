use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::LazyLock;

static SHARED_PG_POOL: LazyLock<PgPool> = LazyLock::new(|| {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url())
        .expect("database URL should be valid")
});

fn database_url() -> String {
    let _ = dotenvy::dotenv();

    std::env::var("DATABASE_URL").unwrap_or_else(|error| {
        panic!("DATABASE_URL must be set in the environment or .env: {error}")
    })
}

pub(crate) fn shared_pg_pool() -> &'static PgPool {
    &SHARED_PG_POOL
}
