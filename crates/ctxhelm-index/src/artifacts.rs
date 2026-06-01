use crate::inventory::{repo_id_for_path, InventoryError};
use ctxhelm_core::{
    DiagnosticSeverity, SharedArtifactEntry, SharedArtifactInspectionReport, SharedArtifactKind,
    SharedArtifactManifest, SharedArtifactStatus, TeamPolicyReport, TeamPrivacyPolicy,
    WorkspaceRepoDiagnostic, WorkspaceRepoPrivacyStatus,
};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SHARED_ARTIFACT_SCHEMA_VERSION: u32 = 1;
pub const TEAM_POLICY_SCHEMA_VERSION: u32 = 1;

pub fn shared_artifact_manifest_path(repo_root: impl AsRef<Path>) -> PathBuf {
    repo_root
        .as_ref()
        .join(".ctxhelm")
        .join("shared-artifacts.json")
}

pub fn imported_shared_artifact_manifest_path(repo_root: impl AsRef<Path>) -> PathBuf {
    repo_root
        .as_ref()
        .join(".ctxhelm")
        .join("imported-shared-artifacts.json")
}

pub fn team_policy_path(repo_root: impl AsRef<Path>) -> PathBuf {
    repo_root.as_ref().join(".ctxhelm").join("team-policy.json")
}

pub fn export_shared_artifact_manifest(
    repo_root: impl AsRef<Path>,
) -> Result<SharedArtifactManifest, InventoryError> {
    let repo_root = canonicalize_existing(repo_root.as_ref())?;
    let manifest = build_shared_artifact_manifest(&repo_root)?;
    let path = shared_artifact_manifest_path(&repo_root);
    write_json(&path, &manifest)?;
    Ok(manifest)
}

pub fn inspect_shared_artifact_manifest(
    manifest_path: impl AsRef<Path>,
) -> Result<SharedArtifactInspectionReport, InventoryError> {
    let path = manifest_path.as_ref();
    let json = fs::read_to_string(path).map_err(|source| InventoryError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: SharedArtifactManifest =
        serde_json::from_str(&json).map_err(|source| InventoryError::Deserialize {
            path: path.to_path_buf(),
            source,
        })?;
    let mut diagnostics = manifest.diagnostics.clone();
    let compatible = manifest.schema_version == SHARED_ARTIFACT_SCHEMA_VERSION
        && !manifest.source_text_logged
        && manifest
            .artifacts
            .iter()
            .all(|artifact| !artifact.source_text_logged);
    if manifest.schema_version != SHARED_ARTIFACT_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            "shared_artifact_schema_mismatch",
            DiagnosticSeverity::Error,
            "Shared artifact manifest schema version is not supported.",
            vec![path.display().to_string()],
        ));
    }
    if manifest.source_text_logged
        || manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.source_text_logged)
    {
        diagnostics.push(diagnostic(
            "shared_artifact_source_text_logged",
            DiagnosticSeverity::Error,
            "Shared artifact manifest is not source-free.",
            vec![path.display().to_string()],
        ));
    }
    Ok(SharedArtifactInspectionReport {
        manifest_path: path.display().to_string(),
        artifact_count: manifest.artifacts.len(),
        compatible,
        source_text_logged: manifest.source_text_logged,
        privacy_status: manifest.privacy_status,
        artifacts: manifest.artifacts,
        diagnostics,
    })
}

pub fn import_shared_artifact_manifest(
    repo_root: impl AsRef<Path>,
    input: impl AsRef<Path>,
) -> Result<SharedArtifactInspectionReport, InventoryError> {
    let repo_root = canonicalize_existing(repo_root.as_ref())?;
    let input = input.as_ref();
    let report = inspect_shared_artifact_manifest(input)?;
    if !report.compatible {
        return Ok(report);
    }
    let destination = imported_shared_artifact_manifest_path(&repo_root);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(input, &destination).map_err(|source| InventoryError::Write {
        path: destination.clone(),
        source,
    })?;
    inspect_shared_artifact_manifest(destination)
}

pub fn write_team_policy_template(
    repo_root: impl AsRef<Path>,
) -> Result<TeamPolicyReport, InventoryError> {
    let repo_root = canonicalize_existing(repo_root.as_ref())?;
    let policy = default_team_policy();
    let path = team_policy_path(&repo_root);
    write_json(&path, &policy)?;
    team_policy_report(&repo_root)
}

pub fn team_policy_report(repo_root: impl AsRef<Path>) -> Result<TeamPolicyReport, InventoryError> {
    let repo_root = canonicalize_existing(repo_root.as_ref())?;
    let path = team_policy_path(&repo_root);
    let policy = if path.exists() {
        let json = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
            path: path.clone(),
            source,
        })?;
        serde_json::from_str(&json).map_err(|source| InventoryError::Deserialize {
            path: path.clone(),
            source,
        })?
    } else {
        default_team_policy()
    };
    let mut diagnostics = Vec::new();
    if policy.allow_cloud_embeddings || policy.allow_cloud_reranking {
        diagnostics.push(diagnostic(
            "team_policy_cloud_enabled",
            DiagnosticSeverity::Warning,
            "Cloud retrieval features are allowed by this policy.",
            vec![path.display().to_string()],
        ));
    }
    let all = vec![
        SharedArtifactKind::ContextCards,
        SharedArtifactKind::BenchmarkReport,
        SharedArtifactKind::PolicyProfile,
        SharedArtifactKind::FeedbackSummary,
        SharedArtifactKind::ProofReport,
        SharedArtifactKind::WorkspaceManifest,
        SharedArtifactKind::TeamPolicy,
    ];
    let (allowed_artifacts, blocked_artifacts) = if policy.allow_artifact_export {
        (all.clone(), Vec::new())
    } else {
        (Vec::new(), all)
    };
    let redacted_artifacts = if policy.redact_secrets {
        vec![
            SharedArtifactKind::ContextCards,
            SharedArtifactKind::FeedbackSummary,
            SharedArtifactKind::PolicyProfile,
        ]
    } else {
        Vec::new()
    };
    let degraded_artifacts = if policy.allow_source_snippets_in_shared_artifacts {
        vec![SharedArtifactKind::ContextCards]
    } else {
        Vec::new()
    };
    Ok(TeamPolicyReport {
        policy_path: path.display().to_string(),
        policy,
        allowed_artifacts,
        blocked_artifacts,
        degraded_artifacts,
        redacted_artifacts,
        source_text_logged: false,
        privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
        diagnostics,
    })
}

fn build_shared_artifact_manifest(
    repo_root: &Path,
) -> Result<SharedArtifactManifest, InventoryError> {
    let repo_id = repo_id_for_path(repo_root);
    let repo_label = repo_root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string());
    let candidates = [
        (
            SharedArtifactKind::ContextCards,
            ".ctxhelm/cards",
            repo_root.join(".ctxhelm/cards"),
        ),
        (
            SharedArtifactKind::BenchmarkReport,
            ".ctxhelm/benchmark-report.json",
            repo_root.join(".ctxhelm/benchmark-report.json"),
        ),
        (
            SharedArtifactKind::PolicyProfile,
            ".ctxhelm/policy-profiles.jsonl",
            repo_root.join(".ctxhelm/policy-profiles.jsonl"),
        ),
        (
            SharedArtifactKind::FeedbackSummary,
            ".ctxhelm/feedback-summary.json",
            repo_root.join(".ctxhelm/feedback-summary.json"),
        ),
        (
            SharedArtifactKind::ProofReport,
            ".ctxhelm/proof-report.json",
            repo_root.join(".ctxhelm/proof-report.json"),
        ),
        (
            SharedArtifactKind::WorkspaceManifest,
            ".ctxhelm/workspace.json",
            repo_root.join(".ctxhelm/workspace.json"),
        ),
        (
            SharedArtifactKind::TeamPolicy,
            ".ctxhelm/team-policy.json",
            team_policy_path(repo_root),
        ),
    ];
    let artifacts = candidates
        .into_iter()
        .map(|(kind, label, path)| artifact_entry(kind, label, &path))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SharedArtifactManifest {
        schema_version: SHARED_ARTIFACT_SCHEMA_VERSION,
        repo_id,
        repo_label,
        exported_at_unix_seconds: current_unix_seconds(),
        source_text_logged: false,
        privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
        artifacts,
        diagnostics: Vec::new(),
    })
}

fn artifact_entry(
    kind: SharedArtifactKind,
    label: &str,
    path: &Path,
) -> Result<SharedArtifactEntry, InventoryError> {
    let metadata = fs::metadata(path).ok();
    let present = metadata.is_some();
    let (content_hash, size_bytes) = if present {
        if path.is_file() {
            let bytes = fs::read(path).map_err(|source| InventoryError::Read {
                path: path.to_path_buf(),
                source,
            })?;
            (
                Some(blake3::hash(&bytes).to_hex().to_string()),
                bytes.len() as u64,
            )
        } else {
            (Some(directory_hash(path)?), 0)
        }
    } else {
        (None, 0)
    };
    Ok(SharedArtifactEntry {
        id: artifact_id(&kind, label),
        kind,
        status: if present {
            SharedArtifactStatus::Present
        } else {
            SharedArtifactStatus::Missing
        },
        path_label: normalize_label(label),
        content_hash,
        size_bytes,
        generated_at_unix_seconds: current_unix_seconds(),
        source_text_logged: false,
        privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
        diagnostics: Vec::new(),
    })
}

fn directory_hash(path: &Path) -> Result<String, InventoryError> {
    let mut hasher = blake3::Hasher::new();
    let mut files = Vec::new();
    collect_files(path, &mut files)?;
    files.sort();
    for file in files {
        hasher.update(normalize_label(&file.display().to_string()).as_bytes());
        let bytes = fs::read(&file).map_err(|source| InventoryError::Read {
            path: file.clone(),
            source,
        })?;
        hasher.update(blake3::hash(&bytes).as_bytes());
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), InventoryError> {
    for entry in fs::read_dir(path).map_err(|source| InventoryError::Read {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| InventoryError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_files(&entry_path, files)?;
        } else if entry_path.is_file() {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn artifact_id(kind: &SharedArtifactKind, label: &str) -> String {
    format!("{kind:?}:{}", blake3::hash(label.as_bytes()).to_hex())
}

fn default_team_policy() -> TeamPrivacyPolicy {
    TeamPrivacyPolicy {
        schema_version: TEAM_POLICY_SCHEMA_VERSION,
        name: "local-source-free".to_string(),
        allow_workspace_indexing: true,
        allow_artifact_export: true,
        allow_cloud_embeddings: false,
        allow_cloud_reranking: false,
        allow_source_snippets_in_shared_artifacts: false,
        redact_secrets: true,
        source_text_logged: false,
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), InventoryError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(value).map_err(InventoryError::Serialize)?;
    fs::write(path, json).map_err(|source| InventoryError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn diagnostic(
    code: impl Into<String>,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
    paths: Vec<String>,
) -> WorkspaceRepoDiagnostic {
    WorkspaceRepoDiagnostic {
        code: code.into(),
        severity,
        message: message.into(),
        repo_id: None,
        label: None,
        paths,
    }
}

fn normalize_label(label: &str) -> String {
    Path::new(label)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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

    #[test]
    fn shared_manifest_and_policy_are_source_free() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join(".ctxhelm/cards")).unwrap();
        fs::write(
            repo.join(".ctxhelm/cards/auth.md"),
            "source-free auth summary\n",
        )
        .unwrap();
        fs::write(repo.join(".env"), "CTXHELM_ARTIFACT_SECRET_SENTINEL\n").unwrap();

        let policy = write_team_policy_template(&repo).unwrap();
        assert!(!policy.policy.allow_cloud_embeddings);
        assert!(!policy.source_text_logged);

        let manifest = export_shared_artifact_manifest(&repo).unwrap();
        assert!(!manifest.source_text_logged);
        assert!(manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == SharedArtifactKind::ContextCards
                && artifact.status == SharedArtifactStatus::Present));
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(!json.contains("CTXHELM_ARTIFACT_SECRET_SENTINEL"));

        let report =
            inspect_shared_artifact_manifest(shared_artifact_manifest_path(&repo)).unwrap();
        assert!(report.compatible);
        assert!(!report.source_text_logged);

        let imported =
            import_shared_artifact_manifest(&repo, shared_artifact_manifest_path(&repo)).unwrap();
        assert!(imported.compatible);
        assert!(imported_shared_artifact_manifest_path(&repo).exists());
    }
}
