use reqwest::header::USER_AGENT;
use serde::Deserialize;

use crate::sync::SyncError;

const NPM_REGISTRY_API: &str = "https://registry.npmjs.org";
const NPM_DOWNLOADS_API: &str = "https://api.npmjs.org/downloads/point/last-week";

pub struct NpmRegistry {
    client: reqwest::Client,
    username: String,
}

#[derive(Debug, Clone)]
pub struct NpmPackageSummary {
    pub name: String,
    pub scope: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub npm_url: String,
    pub downloads_weekly: i32,
    pub version: Option<String>,
    pub keywords: Vec<String>,
}

impl NpmRegistry {
    pub fn new(username: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            username,
        }
    }

    pub fn from_env() -> Option<Self> {
        let username = std::env::var("DJV_NPM_USER").ok()?;
        Some(Self::new(username))
    }

    #[tracing::instrument(skip(self), fields(username = %self.username))]
    pub async fn fetch_packages(&self) -> Result<Vec<NpmPackageSummary>, SyncError> {
        let url = format!(
            "{}/-/v1/search?text=maintainer:{}&size=250",
            NPM_REGISTRY_API, self.username
        );

        let response: SearchResponse = self
            .client
            .get(&url)
            .header(USER_AGENT, "djv-sync/1.0 (https://djv.sh)")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        tracing::debug!(
            count = response.objects.len(),
            "fetched packages from search"
        );

        let mut packages = Vec::new();

        for obj in response.objects {
            let pkg = &obj.package;

            // Parse scope from name (e.g., "@scope/name" -> Some("scope"))
            let (scope, _name) = if pkg.name.starts_with('@') {
                let parts: Vec<&str> = pkg.name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    (Some(parts[0].trim_start_matches('@').to_string()), parts[1])
                } else {
                    (None, pkg.name.as_str())
                }
            } else {
                (None, pkg.name.as_str())
            };

            // Fetch weekly downloads for this package
            let downloads = self.fetch_downloads(&pkg.name).await.unwrap_or(0);

            // Extract repository URL from links
            let repository_url = pkg
                .links
                .as_ref()
                .and_then(|l| l.repository.clone())
                .or_else(|| {
                    pkg.links
                        .as_ref()
                        .and_then(|l| l.homepage.clone())
                        .filter(|h| h.contains("github.com") || h.contains("gitlab.com"))
                });

            packages.push(NpmPackageSummary {
                name: pkg.name.clone(),
                scope,
                description: pkg.description.clone(),
                repository_url,
                npm_url: format!("https://www.npmjs.com/package/{}", pkg.name),
                downloads_weekly: downloads,
                version: pkg.version.clone(),
                keywords: pkg.keywords.clone().unwrap_or_default(),
            });
        }

        tracing::info!(count = packages.len(), "fetched all npm packages");
        Ok(packages)
    }

    async fn fetch_downloads(&self, package_name: &str) -> Result<i32, SyncError> {
        let url = format!("{}/{}", NPM_DOWNLOADS_API, package_name);

        let response: DownloadsResponse = self
            .client
            .get(&url)
            .header(USER_AGENT, "djv-sync/1.0 (https://djv.sh)")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response.downloads as i32)
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    objects: Vec<SearchObject>,
}

#[derive(Debug, Deserialize)]
struct SearchObject {
    package: PackageInfo,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    name: String,
    version: Option<String>,
    description: Option<String>,
    keywords: Option<Vec<String>>,
    links: Option<PackageLinks>,
}

#[derive(Debug, Deserialize)]
struct PackageLinks {
    #[allow(dead_code)]
    npm: Option<String>,
    #[allow(dead_code)]
    homepage: Option<String>,
    repository: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DownloadsResponse {
    downloads: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_search_response() {
        let json = r#"{
            "objects": [{
                "package": {
                    "name": "my-package",
                    "version": "1.0.0",
                    "description": "A test package",
                    "keywords": ["test", "example"],
                    "links": {
                        "npm": "https://www.npmjs.com/package/my-package",
                        "homepage": "https://example.com",
                        "repository": "https://github.com/user/my-package"
                    }
                }
            }],
            "total": 1
        }"#;

        let response: SearchResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.objects.len(), 1);
        assert_eq!(response.objects[0].package.name, "my-package");
        assert_eq!(
            response.objects[0].package.version,
            Some("1.0.0".to_string())
        );
    }

    #[test]
    fn parses_scoped_package_name() {
        let name = "@scope/package-name";
        let (scope, pkg_name) = if name.starts_with('@') {
            let parts: Vec<&str> = name.splitn(2, '/').collect();
            if parts.len() == 2 {
                (Some(parts[0].trim_start_matches('@').to_string()), parts[1])
            } else {
                (None, name)
            }
        } else {
            (None, name)
        };

        assert_eq!(scope, Some("scope".to_string()));
        assert_eq!(pkg_name, "package-name");
    }

    #[test]
    fn parses_unscoped_package_name() {
        let name = "simple-package";
        let (scope, pkg_name) = if name.starts_with('@') {
            let parts: Vec<&str> = name.splitn(2, '/').collect();
            if parts.len() == 2 {
                (Some(parts[0].trim_start_matches('@').to_string()), parts[1])
            } else {
                (None, name)
            }
        } else {
            (None, name)
        };

        assert!(scope.is_none());
        assert_eq!(pkg_name, "simple-package");
    }

    #[test]
    fn parses_downloads_response() {
        let json = r#"{"downloads": 12345, "start": "2024-01-01", "end": "2024-01-07", "package": "test"}"#;
        let response: DownloadsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.downloads, 12345);
    }

    #[test]
    fn creates_registry_instance() {
        let registry = NpmRegistry::new("testuser".to_string());
        assert_eq!(registry.username, "testuser");
    }
}
