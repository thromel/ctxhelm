use crate::inventory::{
    build_inventory, build_inventory_freshness_metadata, canonicalize, inventory_path,
    load_inventory, options_fingerprint, persist_inventory, repo_id_for_path, InventoryError,
    InventoryManifestEntry, InventoryMetadata, InventoryOptions, RepoInventory,
    INVENTORY_SCHEMA_VERSION,
};
use crate::policy::POLICY_VERSION;
use ctxpack_core::{CacheStatus, CacheStatusKind, Diagnostic, DiagnosticSeverity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryFreshness {
    pub fresh: bool,
    pub reasons: Vec<InventoryStaleReason>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum InventoryStaleReason {
    MissingMetadata,
    SchemaChanged,
    PolicyChanged,
    OptionsChanged,
    RepoRootChanged,
    IgnoreFileChanged,
    FileCreated,
    FileDeleted,
    FileChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoadCacheStatus {
    Hit,
    Miss,
    Rebuilt,
    WriteFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryLoadReport {
    pub inventory: RepoInventory,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub freshness: InventoryFreshness,
    pub cache_status: CacheStatus,
}

pub fn check_inventory_freshness(
    repo_root: impl AsRef<Path>,
    cached: &RepoInventory,
    options: &InventoryOptions,
) -> Result<InventoryFreshness, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let current_metadata =
        build_inventory_freshness_metadata(&repo_root, options, &cached.metadata.manifest)?;
    Ok(compare_inventory_metadata(
        &repo_root,
        cached,
        &current_metadata,
        options,
    ))
}

pub fn load_or_refresh_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<InventoryLoadReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = inventory_path(&repo_id);

    match load_inventory(&repo_id) {
        Ok(cached) => {
            let cached_freshness = check_inventory_freshness(&repo_root, &cached, options)?;
            if cached_freshness.fresh {
                return Ok(InventoryLoadReport {
                    inventory: cached,
                    diagnostics: Vec::new(),
                    freshness: cached_freshness,
                    cache_status: cache_status(CacheStatusKind::Hit, &path, Vec::new()),
                });
            }

            let mut diagnostics = vec![diagnostic(
                "inventory_stale",
                DiagnosticSeverity::Warning,
                "Cached inventory was stale and was rebuilt before returning results.",
                cached_freshness
                    .diagnostics
                    .iter()
                    .flat_map(|diagnostic| diagnostic.paths.clone())
                    .collect(),
            )];
            diagnostics.extend(cached_freshness.diagnostics);
            rebuild_inventory_report(&repo_root, options, diagnostics, &path)
        }
        Err(InventoryError::Read { .. }) => rebuild_inventory_report(
            &repo_root,
            options,
            vec![diagnostic(
                "inventory_cache_miss",
                DiagnosticSeverity::Info,
                "Cached inventory was missing or unreadable and was rebuilt.",
                Vec::new(),
            )],
            &path,
        ),
        Err(error) => Err(error),
    }
}

fn rebuild_inventory_report(
    repo_root: &Path,
    options: &InventoryOptions,
    mut diagnostics: Vec<Diagnostic>,
    path: &Path,
) -> Result<InventoryLoadReport, InventoryError> {
    let inventory = match build_inventory(repo_root, options) {
        Ok(inventory) => inventory,
        Err(error) => {
            diagnostics.push(diagnostic(
                "inventory_rebuild_failed",
                DiagnosticSeverity::Error,
                "Inventory rebuild failed before fresh results could be returned.",
                Vec::new(),
            ));
            return Err(error);
        }
    };

    diagnostics.push(diagnostic(
        "inventory_rebuilt",
        DiagnosticSeverity::Info,
        "Inventory was rebuilt before returning results.",
        Vec::new(),
    ));

    let freshness = InventoryFreshness {
        fresh: true,
        reasons: Vec::new(),
        diagnostics: Vec::new(),
    };

    match persist_inventory(&inventory) {
        Ok(path) => Ok(InventoryLoadReport {
            inventory,
            diagnostics,
            freshness,
            cache_status: cache_status(CacheStatusKind::Rebuilt, &path, Vec::new()),
        }),
        Err(error) => {
            let cache_diagnostic = diagnostic(
                "cache_write_failed",
                DiagnosticSeverity::Warning,
                "Fresh inventory was returned, but ctxpack could not persist the cache.",
                vec![path.to_string_lossy().to_string()],
            );
            diagnostics.push(cache_diagnostic.clone());
            Ok(InventoryLoadReport {
                inventory,
                diagnostics,
                freshness,
                cache_status: cache_status(
                    CacheStatusKind::WriteFailed,
                    path,
                    vec![cache_diagnostic],
                ),
            })
            .map(|mut report| {
                report.cache_status.diagnostics[0].message =
                    format!("{} ({error})", report.cache_status.diagnostics[0].message);
                report
            })
        }
    }
}

fn compare_inventory_metadata(
    repo_root: &Path,
    cached: &RepoInventory,
    current_metadata: &InventoryMetadata,
    options: &InventoryOptions,
) -> InventoryFreshness {
    let mut reasons = Vec::new();
    let mut diagnostics = Vec::new();
    let metadata = &cached.metadata;

    if metadata.schema_version == 0 {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::MissingMetadata,
            "inventory_stale_missing_metadata",
            "Cached inventory was built before freshness metadata existed.",
            Vec::new(),
        );
    } else if metadata.schema_version != INVENTORY_SCHEMA_VERSION {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::SchemaChanged,
            "inventory_stale_schema_changed",
            "Cached inventory schema version changed.",
            Vec::new(),
        );
    }

    if metadata.policy_version != POLICY_VERSION {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::PolicyChanged,
            "inventory_stale_policy_changed",
            "Cached inventory policy version changed.",
            Vec::new(),
        );
    }

    if metadata.options_fingerprint != options_fingerprint(options) {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::OptionsChanged,
            "inventory_stale_options_changed",
            "Cached inventory options changed.",
            Vec::new(),
        );
    }

    if metadata.repo_root != repo_root || cached.repo_root != repo_root {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::RepoRootChanged,
            "inventory_stale_repo_root_changed",
            "Cached inventory was built for a different repository root.",
            vec![metadata.repo_root.to_string_lossy().to_string()],
        );
    }

    let changed_ignores = metadata
        .ignore_fingerprints
        .iter()
        .zip(current_metadata.ignore_fingerprints.iter())
        .filter_map(|(cached, current)| (cached != current).then(|| current.path.clone()))
        .collect::<Vec<_>>();
    if !changed_ignores.is_empty() {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::IgnoreFileChanged,
            "inventory_stale_ignore_changed",
            "Repository ignore configuration changed.",
            changed_ignores,
        );
    }

    let cached_manifest = manifest_by_path(&metadata.manifest);
    let current_manifest = manifest_by_path(&current_metadata.manifest);

    let created = current_manifest
        .keys()
        .filter(|path| !cached_manifest.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !created.is_empty() {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::FileCreated,
            "inventory_stale_file_created",
            "Repository files were created since cached inventory was built.",
            created,
        );
    }

    let deleted = cached_manifest
        .keys()
        .filter(|path| !current_manifest.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !deleted.is_empty() {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::FileDeleted,
            "inventory_stale_file_deleted",
            "Repository files were deleted or excluded since cached inventory was built.",
            deleted,
        );
    }

    let changed = cached_manifest
        .iter()
        .filter_map(|(path, cached)| {
            current_manifest
                .get(path)
                .is_some_and(|current| current != cached)
                .then(|| path.clone())
        })
        .collect::<Vec<_>>();
    if !changed.is_empty() {
        push_reason(
            &mut reasons,
            &mut diagnostics,
            InventoryStaleReason::FileChanged,
            "inventory_stale_file_changed",
            "Repository file metadata changed since cached inventory was built.",
            changed,
        );
    }

    reasons.sort();
    reasons.dedup();

    InventoryFreshness {
        fresh: reasons.is_empty(),
        reasons,
        diagnostics,
    }
}

fn cache_status(status: CacheStatusKind, path: &Path, diagnostics: Vec<Diagnostic>) -> CacheStatus {
    CacheStatus {
        status,
        path: Some(path.to_string_lossy().to_string()),
        diagnostics,
    }
}

fn manifest_by_path(
    manifest: &[InventoryManifestEntry],
) -> BTreeMap<String, &InventoryManifestEntry> {
    manifest
        .iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect()
}

fn diagnostic(
    code: &str,
    severity: DiagnosticSeverity,
    message: &str,
    paths: Vec<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity,
        message: message.to_string(),
        count: paths.len(),
        paths,
    }
}

fn push_reason(
    reasons: &mut Vec<InventoryStaleReason>,
    diagnostics: &mut Vec<Diagnostic>,
    reason: InventoryStaleReason,
    code: &str,
    message: &str,
    mut paths: Vec<String>,
) {
    paths.sort();
    reasons.push(reason);
    diagnostics.push(Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: message.to_string(),
        count: paths.len(),
        paths,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        load_inventory, repo_id_for_path, write_inventory, InventoryMetadata,
        INVENTORY_SCHEMA_VERSION,
    };
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn paths(inventory: &RepoInventory) -> Vec<&str> {
        inventory
            .files
            .iter()
            .map(|entry| entry.path.as_str())
            .collect()
    }

    #[test]
    fn freshness_detects_file_create_delete_and_rename() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn old() {}\n").unwrap();

        let cached = build_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(repo.join("src/new.rs"), "pub fn new() {}\n").unwrap();
        fs::rename(repo.join("src/lib.rs"), repo.join("src/renamed.rs")).unwrap();
        let freshness =
            check_inventory_freshness(&repo, &cached, &InventoryOptions::default()).unwrap();

        assert!(!freshness.fresh);
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::FileCreated));
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::FileDeleted));
        assert!(freshness.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "inventory_stale_file_created"
                && diagnostic.paths == vec!["src/new.rs".to_string(), "src/renamed.rs".to_string()]
        }));
        assert!(freshness.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "inventory_stale_file_deleted"
                && diagnostic.paths == vec!["src/lib.rs".to_string()]
        }));
    }

    #[test]
    fn freshness_detects_content_and_ignore_file_mutations() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn old() {}\n").unwrap();
        fs::write(repo.join(".ctxpackignore"), "ignored-before.rs\n").unwrap();

        let cached = build_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
        fs::write(repo.join(".ctxpackignore"), "src/lib.rs\n").unwrap();
        let freshness =
            check_inventory_freshness(&repo, &cached, &InventoryOptions::default()).unwrap();

        assert!(!freshness.fresh);
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::IgnoreFileChanged));
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::FileDeleted));
        assert!(freshness
            .diagnostics
            .iter()
            .any(
                |diagnostic| diagnostic.code == "inventory_stale_ignore_changed"
                    && diagnostic.paths == vec![".ctxpackignore".to_string()]
            ));
    }

    #[test]
    fn freshness_detects_option_policy_and_repo_root_drift() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let other_repo = temp.path().join("other");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(other_repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(repo.join("src.rs"), "pub fn source() {}\n").unwrap();
        fs::write(repo.join("dist/app.min.js"), "minified\n").unwrap();

        let cached = build_inventory(
            &repo,
            &InventoryOptions {
                include_generated: false,
                include_sensitive: false,
            },
        )
        .unwrap();
        let freshness = check_inventory_freshness(
            &repo,
            &cached,
            &InventoryOptions {
                include_generated: true,
                include_sensitive: false,
            },
        )
        .unwrap();
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::OptionsChanged));

        let mut policy_drift = cached.clone();
        policy_drift.metadata.policy_version = "old-policy".to_string();
        let freshness =
            check_inventory_freshness(&repo, &policy_drift, &InventoryOptions::default()).unwrap();
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::PolicyChanged));

        let freshness =
            check_inventory_freshness(&other_repo, &cached, &InventoryOptions::default()).unwrap();
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::RepoRootChanged));
    }

    #[test]
    fn old_inventory_json_without_metadata_deserializes_as_stale() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join("src.rs"), "pub fn source() {}\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        let repo_id = repo_id_for_path(&fs::canonicalize(&repo).unwrap());
        let path = crate::inventory_path(&repo_id);
        let mut json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        json.as_object_mut().unwrap().remove("metadata");
        fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let cached = load_inventory(&repo_id).unwrap();
        assert_eq!(cached.metadata, InventoryMetadata::default());

        let freshness =
            check_inventory_freshness(&repo, &cached, &InventoryOptions::default()).unwrap();
        assert!(!freshness.fresh);
        assert!(freshness
            .reasons
            .contains(&InventoryStaleReason::MissingMetadata));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn inventory_metadata_records_safe_file_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn source() {}\n").unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();

        let inventory = build_inventory(&repo, &InventoryOptions::default()).unwrap();

        assert_eq!(paths(&inventory), vec!["src/lib.rs"]);
        assert_eq!(inventory.metadata.schema_version, INVENTORY_SCHEMA_VERSION);
        assert_eq!(inventory.metadata.manifest.len(), 1);
        assert_eq!(inventory.metadata.manifest[0].path, "src/lib.rs");
        assert_eq!(inventory.metadata.manifest[0].hash.len(), 64);
        assert!(inventory
            .metadata
            .ignore_fingerprints
            .iter()
            .any(|fingerprint| fingerprint.path == ".gitignore"));
    }

    #[test]
    fn load_or_refresh_rebuilds_stale_cache_after_file_mutation() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn old() {}\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(repo.join("src/new.rs"), "pub fn new() {}\n").unwrap();

        let report = load_or_refresh_inventory(&repo, &InventoryOptions::default()).unwrap();
        let paths = paths(&report.inventory);

        assert!(paths.contains(&"src/lib.rs"));
        assert!(paths.contains(&"src/new.rs"));
        assert!(report.freshness.fresh);
        assert_eq!(
            report.cache_status.status,
            ctxpack_core::CacheStatusKind::Rebuilt
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_stale"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_rebuilt"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn load_or_refresh_returns_inventory_when_cache_write_fails() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let blocked_home = temp.path().join("blocked-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join("src.rs"), "pub fn source() {}\n").unwrap();
        fs::write(&blocked_home, "not a directory\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &blocked_home);

        let explicit_write = write_inventory(&repo, &InventoryOptions::default());
        assert!(matches!(
            explicit_write,
            Err(crate::InventoryError::CreateDir { .. })
        ));

        let report = load_or_refresh_inventory(&repo, &InventoryOptions::default()).unwrap();
        assert_eq!(paths(&report.inventory), vec!["src.rs"]);
        assert_eq!(
            report.cache_status.status,
            ctxpack_core::CacheStatusKind::WriteFailed
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "cache_write_failed"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_rebuilt"));

        std::env::remove_var("CTXPACK_HOME");
    }
}
