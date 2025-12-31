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
    gitlab_username: Option<String>,
    gitlab_host: Option<String>,
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
            gitlab_username: None,
            gitlab_host: None,
        }
    }

    pub fn with_gitlab(mut self, username: String, host: Option<String>) -> Self {
        self.gitlab_username = Some(username);
        self.gitlab_host = host;
        self
    }

    pub fn from_env() -> Option<Self> {
        let username = std::env::var("DJV_CONTRIBUTIONS_USER").ok()?;
        let token = std::env::var("DJV_GITHUB_TOKEN").ok();
        let exclude_owner = std::env::var("DJV_GITHUB_USER").ok();

        let mut sync = Self::new(username, token, exclude_owner);

        // Add GitLab if configured
        if let Ok(gitlab_user) = std::env::var("DJV_GITLAB_USER") {
            let gitlab_host = std::env::var("DJV_GITLAB_HOST").ok();
            sync = sync.with_gitlab(gitlab_user, gitlab_host);
        }

        Some(sync)
    }

    #[tracing::instrument(skip(self), fields(username = %self.username))]
    pub async fn fetch_contributions(&self) -> Result<Vec<FetchedContribution>, SyncError> {
        let mut all_contributions = Vec::new();

        // Fetch GitHub contributions
        let github_contribs = self.fetch_github_contributions().await?;
        all_contributions.extend(github_contribs);

        // Fetch GitLab contributions if configured
        if self.gitlab_username.is_some() {
            match self.fetch_gitlab_contributions().await {
                Ok(gitlab_contribs) => all_contributions.extend(gitlab_contribs),
                Err(e) => tracing::warn!(error = %e, "failed to fetch GitLab contributions"),
            }
        }

        tracing::info!(count = all_contributions.len(), "fetched all contributions");
        Ok(all_contributions)
    }

    async fn fetch_github_contributions(&self) -> Result<Vec<FetchedContribution>, SyncError> {
        let mut all_contributions = Vec::new();
        let mut page = 1;

        loop {
            let contributions = self.fetch_github_page(page).await?;
            let count = contributions.len();

            tracing::debug!(page, count, "fetched GitHub page");

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

        Ok(all_contributions)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_github_page(&self, page: u32) -> Result<Vec<FetchedContribution>, SyncError> {
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

    #[tracing::instrument(skip(self))]
    async fn fetch_gitlab_contributions(&self) -> Result<Vec<FetchedContribution>, SyncError> {
        let username = match &self.gitlab_username {
            Some(u) => u,
            None => return Ok(Vec::new()),
        };

        let host = self.gitlab_host.as_deref().unwrap_or("gitlab.com");

        let mut all_contributions = Vec::new();
        let mut page = 1;

        loop {
            let contributions = self.fetch_gitlab_page(username, host, page).await?;
            let count = contributions.len();

            tracing::debug!(page, count, "fetched GitLab page");

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

        Ok(all_contributions)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_gitlab_page(
        &self,
        username: &str,
        host: &str,
        page: u32,
    ) -> Result<Vec<FetchedContribution>, SyncError> {
        // Fetch merged MRs by the user, excluding their own projects
        let url = format!(
            "https://{}/api/v4/merge_requests?author_username={}&state=merged&scope=all&per_page=100&page={}",
            host, username, page
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

        let merge_requests: Vec<GitLabMergeRequest> = response.error_for_status()?.json().await?;

        // Filter out MRs to the user's own projects and map to contributions
        let contributions = merge_requests
            .into_iter()
            .filter_map(|mr| {
                // Parse project info from web_url
                let (repo_owner, repo_name, repo_url) = mr.parse_project_info()?;

                // Exclude MRs to the user's own projects
                if repo_owner.eq_ignore_ascii_case(username) {
                    return None;
                }

                Some(FetchedContribution {
                    forge: "gitlab".to_string(),
                    repo_owner,
                    repo_name,
                    repo_url,
                    contribution_type: "mr".to_string(),
                    title: Some(mr.title),
                    url: mr.web_url,
                    merged_at: mr.merged_at,
                })
            })
            .collect();

        Ok(contributions)
    }
}

#[derive(Debug, Deserialize)]
struct GitLabMergeRequest {
    title: String,
    web_url: String,
    merged_at: Option<DateTime<Utc>>,
}

impl GitLabMergeRequest {
    /// Parse project owner and name from the MR web_url.
    /// Format: https://gitlab.com/owner/repo/-/merge_requests/1
    fn parse_project_info(&self) -> Option<(String, String, String)> {
        let url = &self.web_url;
        // Remove the /-/merge_requests/N suffix
        let project_part = url.split("/-/merge_requests").next()?;
        // Parse the URL to get the path
        let parsed = reqwest::Url::parse(project_part).ok()?;
        let path = parsed.path().trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() >= 2 {
            let owner = parts[0].to_string();
            let repo = parts[parts.len() - 1].to_string();
            let project_url = project_part.to_string();
            Some((owner, repo, project_url))
        } else {
            None
        }
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
