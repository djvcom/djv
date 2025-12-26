//! Database integration tests using sqlx::test
//!
//! These tests require a PostgreSQL database. Each test runs in a transaction
//! that is rolled back after the test completes.
//!
//! Run with: DATABASE_URL="postgres:///djv_test" cargo test --features ssr

use chrono::Utc;
use sqlx::PgPool;

use super::models::*;
use super::queries::*;

// ============================================================================
// Repository upsert tests
// ============================================================================

#[sqlx::test(migrations = "./migrations")]
async fn upsert_repository_insert(pool: PgPool) {
    let id = upsert_repository(
        &pool,
        "github",
        "user/new-repo",
        "new-repo",
        Some("A new repository"),
        "https://github.com/user/new-repo",
        Some("Rust"),
        42,
        &["rust".to_string(), "cli".to_string()],
        Some(Utc::now()),
    )
    .await
    .expect("should insert repository");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_repository_update(pool: PgPool) {
    // Insert initial
    let id1 = upsert_repository(
        &pool,
        "github",
        "user/repo",
        "repo",
        Some("Original description"),
        "https://github.com/user/repo",
        Some("Rust"),
        10,
        &[],
        None,
    )
    .await
    .expect("should insert");

    // Update same repo
    let id2 = upsert_repository(
        &pool,
        "github",
        "user/repo",
        "repo",
        Some("Updated description"),
        "https://github.com/user/repo",
        Some("Rust"),
        100,
        &["updated".to_string()],
        Some(Utc::now()),
    )
    .await
    .expect("should update");

    // Should return same ID (upsert)
    assert_eq!(id1, id2);
}

// ============================================================================
// Crate upsert tests
// ============================================================================

#[sqlx::test(migrations = "./migrations")]
async fn upsert_crate_standalone(pool: PgPool) {
    let id = upsert_crate(
        &pool,
        "my-crate",
        Some("A standalone crate"),
        None, // No repository link
        "https://crates.io/crates/my-crate",
        Some("https://docs.rs/my-crate"),
        1000,
        Some("1.0.0"),
        &["cli".to_string()],
        &["command-line-utilities".to_string()],
    )
    .await
    .expect("should insert crate");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_crate_with_repo_link(pool: PgPool) {
    // First create a repository
    let repo_id = upsert_repository(
        &pool,
        "github",
        "user/my-crate",
        "my-crate",
        Some("Source repo"),
        "https://github.com/user/my-crate",
        Some("Rust"),
        50,
        &[],
        None,
    )
    .await
    .expect("should insert repository");

    // Then create crate linked to it
    let crate_id = upsert_crate(
        &pool,
        "my-crate",
        Some("A crate with repo"),
        Some(repo_id),
        "https://crates.io/crates/my-crate",
        Some("https://docs.rs/my-crate"),
        5000,
        Some("2.0.0"),
        &[],
        &[],
    )
    .await
    .expect("should insert crate");

    assert!(!crate_id.is_nil());
}

// ============================================================================
// NPM package upsert tests
// ============================================================================

#[sqlx::test(migrations = "./migrations")]
async fn upsert_npm_package_insert(pool: PgPool) {
    let id = upsert_npm_package(
        &pool,
        "my-package",
        Some("@scope"),
        Some("An NPM package"),
        None,
        "https://www.npmjs.com/package/my-package",
        10000,
        Some("3.0.0"),
        &["typescript".to_string()],
    )
    .await
    .expect("should insert npm package");

    assert!(!id.is_nil());
}

// ============================================================================
// Contribution upsert tests
// ============================================================================

#[sqlx::test(migrations = "./migrations")]
async fn upsert_contribution_insert(pool: PgPool) {
    let id = upsert_contribution(
        &pool,
        "github",
        "rust-lang",
        "rust",
        "https://github.com/rust-lang/rust",
        "pull_request",
        Some("Fix compiler bug"),
        "https://github.com/rust-lang/rust/pull/12345",
        Some(Utc::now()),
    )
    .await
    .expect("should insert contribution");

    assert!(!id.is_nil());
}

#[sqlx::test(migrations = "./migrations")]
async fn upsert_contribution_update(pool: PgPool) {
    let id1 = upsert_contribution(
        &pool,
        "github",
        "owner",
        "repo",
        "https://github.com/owner/repo",
        "pull_request",
        Some("Original title"),
        "https://github.com/owner/repo/pull/1",
        None,
    )
    .await
    .expect("should insert");

    let id2 = upsert_contribution(
        &pool,
        "github",
        "owner",
        "repo",
        "https://github.com/owner/repo",
        "pull_request",
        Some("Updated title"),
        "https://github.com/owner/repo/pull/1",
        Some(Utc::now()),
    )
    .await
    .expect("should update");

    assert_eq!(id1, id2);
}

// ============================================================================
// Project query tests
// ============================================================================

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_unfiltered(pool: PgPool) {
    // Insert a repository (will show in projects view)
    upsert_repository(
        &pool,
        "github",
        "user/test-repo",
        "test-repo",
        Some("Test repository"),
        "https://github.com/user/test-repo",
        Some("Rust"),
        100,
        &["rust".to_string()],
        Some(Utc::now()),
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
    // Insert a crate
    upsert_crate(
        &pool,
        "filter-test-crate",
        Some("A crate for testing"),
        None,
        "https://crates.io/crates/filter-test-crate",
        None,
        500,
        Some("1.0.0"),
        &[],
        &[],
    )
    .await
    .expect("should insert crate");

    // Insert a repo
    upsert_repository(
        &pool,
        "github",
        "user/filter-test-repo",
        "filter-test-repo",
        None,
        "https://github.com/user/filter-test-repo",
        Some("Python"),
        10,
        &[],
        None,
    )
    .await
    .expect("should insert repo");

    // Filter by crate
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
    // Insert Rust repo
    upsert_repository(
        &pool,
        "github",
        "user/rust-project",
        "rust-project",
        None,
        "https://github.com/user/rust-project",
        Some("Rust"),
        50,
        &[],
        None,
    )
    .await
    .expect("should insert");

    // Insert Python repo
    upsert_repository(
        &pool,
        "github",
        "user/python-project",
        "python-project",
        None,
        "https://github.com/user/python-project",
        Some("Python"),
        50,
        &[],
        None,
    )
    .await
    .expect("should insert");

    // Filter by Rust
    let filters = ProjectFilters {
        language: Some("Rust".to_string()),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    assert!(projects
        .iter()
        .all(|p| p.language.as_deref() == Some("Rust") || p.language.as_deref() == Some("rust")));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_filter_by_topic(pool: PgPool) {
    // Insert repo with topic
    upsert_repository(
        &pool,
        "github",
        "user/otel-project",
        "otel-project",
        None,
        "https://github.com/user/otel-project",
        Some("Rust"),
        100,
        &["opentelemetry".to_string(), "tracing".to_string()],
        None,
    )
    .await
    .expect("should insert");

    // Insert repo without topic
    upsert_repository(
        &pool,
        "github",
        "user/other-project",
        "other-project",
        None,
        "https://github.com/user/other-project",
        Some("Rust"),
        50,
        &["other".to_string()],
        None,
    )
    .await
    .expect("should insert");

    // Filter by topic
    let filters = ProjectFilters {
        topic: Some("opentelemetry".to_string()),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    assert!(projects
        .iter()
        .all(|p| p.topics.contains(&"opentelemetry".to_string())));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_sort_by_name(pool: PgPool) {
    // Insert repos with different names
    for name in ["zebra", "alpha", "middle"] {
        upsert_repository(
            &pool,
            "github",
            &format!("user/{}", name),
            name,
            None,
            &format!("https://github.com/user/{}", name),
            Some("Rust"),
            50,
            &[],
            None,
        )
        .await
        .expect("should insert");
    }

    let filters = ProjectFilters {
        sort: Some(SortOrder::Name),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    // Check alphabetical order
    let names: Vec<_> = projects.iter().map(|p| p.name.as_str()).collect();
    let mut sorted_names = names.clone();
    sorted_names.sort();
    assert_eq!(names, sorted_names);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_projects_sort_by_popularity(pool: PgPool) {
    // Insert repos with different star counts
    upsert_repository(
        &pool,
        "github",
        "user/popular",
        "popular",
        None,
        "https://github.com/user/popular",
        Some("Rust"),
        1000,
        &[],
        None,
    )
    .await
    .expect("should insert");

    upsert_repository(
        &pool,
        "github",
        "user/unpopular",
        "unpopular",
        None,
        "https://github.com/user/unpopular",
        Some("Rust"),
        10,
        &[],
        None,
    )
    .await
    .expect("should insert");

    let filters = ProjectFilters {
        sort: Some(SortOrder::Popularity),
        ..Default::default()
    };

    let projects = get_projects(&pool, &filters).await.expect("should query");

    // First should be more popular
    if projects.len() >= 2 {
        assert!(projects[0].popularity >= projects[1].popularity);
    }
}

// ============================================================================
// Contributions query tests
// ============================================================================

#[sqlx::test(migrations = "./migrations")]
async fn get_contributions_with_limit(pool: PgPool) {
    // Insert multiple contributions
    for i in 1..=5 {
        upsert_contribution(
            &pool,
            "github",
            "owner",
            "repo",
            "https://github.com/owner/repo",
            "pull_request",
            Some(&format!("PR {}", i)),
            &format!("https://github.com/owner/repo/pull/{}", i),
            Some(Utc::now()),
        )
        .await
        .expect("should insert");
    }

    let contributions = get_contributions(&pool, 3, 5).await.expect("should query");

    assert!(contributions.len() <= 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_distinct_topics_returns_unique(pool: PgPool) {
    // Insert repos with overlapping topics
    upsert_repository(
        &pool,
        "github",
        "user/repo1",
        "repo1",
        None,
        "https://github.com/user/repo1",
        Some("Rust"),
        50,
        &["rust".to_string(), "cli".to_string()],
        None,
    )
    .await
    .expect("should insert");

    upsert_repository(
        &pool,
        "github",
        "user/repo2",
        "repo2",
        None,
        "https://github.com/user/repo2",
        Some("Rust"),
        50,
        &["rust".to_string(), "web".to_string()],
        None,
    )
    .await
    .expect("should insert");

    let topics = get_distinct_topics(&pool).await.expect("should query");

    // Should have unique topics
    let unique_count = topics.len();
    let mut deduped = topics.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(unique_count, deduped.len());

    // Should contain expected topics
    assert!(topics.contains(&"rust".to_string()));
    assert!(topics.contains(&"cli".to_string()));
    assert!(topics.contains(&"web".to_string()));
}
