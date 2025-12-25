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
