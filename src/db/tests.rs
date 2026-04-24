//! Database integration tests using sqlx::test
//!
//! These tests require a PostgreSQL database. Each test runs in a transaction
//! that is rolled back after the test completes.
//!
//! Run with: DATABASE_URL="postgres:///djv_test" cargo test --features ssr

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_imports
)]

use chrono::Utc;
use sqlx::PgPool;

use super::models::*;
use super::queries::*;

fn repo<'a>(
    forge_id: &'a str,
    name: &'a str,
    url: &'a str,
    language: Option<&'a str>,
    stars: i32,
    topics: &'a [String],
) -> NewRepository<'a> {
    NewRepository {
        forge: "github",
        forge_id,
        name,
        description: None,
        url,
        language,
        stars,
        topics,
        updated_at: None,
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_repository_insert(pool: PgPool) {
    let topics = ["rust".to_owned(), "cli".to_owned()];
    let id = upsert_repository(
        &pool,
        &NewRepository {
            forge: "github",
            forge_id: "user/new-repo",
            name: "new-repo",
            description: Some("A new repository"),
            url: "https://github.com/user/new-repo",
            language: Some("Rust"),
            stars: 42,
            topics: &topics,
            updated_at: Some(Utc::now()),
        },
    )
    .await
    .expect("should insert repository");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_repository_update(pool: PgPool) {
    let id1 = upsert_repository(
        &pool,
        &NewRepository {
            forge: "github",
            forge_id: "user/repo",
            name: "repo",
            description: Some("Original description"),
            url: "https://github.com/user/repo",
            language: Some("Rust"),
            stars: 10,
            topics: &[],
            updated_at: None,
        },
    )
    .await
    .expect("should insert");

    let updated_topics = ["updated".to_owned()];
    let id2 = upsert_repository(
        &pool,
        &NewRepository {
            forge: "github",
            forge_id: "user/repo",
            name: "repo",
            description: Some("Updated description"),
            url: "https://github.com/user/repo",
            language: Some("Rust"),
            stars: 100,
            topics: &updated_topics,
            updated_at: Some(Utc::now()),
        },
    )
    .await
    .expect("should update");

    assert_eq!(id1, id2);
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_crate_standalone(pool: PgPool) {
    let keywords = ["cli".to_owned()];
    let categories = ["command-line-utilities".to_owned()];
    let id = upsert_crate(
        &pool,
        &NewCrate {
            name: "my-crate",
            description: Some("A standalone crate"),
            repository_id: None,
            crates_io_url: "https://crates.io/crates/my-crate",
            documentation_url: Some("https://docs.rs/my-crate"),
            downloads: 1000,
            version: Some("1.0.0"),
            keywords: &keywords,
            categories: &categories,
        },
    )
    .await
    .expect("should insert crate");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_crate_with_repo_link(pool: PgPool) {
    let repo_id = upsert_repository(
        &pool,
        &NewRepository {
            forge: "github",
            forge_id: "user/my-crate",
            name: "my-crate",
            description: Some("Source repo"),
            url: "https://github.com/user/my-crate",
            language: Some("Rust"),
            stars: 50,
            topics: &[],
            updated_at: None,
        },
    )
    .await
    .expect("should insert repository");

    let crate_id = upsert_crate(
        &pool,
        &NewCrate {
            name: "my-crate",
            description: Some("A crate with repo"),
            repository_id: Some(repo_id),
            crates_io_url: "https://crates.io/crates/my-crate",
            documentation_url: Some("https://docs.rs/my-crate"),
            downloads: 5000,
            version: Some("2.0.0"),
            keywords: &[],
            categories: &[],
        },
    )
    .await
    .expect("should insert crate");

    assert!(!crate_id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_npm_package_insert(pool: PgPool) {
    let keywords = ["typescript".to_owned()];
    let id = upsert_npm_package(
        &pool,
        &NewNpmPackage {
            name: "my-package",
            scope: Some("@scope"),
            description: Some("An NPM package"),
            repository_id: None,
            npm_url: "https://www.npmjs.com/package/my-package",
            downloads_weekly: 10_000,
            version: Some("3.0.0"),
            keywords: &keywords,
        },
    )
    .await
    .expect("should insert npm package");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_contribution_insert(pool: PgPool) {
    let id = upsert_contribution(
        &pool,
        &NewContribution {
            forge: "github",
            repo_owner: "rust-lang",
            repo_name: "rust",
            repo_url: "https://github.com/rust-lang/rust",
            contribution_type: "pull_request",
            title: Some("Fix compiler bug"),
            url: "https://github.com/rust-lang/rust/pull/12345",
            merged_at: Some(Utc::now()),
        },
    )
    .await
    .expect("should insert contribution");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_contribution_update(pool: PgPool) {
    let id1 = upsert_contribution(
        &pool,
        &NewContribution {
            forge: "github",
            repo_owner: "owner",
            repo_name: "repo",
            repo_url: "https://github.com/owner/repo",
            contribution_type: "pull_request",
            title: Some("Original title"),
            url: "https://github.com/owner/repo/pull/1",
            merged_at: None,
        },
    )
    .await
    .expect("should insert");

    let id2 = upsert_contribution(
        &pool,
        &NewContribution {
            forge: "github",
            repo_owner: "owner",
            repo_name: "repo",
            repo_url: "https://github.com/owner/repo",
            contribution_type: "pull_request",
            title: Some("Updated title"),
            url: "https://github.com/owner/repo/pull/1",
            merged_at: Some(Utc::now()),
        },
    )
    .await
    .expect("should update");

    assert_eq!(id1, id2);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_unfiltered(pool: PgPool) {
    let topics = ["rust".to_owned()];
    upsert_repository(
        &pool,
        &NewRepository {
            forge: "github",
            forge_id: "user/test-repo",
            name: "test-repo",
            description: Some("Test repository"),
            url: "https://github.com/user/test-repo",
            language: Some("Rust"),
            stars: 100,
            topics: &topics,
            updated_at: Some(Utc::now()),
        },
    )
    .await
    .expect("should insert");

    let projects = get_projects(&pool, &ProjectFilters::default())
        .await
        .expect("should query projects");

    assert!(!projects.is_empty());
    assert!(projects.iter().any(|p| p.name == "test-repo"));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_filter_by_kind_crate(pool: PgPool) {
    upsert_crate(
        &pool,
        &NewCrate {
            name: "filter-test-crate",
            description: Some("A crate for testing"),
            repository_id: None,
            crates_io_url: "https://crates.io/crates/filter-test-crate",
            documentation_url: None,
            downloads: 500,
            version: Some("1.0.0"),
            keywords: &[],
            categories: &[],
        },
    )
    .await
    .expect("should insert crate");

    upsert_repository(
        &pool,
        &repo(
            "user/filter-test-repo",
            "filter-test-repo",
            "https://github.com/user/filter-test-repo",
            Some("Python"),
            10,
            &[],
        ),
    )
    .await
    .expect("should insert repo");

    let filters = ProjectFilters {
        kind: Some(ProjectKind::Crate),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    assert!(projects.iter().all(|p| p.kind == ProjectKind::Crate));
    assert!(projects.iter().any(|p| p.name == "filter-test-crate"));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_filter_by_language(pool: PgPool) {
    upsert_repository(
        &pool,
        &repo(
            "user/rust-project",
            "rust-project",
            "https://github.com/user/rust-project",
            Some("Rust"),
            50,
            &[],
        ),
    )
    .await
    .expect("should insert");

    upsert_repository(
        &pool,
        &repo(
            "user/python-project",
            "python-project",
            "https://github.com/user/python-project",
            Some("Python"),
            50,
            &[],
        ),
    )
    .await
    .expect("should insert");

    let filters = ProjectFilters {
        language: Some("Rust".to_owned()),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    assert!(projects
        .iter()
        .all(|p| p.language.as_deref() == Some("Rust") || p.language.as_deref() == Some("rust")));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_filter_by_topic(pool: PgPool) {
    let otel_topics = ["opentelemetry".to_owned(), "tracing".to_owned()];
    upsert_repository(
        &pool,
        &repo(
            "user/otel-project",
            "otel-project",
            "https://github.com/user/otel-project",
            Some("Rust"),
            100,
            &otel_topics,
        ),
    )
    .await
    .expect("should insert");

    let other_topics = ["other".to_owned()];
    upsert_repository(
        &pool,
        &repo(
            "user/other-project",
            "other-project",
            "https://github.com/user/other-project",
            Some("Rust"),
            50,
            &other_topics,
        ),
    )
    .await
    .expect("should insert");

    let filters = ProjectFilters {
        topic: Some("opentelemetry".to_owned()),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    assert!(projects
        .iter()
        .all(|p| p.topics.contains(&"opentelemetry".to_owned())));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_sort_by_name(pool: PgPool) {
    for name in ["zebra", "alpha", "middle"] {
        let forge_id = format!("user/{name}");
        let url = format!("https://github.com/user/{name}");
        upsert_repository(&pool, &repo(&forge_id, name, &url, Some("Rust"), 50, &[]))
            .await
            .expect("should insert");
    }

    let filters = ProjectFilters {
        sort: Some(SortOrder::Name),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    let names: Vec<_> = projects.iter().map(|p| p.name.as_str()).collect();
    let mut sorted_names = names.clone();
    sorted_names.sort();
    assert_eq!(names, sorted_names);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_sort_by_popularity(pool: PgPool) {
    upsert_repository(
        &pool,
        &repo(
            "user/popular",
            "popular",
            "https://github.com/user/popular",
            Some("Rust"),
            1000,
            &[],
        ),
    )
    .await
    .expect("should insert");

    upsert_repository(
        &pool,
        &repo(
            "user/unpopular",
            "unpopular",
            "https://github.com/user/unpopular",
            Some("Rust"),
            10,
            &[],
        ),
    )
    .await
    .expect("should insert");

    let filters = ProjectFilters {
        sort: Some(SortOrder::Popularity),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    if projects.len() >= 2 {
        assert!(projects[0].popularity >= projects[1].popularity);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn get_contributions_with_limit(pool: PgPool) {
    for i in 1..=5 {
        let title = format!("PR {i}");
        let url = format!("https://github.com/owner/repo/pull/{i}");
        upsert_contribution(
            &pool,
            &NewContribution {
                forge: "github",
                repo_owner: "owner",
                repo_name: "repo",
                repo_url: "https://github.com/owner/repo",
                contribution_type: "pull_request",
                title: Some(&title),
                url: &url,
                merged_at: Some(Utc::now()),
            },
        )
        .await
        .expect("should insert");
    }

    let contributions = get_contributions(&pool, 3, 5).await.expect("should query");

    assert!(contributions.len() <= 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_distinct_topics_returns_unique(pool: PgPool) {
    let topics_1 = ["rust".to_owned(), "cli".to_owned()];
    upsert_repository(
        &pool,
        &repo(
            "user/repo1",
            "repo1",
            "https://github.com/user/repo1",
            Some("Rust"),
            50,
            &topics_1,
        ),
    )
    .await
    .expect("should insert");

    let topics_2 = ["rust".to_owned(), "web".to_owned()];
    upsert_repository(
        &pool,
        &repo(
            "user/repo2",
            "repo2",
            "https://github.com/user/repo2",
            Some("Rust"),
            50,
            &topics_2,
        ),
    )
    .await
    .expect("should insert");

    let topics = get_distinct_topics(&pool).await.expect("should query");

    let unique_count = topics.len();
    let mut deduped = topics.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(unique_count, deduped.len());

    assert!(topics.contains(&"rust".to_owned()));
    assert!(topics.contains(&"cli".to_owned()));
    assert!(topics.contains(&"web".to_owned()));
}
