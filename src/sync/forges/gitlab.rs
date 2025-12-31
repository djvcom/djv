use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::Value;

use crate::sync::{FetchedRepository, SyncError, SyncSource};

pub struct GitLabForge {
    client: reqwest::Client,
    host: String,
    username: String,
}

impl GitLabForge {
    pub fn new(username: String, host: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            host: host.unwrap_or_else(|| "gitlab.com".to_string()),
            username,
        }
    }

    pub fn from_env() -> Option<Self> {
        let username = std::env::var("DJV_GITLAB_USER").ok()?;
        let host = std::env::var("DJV_GITLAB_HOST").ok();

        Some(Self::new(username, host))
    }

    fn api_base(&self) -> String {
        format!("https://{}/api/v4", self.host)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_page(&self, page: u32) -> Result<Vec<GitLabProject>, SyncError> {
        let url = format!(
            "{}/users/{}/projects?per_page=100&page={}&order_by=updated_at&visibility=public",
            self.api_base(),
            self.username,
            page
        );

        let response = self
            .client
            .get(&url)
            .header(USER_AGENT, "djv-sync/1.0 (https://djv.sh)")
            .send()
            .await?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(SyncError::RateLimited(retry_after));
        }

        let projects: Vec<GitLabProject> = response.error_for_status()?.json().await?;
        Ok(projects)
    }
}

#[async_trait]
impl SyncSource for GitLabForge {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    #[tracing::instrument(skip(self), fields(username = %self.username, host = %self.host))]
    async fn fetch_repositories(&self) -> Result<Vec<FetchedRepository>, SyncError> {
        let mut all_repos = Vec::new();
        let mut page = 1;

        loop {
            let projects = self.fetch_page(page).await?;
            let count = projects.len();

            tracing::debug!(page, count, "fetched page");

            // Filter out archived projects and forks
            all_repos.extend(
                projects
                    .into_iter()
                    .filter(|p| !p.archived && p.forked_from_project.is_none())
                    .map(|p| p.into()),
            );

            if count < 100 {
                break;
            }
            page += 1;
        }

        tracing::info!(count = all_repos.len(), "fetched all repositories");
        Ok(all_repos)
    }
}

#[derive(Debug, Deserialize)]
struct GitLabProject {
    id: i64,
    #[allow(dead_code)]
    path_with_namespace: String,
    name: String,
    description: Option<String>,
    web_url: String,
    star_count: i32,
    #[serde(default)]
    archived: bool,
    #[serde(default)]
    forked_from_project: Option<Value>,
    topics: Option<Vec<String>>,
    last_activity_at: Option<DateTime<Utc>>,
}

impl From<GitLabProject> for FetchedRepository {
    fn from(project: GitLabProject) -> Self {
        Self {
            forge: "gitlab".to_string(),
            forge_id: project.id.to_string(),
            name: project.name,
            description: project.description,
            url: project.web_url,
            language: None, // GitLab doesn't return primary language in this endpoint
            stars: project.star_count,
            topics: project.topics.unwrap_or_default(),
            updated_at: project.last_activity_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gitlab_project() {
        let json = r#"{
            "id": 12345,
            "path_with_namespace": "user/project",
            "name": "project",
            "description": "A test project",
            "web_url": "https://gitlab.com/user/project",
            "star_count": 42,
            "archived": false,
            "forked_from_project": null,
            "topics": ["rust", "testing"],
            "last_activity_at": "2024-01-15T10:30:00Z"
        }"#;

        let project: GitLabProject = serde_json::from_str(json).unwrap();

        assert_eq!(project.id, 12345);
        assert_eq!(project.name, "project");
        assert_eq!(project.path_with_namespace, "user/project");
        assert_eq!(project.star_count, 42);
        assert!(!project.archived);
        assert!(project.forked_from_project.is_none());
    }

    #[test]
    fn converts_gitlab_project_to_fetched_repository() {
        let project = GitLabProject {
            id: 12345,
            path_with_namespace: "user/project".to_string(),
            name: "project".to_string(),
            description: Some("A test project".to_string()),
            web_url: "https://gitlab.com/user/project".to_string(),
            star_count: 42,
            archived: false,
            forked_from_project: None,
            topics: Some(vec!["rust".to_string(), "testing".to_string()]),
            last_activity_at: None,
        };

        let fetched: FetchedRepository = project.into();

        assert_eq!(fetched.forge, "gitlab");
        assert_eq!(fetched.forge_id, "12345");
        assert_eq!(fetched.name, "project");
        assert_eq!(fetched.description, Some("A test project".to_string()));
        assert_eq!(fetched.url, "https://gitlab.com/user/project");
        assert_eq!(fetched.stars, 42);
        assert_eq!(fetched.topics, vec!["rust", "testing"]);
    }

    #[test]
    fn filters_archived_projects() {
        let projects = vec![
            GitLabProject {
                id: 1,
                path_with_namespace: "user/active".to_string(),
                name: "active".to_string(),
                description: None,
                web_url: "https://gitlab.com/user/active".to_string(),
                star_count: 10,
                archived: false,
                forked_from_project: None,
                topics: None,
                last_activity_at: None,
            },
            GitLabProject {
                id: 2,
                path_with_namespace: "user/archived".to_string(),
                name: "archived".to_string(),
                description: None,
                web_url: "https://gitlab.com/user/archived".to_string(),
                star_count: 100,
                archived: true,
                forked_from_project: None,
                topics: None,
                last_activity_at: None,
            },
        ];

        let filtered: Vec<FetchedRepository> = projects
            .into_iter()
            .filter(|p| !p.archived && p.forked_from_project.is_none())
            .map(|p| p.into())
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "active");
    }

    #[test]
    fn creates_forge_instance() {
        let forge = GitLabForge::new("testuser".to_string(), None);
        assert_eq!(forge.username, "testuser");
        assert_eq!(forge.host, "gitlab.com");
    }

    #[test]
    fn creates_forge_with_custom_host() {
        let forge = GitLabForge::new(
            "testuser".to_string(),
            Some("gitlab.example.com".to_string()),
        );
        assert_eq!(forge.username, "testuser");
        assert_eq!(forge.host, "gitlab.example.com");
    }
}
