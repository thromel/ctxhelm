use crate::policy::{classify_path, language_for_path, POLICY_VERSION};
use ctxpack_core::FileRole;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

pub const INVENTORY_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("failed to canonicalize {path}: {source}")]
    Canonicalize { path: PathBuf, source: io::Error },
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("failed to create directory {path}: {source}")]
    CreateDir { path: PathBuf, source: io::Error },
    #[error("failed to serialize inventory: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize inventory {path}: {source}")]
    Deserialize {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid inventory input: {0}")]
    InvalidInput(String),
    #[error("git command failed in {repo_root}: {message}")]
    Git { repo_root: PathBuf, message: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryOptions {
    pub include_generated: bool,
    pub include_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileInventoryEntry {
    pub path: String,
    pub language: Option<String>,
    pub role: FileRole,
    pub hash: String,
    pub size_bytes: u64,
    pub generated: bool,
    pub ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryManifestEntry {
    pub path: String,
    pub hash: String,
    pub size_bytes: u64,
    pub modified_unix_nanos: u128,
    pub role: FileRole,
    pub generated: bool,
    pub ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IgnoreFileFingerprint {
    pub path: String,
    pub present: bool,
    pub hash: Option<String>,
    pub size_bytes: u64,
    pub modified_unix_nanos: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryMetadata {
    pub schema_version: u32,
    pub policy_version: String,
    pub options_fingerprint: String,
    pub repo_root: PathBuf,
    pub built_at_unix_seconds: u64,
    pub ignore_fingerprints: Vec<IgnoreFileFingerprint>,
    pub manifest: Vec<InventoryManifestEntry>,
}

impl Default for InventoryMetadata {
    fn default() -> Self {
        Self {
            schema_version: 0,
            policy_version: String::new(),
            options_fingerprint: String::new(),
            repo_root: PathBuf::new(),
            built_at_unix_seconds: 0,
            ignore_fingerprints: Vec::new(),
            manifest: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepoInventory {
    pub repo_id: String,
    pub repo_root: PathBuf,
    #[serde(default)]
    pub metadata: InventoryMetadata,
    pub files: Vec<FileInventoryEntry>,
    pub ignored_count: usize,
    pub generated_count: usize,
    pub sensitive_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryReport {
    pub repo_id: String,
    pub repo_root: PathBuf,
    pub inventory_path: PathBuf,
    pub file_count: usize,
    pub ignored_count: usize,
    pub generated_count: usize,
    pub sensitive_count: usize,
}

pub fn build_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<RepoInventory, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let mut files = Vec::new();
    let mut manifest = Vec::new();
    let mut ignored_count = 0;
    let mut generated_count = 0;
    let mut sensitive_count = 0;

    let mut walker = WalkBuilder::new(&repo_root);
    walker
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .parents(true)
        .ignore(false)
        .add_custom_ignore_filename(".ctxpackignore")
        .add_custom_ignore_filename(".cursorignore");

    for result in walker.build() {
        let Ok(entry) = result else {
            continue;
        };
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }

        let path = entry.path();
        let relative = normalize_relative_path(&repo_root, path);
        if relative == ".git" || relative.starts_with(".git/") {
            continue;
        }

        let role = classify_path(&relative);
        let generated = role == FileRole::Generated;
        let sensitive = role == FileRole::Sensitive;

        if generated {
            generated_count += 1;
        }
        if sensitive {
            sensitive_count += 1;
        }
        if (generated && !options.include_generated) || (sensitive && !options.include_sensitive) {
            continue;
        }

        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => {
                ignored_count += 1;
                continue;
            }
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => {
                ignored_count += 1;
                continue;
            }
        };

        let hash = blake3::hash(&bytes).to_hex().to_string();
        let modified_unix_nanos = modified_unix_nanos(&metadata);

        files.push(FileInventoryEntry {
            path: relative.clone(),
            language: language_for_path(&relative).map(str::to_string),
            role: role.clone(),
            hash: hash.clone(),
            size_bytes: metadata.len(),
            generated,
            ignored: false,
        });
        manifest.push(InventoryManifestEntry {
            path: relative,
            hash,
            size_bytes: metadata.len(),
            modified_unix_nanos,
            role,
            generated,
            ignored: false,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    manifest.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(RepoInventory {
        repo_id: repo_id_for_path(&repo_root),
        metadata: InventoryMetadata {
            schema_version: INVENTORY_SCHEMA_VERSION,
            policy_version: POLICY_VERSION.to_string(),
            options_fingerprint: options_fingerprint(options),
            repo_root: repo_root.clone(),
            built_at_unix_seconds: current_unix_seconds(),
            ignore_fingerprints: ignore_fingerprints(&repo_root),
            manifest,
        },
        repo_root,
        files,
        ignored_count,
        generated_count,
        sensitive_count,
    })
}

pub fn write_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<InventoryReport, InventoryError> {
    let inventory = build_inventory(repo_root, options)?;
    let inventory_path = persist_inventory(&inventory)?;

    Ok(InventoryReport {
        repo_id: inventory.repo_id,
        repo_root: inventory.repo_root,
        inventory_path,
        file_count: inventory.files.len(),
        ignored_count: inventory.ignored_count,
        generated_count: inventory.generated_count,
        sensitive_count: inventory.sensitive_count,
    })
}

pub(crate) fn persist_inventory(inventory: &RepoInventory) -> Result<PathBuf, InventoryError> {
    let inventory_path = inventory_path(&inventory.repo_id);
    if let Some(parent) = inventory_path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let json = serde_json::to_string_pretty(&inventory).map_err(InventoryError::Serialize)?;
    fs::write(&inventory_path, json).map_err(|source| InventoryError::Write {
        path: inventory_path.clone(),
        source,
    })?;

    Ok(inventory_path)
}

pub fn load_inventory(repo_id: &str) -> Result<RepoInventory, InventoryError> {
    let path = inventory_path(repo_id);
    let json = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&json).map_err(|source| InventoryError::Deserialize { path, source })
}

pub fn load_or_build_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<RepoInventory, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    match load_inventory(&repo_id) {
        Ok(inventory) => Ok(inventory),
        Err(InventoryError::Read { .. }) => {
            write_inventory(&repo_root, options)?;
            load_inventory(&repo_id)
        }
        Err(error) => Err(error),
    }
}

pub fn inventory_path(repo_id: &str) -> PathBuf {
    ctxpack_home()
        .join("repos")
        .join(repo_id)
        .join("inventory.json")
}

pub fn repo_id_for_path(repo_root: &Path) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, repo_root.to_string_lossy().as_bytes()).to_string()
}

pub fn task_hash(task: &str) -> String {
    blake3::hash(task.trim().as_bytes()).to_hex().to_string()
}

pub(crate) fn options_fingerprint(options: &InventoryOptions) -> String {
    let json = serde_json::to_vec(options).unwrap_or_default();
    blake3::hash(&json).to_hex().to_string()
}

pub(crate) fn ctxpack_home() -> PathBuf {
    std::env::var_os("CTXPACK_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".ctxpack")))
        .unwrap_or_else(|| PathBuf::from(".ctxpack"))
}

pub(crate) fn canonicalize(path: &Path) -> Result<PathBuf, InventoryError> {
    fs::canonicalize(path).map_err(|source| InventoryError::Canonicalize {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn normalize_relative_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn normalize_input_path(repo_root: &Path, path: &str) -> String {
    let path = Path::new(path);
    let path = if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn ignore_fingerprints(repo_root: &Path) -> Vec<IgnoreFileFingerprint> {
    [
        ".gitignore",
        ".git/info/exclude",
        ".ctxpackignore",
        ".cursorignore",
    ]
    .into_iter()
    .map(|path| ignore_file_fingerprint(repo_root, path))
    .collect()
}

fn ignore_file_fingerprint(repo_root: &Path, relative: &str) -> IgnoreFileFingerprint {
    let path = repo_root.join(relative);
    match fs::metadata(&path) {
        Ok(metadata) => {
            let bytes = fs::read(&path).unwrap_or_default();
            IgnoreFileFingerprint {
                path: relative.to_string(),
                present: true,
                hash: Some(blake3::hash(&bytes).to_hex().to_string()),
                size_bytes: metadata.len(),
                modified_unix_nanos: modified_unix_nanos(&metadata),
            }
        }
        Err(_) => IgnoreFileFingerprint {
            path: relative.to_string(),
            present: false,
            hash: None,
            size_bytes: 0,
            modified_unix_nanos: 0,
        },
    }
}

fn modified_unix_nanos(metadata: &fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
