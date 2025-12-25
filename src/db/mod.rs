pub mod models;
pub mod queries;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub use models::*;
pub use queries::*;

/// Initialise the database connection pool with a URL
pub async fn init_pool_with_url(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Initialise the database connection pool from DATABASE_URL env var
pub async fn init_pool() -> Result<PgPool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    init_pool_with_url(&database_url).await
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
