use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    Source,
    Test,
    Config,
    Schema,
    Docs,
    Generated,
    Sensitive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoRoot {
    pub path: PathBuf,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RepoRootError {
    #[error("no git repository found from {start}")]
    NotFound { start: PathBuf },
}

impl RepoRoot {
    pub fn discover_from(start: impl AsRef<Path>) -> Result<Self, RepoRootError> {
        let start = start.as_ref();
        for candidate in start.ancestors() {
            if candidate.join(".git").exists() {
                return Ok(Self {
                    path: candidate.to_path_buf(),
                });
            }
        }

        Err(RepoRootError::NotFound {
            start: start.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_nearest_git_ancestor() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let nested = repo.join("a/b/c");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::create_dir_all(&nested).unwrap();

        let found = RepoRoot::discover_from(&nested).unwrap();
        assert_eq!(found.path, repo);
    }

    #[test]
    fn returns_not_found_when_no_git_ancestor_exists() {
        let temp = tempfile::tempdir().unwrap();
        let err = RepoRoot::discover_from(temp.path()).unwrap_err();
        assert!(matches!(err, RepoRootError::NotFound { .. }));
    }
}
