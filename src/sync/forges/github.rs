use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use crate::sync::{FetchedRepository, SyncError, SyncSource};

const GITHUB_API_BASE: &str = "https://api.github.com";

pub struct GitHubForge {
    client: reqwest::Client,
    username: String,
    token: Option<String>,
}

impl GitHubForge {
    pub fn new(username: String, token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            username,
            token,
        }
    }

    pub fn from_env() -> Option<Self> {
        let username = std::env::var("DJV_GITHUB_USER").ok()?;
        let token = std::env::var("DJV_GITHUB_TOKEN").ok();

        Some(Self::new(username, token))
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_page(&self, page: u32) -> Result<Vec<GitHubRepo>, SyncError> {
        let url = format!(
            "{}/users/{}/repos?per_page=100&page={}&sort=updated",
            GITHUB_API_BASE, self.username, page
        );

        let mut request = self
            .client
            .get(&url)
            .header(USER_AGENT, "djv-sync/1.0")
            .header(ACCEPT, "application/vnd.github+json");

        if let Some(ref token) = self.token {
            request = request.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        let response = request.send().await?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::FORBIDDEN {
            if let Some(reset) = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
            {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let retry_after = reset.saturating_sub(now);
                return Err(SyncError::RateLimited(retry_after));
            }
        }

        let repos: Vec<GitHubRepo> = response.error_for_status()?.json().await?;
        Ok(repos)
    }
}

#[async_trait]
impl SyncSource for GitHubForge {
    fn name(&self) -> &'static str {
        "github"
    }

    #[tracing::instrument(skip(self), fields(username = %self.username))]
    async fn fetch_repositories(&self) -> Result<Vec<FetchedRepository>, SyncError> {
        let mut all_repos = Vec::new();
        let mut page = 1;

        loop {
            let repos = self.fetch_page(page).await?;
            let count = repos.len();

            tracing::debug!(page, count, "fetched page");

            all_repos.extend(repos.into_iter().filter(|r| !r.fork).map(|r| r.into()));

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
struct GitHubRepo {
    full_name: String,
    name: String,
    description: Option<String>,
    html_url: String,
    language: Option<String>,
    stargazers_count: i32,
    fork: bool,
    topics: Option<Vec<String>>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<GitHubRepo> for FetchedRepository {
    fn from(repo: GitHubRepo) -> Self {
        Self {
            forge: "github".to_string(),
            forge_id: repo.full_name.clone(),
            name: repo.name,
            description: repo.description,
            url: repo.html_url,
            language: repo.language,
            stars: repo.stargazers_count,
            topics: repo.topics.unwrap_or_default(),
            updated_at: repo.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn parses_repository_response() {
        let mock_server = MockServer::start().await;

        let response_body = serde_json::json!([
            {
                "full_name": "testuser/testrepo",
                "name": "testrepo",
                "description": "A test repository",
                "html_url": "https://github.com/testuser/testrepo",
                "language": "Rust",
                "stargazers_count": 42,
                "fork": false,
                "topics": ["rust", "testing"],
                "updated_at": "2024-01-15T10:30:00Z"
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/users/testuser/repos"))
            .and(header("User-Agent", "djv-sync/1.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let forge = GitHubForge {
            client: reqwest::Client::new(),
            username: "testuser".to_string(),
            token: None,
        };

        let url = format!(
            "{}/users/{}/repos?per_page=100&page=1&sort=updated",
            mock_server.uri(),
            "testuser"
        );

        let response = forge
            .client
            .get(&url)
            .header(USER_AGENT, "djv-sync/1.0")
            .send()
            .await
            .unwrap();

        let repos: Vec<GitHubRepo> = response.json().await.unwrap();

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "testrepo");
        assert_eq!(repos[0].full_name, "testuser/testrepo");
        assert_eq!(repos[0].description, Some("A test repository".to_string()));
        assert_eq!(repos[0].language, Some("Rust".to_string()));
        assert_eq!(repos[0].stargazers_count, 42);
        assert!(!repos[0].fork);
        assert_eq!(
            repos[0].topics,
            Some(vec!["rust".to_string(), "testing".to_string()])
        );
    }

    #[tokio::test]
    async fn filters_out_forked_repositories() {
        let repos = vec![
            GitHubRepo {
                full_name: "user/owned".to_string(),
                name: "owned".to_string(),
                description: Some("Owned repo".to_string()),
                html_url: "https://github.com/user/owned".to_string(),
                language: Some("Rust".to_string()),
                stargazers_count: 10,
                fork: false,
                topics: None,
                updated_at: None,
            },
            GitHubRepo {
                full_name: "user/forked".to_string(),
                name: "forked".to_string(),
                description: Some("Forked repo".to_string()),
                html_url: "https://github.com/user/forked".to_string(),
                language: Some("Python".to_string()),
                stargazers_count: 100,
                fork: true,
                topics: None,
                updated_at: None,
            },
        ];

        let filtered: Vec<FetchedRepository> = repos
            .into_iter()
            .filter(|r| !r.fork)
            .map(|r| r.into())
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "owned");
    }

    #[test]
    fn converts_github_repo_to_fetched_repository() {
        let github_repo = GitHubRepo {
            full_name: "user/repo".to_string(),
            name: "repo".to_string(),
            description: Some("Description".to_string()),
            html_url: "https://github.com/user/repo".to_string(),
            language: Some("Rust".to_string()),
            stargazers_count: 100,
            fork: false,
            topics: Some(vec!["topic1".to_string(), "topic2".to_string()]),
            updated_at: None,
        };

        let fetched: FetchedRepository = github_repo.into();

        assert_eq!(fetched.forge, "github");
        assert_eq!(fetched.forge_id, "user/repo");
        assert_eq!(fetched.name, "repo");
        assert_eq!(fetched.description, Some("Description".to_string()));
        assert_eq!(fetched.url, "https://github.com/user/repo");
        assert_eq!(fetched.language, Some("Rust".to_string()));
        assert_eq!(fetched.stars, 100);
        assert_eq!(fetched.topics, vec!["topic1", "topic2"]);
    }

    #[test]
    fn handles_missing_optional_fields() {
        let github_repo = GitHubRepo {
            full_name: "user/minimal".to_string(),
            name: "minimal".to_string(),
            description: None,
            html_url: "https://github.com/user/minimal".to_string(),
            language: None,
            stargazers_count: 0,
            fork: false,
            topics: None,
            updated_at: None,
        };

        let fetched: FetchedRepository = github_repo.into();

        assert_eq!(fetched.name, "minimal");
        assert!(fetched.description.is_none());
        assert!(fetched.language.is_none());
        assert_eq!(fetched.stars, 0);
        assert!(fetched.topics.is_empty());
    }
}
