use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
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
    let row = sqlx::query(
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
    )
    .bind(forge)
    .bind(forge_id)
    .bind(name)
    .bind(description)
    .bind(url)
    .bind(language)
    .bind(stars)
    .bind(topics)
    .bind(updated_at)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

pub async fn get_repository_by_url(pool: &PgPool, url: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query("SELECT id FROM repositories WHERE url = $1")
        .bind(url)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| r.get("id")))
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
    let row = sqlx::query(
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
    )
    .bind(name)
    .bind(description)
    .bind(repository_id)
    .bind(crates_io_url)
    .bind(documentation_url)
    .bind(downloads)
    .bind(version)
    .bind(keywords)
    .bind(categories)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
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
    let row = sqlx::query(
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
    )
    .bind(name)
    .bind(scope)
    .bind(description)
    .bind(repository_id)
    .bind(npm_url)
    .bind(downloads_weekly)
    .bind(version)
    .bind(keywords)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
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
    let row = sqlx::query(
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
    )
    .bind(forge)
    .bind(repo_owner)
    .bind(repo_name)
    .bind(repo_url)
    .bind(contribution_type)
    .bind(title)
    .bind(url)
    .bind(merged_at)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

pub async fn get_contributions(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<Contribution>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, forge, repo_owner, repo_name, repo_url, contribution_type, title, url, merged_at, synced_at
        FROM contributions
        ORDER BY merged_at DESC NULLS LAST
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Contribution {
            id: row.get("id"),
            forge: row.get("forge"),
            repo_owner: row.get("repo_owner"),
            repo_name: row.get("repo_name"),
            repo_url: row.get("repo_url"),
            contribution_type: row.get("contribution_type"),
            title: row.get("title"),
            url: row.get("url"),
            merged_at: row.get("merged_at"),
            synced_at: row.get("synced_at"),
        })
        .collect())
}

// ============================================================================
// Project view queries
// ============================================================================

pub async fn get_projects(
    pool: &PgPool,
    filters: &ProjectFilters,
) -> Result<Vec<ProjectView>, sqlx::Error> {
    let kind_filter = filters.kind.map(|k| k.to_string());
    let language_filter = filters.language.as_deref();
    let topic_filter = filters.topic.as_deref();

    let order_by = match filters.sort.unwrap_or_default() {
        SortOrder::Popularity => "popularity DESC NULLS LAST",
        SortOrder::Name => "name ASC",
        SortOrder::Updated => "synced_at DESC",
    };

    let query = format!(
        r#"
        SELECT id, kind, name, description, url, language, topics, popularity, synced_at
        FROM projects
        WHERE ($1::TEXT IS NULL OR kind = $1)
          AND ($2::TEXT IS NULL OR LOWER(language) = LOWER($2))
          AND ($3::TEXT IS NULL OR $3 = ANY(topics))
        ORDER BY {}
        LIMIT 50
        "#,
        order_by
    );

    let rows = sqlx::query(&query)
        .bind(kind_filter)
        .bind(language_filter)
        .bind(topic_filter)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let kind_str: String = row.get("kind");
            ProjectView {
                id: row.get("id"),
                kind: kind_str.parse().unwrap_or(ProjectKind::Repo),
                name: row.get("name"),
                description: row.get("description"),
                url: row.get("url"),
                language: row.get("language"),
                topics: row
                    .get::<Option<Vec<String>>, _>("topics")
                    .unwrap_or_default(),
                popularity: row.get::<Option<i32>, _>("popularity").unwrap_or(0),
                synced_at: row.get("synced_at"),
            }
        })
        .collect())
}

pub async fn get_all_projects(pool: &PgPool) -> Result<Vec<ProjectView>, sqlx::Error> {
    get_projects(pool, &ProjectFilters::default()).await
}

pub async fn get_distinct_topics(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT unnest(topics) as topic
        FROM projects
        WHERE topics IS NOT NULL AND array_length(topics, 1) > 0
        ORDER BY topic
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| row.get("topic")).collect())
}
