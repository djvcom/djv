use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{Contribution, ProjectFilters, ProjectKind, ProjectView, SortOrder};

// ============================================================================
// Repository queries
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn upsert_repository(
    pool: &PgPool,
    forge: &str,
    forge_id: &str,
    name: &str,
    description: Option<&str>,
    url: &str,
    language: Option<&str>,
    stars: i32,
    topics: &[String],
    updated_at: Option<DateTime<Utc>>,
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO repositories (forge, forge_id, name, description, url, language, stars, topics, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
        ON CONFLICT (forge, forge_id) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            url = EXCLUDED.url,
            language = EXCLUDED.language,
            stars = EXCLUDED.stars,
            topics = EXCLUDED.topics,
            updated_at = EXCLUDED.updated_at,
            synced_at = now()
        RETURNING id
        "#,
        forge,
        forge_id,
        name,
        description,
        url,
        language,
        stars,
        topics,
        updated_at,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn get_repository_by_url(pool: &PgPool, url: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let id = sqlx::query_scalar!("SELECT id FROM repositories WHERE url = $1", url)
        .fetch_optional(pool)
        .await?;

    Ok(id)
}

/// Delete repositories from a forge that are no longer present in the source.
/// Returns the number of deleted rows.
pub async fn delete_stale_repositories(
    pool: &PgPool,
    forge: &str,
    synced_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM repositories WHERE forge = $1 AND id != ALL($2)",
        forge,
        synced_ids,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn get_repositories_by_urls(
    pool: &PgPool,
    urls: &[String],
) -> Result<std::collections::HashMap<String, Uuid>, sqlx::Error> {
    if urls.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows = sqlx::query!("SELECT url, id FROM repositories WHERE url = ANY($1)", urls)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| (r.url, r.id)).collect())
}

// ============================================================================
// Crate queries
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn upsert_crate(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    repository_id: Option<Uuid>,
    crates_io_url: &str,
    documentation_url: Option<&str>,
    downloads: i32,
    version: Option<&str>,
    keywords: &[String],
    categories: &[String],
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO crates (name, description, repository_id, crates_io_url, documentation_url, downloads, version, keywords, categories, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
        ON CONFLICT (name) DO UPDATE SET
            description = EXCLUDED.description,
            repository_id = EXCLUDED.repository_id,
            crates_io_url = EXCLUDED.crates_io_url,
            documentation_url = EXCLUDED.documentation_url,
            downloads = EXCLUDED.downloads,
            version = EXCLUDED.version,
            keywords = EXCLUDED.keywords,
            categories = EXCLUDED.categories,
            synced_at = now()
        RETURNING id
        "#,
        name,
        description,
        repository_id,
        crates_io_url,
        documentation_url,
        downloads,
        version,
        keywords,
        categories,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ============================================================================
// NPM package queries
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn upsert_npm_package(
    pool: &PgPool,
    name: &str,
    scope: Option<&str>,
    description: Option<&str>,
    repository_id: Option<Uuid>,
    npm_url: &str,
    downloads_weekly: i32,
    version: Option<&str>,
    keywords: &[String],
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO npm_packages (name, scope, description, repository_id, npm_url, downloads_weekly, version, keywords, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
        ON CONFLICT (name) DO UPDATE SET
            scope = EXCLUDED.scope,
            description = EXCLUDED.description,
            repository_id = EXCLUDED.repository_id,
            npm_url = EXCLUDED.npm_url,
            downloads_weekly = EXCLUDED.downloads_weekly,
            version = EXCLUDED.version,
            keywords = EXCLUDED.keywords,
            synced_at = now()
        RETURNING id
        "#,
        name,
        scope,
        description,
        repository_id,
        npm_url,
        downloads_weekly,
        version,
        keywords,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ============================================================================
// Contribution queries
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn upsert_contribution(
    pool: &PgPool,
    forge: &str,
    repo_owner: &str,
    repo_name: &str,
    repo_url: &str,
    contribution_type: &str,
    title: Option<&str>,
    url: &str,
    merged_at: Option<DateTime<Utc>>,
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO contributions (forge, repo_owner, repo_name, repo_url, contribution_type, title, url, merged_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
        ON CONFLICT (forge, repo_owner, repo_name, url) DO UPDATE SET
            repo_url = EXCLUDED.repo_url,
            contribution_type = EXCLUDED.contribution_type,
            title = EXCLUDED.title,
            merged_at = EXCLUDED.merged_at,
            synced_at = now()
        RETURNING id
        "#,
        forge,
        repo_owner,
        repo_name,
        repo_url,
        contribution_type,
        title,
        url,
        merged_at,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn get_contributions(
    pool: &PgPool,
    limit: i64,
    max_age_years: i32,
) -> Result<Vec<Contribution>, sqlx::Error> {
    let max_age = f64::from(max_age_years);
    let rows = sqlx::query!(
        r#"
        SELECT id, forge, repo_owner, repo_name, repo_url, contribution_type, title, url, merged_at, synced_at
        FROM contributions
        WHERE merged_at IS NULL OR merged_at > NOW() - INTERVAL '1 year' * $2
        ORDER BY merged_at DESC NULLS LAST
        LIMIT $1
        "#,
        limit,
        max_age,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Contribution {
            id: row.id,
            forge: row.forge,
            repo_owner: row.repo_owner,
            repo_name: row.repo_name,
            repo_url: row.repo_url,
            contribution_type: row.contribution_type,
            title: row.title,
            url: row.url,
            merged_at: row.merged_at,
            synced_at: row.synced_at,
        })
        .collect())
}

// ============================================================================
// Project view queries
// ============================================================================

/// Internal struct for database rows from projects view
struct ProjectRow {
    id: Uuid,
    kind: String,
    name: String,
    description: Option<String>,
    url: String,
    language: Option<String>,
    topics: Option<Vec<String>>,
    popularity: Option<i32>,
    version: Option<String>,
    commit_count: Option<i32>,
    updated_at: Option<DateTime<Utc>>,
    synced_at: DateTime<Utc>,
}

impl From<ProjectRow> for ProjectView {
    fn from(row: ProjectRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind.parse().unwrap_or(ProjectKind::Repo),
            name: row.name,
            description: row.description,
            url: row.url,
            language: row.language,
            topics: row.topics.unwrap_or_default(),
            popularity: row.popularity.unwrap_or(0),
            version: row.version,
            commit_count: row.commit_count,
            updated_at: row.updated_at,
            synced_at: row.synced_at,
        }
    }
}

pub async fn get_projects(
    pool: &PgPool,
    filters: &ProjectFilters,
) -> Result<Vec<ProjectView>, sqlx::Error> {
    let kind_filter = filters.kind.map(|k| k.to_string());
    let language_filter = filters.language.clone();
    let topic_filter = filters.topic.clone();
    // Use provided limit, or a large number to effectively mean "no limit"
    let limit = filters.limit.unwrap_or(1000) as i64;

    let rows = match filters.sort.unwrap_or_default() {
        SortOrder::Popularity => {
            sqlx::query_as!(
                ProjectRow,
                r#"
                SELECT
                    id as "id!",
                    kind as "kind!",
                    name as "name!",
                    description,
                    url as "url!",
                    language,
                    topics,
                    popularity,
                    version,
                    commit_count,
                    updated_at,
                    synced_at as "synced_at!"
                FROM projects
                WHERE ($1::TEXT IS NULL OR kind = $1)
                  AND ($2::TEXT IS NULL OR LOWER(language) = LOWER($2))
                  AND ($3::TEXT IS NULL OR $3 = ANY(topics))
                ORDER BY popularity DESC NULLS LAST
                LIMIT $4
                "#,
                kind_filter,
                language_filter,
                topic_filter,
                limit,
            )
            .fetch_all(pool)
            .await?
        }
        SortOrder::Name => {
            sqlx::query_as!(
                ProjectRow,
                r#"
                SELECT
                    id as "id!",
                    kind as "kind!",
                    name as "name!",
                    description,
                    url as "url!",
                    language,
                    topics,
                    popularity,
                    version,
                    commit_count,
                    updated_at,
                    synced_at as "synced_at!"
                FROM projects
                WHERE ($1::TEXT IS NULL OR kind = $1)
                  AND ($2::TEXT IS NULL OR LOWER(language) = LOWER($2))
                  AND ($3::TEXT IS NULL OR $3 = ANY(topics))
                ORDER BY name ASC
                LIMIT $4
                "#,
                kind_filter,
                language_filter,
                topic_filter,
                limit,
            )
            .fetch_all(pool)
            .await?
        }
        SortOrder::Updated => {
            sqlx::query_as!(
                ProjectRow,
                r#"
                SELECT
                    id as "id!",
                    kind as "kind!",
                    name as "name!",
                    description,
                    url as "url!",
                    language,
                    topics,
                    popularity,
                    version,
                    commit_count,
                    updated_at,
                    synced_at as "synced_at!"
                FROM projects
                WHERE ($1::TEXT IS NULL OR kind = $1)
                  AND ($2::TEXT IS NULL OR LOWER(language) = LOWER($2))
                  AND ($3::TEXT IS NULL OR $3 = ANY(topics))
                ORDER BY synced_at DESC
                LIMIT $4
                "#,
                kind_filter,
                language_filter,
                topic_filter,
                limit,
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(rows.into_iter().map(ProjectView::from).collect())
}

pub async fn get_all_projects(pool: &PgPool) -> Result<Vec<ProjectView>, sqlx::Error> {
    get_projects(pool, &ProjectFilters::default()).await
}

pub async fn get_distinct_topics(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT unnest(topics) as "topic!"
        FROM projects
        WHERE topics IS NOT NULL AND array_length(topics, 1) > 0
        ORDER BY 1
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
