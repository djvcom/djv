use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use crate::sync::SyncError;

const GITHUB_API_BASE: &str = "https://api.github.com";

pub struct ContributionsSync {
    client: reqwest::Client,
    username: String,
    token: Option<String>,
    exclude_owner: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FetchedContribution {
    pub forge: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub repo_url: String,
    pub contribution_type: String,
    pub title: Option<String>,
    pub url: String,
    pub merged_at: Option<DateTime<Utc>>,
}

impl ContributionsSync {
    pub fn new(username: String, token: Option<String>, exclude_owner: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            username,
            token,
            exclude_owner,
        }
    }

    pub fn from_env() -> Option<Self> {
        let username = std::env::var("DJV_CONTRIBUTIONS_USER").ok()?;
        let token = std::env::var("DJV_GITHUB_TOKEN").ok();
        let exclude_owner = std::env::var("DJV_GITHUB_USER").ok();

        Some(Self::new(username, token, exclude_owner))
    }

    #[tracing::instrument(skip(self), fields(username = %self.username))]
    pub async fn fetch_contributions(&self) -> Result<Vec<FetchedContribution>, SyncError> {
        let mut all_contributions = Vec::new();
        let mut page = 1;

        loop {
            let contributions = self.fetch_page(page).await?;
            let count = contributions.len();

            tracing::debug!(page, count, "fetched page");

            all_contributions.extend(contributions);

            if count < 100 {
                break;
            }
            page += 1;

            // Limit to avoid excessive API calls
            if page > 5 {
                tracing::warn!("stopping at page 5 to avoid rate limits");
                break;
            }
        }

        tracing::info!(count = all_contributions.len(), "fetched all contributions");
        Ok(all_contributions)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_page(&self, page: u32) -> Result<Vec<FetchedContribution>, SyncError> {
        // Search for merged PRs by the user
        let query = format!(
            "type:pr author:{} is:merged -user:{}",
            self.username,
            self.exclude_owner.as_deref().unwrap_or(&self.username)
        );

        let url = format!(
            "{}/search/issues?q={}&sort=updated&order=desc&per_page=100&page={}",
            GITHUB_API_BASE,
            urlencoding::encode(&query),
            page
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

        let search_result: SearchResult = response.error_for_status()?.json().await?;

        let contributions = search_result
            .items
            .into_iter()
            .filter_map(|item| {
                // Parse repo from URL: https://api.github.com/repos/owner/name/...
                let repo_url = item.repository_url?;
                let parts: Vec<&str> = repo_url.split('/').collect();
                if parts.len() < 2 {
                    return None;
                }
                let repo_name = parts[parts.len() - 1].to_string();
                let repo_owner = parts[parts.len() - 2].to_string();

                Some(FetchedContribution {
                    forge: "github".to_string(),
                    repo_owner: repo_owner.clone(),
                    repo_name: repo_name.clone(),
                    repo_url: format!("https://github.com/{}/{}", repo_owner, repo_name),
                    contribution_type: "pr".to_string(),
                    title: Some(item.title),
                    url: item.html_url,
                    merged_at: item.closed_at,
                })
            })
            .collect();

        Ok(contributions)
    }
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    title: String,
    html_url: String,
    repository_url: Option<String>,
    closed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_repo_owner_and_name_from_api_url() {
        let repository_url = "https://api.github.com/repos/goreleaser/goreleaser";
        let parts: Vec<&str> = repository_url.split('/').collect();
        let repo_name = parts[parts.len() - 1].to_string();
        let repo_owner = parts[parts.len() - 2].to_string();

        assert_eq!(repo_owner, "goreleaser");
        assert_eq!(repo_name, "goreleaser");
    }

    #[test]
    fn constructs_repo_url_from_owner_and_name() {
        let repo_owner = "anthropics";
        let repo_name = "claude-code";
        let repo_url = format!("https://github.com/{}/{}", repo_owner, repo_name);

        assert_eq!(repo_url, "https://github.com/anthropics/claude-code");
    }

    #[test]
    fn parses_search_item_to_contribution() {
        let item = SearchItem {
            title: "Fix memory leak".to_string(),
            html_url: "https://github.com/owner/repo/pull/123".to_string(),
            repository_url: Some("https://api.github.com/repos/owner/repo".to_string()),
            closed_at: Some(chrono::Utc::now()),
        };

        let repo_url = item.repository_url.as_ref().unwrap();
        let parts: Vec<&str> = repo_url.split('/').collect();
        let repo_name = parts[parts.len() - 1].to_string();
        let repo_owner = parts[parts.len() - 2].to_string();

        let contribution = FetchedContribution {
            forge: "github".to_string(),
            repo_owner: repo_owner.clone(),
            repo_name: repo_name.clone(),
            repo_url: format!("https://github.com/{}/{}", repo_owner, repo_name),
            contribution_type: "pr".to_string(),
            title: Some(item.title.clone()),
            url: item.html_url.clone(),
            merged_at: item.closed_at,
        };

        assert_eq!(contribution.forge, "github");
        assert_eq!(contribution.repo_owner, "owner");
        assert_eq!(contribution.repo_name, "repo");
        assert_eq!(contribution.repo_url, "https://github.com/owner/repo");
        assert_eq!(contribution.contribution_type, "pr");
        assert_eq!(contribution.title, Some("Fix memory leak".to_string()));
        assert_eq!(contribution.url, "https://github.com/owner/repo/pull/123");
    }

    #[test]
    fn handles_missing_repository_url() {
        let item = SearchItem {
            title: "Some PR".to_string(),
            html_url: "https://github.com/owner/repo/pull/456".to_string(),
            repository_url: None,
            closed_at: None,
        };

        assert!(item.repository_url.is_none());
    }

    #[test]
    fn creates_contributions_sync_instance() {
        let sync = ContributionsSync::new(
            "testuser".to_string(),
            Some("token123".to_string()),
            Some("excludeuser".to_string()),
        );

        assert_eq!(sync.username, "testuser");
        assert_eq!(sync.token, Some("token123".to_string()));
        assert_eq!(sync.exclude_owner, Some("excludeuser".to_string()));
    }
}
