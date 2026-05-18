use crate::inventory::{build_inventory, repo_id_for_path, InventoryError, InventoryOptions};
use crate::policy::classify_path;
use crate::storage::{storage_status_for_repo, StorageCompatibility, StoreConfig};
use ctxpack_core::{
    DiagnosticSeverity, FileRole, WorkspaceInventoryReport, WorkspaceManifest, WorkspaceRepo,
    WorkspaceRepoDiagnostic, WorkspaceRepoPrivacyStatus, WorkspaceRepoState, WorkspaceRepoStatus,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const WORKSPACE_MANIFEST_SCHEMA_VERSION: u32 = 1;

pub fn default_workspace_manifest_path(
    workspace_root: impl AsRef<Path>,
) -> Result<PathBuf, InventoryError> {
    let root = canonicalize_existing(workspace_root.as_ref())?;
    Ok(root.join(".ctxpack").join("workspace.json"))
}

pub fn write_workspace_manifest(
    workspace_root: impl AsRef<Path>,
    manifest: &WorkspaceManifest,
) -> Result<PathBuf, InventoryError> {
    let path = default_workspace_manifest_path(workspace_root)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(manifest).map_err(InventoryError::Serialize)?;
    fs::write(&path, json).map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

pub fn load_workspace_manifest(
    workspace_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<(PathBuf, WorkspaceManifest), InventoryError> {
    let path = match manifest_path {
        Some(path) => path.to_path_buf(),
        None => default_workspace_manifest_path(workspace_root)?,
    };
    let json = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
        path: path.clone(),
        source,
    })?;
    let manifest = serde_json::from_str(&json).map_err(|source| InventoryError::Deserialize {
        path: path.clone(),
        source,
    })?;
    Ok((path, manifest))
}

pub fn workspace_inventory_status(
    workspace_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<WorkspaceInventoryReport, InventoryError> {
    let workspace_root = canonicalize_existing(workspace_root.as_ref())?;
    let (manifest_path, manifest) = load_workspace_manifest(&workspace_root, manifest_path)?;
    Ok(workspace_inventory_status_for_manifest(
        &workspace_root,
        &manifest_path,
        &manifest,
    ))
}

pub fn workspace_inventory_status_for_manifest(
    workspace_root: &Path,
    manifest_path: &Path,
    manifest: &WorkspaceManifest,
) -> WorkspaceInventoryReport {
    let manifest_base = manifest_repo_base(manifest_path, workspace_root);
    let mut repos = Vec::new();
    let mut diagnostics = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut seen_labels = BTreeMap::<String, String>::new();

    for repo in &manifest.repos {
        let status = repo_status(workspace_root, manifest_base, repo);
        let duplicates = duplicate_diagnostics(repo, &status, &mut seen_ids, &mut seen_labels);
        if !duplicates.is_empty() {
            let mut repo_diagnostics = status.diagnostics.clone();
            repo_diagnostics.extend(duplicates.clone());
            diagnostics.extend(duplicates);
            diagnostics.extend(status.diagnostics.clone());
            repos.push(WorkspaceRepoStatus {
                diagnostics: repo_diagnostics,
                ..status
            });
            continue;
        }
        diagnostics.extend(status.diagnostics.clone());
        repos.push(status);
    }

    let available_repo_count = repos
        .iter()
        .filter(|repo| repo.state == WorkspaceRepoState::Available)
        .count();
    let file_count = repos.iter().map(|repo| repo.file_count).sum();
    let ignored_count = repos.iter().map(|repo| repo.ignored_count).sum();
    let generated_count = repos.iter().map(|repo| repo.generated_count).sum();
    let sensitive_count = repos.iter().map(|repo| repo.sensitive_count).sum();

    WorkspaceInventoryReport {
        schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
        workspace_root: workspace_root.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        repo_count: repos.len(),
        available_repo_count,
        file_count,
        ignored_count,
        generated_count,
        sensitive_count,
        source_text_logged: false,
        privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
        repos,
        diagnostics,
    }
}

fn repo_status(
    workspace_root: &Path,
    manifest_base: &Path,
    repo: &WorkspaceRepo,
) -> WorkspaceRepoStatus {
    let resolved = resolve_repo_path(manifest_base, &repo.path);
    let path_label = path_label(workspace_root, &resolved);
    let label = repo.label.clone().unwrap_or_else(|| path_label.clone());
    let mut diagnostics = validate_repo_metadata(repo, &label, &path_label);

    if !resolved.exists() {
        let repo_id = repo
            .id
            .clone()
            .unwrap_or_else(|| fallback_repo_id(&repo.path, &label));
        diagnostics.push(repo_diagnostic(
            "workspace_repo_missing",
            DiagnosticSeverity::Error,
            "Workspace repository path does not exist.",
            Some(repo_id.clone()),
            Some(label.clone()),
            vec![path_label.clone()],
        ));
        return empty_repo_status(
            repo_id,
            label,
            path_label,
            repo.tags.clone(),
            WorkspaceRepoState::Missing,
            false,
            diagnostics,
        );
    }

    let canonical = match canonicalize_existing(&resolved) {
        Ok(path) => path,
        Err(error) => {
            let repo_id = repo
                .id
                .clone()
                .unwrap_or_else(|| fallback_repo_id(&repo.path, &label));
            diagnostics.push(repo_diagnostic(
                "workspace_repo_inaccessible",
                DiagnosticSeverity::Error,
                format!("Workspace repository path could not be accessed: {error}"),
                Some(repo_id.clone()),
                Some(label.clone()),
                vec![path_label.clone()],
            ));
            return empty_repo_status(
                repo_id,
                label,
                path_label,
                repo.tags.clone(),
                WorkspaceRepoState::Invalid,
                false,
                diagnostics,
            );
        }
    };

    let repo_id = repo
        .id
        .clone()
        .unwrap_or_else(|| repo_id_for_path(&canonical));
    let git_root = canonical.join(".git").exists();
    if !git_root {
        diagnostics.push(repo_diagnostic(
            "workspace_repo_not_git_root",
            DiagnosticSeverity::Warning,
            "Workspace repository path is not a git root.",
            Some(repo_id.clone()),
            Some(label.clone()),
            vec![path_label.clone()],
        ));
    }

    match build_inventory(&canonical, &InventoryOptions::default()) {
        Ok(inventory) => {
            let storage = storage_status_for_repo(&canonical, &StoreConfig::default());
            let (storage_compatibility, storage_database_present, memory_card_count) = match storage
            {
                Ok(status) => (
                    Some(storage_compatibility_label(status.compatibility).to_string()),
                    status.database_path.exists(),
                    status.memory_card_records,
                ),
                Err(error) => {
                    diagnostics.push(repo_diagnostic(
                        "workspace_storage_status_unavailable",
                        DiagnosticSeverity::Warning,
                        format!("Storage status could not be inspected: {error}"),
                        Some(repo_id.clone()),
                        Some(label.clone()),
                        vec![path_label.clone()],
                    ));
                    (None, false, 0)
                }
            };
            WorkspaceRepoStatus {
                repo_id,
                label,
                path_label,
                tags: repo.tags.clone(),
                state: WorkspaceRepoState::Available,
                git_root,
                file_count: inventory.files.len(),
                ignored_count: inventory.ignored_count,
                generated_count: inventory.generated_count,
                sensitive_count: inventory.sensitive_count,
                storage_compatibility,
                storage_database_present,
                memory_card_count,
                privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
                diagnostics,
            }
        }
        Err(error) => {
            diagnostics.push(repo_diagnostic(
                "workspace_inventory_unavailable",
                DiagnosticSeverity::Error,
                format!("Workspace inventory could not be built: {error}"),
                Some(repo_id.clone()),
                Some(label.clone()),
                vec![path_label.clone()],
            ));
            empty_repo_status(
                repo_id,
                label,
                path_label,
                repo.tags.clone(),
                WorkspaceRepoState::Invalid,
                git_root,
                diagnostics,
            )
        }
    }
}

fn duplicate_diagnostics(
    repo: &WorkspaceRepo,
    status: &WorkspaceRepoStatus,
    seen_ids: &mut BTreeSet<String>,
    seen_labels: &mut BTreeMap<String, String>,
) -> Vec<WorkspaceRepoDiagnostic> {
    let mut diagnostics = Vec::new();
    if !seen_ids.insert(status.repo_id.clone()) {
        diagnostics.push(repo_diagnostic(
            "workspace_repo_duplicate_id",
            DiagnosticSeverity::Error,
            "Workspace manifest contains a duplicate repository ID.",
            Some(status.repo_id.clone()),
            Some(status.label.clone()),
            vec![status.path_label.clone()],
        ));
    }
    if let Some(previous_id) = seen_labels.insert(status.label.clone(), status.repo_id.clone()) {
        diagnostics.push(repo_diagnostic(
            "workspace_repo_duplicate_label",
            DiagnosticSeverity::Warning,
            format!("Workspace manifest label is also used by repository ID {previous_id}."),
            repo.id.clone().or_else(|| Some(status.repo_id.clone())),
            Some(status.label.clone()),
            vec![status.path_label.clone()],
        ));
    }
    diagnostics
}

fn validate_repo_metadata(
    repo: &WorkspaceRepo,
    label: &str,
    path_label: &str,
) -> Vec<WorkspaceRepoDiagnostic> {
    let mut diagnostics = Vec::new();
    for (field, value) in [
        ("id", repo.id.as_deref()),
        ("label", repo.label.as_deref()),
        ("path", Some(repo.path.as_str())),
    ] {
        if let Some(value) = value {
            if has_unsafe_string(value) {
                diagnostics.push(repo_diagnostic(
                    format!("workspace_repo_invalid_{field}"),
                    DiagnosticSeverity::Error,
                    format!(
                        "Workspace repository {field} contains control characters or is too long."
                    ),
                    repo.id.clone(),
                    Some(label.to_string()),
                    vec![path_label.to_string()],
                ));
            }
            let role = classify_path(value);
            if matches!(role, FileRole::Sensitive | FileRole::Generated) {
                diagnostics.push(repo_diagnostic(
                    format!("workspace_repo_{field}_looks_{}", role_label(&role)),
                    DiagnosticSeverity::Warning,
                    format!("Workspace repository {field} looks like a {role:?} path label."),
                    repo.id.clone(),
                    Some(label.to_string()),
                    vec![path_label.to_string()],
                ));
            }
        }
    }
    for tag in &repo.tags {
        if has_unsafe_string(tag) {
            diagnostics.push(repo_diagnostic(
                "workspace_repo_invalid_tag",
                DiagnosticSeverity::Error,
                "Workspace repository tag contains control characters or is too long.",
                repo.id.clone(),
                Some(label.to_string()),
                vec![path_label.to_string()],
            ));
        }
        let role = classify_path(tag);
        if matches!(role, FileRole::Sensitive | FileRole::Generated) {
            diagnostics.push(repo_diagnostic(
                format!("workspace_repo_tag_looks_{}", role_label(&role)),
                DiagnosticSeverity::Warning,
                format!("Workspace repository tag looks like a {role:?} path label."),
                repo.id.clone(),
                Some(label.to_string()),
                vec![path_label.to_string()],
            ));
        }
    }
    diagnostics
}

fn empty_repo_status(
    repo_id: String,
    label: String,
    path_label: String,
    tags: Vec<String>,
    state: WorkspaceRepoState,
    git_root: bool,
    diagnostics: Vec<WorkspaceRepoDiagnostic>,
) -> WorkspaceRepoStatus {
    WorkspaceRepoStatus {
        repo_id,
        label,
        path_label,
        tags,
        state,
        git_root,
        file_count: 0,
        ignored_count: 0,
        generated_count: 0,
        sensitive_count: 0,
        storage_compatibility: None,
        storage_database_present: false,
        memory_card_count: 0,
        privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
        diagnostics,
    }
}

fn repo_diagnostic(
    code: impl Into<String>,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
    repo_id: Option<String>,
    label: Option<String>,
    paths: Vec<String>,
) -> WorkspaceRepoDiagnostic {
    WorkspaceRepoDiagnostic {
        code: code.into(),
        severity,
        message: message.into(),
        repo_id,
        label,
        paths,
    }
}

fn resolve_repo_path(manifest_base: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        manifest_base.join(path)
    }
}

fn manifest_repo_base<'a>(manifest_path: &'a Path, workspace_root: &'a Path) -> &'a Path {
    let Some(parent) = manifest_path.parent() else {
        return workspace_root;
    };
    if parent.file_name().is_some_and(|name| name == ".ctxpack") {
        parent.parent().unwrap_or(workspace_root)
    } else {
        parent
    }
}

fn path_label(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn fallback_repo_id(path: &str, label: &str) -> String {
    let hash = blake3::hash(format!("{path}:{label}").as_bytes())
        .to_hex()
        .to_string();
    format!("unresolved-{}", &hash[..12])
}

fn storage_compatibility_label(value: StorageCompatibility) -> &'static str {
    match value {
        StorageCompatibility::Compatible => "compatible",
        StorageCompatibility::MissingMetadata => "missing_metadata",
        StorageCompatibility::MissingTables => "missing_tables",
        StorageCompatibility::IncompatibleSchema => "incompatible_schema",
    }
}

fn role_label(role: &FileRole) -> &'static str {
    match role {
        FileRole::Sensitive => "sensitive",
        FileRole::Generated => "generated",
        _ => "unsafe",
    }
}

fn has_unsafe_string(value: &str) -> bool {
    value.len() > 160 || value.chars().any(|character| character.is_control())
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, InventoryError> {
    fs::canonicalize(path).map_err(|source| InventoryError::Canonicalize {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git_repo(path: &Path) {
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/lib.rs"), "pub fn sentinel_free() {}\n").unwrap();
        fs::write(path.join(".env"), "CTXPACK_WORKSPACE_SOURCE_SENTINEL\n").unwrap();
        let output = Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn workspace_status_reports_multiple_repos_without_source_text() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        let api = workspace.join("api");
        let web = workspace.join("web");
        git_repo(&api);
        git_repo(&web);

        let manifest = WorkspaceManifest {
            schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
            workspace_id: Some("local-stack".to_string()),
            repos: vec![
                WorkspaceRepo {
                    id: Some("api".to_string()),
                    path: "api".to_string(),
                    label: Some("api".to_string()),
                    tags: vec!["backend".to_string()],
                },
                WorkspaceRepo {
                    id: Some("web".to_string()),
                    path: "web".to_string(),
                    label: Some("web".to_string()),
                    tags: vec!["frontend".to_string()],
                },
            ],
        };
        let manifest_path = workspace.join(".ctxpack/workspace.json");
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let report = workspace_inventory_status(&workspace, None).unwrap();
        assert_eq!(report.repo_count, 2);
        assert_eq!(report.available_repo_count, 2);
        assert_eq!(report.source_text_logged, false);
        assert!(report.file_count >= 2);
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("CTXPACK_WORKSPACE_SOURCE_SENTINEL"));
        assert!(!json.contains("pub fn sentinel_free"));
    }

    #[test]
    fn workspace_status_diagnoses_missing_and_duplicate_repos() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        let api = workspace.join("api");
        git_repo(&api);
        fs::create_dir_all(workspace.join(".ctxpack")).unwrap();
        let manifest = WorkspaceManifest {
            schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
            workspace_id: None,
            repos: vec![
                WorkspaceRepo {
                    id: Some("api".to_string()),
                    path: "api".to_string(),
                    label: Some("api".to_string()),
                    tags: vec![],
                },
                WorkspaceRepo {
                    id: Some("api".to_string()),
                    path: "missing".to_string(),
                    label: Some("api".to_string()),
                    tags: vec![".env".to_string()],
                },
            ],
        };
        let manifest_path = write_workspace_manifest(&workspace, &manifest).unwrap();
        let report = workspace_inventory_status(&workspace, Some(&manifest_path)).unwrap();
        let codes = report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert!(codes.contains(&"workspace_repo_missing"));
        assert!(codes.contains(&"workspace_repo_duplicate_id"));
        assert!(codes.contains(&"workspace_repo_duplicate_label"));
        assert!(codes.contains(&"workspace_repo_tag_looks_sensitive"));
    }
}
