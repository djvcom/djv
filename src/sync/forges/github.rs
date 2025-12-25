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
