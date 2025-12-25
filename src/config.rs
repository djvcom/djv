use figment::{
    providers::{Env, Serialized},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Listen address (e.g. "127.0.0.1:3000")
    #[serde(default = "default_listen")]
    pub listen: String,

    /// Database configuration
    pub database: Option<DatabaseConfig>,

    /// OpenTelemetry configuration
    #[serde(default)]
    pub otel: OtelConfig,

    /// Sync configuration
    #[serde(default)]
    pub sync: SyncConfig,
}

fn default_listen() -> String {
    "127.0.0.1:3000".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OtelConfig {
    /// OTLP endpoint (if None, uses OTEL_EXPORTER_OTLP_ENDPOINT env var)
    pub endpoint: Option<String>,

    /// Deployment environment name
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String {
    "development".to_string()
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            environment: default_environment(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    /// Enable background sync
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Sync interval in seconds
    #[serde(default = "default_interval")]
    pub interval_secs: u64,

    /// GitHub sync configuration
    pub github: Option<GitHubConfig>,

    /// crates.io sync configuration
    pub crates_io: Option<CratesIoConfig>,

    /// Contributions sync configuration
    pub contributions: Option<ContributionsConfig>,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    3600
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600,
            github: None,
            crates_io: None,
            contributions: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubConfig {
    /// GitHub username to sync repositories from
    pub user: String,

    /// GitHub personal access token (optional, increases rate limits)
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CratesIoConfig {
    /// crates.io username to sync crates from
    pub user: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContributionsConfig {
    /// GitHub username to track contributions from
    pub user: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Supports the following env vars:
    /// - DJV_LISTEN
    /// - DJV_DATABASE_URL or DATABASE_URL
    /// - DJV_OTEL_ENDPOINT
    /// - DJV_OTEL_ENVIRONMENT
    /// - DJV_SYNC_ENABLED
    /// - DJV_SYNC_INTERVAL_SECS
    /// - DJV_SYNC_GITHUB_USER
    /// - DJV_SYNC_GITHUB_TOKEN
    /// - DJV_SYNC_CRATES_IO_USER
    /// - DJV_SYNC_CONTRIBUTIONS_USER
    pub fn load() -> Result<Self, figment::Error> {
        // Start with defaults
        let figment = Figment::new()
            .merge(Serialized::defaults(ConfigDefaults::default()))
            // Merge DJV_ prefixed env vars with nested structure
            .merge(Env::prefixed("DJV_").split("_"));

        // Extract base config
        let mut config: Config = figment.extract()?;

        // Handle DATABASE_URL without prefix for compatibility
        if config.database.is_none() {
            if let Ok(url) = std::env::var("DATABASE_URL") {
                config.database = Some(DatabaseConfig { url });
            }
        }

        // Handle legacy flat env vars for backwards compatibility
        config.apply_legacy_env_vars();

        Ok(config)
    }

    /// Apply legacy flat environment variables for backwards compatibility
    fn apply_legacy_env_vars(&mut self) {
        // DJV_GITHUB_USER -> sync.github.user
        if self.sync.github.is_none() {
            if let Ok(user) = std::env::var("DJV_GITHUB_USER") {
                let token = std::env::var("DJV_GITHUB_TOKEN").ok();
                self.sync.github = Some(GitHubConfig { user, token });
            }
        }

        // DJV_CRATES_IO_USER -> sync.crates_io.user
        if self.sync.crates_io.is_none() {
            if let Ok(user) = std::env::var("DJV_CRATES_IO_USER") {
                self.sync.crates_io = Some(CratesIoConfig { user });
            }
        }

        // DJV_CONTRIBUTIONS_USER -> sync.contributions.user
        if self.sync.contributions.is_none() {
            if let Ok(user) = std::env::var("DJV_CONTRIBUTIONS_USER") {
                self.sync.contributions = Some(ContributionsConfig { user });
            }
        }

        // DJV_SYNC_INTERVAL -> sync.interval_secs
        if let Ok(interval) = std::env::var("DJV_SYNC_INTERVAL") {
            if let Ok(secs) = interval.parse() {
                self.sync.interval_secs = secs;
            }
        }
    }
}

/// Helper struct for default values in figment
#[derive(Debug, Serialize)]
struct ConfigDefaults {
    listen: String,
    otel: OtelConfig,
    sync: SyncConfig,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            otel: OtelConfig::default(),
            sync: SyncConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::load().unwrap();
        assert_eq!(config.listen, "127.0.0.1:3000");
        assert!(config.sync.enabled);
        assert_eq!(config.sync.interval_secs, 3600);
    }
}
