use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A git repository from a forge (GitHub, Codeberg, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub forge: String,
    pub forge_id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub language: Option<String>,
    pub stars: i32,
    pub topics: Vec<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// A Rust crate from crates.io
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Crate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub repository_id: Option<Uuid>,
    pub crates_io_url: String,
    pub documentation_url: Option<String>,
    pub downloads: i32,
    pub version: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub synced_at: DateTime<Utc>,
}

/// An NPM package from npmjs.com
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NpmPackage {
    pub id: Uuid,
    pub name: String,
    pub scope: Option<String>,
    pub description: Option<String>,
    pub repository_id: Option<Uuid>,
    pub npm_url: String,
    pub downloads_weekly: i32,
    pub version: Option<String>,
    pub keywords: Vec<String>,
    pub synced_at: DateTime<Utc>,
}

/// An open source contribution to an external repository
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contribution {
    pub id: Uuid,
    pub forge: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub repo_url: String,
    pub contribution_type: String,
    pub title: Option<String>,
    pub url: String,
    pub merged_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// The kind of project in the unified view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind {
    Crate,
    Npm,
    Repo,
}

impl std::fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectKind::Crate => write!(f, "crate"),
            ProjectKind::Npm => write!(f, "npm"),
            ProjectKind::Repo => write!(f, "repo"),
        }
    }
}

impl std::str::FromStr for ProjectKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "crate" => Ok(ProjectKind::Crate),
            "npm" => Ok(ProjectKind::Npm),
            "repo" => Ok(ProjectKind::Repo),
            _ => Err(format!("unknown project kind: {}", s)),
        }
    }
}

/// A unified project view for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectView {
    pub id: Uuid,
    pub kind: ProjectKind,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub popularity: i32,
    pub version: Option<String>,
    pub commit_count: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// Filters for querying projects
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProjectFilters {
    pub kind: Option<ProjectKind>,
    pub language: Option<String>,
    pub topic: Option<String>,
    pub sort: Option<SortOrder>,
}

/// Sort order for projects
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Popularity,
    Name,
    Updated,
}

impl std::str::FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "popularity" => Ok(SortOrder::Popularity),
            "name" => Ok(SortOrder::Name),
            "updated" => Ok(SortOrder::Updated),
            _ => Err(format!("unknown sort order: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_kind_from_str_valid() {
        assert_eq!("crate".parse::<ProjectKind>().unwrap(), ProjectKind::Crate);
        assert_eq!("npm".parse::<ProjectKind>().unwrap(), ProjectKind::Npm);
        assert_eq!("repo".parse::<ProjectKind>().unwrap(), ProjectKind::Repo);
    }

    #[test]
    fn project_kind_from_str_case_insensitive() {
        assert_eq!("CRATE".parse::<ProjectKind>().unwrap(), ProjectKind::Crate);
        assert_eq!("Npm".parse::<ProjectKind>().unwrap(), ProjectKind::Npm);
        assert_eq!("REPO".parse::<ProjectKind>().unwrap(), ProjectKind::Repo);
    }

    #[test]
    fn project_kind_from_str_invalid() {
        assert!("invalid".parse::<ProjectKind>().is_err());
        assert!("".parse::<ProjectKind>().is_err());
    }

    #[test]
    fn project_kind_display() {
        assert_eq!(ProjectKind::Crate.to_string(), "crate");
        assert_eq!(ProjectKind::Npm.to_string(), "npm");
        assert_eq!(ProjectKind::Repo.to_string(), "repo");
    }

    #[test]
    fn project_kind_roundtrip() {
        for kind in [ProjectKind::Crate, ProjectKind::Npm, ProjectKind::Repo] {
            let s = kind.to_string();
            let parsed: ProjectKind = s.parse().unwrap();
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn sort_order_from_str_valid() {
        assert_eq!(
            "popularity".parse::<SortOrder>().unwrap(),
            SortOrder::Popularity
        );
        assert_eq!("name".parse::<SortOrder>().unwrap(), SortOrder::Name);
        assert_eq!("updated".parse::<SortOrder>().unwrap(), SortOrder::Updated);
    }

    #[test]
    fn sort_order_from_str_case_insensitive() {
        assert_eq!(
            "POPULARITY".parse::<SortOrder>().unwrap(),
            SortOrder::Popularity
        );
        assert_eq!("Name".parse::<SortOrder>().unwrap(), SortOrder::Name);
    }

    #[test]
    fn sort_order_from_str_invalid() {
        assert!("invalid".parse::<SortOrder>().is_err());
        assert!("".parse::<SortOrder>().is_err());
    }

    #[test]
    fn sort_order_default() {
        assert_eq!(SortOrder::default(), SortOrder::Popularity);
    }

    #[test]
    fn project_kind_serde_roundtrip() {
        for kind in [ProjectKind::Crate, ProjectKind::Npm, ProjectKind::Repo] {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: ProjectKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn sort_order_serde_roundtrip() {
        for order in [SortOrder::Popularity, SortOrder::Name, SortOrder::Updated] {
            let json = serde_json::to_string(&order).unwrap();
            let parsed: SortOrder = serde_json::from_str(&json).unwrap();
            assert_eq!(order, parsed);
        }
    }

    #[test]
    fn project_filters_default() {
        let filters = ProjectFilters::default();
        assert!(filters.kind.is_none());
        assert!(filters.language.is_none());
        assert!(filters.topic.is_none());
        assert!(filters.sort.is_none());
    }
}
