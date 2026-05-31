mod artifacts;
mod dependencies;
mod feedback;
mod freshness;
mod git;
mod inventory;
mod policy;
mod related_tests;
mod search;
mod semantic;
mod storage;
mod symbols;
mod traces;
mod tree_sitter_backend;
mod workspace;

pub use artifacts::{
    export_shared_artifact_manifest, import_shared_artifact_manifest,
    imported_shared_artifact_manifest_path, inspect_shared_artifact_manifest,
    shared_artifact_manifest_path, team_policy_path, team_policy_report,
    write_team_policy_template, SHARED_ARTIFACT_SCHEMA_VERSION, TEAM_POLICY_SCHEMA_VERSION,
};
pub use dependencies::{
    dependency_edges, dependency_edges_report, discover_precision_edges, import_precision_edges,
    precision_edges_path, related_dependency_edges, related_dependency_edges_report,
    DependencyEdge, DependencyEdgesReport, DependencyOptions, PrecisionDiscoverOptions,
    PrecisionDiscoverReport, PrecisionEdgeRecord, PrecisionEdgesFile, PrecisionImportReport,
    PRECISION_EDGES_SCHEMA_VERSION,
};
pub use feedback::{
    append_feedback_event, apply_policy_profile, disable_policy_profile, feedback_path,
    list_feedback_events, list_policy_profiles, outcome_comparison_report, policy_profiles_path,
    policy_quality_report, propose_learned_policy_profile, propose_policy_profile,
    rollback_policy_profile, summarize_feedback_events, try_append_feedback_event,
    LearnedPolicyOptions, FEEDBACK_EVENT_SCHEMA_VERSION, LEARNED_POLICY_PROFILE_SCHEMA_VERSION,
};
pub use freshness::{
    check_inventory_freshness, load_or_refresh_inventory, InventoryFreshness, InventoryLoadReport,
    InventoryStaleReason, LoadCacheStatus,
};
pub use git::{
    co_change_hints, co_change_hints_report, current_diff_summary, current_diff_summary_report,
    historical_commit_samples, historical_commit_samples_report,
    historical_commit_samples_with_safe_paths, write_eval_history_sidecar, ChangeKind,
    CoChangeHint, CoChangeOptions, CoChangeReport, CurrentDiffExcluded, CurrentDiffOptions,
    CurrentDiffPrivacyStatus, CurrentDiffReport, CurrentDiffSummary, HistoricalChangedPath,
    HistoricalCommitOptions, HistoricalCommitReport, HistoricalCommitSample,
    HistoricalPathExclusionReason, LabelScope,
};
pub use inventory::{
    build_inventory, inventory_path, load_inventory, load_or_build_inventory, repo_id_for_path,
    task_hash, write_inventory, FileInventoryEntry, IgnoreFileFingerprint, InventoryError,
    InventoryManifestEntry, InventoryMetadata, InventoryOptions, InventoryReport, RepoInventory,
    INVENTORY_SCHEMA_VERSION,
};
pub use policy::{
    classify_path, read_safe_source, SourceRead, SourceReadReason, SourceReadStatus,
    SOURCE_READ_MAX_BYTES,
};
pub use related_tests::{
    related_tests, related_tests_report, test_map, test_map_report, RelatedTestResult,
    RelatedTestsReport,
};
pub use search::{
    lexical_search, lexical_search_report, SearchOptions, SearchReport, SearchResult,
};
pub use semantic::{
    normalized_provider, semantic_document_report, semantic_search, semantic_search_report,
    semantic_vector_records, sync_semantic_index_to_store, SemanticDocumentOptions,
    SemanticOptions, SemanticProviderConfig, SemanticSearchReport, SemanticSearchResult,
    SemanticVectorRecord, DEFAULT_SEMANTIC_DIMENSIONS, DEFAULT_SEMANTIC_DISTANCE,
    DEFAULT_SEMANTIC_MODEL, DEFAULT_SEMANTIC_PROVIDER,
};
pub use storage::{
    initialize_store, inspect_store_schema, list_memory_cards, open_store_report,
    persist_benchmark_run_record, persist_context_pack_record, persist_memory_card_records,
    persist_proof_report_record, persist_semantic_vector_records, required_table_names,
    storage_status_for_path, storage_status_for_repo, sync_inventory_to_store,
    update_memory_card_review_status, vacuum_store, StorageBenchmarkRunRecord,
    StorageCompatibility, StorageContextPackRecord, StorageError, StorageGapRecord,
    StorageIndexReport, StorageMemoryCardRecord, StorageMetadata, StorageMetricRecord,
    StoragePrivacyMode, StorageProofReportRecord, StorageReport, StorageSchemaReport,
    StorageSemanticIndexReport, StorageSemanticVectorRecord, StorageStatusReport, StoreConfig,
    StorePaths, STORAGE_SCHEMA_VERSION,
};
pub use symbols::{
    extract_symbols, extract_symbols_report, symbol_search, symbol_search_report, CodeSymbol,
    SymbolExtractionReport, SymbolKind, SymbolOptions, SymbolSearchReport, SymbolSearchResult,
};
pub use traces::{append_eval_trace, list_eval_traces, trace_path, try_append_eval_trace};
pub use workspace::{
    default_workspace_manifest_path, load_workspace_manifest, workspace_inventory_status,
    workspace_inventory_status_for_manifest, write_workspace_manifest,
    WORKSPACE_MANIFEST_SCHEMA_VERSION,
};

#[cfg(test)]
use ctxpack_core::{EvalTrace, FeedbackOutcome, FileRole, PackBudget, SessionFeedbackEvent};
#[cfg(test)]
use git::{
    git_commit_subject_file_sets_with_timeouts, git_stdout_with_timeout, parse_git_log_name_only,
    parse_git_log_subject_name_only,
};
#[cfg(test)]
use std::{fs, path::Path, process::Command, time::Duration};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]

mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn classifies_common_file_roles() {
        assert_eq!(classify_path("src/lib.rs"), FileRole::Source);
        assert_eq!(classify_path("src/lib.test.ts"), FileRole::Test);
        assert_eq!(classify_path("tests/auth_test.py"), FileRole::Test);
        assert_eq!(classify_path("package.json"), FileRole::Config);
        assert_eq!(classify_path("db/migrations/001.sql"), FileRole::Schema);
        assert_eq!(classify_path("README.md"), FileRole::Docs);
        assert_eq!(classify_path(".env"), FileRole::Sensitive);
        assert_eq!(classify_path("dist/app.min.js"), FileRole::Generated);
        assert_eq!(
            classify_path(".gradle/7.4/fileHashes.bin"),
            FileRole::Generated
        );
        assert_eq!(classify_path("build 2/tmp/cache.bin"), FileRole::Generated);
        assert_eq!(
            classify_path("src/test/resources/oracle/commits/example.json"),
            FileRole::Generated
        );
        assert_eq!(
            classify_path("src/test/resources/astDiff/defects4j/example.json"),
            FileRole::Generated
        );
        assert_eq!(
            classify_path("src/test/resources/mappings/example.json"),
            FileRole::Generated
        );
        assert_eq!(
            classify_path("src/main/resources/web/monaco/min/vs/editor/editor.main.js"),
            FileRole::Generated
        );
    }

    #[test]
    fn inventory_respects_ignores_and_default_exclusions() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::write(repo.join(".gitignore"), "ignored-by-git.ts\n").unwrap();
        fs::write(repo.join(".ctxpackignore"), "ignored-by-ctxpack.ts\n").unwrap();
        fs::write(repo.join(".cursorignore"), "ignored-by-cursor.ts\n").unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::create_dir_all(repo.join(".gradle/7.4")).unwrap();
        fs::create_dir_all(
            repo.join(".fastembed_cache/models--jinaai--jina-embeddings-v2-base-code/blobs"),
        )
        .unwrap();
        fs::create_dir_all(repo.join("build 2/tmp")).unwrap();
        fs::create_dir_all(repo.join("src/test/resources/oracle/commits")).unwrap();
        fs::create_dir_all(repo.join("src/test/resources/astDiff/defects4j")).unwrap();
        fs::create_dir_all(repo.join("src/test/resources/mappings")).unwrap();
        fs::create_dir_all(repo.join("src/main/resources/web/monaco/min/vs/editor")).unwrap();
        fs::write(repo.join("src/lib.ts"), "export const x = 1;\n").unwrap();
        fs::write(repo.join("tests/lib.test.ts"), "test('x', () => {});\n").unwrap();
        fs::write(repo.join("README.md"), "# Repo\n").unwrap();
        fs::write(repo.join("schema.sql"), "create table users(id int);\n").unwrap();
        fs::write(repo.join("package.json"), "{}\n").unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();
        fs::write(repo.join("private.key"), "secret\n").unwrap();
        fs::write(repo.join("dist/app.min.js"), "minified\n").unwrap();
        fs::write(repo.join(".gradle/7.4/fileHashes.bin"), "cache\n").unwrap();
        fs::write(
            repo.join(
                ".fastembed_cache/models--jinaai--jina-embeddings-v2-base-code/blobs/model.onnx",
            ),
            "local model cache\n",
        )
        .unwrap();
        fs::write(repo.join("build 2/tmp/cache.bin"), "cache\n").unwrap();
        fs::write(
            repo.join("src/test/resources/oracle/commits/example.json"),
            "{}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/test/resources/astDiff/defects4j/example.json"),
            "{}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/test/resources/mappings/example.json"),
            "{}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/resources/web/monaco/min/vs/editor/editor.main.js"),
            "generated\n",
        )
        .unwrap();
        fs::write(repo.join("ignored-by-git.ts"), "ignored\n").unwrap();
        fs::write(repo.join("ignored-by-ctxpack.ts"), "ignored\n").unwrap();
        fs::write(repo.join("ignored-by-cursor.ts"), "ignored\n").unwrap();

        let inventory = build_inventory(repo, &InventoryOptions::default()).unwrap();
        let paths = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"src/lib.ts"));
        assert!(paths.contains(&"tests/lib.test.ts"));
        assert!(paths.contains(&"README.md"));
        assert!(paths.contains(&"schema.sql"));
        assert!(paths.contains(&"package.json"));
        assert!(!paths.contains(&".env"));
        assert!(!paths.contains(&"private.key"));
        assert!(!paths.contains(&"dist/app.min.js"));
        assert!(!paths.contains(&".gradle/7.4/fileHashes.bin"));
        assert!(!paths.contains(
            &".fastembed_cache/models--jinaai--jina-embeddings-v2-base-code/blobs/model.onnx"
        ));
        assert!(!paths.contains(&"build 2/tmp/cache.bin"));
        assert!(!paths.contains(&"src/test/resources/oracle/commits/example.json"));
        assert!(!paths.contains(&"src/test/resources/astDiff/defects4j/example.json"));
        assert!(!paths.contains(&"src/test/resources/mappings/example.json"));
        assert!(!paths.contains(&"src/main/resources/web/monaco/min/vs/editor/editor.main.js"));
        assert!(!paths.contains(&"ignored-by-git.ts"));
        assert!(!paths.contains(&"ignored-by-ctxpack.ts"));
        assert!(!paths.contains(&"ignored-by-cursor.ts"));

        let lib = inventory
            .files
            .iter()
            .find(|file| file.path == "src/lib.ts")
            .unwrap();
        assert_eq!(lib.language.as_deref(), Some("typescript"));
        assert_eq!(lib.role, FileRole::Source);
        assert_eq!(lib.hash.len(), 64);
        assert!(inventory.generated_count >= 1);
        assert!(inventory.sensitive_count >= 2);
    }

    #[test]
    fn inventory_can_include_sensitive_and_generated_when_requested() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();
        fs::write(repo.join("dist/app.min.js"), "minified\n").unwrap();

        let inventory = build_inventory(
            repo,
            &InventoryOptions {
                include_generated: true,
                include_sensitive: true,
            },
        )
        .unwrap();
        let paths = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&".env"));
        assert!(paths.contains(&"dist/app.min.js"));
    }

    #[cfg(unix)]
    #[test]
    fn inventory_skips_unreadable_files_instead_of_aborting() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.ts"), "export const x = 1;\n").unwrap();
        fs::write(repo.join("src/unreadable.ts"), "export const secret = 1;\n").unwrap();
        let unreadable = repo.join("src/unreadable.ts");
        let original_permissions = fs::metadata(&unreadable).unwrap().permissions();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();

        let inventory = build_inventory(repo, &InventoryOptions::default()).unwrap();
        fs::set_permissions(&unreadable, original_permissions).unwrap();
        let paths = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"src/lib.ts"));
        assert!(!paths.contains(&"src/unreadable.ts"));
        assert_eq!(inventory.ignored_count, 1);
    }

    #[test]
    fn write_inventory_persists_under_ctxpack_home() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join("src.py"), "print('hi')\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let report = write_inventory(&repo, &InventoryOptions::default()).unwrap();

        assert_eq!(report.file_count, 1);
        assert!(report.inventory_path.starts_with(&home));
        assert!(report.inventory_path.exists());
        let json = fs::read_to_string(report.inventory_path).unwrap();
        let inventory: RepoInventory = serde_json::from_str(&json).unwrap();
        assert_eq!(inventory.files[0].path, "src.py");

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_finds_identifier_content_and_path_matches() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return getSession(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "test('require session', () => {});\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        let results = lexical_search(&repo, "requireSession", &SearchOptions { limit: 5 }).unwrap();

        assert_eq!(results[0].path, "src/auth/session.ts");
        assert!(results[0]
            .reason
            .contains("content matched `requiresession`"));

        let results = lexical_search(&repo, "session test", &SearchOptions { limit: 5 }).unwrap();
        assert_eq!(results[0].path, "tests/auth/session.test.ts");

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_reuses_query_cache_until_inventory_changes() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "export const loginToken = true;\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let first =
            lexical_search_report(&repo, "loginToken", &SearchOptions { limit: 5 }).unwrap();
        let second =
            lexical_search_report(&repo, "loginToken", &SearchOptions { limit: 5 }).unwrap();
        fs::write(
            repo.join("src/other.ts"),
            "export const loginToken = 'fresh';\n",
        )
        .unwrap();
        let third =
            lexical_search_report(&repo, "loginToken", &SearchOptions { limit: 5 }).unwrap();

        assert_eq!(first.results[0].path, "src/session.ts");
        assert_eq!(
            second.cache_status.status,
            ctxpack_core::CacheStatusKind::Hit
        );
        assert!(second
            .cache_status
            .path
            .as_deref()
            .is_some_and(|path| { path.contains("lexical-search") && path.ends_with(".json") }));
        assert!(third
            .results
            .iter()
            .any(|result| result.path == "src/other.ts"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn symbol_search_reuses_extraction_cache_until_inventory_changes() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let first =
            symbol_search_report(&repo, "requireSession", &SymbolOptions { limit: 5 }).unwrap();
        let second =
            symbol_search_report(&repo, "requireSession", &SymbolOptions { limit: 5 }).unwrap();
        fs::write(
            repo.join("src/user.ts"),
            "export function requireUser() { return true; }\n",
        )
        .unwrap();
        let third =
            symbol_search_report(&repo, "requireUser", &SymbolOptions { limit: 5 }).unwrap();

        assert_eq!(first.results[0].symbol.path, "src/session.ts");
        assert_eq!(
            second.cache_status.status,
            ctxpack_core::CacheStatusKind::Hit
        );
        assert!(second
            .cache_status
            .path
            .as_deref()
            .is_some_and(|path| path.contains("symbols") && path.ends_with(".json")));
        assert_eq!(third.results[0].symbol.path, "src/user.ts");

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn dependency_edges_reuse_cache_until_inventory_changes() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/a.ts"), "import { b } from './b';\n").unwrap();
        fs::write(repo.join("src/b.ts"), "export const b = 1;\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let first = dependency_edges_report(&repo, &DependencyOptions { limit: 10 }).unwrap();
        let second = dependency_edges_report(&repo, &DependencyOptions { limit: 10 }).unwrap();
        fs::write(repo.join("src/c.ts"), "export const c = 1;\n").unwrap();
        fs::write(
            repo.join("src/a.ts"),
            "import { b } from './b';\nimport { c } from './c';\n",
        )
        .unwrap();
        let third = dependency_edges_report(&repo, &DependencyOptions { limit: 10 }).unwrap();

        assert_eq!(first.edges.len(), 1);
        assert_eq!(
            second.cache_status.status,
            ctxpack_core::CacheStatusKind::Hit
        );
        assert!(second
            .cache_status
            .path
            .as_deref()
            .is_some_and(|path| { path.contains("dependency-edges") && path.ends_with(".json") }));
        assert!(third
            .edges
            .iter()
            .any(|edge| edge.target_path == "src/c.ts"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_dampens_planning_archive_artifacts_without_excluding_them() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join(".planning/milestones/v1")).unwrap();
        fs::create_dir_all(repo.join(".planning/e2e/run")).unwrap();
        fs::write(
            repo.join("src/current.ts"),
            "export function proofGate() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join(".planning/milestones/v1/old-proof.md"),
            "proofGate proofGate proofGate proofGate proofGate proofGate\n",
        )
        .unwrap();
        fs::write(
            repo.join(".planning/e2e/run/proof.json"),
            "{\"proofGate\":\"proofGate proofGate proofGate proofGate\"}\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        let results = lexical_search(&repo, "proofGate", &SearchOptions { limit: 10 }).unwrap();
        let paths = results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths[0], "src/current.ts");
        assert!(paths.contains(&".planning/milestones/v1/old-proof.md"));
        assert!(paths.contains(&".planning/e2e/run/proof.json"));
        assert!(results
            .iter()
            .filter(|result| result.path.starts_with(".planning/"))
            .all(|result| result.reason.contains("archive context artifact dampened")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_builds_missing_inventory_and_skips_excluded_files() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(repo.join(".ctxpackignore"), "ignored.ts\n").unwrap();
        fs::write(
            repo.join("src/login.ts"),
            "export const loginRedirect = true;\n",
        )
        .unwrap();
        fs::write(repo.join(".env"), "loginRedirect=secret\n").unwrap();
        fs::write(repo.join("dist/generated.min.js"), "loginRedirect\n").unwrap();
        fs::write(repo.join("ignored.ts"), "loginRedirect\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = lexical_search(&repo, "loginRedirect", &SearchOptions { limit: 10 }).unwrap();
        let paths = results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["src/login.ts"]);
        assert!(inventory_path(&repo_id_for_path(&fs::canonicalize(&repo).unwrap())).exists());

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_refreshes_stale_inventory_for_created_files() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/old.ts"), "export const oldValue = true;\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(
            repo.join("src/new-session.ts"),
            "export const requireSession = true;\n",
        )
        .unwrap();

        let report =
            lexical_search_report(&repo, "requireSession", &SearchOptions { limit: 5 }).unwrap();
        let results = lexical_search(&repo, "requireSession", &SearchOptions { limit: 5 }).unwrap();

        assert_eq!(results[0].path, "src/new-session.ts");
        assert_eq!(report.results[0].path, "src/new-session.ts");
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
    fn source_read_diagnostics_cover_non_utf8_oversized_and_deleted_inputs() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/non_utf8.ts"), [0xff, 0xfe, b'\n']).unwrap();
        fs::write(repo.join("src/oversized.ts"), "hugeToken\n".repeat(140_000)).unwrap();
        fs::write(
            repo.join("src/deleted.ts"),
            "export const deletedToken = true;\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::remove_file(repo.join("src/deleted.ts")).unwrap();

        let deleted =
            lexical_search_report(&repo, "deletedToken", &SearchOptions { limit: 10 }).unwrap();
        let non_utf8 =
            lexical_search_report(&repo, "non_utf8", &SearchOptions { limit: 10 }).unwrap();
        let oversized =
            lexical_search_report(&repo, "hugeToken", &SearchOptions { limit: 10 }).unwrap();

        assert!(non_utf8
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source_non_utf8"));
        assert!(oversized
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source_oversized"));
        assert!(deleted
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_stale_file_deleted"));
        assert!(oversized.results.is_empty());
        assert!(non_utf8
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("hugeToken")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_prioritizes_specific_terms_over_common_language_mentions() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/gr/uom/java/xmi")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/refactoringminer/astDiff/utils")).unwrap();
        fs::write(
            repo.join("src/main/java/gr/uom/java/xmi/TypeScriptFileProcessor.java"),
            "final class TypeScriptFileProcessor { String name = \"TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript TypeScript\"; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/java/org/refactoringminer/astDiff/utils/Constants.java"),
            "final class Constants { String rule = \"support enum declarations for parser nodes\"; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = lexical_search(
            &repo,
            "Support enum declarations in TypeScript",
            &SearchOptions { limit: 5 },
        )
        .unwrap();

        assert_eq!(
            results[0].path,
            "src/main/java/org/refactoringminer/astDiff/utils/Constants.java"
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn lexical_search_ignores_common_task_verbs() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/noisy.ts"),
            "handle handle handle handle handle handle handle handle\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/scope.ts"),
            "function getScopeNode() { return iswc4jast; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = lexical_search(
            &repo,
            "Handle NPE in getScopeNode(ISwc4jAst node)",
            &SearchOptions { limit: 5 },
        )
        .unwrap();

        assert_eq!(results[0].path, "src/scope.ts");
        assert!(!results.iter().any(|result| result.path == "src/noisy.ts"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn extract_symbols_finds_language_aware_definitions() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "import { user } from './user';\nexport async function requireSession(req: Request) { return user; }\nclass SessionStore {}\nconst AUTH_COOKIE_NAME = 'sid';\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/service.py"),
            "class AuthService:\n    def require_session(self):\n        return True\n\ndef load_user():\n    return None\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let symbols = extract_symbols(&repo).unwrap();
        let names = symbols
            .iter()
            .map(|symbol| (symbol.name.as_str(), &symbol.kind, symbol.path.as_str()))
            .collect::<Vec<_>>();

        assert!(names.contains(&("requireSession", &SymbolKind::Function, "src/session.ts")));
        assert!(names.contains(&("SessionStore", &SymbolKind::Class, "src/session.ts")));
        assert!(names.contains(&("AUTH_COOKIE_NAME", &SymbolKind::Constant, "src/session.ts")));
        assert!(names.contains(&("AuthService", &SymbolKind::Class, "src/service.py")));
        assert!(names.contains(&("require_session", &SymbolKind::Method, "src/service.py")));
        assert!(names.contains(&("load_user", &SymbolKind::Function, "src/service.py")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn symbol_search_prioritizes_symbol_name_matches() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "export function requireSession() { return true; }\nfunction helper() { return requireSession(); }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = symbol_search(&repo, "requireSession", &SymbolOptions { limit: 5 }).unwrap();

        assert_eq!(results[0].symbol.name, "requireSession");
        assert_eq!(results[0].symbol.path, "src/session.ts");
        assert!(results[0].reason.contains("symbol name exactly matched"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn extract_symbols_finds_java_and_kotlin_definitions() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/example")).unwrap();
        fs::create_dir_all(repo.join("src/main/kotlin/org/example")).unwrap();
        fs::write(
            repo.join("src/main/java/org/example/AuthService.java"),
            "package org.example;\npublic class AuthService {\n  public static final String COOKIE = \"sid\";\n  public Session requireSession(Request request) { return null; }\n}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/kotlin/org/example/AuthController.kt"),
            "package org.example\nclass AuthController {\n  fun redirectToLogin(): String = \"/login\"\n}\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let symbols = extract_symbols(&repo).unwrap();
        let names = symbols
            .iter()
            .map(|symbol| (symbol.name.as_str(), &symbol.kind, symbol.path.as_str()))
            .collect::<Vec<_>>();

        assert!(names.contains(&(
            "AuthService",
            &SymbolKind::Class,
            "src/main/java/org/example/AuthService.java"
        )));
        assert!(names.contains(&(
            "COOKIE",
            &SymbolKind::Constant,
            "src/main/java/org/example/AuthService.java"
        )));
        assert!(names.contains(&(
            "requireSession",
            &SymbolKind::Method,
            "src/main/java/org/example/AuthService.java"
        )));
        assert!(names.contains(&(
            "AuthController",
            &SymbolKind::Class,
            "src/main/kotlin/org/example/AuthController.kt"
        )));
        assert!(names.contains(&(
            "redirectToLogin",
            &SymbolKind::Function,
            "src/main/kotlin/org/example/AuthController.kt"
        )));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn symbol_extraction_refreshes_stale_inventory_for_created_files() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/old.ts"), "export function oldValue() {}\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();

        let report = extract_symbols_report(&repo).unwrap();
        let symbols = extract_symbols(&repo).unwrap();
        let names = symbols
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"requireSession"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_stale"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn dependency_edges_resolve_safe_local_imports() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("src/db")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nimport { findUser } from '../db/user';\nimport express from 'express';\nexport function requireSession() { return parseCookie() && findUser(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/db/user.ts"),
            "export function findUser() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("dist/generated.js"),
            "export const generated = true;\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let edges = dependency_edges(&repo, &DependencyOptions { limit: 10 }).unwrap();
        let pairs = edges
            .iter()
            .map(|edge| (edge.source_path.as_str(), edge.target_path.as_str()))
            .collect::<Vec<_>>();

        assert!(pairs.contains(&("src/auth/session.ts", "src/auth/cookies.ts")));
        assert!(pairs.contains(&("src/auth/session.ts", "src/db/user.ts")));
        assert!(edges.iter().all(|edge| edge.kind == "imports"));
        assert!(edges
            .iter()
            .all(|edge| !edge.target_path.starts_with("dist/")));
        assert!(edges
            .iter()
            .all(|edge| !edge.target_path.contains("express")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn dependency_edges_resolve_python_from_imported_submodules() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("schema_agent/nlp")).unwrap();
        fs::create_dir_all(repo.join("schema_agent/text2r/normalizers")).unwrap();
        fs::write(
            repo.join("schema_agent/agents.py"),
            "from schema_agent.nlp import entity_extractor as ee\nfrom schema_agent.text2r import sql_materializer, triplet_extractor_symbolic\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/reexport_agent.py"),
            "from schema_agent.nlp import EntityExtractor\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/nlp/__init__.py"),
            "from schema_agent.nlp.entity_extractor import EntityExtractor\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/nlp/entity_extractor.py"),
            "class EntityExtractor: pass\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/text2r/sql_materializer.py"),
            "class SqlMaterializer: pass\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/text2r/triplet_extractor_symbolic.py"),
            "class TripletExtractor: pass\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/text2r/normalizers/__init__.py"),
            "from . import currency_normalizer\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/text2r/normalizers/currency_normalizer.py"),
            "class CurrencyNormalizer: pass\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let edges = dependency_edges(&repo, &DependencyOptions { limit: 20 }).unwrap();
        let pairs = edges
            .iter()
            .map(|edge| (edge.source_path.as_str(), edge.target_path.as_str()))
            .collect::<Vec<_>>();

        assert!(pairs.contains(&(
            "schema_agent/agents.py",
            "schema_agent/nlp/entity_extractor.py"
        )));
        assert!(pairs.contains(&(
            "schema_agent/reexport_agent.py",
            "schema_agent/nlp/__init__.py"
        )));
        assert!(pairs.contains(&(
            "schema_agent/agents.py",
            "schema_agent/text2r/sql_materializer.py"
        )));
        assert!(pairs.contains(&(
            "schema_agent/agents.py",
            "schema_agent/text2r/triplet_extractor_symbolic.py"
        )));
        assert!(pairs.contains(&(
            "schema_agent/text2r/normalizers/__init__.py",
            "schema_agent/text2r/normalizers/currency_normalizer.py"
        )));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_dependency_edges_expand_python_package_reexports() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("schema_agent/nlp")).unwrap();
        fs::write(
            repo.join("schema_agent/agent.py"),
            "from schema_agent.nlp import EntityExtractor\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/nlp/__init__.py"),
            "from schema_agent.nlp.entity_extractor import EntityExtractor\n",
        )
        .unwrap();
        fs::write(
            repo.join("schema_agent/nlp/entity_extractor.py"),
            "class EntityExtractor: pass\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let edges = related_dependency_edges(
            &repo,
            &["schema_agent/agent.py".to_string()],
            &DependencyOptions { limit: 10 },
        )
        .unwrap();
        let pairs = edges
            .iter()
            .map(|edge| {
                (
                    edge.source_path.as_str(),
                    edge.target_path.as_str(),
                    edge.kind.as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert!(pairs.contains(&(
            "schema_agent/agent.py",
            "schema_agent/nlp/__init__.py",
            "imports"
        )));
        assert!(pairs.contains(&(
            "schema_agent/agent.py",
            "schema_agent/nlp/entity_extractor.py",
            "python_reexport"
        )));
        assert_eq!(
            edges
                .iter()
                .find(|edge| edge.kind == "python_reexport")
                .map(|edge| edge.target_path.as_str()),
            Some("schema_agent/nlp/entity_extractor.py")
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn dependency_edges_resolve_java_and_kotlin_package_imports() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/example/auth")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/example/user")).unwrap();
        fs::create_dir_all(repo.join("src/main/kotlin/org/example/web")).unwrap();
        fs::write(
            repo.join("src/main/java/org/example/auth/AuthService.java"),
            "package org.example.auth;\nimport org.example.user.UserRepository;\npublic class AuthService { private UserRepository users; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/java/org/example/user/UserRepository.java"),
            "package org.example.user;\npublic class UserRepository {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/kotlin/org/example/web/AuthController.kt"),
            "package org.example.web\nimport org.example.auth.AuthService\nclass AuthController(private val auth: AuthService)\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let edges = dependency_edges(&repo, &DependencyOptions { limit: 10 }).unwrap();
        let pairs = edges
            .iter()
            .map(|edge| (edge.source_path.as_str(), edge.target_path.as_str()))
            .collect::<Vec<_>>();

        assert!(pairs.contains(&(
            "src/main/java/org/example/auth/AuthService.java",
            "src/main/java/org/example/user/UserRepository.java"
        )));
        assert!(pairs.contains(&(
            "src/main/kotlin/org/example/web/AuthController.kt",
            "src/main/java/org/example/auth/AuthService.java"
        )));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn dependency_edges_refresh_stale_inventory_for_created_targets() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() { return parseCookie(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();

        let report = dependency_edges_report(&repo, &DependencyOptions { limit: 10 }).unwrap();
        let edges = dependency_edges(&repo, &DependencyOptions { limit: 10 }).unwrap();
        let pairs = edges
            .iter()
            .map(|edge| (edge.source_path.as_str(), edge.target_path.as_str()))
            .collect::<Vec<_>>();

        assert!(pairs.contains(&("src/auth/session.ts", "src/auth/cookies.ts")));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_stale"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn partial_graph_and_test_map_diagnostics_are_source_free() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(repo.join("src/auth/session.ts"), [0xff, 0xfe, b'\n']).unwrap();
        fs::write(repo.join("tests/auth/session.test.ts"), [0xff, 0xfe, b'\n']).unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let graph = dependency_edges_report(&repo, &DependencyOptions { limit: 10 }).unwrap();
        let tests = related_tests_report(&repo, &["src/auth/session.ts".to_string()]).unwrap();

        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "graph_partial"));
        assert!(tests
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "test_map_partial"));
        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source_non_utf8"));
        assert!(tests
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("requireSession")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_dependency_edges_return_incoming_and_outgoing_edges() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/app.ts"),
            "import { requireSession } from './auth/session';\nexport const app = requireSession();\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() { return parseCookie(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let edges = related_dependency_edges(
            &repo,
            &["src/auth/session.ts".to_string()],
            &DependencyOptions { limit: 10 },
        )
        .unwrap();
        let pairs = edges
            .iter()
            .map(|edge| (edge.source_path.as_str(), edge.target_path.as_str()))
            .collect::<Vec<_>>();

        assert!(pairs.contains(&("src/auth/session.ts", "src/auth/cookies.ts")));
        assert!(pairs.contains(&("src/app.ts", "src/auth/session.ts")));
        assert_eq!(pairs[0], ("src/app.ts", "src/auth/session.ts"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_dependency_edges_preserve_anchor_order_within_same_direction() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/app.ts"),
            "import { requireSession } from './auth/session';\nexport const app = requireSession();\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/admin.ts"),
            "import { requireAdmin } from './auth/admin';\nexport const admin = requireAdmin();\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/admin.ts"),
            "export function requireAdmin() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let edges = related_dependency_edges(
            &repo,
            &[
                "src/auth/admin.ts".to_string(),
                "src/auth/session.ts".to_string(),
            ],
            &DependencyOptions { limit: 2 },
        )
        .unwrap();
        let pairs = edges
            .iter()
            .map(|edge| (edge.source_path.as_str(), edge.target_path.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(pairs[0], ("src/admin.ts", "src/auth/admin.ts"));
        assert_eq!(pairs[1], ("src/app.ts", "src/auth/session.ts"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn precision_edge_import_is_source_free_and_additive() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("src/routes")).unwrap();
        fs::write(
            repo.join("src/routes/login.ts"),
            "export function loginRoute() { return '/login'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/middleware.ts"),
            "export function authMiddleware() { return true; }\n",
        )
        .unwrap();
        let input = temp.path().join("precision.json");
        fs::write(
            &input,
            r#"{
  "schemaVersion": 1,
  "provider": "scip-json-fixture",
  "edges": [
    {
      "sourcePath": "src/auth/middleware.ts",
      "targetPath": "src/routes/login.ts",
      "edgeType": "calls",
      "symbol": "loginRoute",
      "confidence": 0.98,
      "reason": "local SCIP fixture edge"
    },
    {
      "sourcePath": ".env",
      "targetPath": "src/routes/login.ts",
      "edgeType": "calls",
      "reason": "SHOULD_NOT_PERSIST_SECRET"
    }
  ]
}
"#,
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let report = import_precision_edges(&repo, &input).unwrap();
        let overlay_path = precision_edges_path(&repo).unwrap();
        let overlay = fs::read_to_string(overlay_path).unwrap();
        let edges = dependency_edges(&repo, &DependencyOptions { limit: 10 }).unwrap();

        assert_eq!(report.accepted_edges, 1);
        assert_eq!(report.rejected_edges, 1);
        assert!(!overlay.contains("SHOULD_NOT_PERSIST_SECRET"));
        assert!(edges.iter().any(|edge| {
            edge.kind == "precision:calls"
                && edge.source_path == "src/auth/middleware.ts"
                && edge.target_path == "src/routes/login.ts"
                && edge.reason == "local SCIP fixture edge"
        }));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn precision_discovery_generates_source_free_symbol_edges() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("src/routes")).unwrap();
        fs::write(
            repo.join("src/routes/login.ts"),
            "export function loginRoute() { return '/login'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/middleware.ts"),
            "import { loginRoute } from '../routes/login';\nexport function authMiddleware() { return loginRoute(); }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let report = discover_precision_edges(
            &repo,
            &PrecisionDiscoverOptions {
                limit: 20,
                include_private_symbols: false,
            },
        )
        .unwrap();
        let overlay = fs::read_to_string(precision_edges_path(&repo).unwrap()).unwrap();
        let edges = dependency_edges(&repo, &DependencyOptions { limit: 20 }).unwrap();

        assert_eq!(report.provider, "local_tree_sitter_reference_scan");
        assert!(report.discovered_edges > 0);
        assert!(!overlay.contains("return '/login'"));
        assert!(edges.iter().any(|edge| {
            edge.kind == "precision:calls"
                && edge.source_path == "src/auth/middleware.ts"
                && edge.target_path == "src/routes/login.ts"
                && edge.reason.contains("loginRoute")
        }));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_maps_source_to_conventional_test_file() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(&repo, &["src/auth/session.ts".to_string()]).unwrap();

        assert_eq!(results[0].path, "tests/auth/session.test.ts");
        assert_eq!(
            results[0].command.as_deref(),
            Some("pnpm test tests/auth/session.test.ts")
        );
        assert!(results[0].confidence > 0.5);
        assert!(results[0].reason.contains("source stem `session`"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_refreshes_stale_inventory_for_created_test_files() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() {}\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();

        let report = related_tests_report(&repo, &["src/auth/session.ts".to_string()]).unwrap();
        let results = related_tests(&repo, &["src/auth/session.ts".to_string()]).unwrap();

        assert_eq!(results[0].path, "tests/auth/session.test.ts");
        assert_eq!(report.results[0].path, "tests/auth/session.test.ts");
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_stale"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_uses_vitest_script_and_package_manager_lockfile() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("package.json"),
            r#"{"scripts":{"test":"vitest run"}}"#,
        )
        .unwrap();
        fs::write(repo.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "requireSession keeps auth tests connected\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(&repo, &["src/auth/session.ts".to_string()]).unwrap();

        assert_eq!(
            results[0].command.as_deref(),
            Some("pnpm vitest run tests/auth/session.test.ts")
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_uses_npm_test_script_with_separator() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::write(repo.join("package.json"), r#"{"scripts":{"test":"mocha"}}"#).unwrap();
        fs::write(repo.join("package-lock.json"), "{}\n").unwrap();
        fs::write(
            repo.join("src/payment.js"),
            "export function normalizePayment() {}\n",
        )
        .unwrap();
        fs::write(repo.join("tests/payment.test.js"), "normalizePayment\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(&repo, &["src/payment.js".to_string()]).unwrap();

        assert_eq!(
            results[0].command.as_deref(),
            Some("npm test -- tests/payment.test.js")
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_uses_rust_integration_test_command() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::write(repo.join("src/auth.rs"), "pub fn require_session() {}\n").unwrap();
        fs::write(repo.join("tests/auth.rs"), "require_session();\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(&repo, &["src/auth.rs".to_string()]).unwrap();

        assert_eq!(
            results[0].command.as_deref(),
            Some("cargo test --test auth")
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_uses_gradle_java_test_class_command() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/example/auth")).unwrap();
        fs::create_dir_all(repo.join("src/test/java/org/example/auth")).unwrap();
        fs::write(repo.join("build.gradle"), "plugins { id 'java' }\n").unwrap();
        fs::write(
            repo.join("src/main/java/org/example/auth/SessionService.java"),
            "package org.example.auth; class SessionService {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/test/java/org/example/auth/SessionServiceTest.java"),
            "package org.example.auth; class SessionServiceTest { SessionService service; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(
            &repo,
            &["src/main/java/org/example/auth/SessionService.java".to_string()],
        )
        .unwrap();

        assert_eq!(
            results[0].command.as_deref(),
            Some("./gradlew test --tests org.example.auth.SessionServiceTest")
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_deduplicates_shared_source_terms_across_many_sources() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/refactoringminer/mcp")).unwrap();
        fs::create_dir_all(repo.join("src/test/java/org/refactoringminer/mcp")).unwrap();
        fs::create_dir_all(repo.join("src/test/java/org/refactoringminer/test")).unwrap();
        fs::write(repo.join("build.gradle"), "plugins { id 'java' }\n").unwrap();
        fs::write(
            repo.join("src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java"),
            "package org.refactoringminer.mcp; class RefactoringMinerMcpTools {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpService.java"),
            "package org.refactoringminer.mcp; class RefactoringMinerMcpService {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java"),
            "package org.refactoringminer.mcp; class RefactoringMinerMcpToolsTest { RefactoringMinerMcpTools tools; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/test/java/org/refactoringminer/test/TestBuilder.java"),
            "package org.refactoringminer.test; class TestBuilder { String s = \"refactoringminer mcp refactoringminer mcp refactoringminer mcp\"; }\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(
            &repo,
            &[
                "src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java".to_string(),
                "src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpService.java"
                    .to_string(),
            ],
        )
        .unwrap();

        assert_eq!(
            results[0].path,
            "src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java"
        );
        let tools_test_position = results
            .iter()
            .position(|result| result.path.ends_with("RefactoringMinerMcpToolsTest.java"))
            .unwrap();
        let helper_position = results
            .iter()
            .position(|result| result.path.ends_with("TestBuilder.java"))
            .unwrap();
        assert!(tools_test_position < helper_position);

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_prefers_exact_tests_for_higher_ranked_source_seeds() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/org/refactoringminer/mcp")).unwrap();
        fs::create_dir_all(repo.join("src/test/java/org/refactoringminer/mcp")).unwrap();
        fs::write(repo.join("build.gradle"), "plugins { id 'java' }\n").unwrap();
        for source in [
            "RefactoringMinerMcpService",
            "RefactoringMinerMcpTools",
            "McpIntentValidator",
        ] {
            fs::write(
                repo.join(format!(
                    "src/main/java/org/refactoringminer/mcp/{source}.java"
                )),
                format!("package org.refactoringminer.mcp; class {source} {{}}\n"),
            )
            .unwrap();
        }
        for test in [
            "McpIntentValidatorTest",
            "RefactoringMinerMcpServiceRepositoryTest",
            "RefactoringMinerMcpToolsTest",
        ] {
            fs::write(
                repo.join(format!(
                    "src/test/java/org/refactoringminer/mcp/{test}.java"
                )),
                format!("package org.refactoringminer.mcp; class {test} {{}}\n"),
            )
            .unwrap();
        }
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(
            &repo,
            &[
                "src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpService.java"
                    .to_string(),
                "src/main/java/org/refactoringminer/mcp/RefactoringMinerMcpTools.java".to_string(),
                "src/main/java/org/refactoringminer/mcp/McpIntentValidator.java".to_string(),
            ],
        )
        .unwrap();
        let paths = results
            .iter()
            .take(3)
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            paths,
            vec![
                "src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpServiceRepositoryTest.java",
                "src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java",
                "src/test/java/org/refactoringminer/mcp/McpIntentValidatorTest.java",
            ]
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn related_tests_uses_content_mentions_and_excludes_non_inventory_files() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(repo.join(".ctxpackignore"), "tests/ignored.test.ts\n").unwrap();
        fs::write(
            repo.join("src/payment.ts"),
            "export function normalizePayment() {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/billing.test.ts"),
            "normalizePayment handles payment edge cases\n",
        )
        .unwrap();
        fs::write(repo.join("tests/ignored.test.ts"), "normalizePayment\n").unwrap();
        fs::write(repo.join("dist/payment.test.js"), "normalizePayment\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let results = related_tests(&repo, &["src/payment.ts".to_string()]).unwrap();
        let paths = results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["tests/billing.test.ts"]);

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn test_map_lists_safe_tests_with_package_aware_commands() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(
            repo.join("package.json"),
            r#"{"scripts":{"test":"vitest run"}}"#,
        )
        .unwrap();
        fs::write(repo.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        fs::write(
            repo.join("dist/generated.test.ts"),
            "test('generated', () => {});\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let tests = test_map(&repo).unwrap();

        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].path, "tests/auth/session.test.ts");
        assert_eq!(
            tests[0].command.as_deref(),
            Some("pnpm vitest run tests/auth/session.test.ts")
        );
        assert_eq!(tests[0].confidence, 1.0);
        assert_eq!(tests[0].reason, "safe test file from inventory");

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn parses_git_log_name_only_output() {
        let commits = parse_git_log_name_only(
            "commit:abc\nsrc/a.ts\ntests/a.test.ts\n\ncommit:def\nsrc/b.ts\n",
        );

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, "abc");
        assert_eq!(commits[0].files, vec!["src/a.ts", "tests/a.test.ts"]);
        assert_eq!(commits[1].sha, "def");
        assert_eq!(commits[1].files, vec!["src/b.ts"]);
    }

    #[test]
    fn parses_git_log_subject_name_only_output() {
        let commits = parse_git_log_subject_name_only(
            "commit:abc\u{0}fix auth redirect\nsrc/a.ts\ntests/a.test.ts\n\ncommit:def\u{0}add billing\nsrc/b.ts\n",
        );

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, "abc");
        assert_eq!(commits[0].title, "fix auth redirect");
        assert_eq!(
            commits[0]
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/a.ts", "tests/a.test.ts"]
        );
        assert_eq!(commits[1].sha, "def");
        assert_eq!(commits[1].title, "add billing");
        assert_eq!(
            commits[1]
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/b.ts"]
        );
    }

    #[test]
    fn historical_commit_samples_return_safe_labels_without_excluded_paths() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        fs::write(repo.join("dist/generated.min.js"), "generated\n").unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "fix requireSession bug"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let samples = historical_commit_samples(
            &repo,
            &HistoricalCommitOptions {
                limit: 10,
                base: None,
                head: None,
            },
        )
        .unwrap();

        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].title, "fix requireSession bug");
        assert_eq!(
            samples[0].safe_changed_files,
            vec!["src/auth/session.ts", "tests/auth/session.test.ts"]
        );
        assert_eq!(samples[0].excluded_changed_file_count, 2);
        assert!(!serde_json::to_string(&samples).unwrap().contains("TOKEN"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn historical_commit_samples_include_bounded_rename_paths() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export const session = 1;\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        fs::rename(
            repo.join("src/auth/session.ts"),
            repo.join("src/auth/session-store.ts"),
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "rename session store"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let samples = historical_commit_samples(
            &repo,
            &HistoricalCommitOptions {
                limit: 1,
                base: None,
                head: None,
            },
        )
        .unwrap();

        assert_eq!(samples.len(), 1);
        assert_eq!(
            samples[0].safe_changed_files,
            vec!["src/auth/session-store.ts"]
        );
        assert_eq!(samples[0].changed_paths.len(), 2);
        assert_eq!(
            samples[0].changed_paths[0].path,
            "src/auth/session-store.ts"
        );
        assert_eq!(samples[0].changed_paths[0].change_kind, ChangeKind::Added);
        assert_eq!(samples[0].changed_paths[0].label_scope, LabelScope::Safe);
        assert_eq!(samples[0].changed_paths[1].path, "src/auth/session.ts");
        assert_eq!(samples[0].changed_paths[1].change_kind, ChangeKind::Deleted);
        assert_eq!(
            samples[0].changed_paths[1].label_scope,
            LabelScope::HistoricalOnly
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn historical_commit_samples_include_delete_records() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export const session = 1;\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        fs::remove_file(repo.join("src/auth/session.ts")).unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "delete stale session"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let samples = historical_commit_samples(
            &repo,
            &HistoricalCommitOptions {
                limit: 1,
                base: None,
                head: None,
            },
        )
        .unwrap();

        assert_eq!(samples.len(), 1);
        assert!(samples[0].safe_changed_files.is_empty());
        assert_eq!(samples[0].changed_paths.len(), 1);
        assert_eq!(samples[0].changed_paths[0].change_kind, ChangeKind::Deleted);
        assert_eq!(samples[0].changed_paths[0].path, "src/auth/session.ts");
        assert_eq!(
            samples[0].changed_paths[0].label_scope,
            LabelScope::HistoricalOnly
        );

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn historical_commit_samples_label_generated_and_sensitive_exclusions() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(repo.join("dist/generated.min.js"), "generated\n").unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(
            &repo,
            &["commit", "-m", "add generated and sensitive files"],
        );
        std::env::set_var("CTXPACK_HOME", &home);

        let samples = historical_commit_samples(
            &repo,
            &HistoricalCommitOptions {
                limit: 1,
                base: None,
                head: None,
            },
        )
        .unwrap();

        assert_eq!(samples.len(), 1);
        assert!(samples[0].safe_changed_files.is_empty());
        assert_eq!(samples[0].excluded_changed_file_count, 2);
        assert!(samples[0].changed_paths.iter().any(|label| {
            label.path == "dist/generated.min.js"
                && label.label_scope == LabelScope::Generated
                && label.excluded_reason == Some(HistoricalPathExclusionReason::Generated)
        }));
        assert!(samples[0].changed_paths.iter().any(|label| {
            label.path == ".env"
                && label.label_scope == LabelScope::Sensitive
                && label.excluded_reason == Some(HistoricalPathExclusionReason::Sensitive)
        }));
        let serialized = serde_json::to_string(&samples).unwrap();
        assert!(!serialized.contains("TOKEN"));
        assert!(!serialized.contains("generated\\n"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn historical_commit_limit_bounds_revision_scan_before_filtering() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(repo.join("src/old.ts"), "export const old = true;\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "old safe change"]);
        fs::write(repo.join("dist/new.min.js"), "generated\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "new generated change"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let samples = historical_commit_samples(
            &repo,
            &HistoricalCommitOptions {
                limit: 1,
                base: None,
                head: None,
            },
        )
        .unwrap();

        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].title, "new generated change");
        assert!(samples[0].safe_changed_files.is_empty());
        assert!(samples[0]
            .changed_paths
            .iter()
            .all(|label| label.path != "src/old.ts"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn diagnostics_report_git_missing_as_history_partial() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u8 { 42 }\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        let old_path = std::env::var_os("PATH");
        std::env::set_var("PATH", "");

        let report = historical_commit_samples_report(
            &repo,
            &HistoricalCommitOptions {
                limit: 5,
                base: None,
                head: None,
            },
        )
        .unwrap();

        assert!(report.samples.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "git_missing"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "history_partial"));

        if let Some(path) = old_path {
            std::env::set_var("PATH", path);
        } else {
            std::env::remove_var("PATH");
        }
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn historical_commit_samples_can_use_stable_revision_range() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(repo.join("src/auth/first.ts"), "export const first = 1;\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "first auth change"]);
        let base = git_stdout_with_timeout(&repo, &["rev-parse", "HEAD"], Duration::from_secs(1))
            .unwrap()
            .trim()
            .to_string();
        fs::write(
            repo.join("src/auth/second.ts"),
            "export const second = 2;\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "second auth change"]);
        fs::write(repo.join("src/auth/third.ts"), "export const third = 3;\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "third auth change"]);
        let head = git_stdout_with_timeout(&repo, &["rev-parse", "HEAD"], Duration::from_secs(1))
            .unwrap()
            .trim()
            .to_string();
        std::env::set_var("CTXPACK_HOME", &home);

        let samples = historical_commit_samples(
            &repo,
            &HistoricalCommitOptions {
                limit: 10,
                base: Some(base),
                head: Some(head),
            },
        )
        .unwrap();

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].title, "third auth change");
        assert_eq!(samples[1].title, "second auth change");
        assert!(samples
            .iter()
            .all(|sample| sample.title != "first auth change"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn historical_commit_collection_uses_fast_no_rename_diff() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(repo.join("src/first.ts"), "export const first = 1;\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "first change"]);
        fs::write(repo.join("src/second.ts"), "export const second = 2;\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "second change"]);

        let commits = git_commit_subject_file_sets_with_timeouts(
            &repo,
            2,
            None,
            None,
            Duration::from_secs(1),
            Duration::ZERO,
        )
        .unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].title, "second change");
        assert!(commits[0]
            .files
            .iter()
            .any(|file| file.path == "src/second.ts"));
    }

    #[test]
    fn co_change_hints_use_local_git_history_and_safe_inventory() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(repo.join(".ctxpackignore"), "ignored.ts\n").unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export const session = 1;\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "test('session', () => {});\n",
        )
        .unwrap();
        fs::write(repo.join("dist/session.min.js"), "session\n").unwrap();
        fs::write(repo.join("ignored.ts"), "session\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "initial auth session"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export const session = 2;\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "test('session changed', () => {});\n",
        )
        .unwrap();
        fs::write(repo.join("dist/session.min.js"), "session changed\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "change auth session"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let hints = co_change_hints(
            &repo,
            &["src/auth/session.ts".to_string()],
            &CoChangeOptions { limit: 10 },
        )
        .unwrap();
        let paths = hints
            .iter()
            .map(|hint| hint.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"tests/auth/session.test.ts"));
        assert!(!paths.contains(&"dist/session.min.js"));
        assert!(!paths.contains(&"ignored.ts"));
        let test_hint = hints
            .iter()
            .find(|hint| hint.path == "tests/auth/session.test.ts")
            .unwrap();
        assert_eq!(test_hint.commit_count, 2);
        assert_eq!(test_hint.sample_commits.len(), 2);

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn co_change_hints_use_eval_history_sidecar_when_snapshot_has_no_git() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let source_repo = temp.path().join("source-repo");
        let snapshot_repo = temp.path().join("snapshot-repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(source_repo.join("src/auth")).unwrap();
        fs::create_dir_all(source_repo.join("tests/auth")).unwrap();
        run_git(&source_repo, &["init"]);
        run_git(
            &source_repo,
            &["config", "user.email", "ctxpack@example.com"],
        );
        run_git(&source_repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            source_repo.join("src/auth/session.ts"),
            "export const session = 1;\n",
        )
        .unwrap();
        fs::write(
            source_repo.join("tests/auth/session.test.ts"),
            "test('session', () => {});\n",
        )
        .unwrap();
        run_git(&source_repo, &["add", "."]);
        run_git(&source_repo, &["commit", "-m", "initial auth session"]);
        fs::write(
            source_repo.join("src/auth/session.ts"),
            "export const session = 2;\n",
        )
        .unwrap();
        fs::write(
            source_repo.join("tests/auth/session.test.ts"),
            "test('session changed', () => {});\n",
        )
        .unwrap();
        run_git(&source_repo, &["add", "."]);
        run_git(&source_repo, &["commit", "-m", "change auth session"]);
        let head =
            git_stdout_with_timeout(&source_repo, &["rev-parse", "HEAD"], Duration::from_secs(1))
                .unwrap();
        let head = head.trim();

        fs::create_dir_all(snapshot_repo.join("src/auth")).unwrap();
        fs::create_dir_all(snapshot_repo.join("tests/auth")).unwrap();
        fs::write(
            snapshot_repo.join("src/auth/session.ts"),
            "export const session = 2;\n",
        )
        .unwrap();
        fs::write(
            snapshot_repo.join("tests/auth/session.test.ts"),
            "test('session changed', () => {});\n",
        )
        .unwrap();
        write_eval_history_sidecar(&source_repo, head, &snapshot_repo).unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let report = co_change_hints_report(
            &snapshot_repo,
            &["src/auth/session.ts".to_string()],
            &CoChangeOptions { limit: 10 },
        )
        .unwrap();

        assert!(report
            .hints
            .iter()
            .any(|hint| hint.path == "tests/auth/session.test.ts" && hint.commit_count == 2));
        assert!(!report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "history_partial"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn co_change_hints_treat_empty_git_history_as_no_hints() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export const session = 1;\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let hints = co_change_hints(
            &repo,
            &["src/auth/session.ts".to_string()],
            &CoChangeOptions { limit: 10 },
        )
        .unwrap();

        assert!(hints.is_empty());
        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn current_diff_summary_returns_safe_changed_paths_without_source_text() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(repo.join("src/lib.ts"), "export const value = 1;\n").unwrap();
        fs::write(repo.join("README.md"), "# Repo\n").unwrap();
        fs::write(repo.join("dist/app.min.js"), "generated\n").unwrap();
        fs::write(repo.join("private.key"), "secret\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "initial"]);
        fs::write(repo.join("src/lib.ts"), "export const value = 2;\n").unwrap();
        fs::write(repo.join("README.md"), "# Repo changed\n").unwrap();
        run_git(&repo, &["add", "README.md"]);
        fs::write(repo.join("tests/new.test.ts"), "test('new', () => {});\n").unwrap();
        fs::write(repo.join("dist/new.min.js"), "generated\n").unwrap();
        fs::write(repo.join("local.key"), "secret\n").unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let summary = current_diff_summary(
            &repo,
            &CurrentDiffOptions {
                include_untracked: true,
            },
        )
        .unwrap();

        assert_eq!(summary.unstaged, vec!["src/lib.ts"]);
        assert_eq!(summary.staged, vec!["README.md"]);
        assert_eq!(summary.untracked, vec!["tests/new.test.ts"]);
        assert_eq!(summary.excluded.untracked, 2);
        assert!(summary
            .excluded
            .reason
            .contains("source content was not returned"));
        assert!(summary.privacy_status.local_only);
        assert!(!summary.privacy_status.source_text_returned);

        let without_untracked =
            current_diff_summary(&repo, &CurrentDiffOptions::default()).unwrap();
        assert!(without_untracked.untracked.is_empty());

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn eval_traces_append_and_list_without_source_text() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        let repo_id = repo_id_for_path(&fs::canonicalize(&repo).unwrap());

        let first = EvalTrace {
            id: Uuid::nil(),
            repo_id: repo_id.clone(),
            task_hash: task_hash("fix auth"),
            task_type: ctxpack_core::TaskType::BugFix,
            pack_id: None,
            target_agent: "codex".to_string(),
            budget: None,
            recommended_files: vec!["src/auth.ts".to_string()],
            recommended_tests: vec!["tests/auth.test.ts".to_string()],
            recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };
        let second = EvalTrace {
            id: Uuid::nil(),
            created_at_unix_seconds: 2,
            ..first.clone()
        };

        let path = append_eval_trace(&repo, &first).unwrap();
        append_eval_trace(&repo, &second).unwrap();

        assert_eq!(path, trace_path(&repo_id));
        let stored = fs::read_to_string(path).unwrap();
        assert!(!stored.contains("fix auth"));
        assert!(stored.contains("\"sourceTextLogged\":false"));

        let traces = list_eval_traces(&repo, 1).unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].created_at_unix_seconds, 2);
        assert_eq!(traces[0].recommended_files, vec!["src/auth.ts"]);

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn feedback_events_append_list_and_summarize_without_source_text() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        std::env::set_var("CTXPACK_HOME", &home);
        let repo_id = repo_id_for_path(&fs::canonicalize(&repo).unwrap());

        let first = SessionFeedbackEvent {
            id: Uuid::nil(),
            schema_version: FEEDBACK_EVENT_SCHEMA_VERSION,
            repo_id: repo_id.clone(),
            task_hash: task_hash("fix auth"),
            task_type: ctxpack_core::TaskType::BugFix,
            pack_id: Some(Uuid::nil()),
            target_agent: "codex".to_string(),
            budget: Some(PackBudget::Brief),
            outcome: FeedbackOutcome::Passed,
            recommended_files: vec!["src/auth.ts".to_string()],
            recommended_tests: vec!["tests/auth.test.ts".to_string()],
            recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            read_files: vec!["src/auth.ts".to_string()],
            edited_files: vec!["src/auth.ts".to_string()],
            tested_files: vec!["tests/auth.test.ts".to_string()],
            tested_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            user_corrected_files: Vec::new(),
            tags: vec!["accepted_fix".to_string()],
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };
        let second = SessionFeedbackEvent {
            id: Uuid::nil(),
            outcome: FeedbackOutcome::Failed,
            created_at_unix_seconds: 2,
            ..first.clone()
        };

        let path = append_feedback_event(&repo, &first).unwrap();
        append_feedback_event(&repo, &second).unwrap();

        assert_eq!(path, feedback_path(&repo_id));
        let stored = fs::read_to_string(path).unwrap();
        assert!(!stored.contains("fix auth"));
        assert!(!stored.contains("pub fn"));
        assert!(stored.contains("\"sourceTextLogged\":false"));

        let events = list_feedback_events(&repo, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].outcome, FeedbackOutcome::Failed);
        let summary = summarize_feedback_events(&repo_id, &events);
        assert_eq!(summary.event_count, 2);
        assert_eq!(summary.passed_count, 1);
        assert_eq!(summary.failed_count, 1);
        assert_eq!(summary.read_file_count, 1);
        assert!(!summary.source_text_logged);

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn feedback_events_reject_source_text_and_unsafe_paths() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        let repo_id = repo_id_for_path(&fs::canonicalize(&repo).unwrap());
        let mut event = SessionFeedbackEvent {
            id: Uuid::nil(),
            schema_version: FEEDBACK_EVENT_SCHEMA_VERSION,
            repo_id,
            task_hash: "hash".to_string(),
            task_type: ctxpack_core::TaskType::BugFix,
            pack_id: None,
            target_agent: "codex".to_string(),
            budget: None,
            outcome: FeedbackOutcome::Unknown,
            recommended_files: vec!["../secret.rs".to_string()],
            recommended_tests: Vec::new(),
            recommended_commands: Vec::new(),
            read_files: Vec::new(),
            edited_files: Vec::new(),
            tested_files: Vec::new(),
            tested_commands: Vec::new(),
            user_corrected_files: Vec::new(),
            tags: Vec::new(),
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };

        assert!(append_feedback_event(&repo, &event).is_err());
        event.recommended_files = vec!["src/auth.ts".to_string()];
        event.source_text_logged = true;
        assert!(append_feedback_event(&repo, &event).is_err());
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
