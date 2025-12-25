pub mod forges;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;

/// Repository data fetched from a forge (before database insertion)
#[derive(Debug, Clone)]
pub struct FetchedRepository {
    pub forge: String,
    pub forge_id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub language: Option<String>,
    pub stars: i32,
    pub topics: Vec<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait SyncSource: Send + Sync {
    fn name(&self) -> &'static str;

    async fn fetch_repositories(&self) -> Result<Vec<FetchedRepository>, SyncError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("{0}")]
    Other(String),
}

pub struct SyncConfig {
    pub enabled: bool,
    pub interval_secs: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600,
        }
    }
}

impl SyncConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("DJV_SYNC_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        let interval_secs = std::env::var("DJV_SYNC_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        Self {
            enabled,
            interval_secs,
        }
    }
}

#[tracing::instrument(skip(pool, sources))]
pub async fn run_sync(pool: &PgPool, sources: &[Box<dyn SyncSource>]) -> Result<(), SyncError> {
    for source in sources {
        sync_source(pool, source.as_ref()).await?;
    }
    Ok(())
}

#[tracing::instrument(skip(pool, source), fields(source = source.name()))]
async fn sync_source(pool: &PgPool, source: &dyn SyncSource) -> Result<(), SyncError> {
    tracing::info!("starting sync");

    let repositories = source.fetch_repositories().await?;
    let count = repositories.len();

    for repo in repositories {
        upsert_repository(pool, &repo).await?;
    }

    tracing::info!(count, "sync complete");
    Ok(())
}

#[tracing::instrument(skip(pool, repo), fields(repo.name = %repo.name, repo.forge = %repo.forge))]
async fn upsert_repository(pool: &PgPool, repo: &FetchedRepository) -> Result<(), SyncError> {
    crate::db::upsert_repository(
        pool,
        &repo.forge,
        &repo.forge_id,
        &repo.name,
        repo.description.as_deref(),
        &repo.url,
        repo.language.as_deref(),
        repo.stars,
        &repo.topics,
        repo.updated_at,
    )
    .await?;

    tracing::debug!("upserted repository");
    Ok(())
}

pub fn spawn_sync_task(pool: PgPool, sources: Vec<Box<dyn SyncSource>>, config: SyncConfig) {
    if !config.enabled {
        tracing::info!("sync disabled");
        return;
    }

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.interval_secs));

        // Run immediately on startup
        if let Err(e) = run_sync(&pool, &sources).await {
            tracing::error!(error = %e, "initial sync failed");
        }

        loop {
            ticker.tick().await;

            if let Err(e) = run_sync(&pool, &sources).await {
                tracing::error!(error = %e, "sync failed");
            }
        }
    });

    tracing::info!(interval_secs = config.interval_secs, "sync task spawned");
}
