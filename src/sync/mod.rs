pub mod contributions;
pub mod forges;
pub mod registries;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;

pub use contributions::{ContributionsSync, FetchedContribution};
pub use registries::{CrateSummary, CratesIoRegistry};

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

/// All sync sources bundled together
pub struct SyncSources {
    pub forges: Vec<Box<dyn SyncSource>>,
    pub crates_io: Option<CratesIoRegistry>,
    pub contributions: Option<ContributionsSync>,
}

impl SyncSources {
    pub fn is_empty(&self) -> bool {
        self.forges.is_empty() && self.crates_io.is_none() && self.contributions.is_none()
    }
}

#[tracing::instrument(skip(pool, sources))]
pub async fn run_sync(pool: &PgPool, sources: &SyncSources) -> Result<(), SyncError> {
    // Sync forges (repositories)
    for source in &sources.forges {
        sync_forge(pool, source.as_ref()).await?;
    }

    // Sync crates.io
    if let Some(ref crates_io) = sources.crates_io {
        sync_crates(pool, crates_io).await?;
    }

    // Sync contributions
    if let Some(ref contributions) = sources.contributions {
        sync_contributions(pool, contributions).await?;
    }

    Ok(())
}

#[tracing::instrument(skip(pool, source), fields(source = source.name()))]
async fn sync_forge(pool: &PgPool, source: &dyn SyncSource) -> Result<(), SyncError> {
    tracing::info!("starting forge sync");

    let repositories = source.fetch_repositories().await?;
    let count = repositories.len();

    for repo in repositories {
        upsert_repository(pool, &repo).await?;
    }

    tracing::info!(count, "forge sync complete");
    Ok(())
}

#[tracing::instrument(skip(pool, crates_io))]
async fn sync_crates(pool: &PgPool, crates_io: &CratesIoRegistry) -> Result<(), SyncError> {
    tracing::info!("starting crates.io sync");

    let crates = crates_io.fetch_crates().await?;
    let count = crates.len();

    // Batch lookup all repository URLs at once to avoid N+1 queries
    let repo_urls: Vec<String> = crates
        .iter()
        .filter_map(|k| k.repository_url.clone())
        .collect();

    let repo_map = crate::db::get_repositories_by_urls(pool, &repo_urls).await?;

    for krate in crates {
        let repository_id = krate
            .repository_url
            .as_ref()
            .and_then(|url| repo_map.get(url).copied());

        crate::db::upsert_crate(
            pool,
            &krate.name,
            krate.description.as_deref(),
            repository_id,
            &krate.crates_io_url,
            krate.documentation_url.as_deref(),
            krate.downloads,
            krate.version.as_deref(),
            &krate.keywords,
            &krate.categories,
        )
        .await?;

        tracing::debug!(name = %krate.name, "upserted crate");
    }

    tracing::info!(count, "crates.io sync complete");
    Ok(())
}

#[tracing::instrument(skip(pool, contributions_sync))]
async fn sync_contributions(
    pool: &PgPool,
    contributions_sync: &ContributionsSync,
) -> Result<(), SyncError> {
    tracing::info!("starting contributions sync");

    let contributions = contributions_sync.fetch_contributions().await?;
    let count = contributions.len();

    for contrib in contributions {
        crate::db::upsert_contribution(
            pool,
            &contrib.forge,
            &contrib.repo_owner,
            &contrib.repo_name,
            &contrib.repo_url,
            &contrib.contribution_type,
            contrib.title.as_deref(),
            &contrib.url,
            contrib.merged_at,
        )
        .await?;

        tracing::debug!(url = %contrib.url, "upserted contribution");
    }

    tracing::info!(count, "contributions sync complete");
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

pub fn spawn_sync_task(pool: PgPool, sources: SyncSources, config: SyncConfig) {
    if !config.enabled {
        tracing::info!("sync disabled");
        return;
    }

    if sources.is_empty() {
        tracing::info!("no sync sources configured");
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
