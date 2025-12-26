use reqwest::header::USER_AGENT;
use serde::Deserialize;

use crate::sync::SyncError;

const CRATES_IO_API_BASE: &str = "https://crates.io/api/v1";

pub struct CratesIoRegistry {
    client: reqwest::Client,
    username: String,
}

#[derive(Debug, Clone)]
pub struct FetchedCrate {
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub crates_io_url: String,
    pub documentation_url: Option<String>,
    pub downloads: i32,
    pub version: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

impl CratesIoRegistry {
    pub fn new(username: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            username,
        }
    }

    pub fn from_env() -> Option<Self> {
        let username = std::env::var("DJV_CRATES_IO_USER").ok()?;
        Some(Self::new(username))
    }

    #[tracing::instrument(skip(self))]
    async fn get_user_id(&self) -> Result<i64, SyncError> {
        let url = format!("{}/users/{}", CRATES_IO_API_BASE, self.username);

        let response: UserResponse = self
            .client
            .get(&url)
            .header(USER_AGENT, "djv-sync/1.0 (https://djv.sh)")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response.user.id)
    }

    #[tracing::instrument(skip(self), fields(username = %self.username))]
    pub async fn fetch_crates(&self) -> Result<Vec<FetchedCrate>, SyncError> {
        let user_id = self.get_user_id().await?;

        let mut all_crates = Vec::new();
        let mut page = 1;

        loop {
            let url = format!(
                "{}/crates?user_id={}&page={}&per_page=100",
                CRATES_IO_API_BASE, user_id, page
            );

            let response: CratesResponse = self
                .client
                .get(&url)
                .header(USER_AGENT, "djv-sync/1.0 (https://djv.sh)")
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            let count = response.crates.len();
            tracing::debug!(page, count, "fetched page");

            all_crates.extend(response.crates.into_iter().map(|c| FetchedCrate {
                name: c.name.clone(),
                description: c.description,
                repository_url: c.repository,
                crates_io_url: format!("https://crates.io/crates/{}", c.name),
                documentation_url: c.documentation,
                downloads: c.downloads as i32,
                version: c.newest_version,
                keywords: Vec::new(),
                categories: Vec::new(),
            }));

            if response.meta.next_page.is_none() {
                break;
            }
            page += 1;
        }

        tracing::info!(count = all_crates.len(), "fetched all crates");
        Ok(all_crates)
    }
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    user: User,
}

#[derive(Debug, Deserialize)]
struct User {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct CratesResponse {
    crates: Vec<CrateInfo>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct CrateInfo {
    name: String,
    description: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    downloads: u64,
    newest_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    next_page: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_response() {
        let json = r#"{"user": {"id": 12345, "login": "testuser", "name": "Test User"}}"#;
        let response: UserResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.user.id, 12345);
    }

    #[test]
    fn parses_crates_response() {
        let json = r#"{
            "crates": [{
                "name": "my-crate",
                "description": "A test crate",
                "repository": "https://github.com/user/my-crate",
                "documentation": "https://docs.rs/my-crate",
                "downloads": 1000,
                "newest_version": "1.0.0"
            }],
            "meta": {"next_page": null}
        }"#;

        let response: CratesResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.crates.len(), 1);
        assert_eq!(response.crates[0].name, "my-crate");
        assert_eq!(response.crates[0].downloads, 1000);
        assert!(response.meta.next_page.is_none());
    }

    #[test]
    fn parses_crates_response_with_pagination() {
        let json = r#"{
            "crates": [],
            "meta": {"next_page": "?page=2"}
        }"#;

        let response: CratesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.meta.next_page, Some("?page=2".to_string()));
    }

    #[test]
    fn converts_crate_info_to_fetched_crate() {
        let crate_info = CrateInfo {
            name: "test-crate".to_string(),
            description: Some("A test crate".to_string()),
            repository: Some("https://github.com/user/test-crate".to_string()),
            documentation: Some("https://docs.rs/test-crate".to_string()),
            downloads: 5000,
            newest_version: Some("2.0.0".to_string()),
        };

        let fetched = FetchedCrate {
            name: crate_info.name.clone(),
            description: crate_info.description,
            repository_url: crate_info.repository,
            crates_io_url: format!("https://crates.io/crates/{}", crate_info.name),
            documentation_url: crate_info.documentation,
            downloads: crate_info.downloads as i32,
            version: crate_info.newest_version,
            keywords: Vec::new(),
            categories: Vec::new(),
        };

        assert_eq!(fetched.name, "test-crate");
        assert_eq!(fetched.description, Some("A test crate".to_string()));
        assert_eq!(fetched.crates_io_url, "https://crates.io/crates/test-crate");
        assert_eq!(fetched.downloads, 5000);
        assert_eq!(fetched.version, Some("2.0.0".to_string()));
    }

    #[test]
    fn handles_missing_optional_fields() {
        let json = r#"{
            "crates": [{
                "name": "minimal-crate",
                "description": null,
                "repository": null,
                "documentation": null,
                "downloads": 0,
                "newest_version": null
            }],
            "meta": {"next_page": null}
        }"#;

        let response: CratesResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.crates[0].name, "minimal-crate");
        assert!(response.crates[0].description.is_none());
        assert!(response.crates[0].repository.is_none());
        assert!(response.crates[0].documentation.is_none());
        assert!(response.crates[0].newest_version.is_none());
    }

    #[test]
    fn creates_registry_instance() {
        let registry = CratesIoRegistry::new("testuser".to_string());
        assert_eq!(registry.username, "testuser");
    }
}
