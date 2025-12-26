use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("sync error: {0}")]
    Sync(#[from] crate::sync::SyncError),

    #[error("not found: {0}")]
    NotFound(String),
}
