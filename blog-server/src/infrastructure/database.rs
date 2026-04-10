use sqlx::Postgres;

const MAX_CONNECTIONS: u32 = 5;

pub async fn run_migrations(pool: &sqlx::Pool<Postgres>) -> Result<(), sqlx::migrate::MigrateError>
{
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn create_pool(database_url: &str) -> Result<sqlx::postgres::PgPool, sqlx::error::Error>
{
    sqlx::postgres::PgPoolOptions::new().max_connections(MAX_CONNECTIONS).connect(database_url).await
}

