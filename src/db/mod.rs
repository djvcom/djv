pub mod models;
pub mod queries;

#[cfg(test)]
mod tests;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub use models::*;
pub use queries::*;

/// Initialise the database connection pool with a URL.
///
/// # Errors
/// Returns a [`sqlx::Error`] if the pool cannot be created or the connect-check fails.
pub async fn init_pool_with_url(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Run database migrations.
///
/// # Errors
/// Returns a [`sqlx::migrate::MigrateError`] if any migration fails to apply.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
