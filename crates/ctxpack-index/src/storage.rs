use crate::inventory::{canonicalize, ctxpack_home, repo_id_for_path};
use ctxpack_core::{Diagnostic, DiagnosticSeverity};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const RANKING_STORAGE_VERSION: u32 = 1;
pub const COMPILER_STORAGE_VERSION: u32 = 1;

const INITIAL_MIGRATION_NAME: &str = "initial_source_free_storage_schema";
const INITIAL_MIGRATION_CHECKSUM: &str = "ctxpack-storage-v1-source-free";

const REQUIRED_TABLES: &[&str] = &[
    "storage_metadata",
    "schema_migrations",
    "repos",
    "files",
    "symbols",
    "chunks",
    "edges",
    "tests",
    "git_history",
    "eval_traces",
    "context_plans",
    "context_packs",
    "benchmark_runs",
    "benchmark_metrics",
    "retrieval_gaps",
    "proof_reports",
];

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to canonicalize {path}: {source}")]
    Canonicalize {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("sqlite error at {path}: {source}")]
    Sqlite {
        path: PathBuf,
        source: rusqlite::Error,
    },
    #[error("storage database does not exist: {path}")]
    MissingDatabase { path: PathBuf },
    #[error("storage metadata missing in {path}")]
    MissingMetadata { path: PathBuf },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoreConfig {
    pub path_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorePaths {
    pub repo_id: String,
    pub database_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoragePrivacyMode {
    SourceFree,
}

impl StoragePrivacyMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::SourceFree => "source_free",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "source_free" => Self::SourceFree,
            _ => Self::SourceFree,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageMetadata {
    pub schema_version: u32,
    pub ctxpack_version: String,
    pub ranking_version: u32,
    pub compiler_version: u32,
    pub privacy_mode: StoragePrivacyMode,
    pub created_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StorageCompatibility {
    Compatible,
    MissingMetadata,
    MissingTables,
    IncompatibleSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageReport {
    pub repo_id: String,
    pub repo_root: PathBuf,
    pub database_path: PathBuf,
    pub schema_version: u32,
    pub ctxpack_version: String,
    pub ranking_version: u32,
    pub compiler_version: u32,
    pub privacy_mode: StoragePrivacyMode,
    pub compatibility: StorageCompatibility,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageSchemaReport {
    pub database_path: PathBuf,
    pub schema_version: Option<u32>,
    pub privacy_mode: Option<StoragePrivacyMode>,
    pub present_tables: Vec<String>,
    pub missing_tables: Vec<String>,
    pub compatibility: StorageCompatibility,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

pub fn required_table_names() -> &'static [&'static str] {
    REQUIRED_TABLES
}

pub fn initialize_store(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
) -> Result<StorageReport, StorageError> {
    let repo_root = canonicalize_storage(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let paths = store_paths_for_repo_id(&repo_id, config);
    if let Some(parent) = paths.database_path.parent() {
        fs::create_dir_all(parent).map_err(|source| StorageError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let connection = open_connection(&paths.database_path)?;
    enable_foreign_keys(&connection, &paths.database_path)?;
    create_schema(&connection, &paths.database_path)?;
    upsert_repo(&connection, &paths.database_path, &repo_id, &repo_root)?;
    let metadata = upsert_metadata(&connection, &paths.database_path, &repo_id)?;
    insert_initial_migration(&connection, &paths.database_path)?;
    let schema_report = inspect_connection_schema(&connection, &paths.database_path)?;

    Ok(StorageReport {
        repo_id,
        repo_root,
        database_path: paths.database_path,
        schema_version: metadata.schema_version,
        ctxpack_version: metadata.ctxpack_version,
        ranking_version: metadata.ranking_version,
        compiler_version: metadata.compiler_version,
        privacy_mode: metadata.privacy_mode,
        compatibility: schema_report.compatibility,
        diagnostics: schema_report.diagnostics,
    })
}

pub fn inspect_store_schema(
    database_path: impl AsRef<Path>,
) -> Result<StorageSchemaReport, StorageError> {
    let database_path = database_path.as_ref().to_path_buf();
    if !database_path.exists() {
        return Ok(StorageSchemaReport {
            database_path: database_path.clone(),
            schema_version: None,
            privacy_mode: None,
            present_tables: Vec::new(),
            missing_tables: REQUIRED_TABLES
                .iter()
                .map(|table| (*table).to_string())
                .collect(),
            compatibility: StorageCompatibility::MissingTables,
            diagnostics: vec![storage_diagnostic(
                "storage_missing",
                format!(
                    "Storage database does not exist: {}",
                    database_path.display()
                ),
                &database_path,
            )],
        });
    }
    let connection = open_connection(&database_path)?;
    inspect_connection_schema(&connection, &database_path)
}

pub fn open_store_report(database_path: impl AsRef<Path>) -> Result<StorageReport, StorageError> {
    let database_path = database_path.as_ref().to_path_buf();
    if !database_path.exists() {
        return Err(StorageError::MissingDatabase {
            path: database_path,
        });
    }
    let connection = open_connection(&database_path)?;
    let metadata = read_metadata(&connection, &database_path)?.ok_or_else(|| {
        StorageError::MissingMetadata {
            path: database_path.clone(),
        }
    })?;
    let repo = read_repo_identity(&connection, &database_path)?;
    let schema_report = inspect_connection_schema(&connection, &database_path)?;

    Ok(StorageReport {
        repo_id: repo.0,
        repo_root: repo.1,
        database_path,
        schema_version: metadata.schema_version,
        ctxpack_version: metadata.ctxpack_version,
        ranking_version: metadata.ranking_version,
        compiler_version: metadata.compiler_version,
        privacy_mode: metadata.privacy_mode,
        compatibility: schema_report.compatibility,
        diagnostics: schema_report.diagnostics,
    })
}

fn store_paths_for_repo_id(repo_id: &str, config: &StoreConfig) -> StorePaths {
    let database_path = config.path_override.clone().unwrap_or_else(|| {
        ctxpack_home()
            .join("repos")
            .join(repo_id)
            .join("ctxpack.sqlite3")
    });
    StorePaths {
        repo_id: repo_id.to_string(),
        database_path,
    }
}

fn canonicalize_storage(path: &Path) -> Result<PathBuf, StorageError> {
    canonicalize(path).map_err(|error| match error {
        crate::InventoryError::Canonicalize { path, source } => {
            StorageError::Canonicalize { path, source }
        }
        _ => StorageError::Canonicalize {
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
        },
    })
}

fn open_connection(path: &Path) -> Result<Connection, StorageError> {
    Connection::open(path).map_err(|source| StorageError::Sqlite {
        path: path.to_path_buf(),
        source,
    })
}

fn sqlite<T>(path: &Path, result: rusqlite::Result<T>) -> Result<T, StorageError> {
    result.map_err(|source| StorageError::Sqlite {
        path: path.to_path_buf(),
        source,
    })
}

fn enable_foreign_keys(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    sqlite(path, connection.pragma_update(None, "foreign_keys", "ON"))
}

fn create_schema(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    sqlite(
        path,
        connection.execute_batch(
            r#"
CREATE TABLE IF NOT EXISTS storage_metadata (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  repo_id TEXT NOT NULL,
  repo_root TEXT NOT NULL,
  schema_version INTEGER NOT NULL,
  ctxpack_version TEXT NOT NULL,
  ranking_version INTEGER NOT NULL,
  compiler_version INTEGER NOT NULL,
  privacy_mode TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL,
  updated_at_unix_seconds INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at_unix_seconds INTEGER NOT NULL,
  checksum TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS repos (
  repo_id TEXT PRIMARY KEY,
  repo_root TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL,
  updated_at_unix_seconds INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS files (
  file_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  language TEXT,
  role TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  generated INTEGER NOT NULL,
  ignored INTEGER NOT NULL,
  updated_at_unix_seconds INTEGER NOT NULL,
  UNIQUE(repo_id, path)
);

CREATE TABLE IF NOT EXISTS symbols (
  symbol_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  file_id TEXT REFERENCES files(file_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  signature TEXT,
  start_line INTEGER,
  end_line INTEGER,
  exported INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS chunks (
  chunk_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  file_id TEXT REFERENCES files(file_id) ON DELETE CASCADE,
  symbol_id TEXT REFERENCES symbols(symbol_id) ON DELETE SET NULL,
  kind TEXT NOT NULL,
  start_line INTEGER,
  end_line INTEGER,
  token_estimate INTEGER,
  content_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS edges (
  edge_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  src_kind TEXT NOT NULL,
  src_id TEXT NOT NULL,
  dst_kind TEXT NOT NULL,
  dst_id TEXT NOT NULL,
  edge_type TEXT NOT NULL,
  weight REAL NOT NULL DEFAULT 1.0,
  evidence_json TEXT
);

CREATE TABLE IF NOT EXISTS tests (
  test_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  framework TEXT,
  command TEXT,
  confidence REAL NOT NULL DEFAULT 0.0
);

CREATE TABLE IF NOT EXISTS git_history (
  history_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  commit_sha TEXT NOT NULL,
  changed_path TEXT NOT NULL,
  change_kind TEXT NOT NULL,
  timestamp_unix_seconds INTEGER
);

CREATE TABLE IF NOT EXISTS eval_traces (
  trace_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  task_hash TEXT NOT NULL,
  task_type TEXT NOT NULL,
  pack_id TEXT,
  target_agent TEXT NOT NULL,
  budget TEXT,
  created_at_unix_seconds INTEGER NOT NULL,
  body_logged INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS context_plans (
  plan_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  task_hash TEXT NOT NULL,
  task_type TEXT NOT NULL,
  confidence REAL NOT NULL,
  selected_candidate_ids_json TEXT,
  warnings_json TEXT,
  privacy_status TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS context_packs (
  pack_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  task_hash TEXT NOT NULL,
  budget TEXT NOT NULL,
  target_agent TEXT NOT NULL,
  confidence REAL NOT NULL,
  selected_candidate_ids_json TEXT,
  warnings_json TEXT,
  privacy_status TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS benchmark_runs (
  run_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  suite_id TEXT NOT NULL,
  revision_id TEXT,
  budget TEXT,
  privacy_status TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS benchmark_metrics (
  metric_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL REFERENCES benchmark_runs(run_id) ON DELETE CASCADE,
  metric_name TEXT NOT NULL,
  metric_value REAL NOT NULL,
  budget TEXT,
  target_kind TEXT
);

CREATE TABLE IF NOT EXISTS retrieval_gaps (
  gap_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL REFERENCES benchmark_runs(run_id) ON DELETE CASCADE,
  family TEXT NOT NULL,
  recommendation_area TEXT,
  target_status TEXT,
  safe_path TEXT,
  count INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS proof_reports (
  proof_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  run_id TEXT REFERENCES benchmark_runs(run_id) ON DELETE SET NULL,
  headline_metrics_json TEXT,
  limitations_json TEXT,
  privacy_status TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL
);
"#,
        ),
    )
}

fn upsert_repo(
    connection: &Connection,
    path: &Path,
    repo_id: &str,
    repo_root: &Path,
) -> Result<(), StorageError> {
    let now = current_unix_seconds();
    sqlite(
        path,
        connection.execute(
            r#"
INSERT INTO repos (repo_id, repo_root, created_at_unix_seconds, updated_at_unix_seconds)
VALUES (?1, ?2, ?3, ?3)
ON CONFLICT(repo_id) DO UPDATE SET
  repo_root = excluded.repo_root,
  updated_at_unix_seconds = excluded.updated_at_unix_seconds
"#,
            params![repo_id, repo_root.display().to_string(), as_i64(now)],
        ),
    )?;
    Ok(())
}

fn upsert_metadata(
    connection: &Connection,
    path: &Path,
    repo_id: &str,
) -> Result<StorageMetadata, StorageError> {
    let now = current_unix_seconds();
    let existing_created = sqlite(
        path,
        connection
            .query_row(
                "SELECT created_at_unix_seconds FROM storage_metadata WHERE id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional(),
    )?
    .map(as_u64)
    .unwrap_or(now);
    sqlite(
        path,
        connection.execute(
            r#"
INSERT INTO storage_metadata (
  id, repo_id, repo_root, schema_version, ctxpack_version, ranking_version,
  compiler_version, privacy_mode, created_at_unix_seconds, updated_at_unix_seconds
)
SELECT 1, repo_id, repo_root, ?2, ?3, ?4, ?5, ?6, ?7, ?8
FROM repos WHERE repo_id = ?1
ON CONFLICT(id) DO UPDATE SET
  repo_id = excluded.repo_id,
  repo_root = excluded.repo_root,
  schema_version = excluded.schema_version,
  ctxpack_version = excluded.ctxpack_version,
  ranking_version = excluded.ranking_version,
  compiler_version = excluded.compiler_version,
  privacy_mode = excluded.privacy_mode,
  updated_at_unix_seconds = excluded.updated_at_unix_seconds
"#,
            params![
                repo_id,
                STORAGE_SCHEMA_VERSION,
                env!("CARGO_PKG_VERSION"),
                RANKING_STORAGE_VERSION,
                COMPILER_STORAGE_VERSION,
                StoragePrivacyMode::SourceFree.as_str(),
                as_i64(existing_created),
                as_i64(now)
            ],
        ),
    )?;
    read_metadata(connection, path)?.ok_or_else(|| StorageError::MissingMetadata {
        path: path.to_path_buf(),
    })
}

fn insert_initial_migration(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    sqlite(
        path,
        connection.execute(
            r#"
INSERT INTO schema_migrations (version, name, applied_at_unix_seconds, checksum)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(version) DO NOTHING
"#,
            params![
                STORAGE_SCHEMA_VERSION,
                INITIAL_MIGRATION_NAME,
                as_i64(current_unix_seconds()),
                INITIAL_MIGRATION_CHECKSUM
            ],
        ),
    )?;
    Ok(())
}

fn read_metadata(
    connection: &Connection,
    path: &Path,
) -> Result<Option<StorageMetadata>, StorageError> {
    sqlite(
        path,
        connection
            .query_row(
                r#"
SELECT schema_version, ctxpack_version, ranking_version, compiler_version,
       privacy_mode, created_at_unix_seconds, updated_at_unix_seconds
FROM storage_metadata WHERE id = 1
"#,
                [],
                |row| {
                    Ok(StorageMetadata {
                        schema_version: row.get::<_, u32>(0)?,
                        ctxpack_version: row.get(1)?,
                        ranking_version: row.get::<_, u32>(2)?,
                        compiler_version: row.get::<_, u32>(3)?,
                        privacy_mode: StoragePrivacyMode::from_str(&row.get::<_, String>(4)?),
                        created_at_unix_seconds: as_u64(row.get::<_, i64>(5)?),
                        updated_at_unix_seconds: as_u64(row.get::<_, i64>(6)?),
                    })
                },
            )
            .optional(),
    )
}

fn read_repo_identity(
    connection: &Connection,
    path: &Path,
) -> Result<(String, PathBuf), StorageError> {
    sqlite(
        path,
        connection.query_row(
            "SELECT repo_id, repo_root FROM repos ORDER BY updated_at_unix_seconds DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    PathBuf::from(row.get::<_, String>(1)?),
                ))
            },
        ),
    )
}

fn inspect_connection_schema(
    connection: &Connection,
    path: &Path,
) -> Result<StorageSchemaReport, StorageError> {
    let present_tables = table_names(connection, path)?;
    let missing_tables = REQUIRED_TABLES
        .iter()
        .filter(|required| !present_tables.iter().any(|present| present == *required))
        .map(|table| (*table).to_string())
        .collect::<Vec<_>>();
    let metadata = if missing_tables
        .iter()
        .any(|table| table == "storage_metadata")
    {
        None
    } else {
        read_metadata(connection, path)?
    };
    let mut diagnostics = Vec::new();
    let compatibility = if metadata.is_none() {
        diagnostics.push(storage_diagnostic(
            "storage_missing_metadata",
            format!("Storage metadata missing in {}", path.display()),
            path,
        ));
        StorageCompatibility::MissingMetadata
    } else if !missing_tables.is_empty() {
        diagnostics.push(storage_diagnostic(
            "storage_missing_tables",
            format!(
                "Storage database {} is missing {} required table(s)",
                path.display(),
                missing_tables.len()
            ),
            path,
        ));
        StorageCompatibility::MissingTables
    } else if metadata
        .as_ref()
        .is_some_and(|metadata| metadata.schema_version != STORAGE_SCHEMA_VERSION)
    {
        diagnostics.push(storage_diagnostic(
            "storage_incompatible_schema",
            format!(
                "Storage database {} uses schema version {}, expected {}",
                path.display(),
                metadata
                    .as_ref()
                    .map_or(0, |metadata| metadata.schema_version),
                STORAGE_SCHEMA_VERSION
            ),
            path,
        ));
        StorageCompatibility::IncompatibleSchema
    } else {
        StorageCompatibility::Compatible
    };

    Ok(StorageSchemaReport {
        database_path: path.to_path_buf(),
        schema_version: metadata.as_ref().map(|metadata| metadata.schema_version),
        privacy_mode: metadata.as_ref().map(|metadata| metadata.privacy_mode),
        present_tables,
        missing_tables,
        compatibility,
        diagnostics,
    })
}

fn table_names(connection: &Connection, path: &Path) -> Result<Vec<String>, StorageError> {
    let mut statement = sqlite(
        path,
        connection.prepare(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        ),
    )?;
    let rows = sqlite(path, statement.query_map([], |row| row.get::<_, String>(0)))?;
    let mut tables =
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|source| StorageError::Sqlite {
                path: path.to_path_buf(),
                source,
            })?;
    tables.sort();
    Ok(tables)
}

fn storage_diagnostic(code: &str, message: String, path: &Path) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message,
        paths: vec![path.display().to_string()],
        count: 1,
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn as_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn as_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        append_eval_trace, load_or_build_inventory, trace_path, write_inventory, InventoryOptions,
    };
    use ctxpack_core::{EvalTrace, PackBudget, TaskType};
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use uuid::Uuid;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn fixture_repo() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn demo() {}\n").unwrap();
        (temp, repo)
    }

    #[test]
    fn initialize_store_uses_ctxpack_home_default_path() {
        let _guard = env_lock();
        let (temp, repo) = fixture_repo();
        let home = temp.path().join("home");
        std::env::set_var("CTXPACK_HOME", &home);
        std::env::remove_var("HOME");

        let report = initialize_store(&repo, &StoreConfig::default()).unwrap();

        assert!(report.database_path.starts_with(&home));
        assert!(report.database_path.ends_with(Path::new("ctxpack.sqlite3")));
        assert_eq!(report.schema_version, STORAGE_SCHEMA_VERSION);
        assert_eq!(report.privacy_mode, StoragePrivacyMode::SourceFree);

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn initialize_store_accepts_explicit_override_path() {
        let _guard = env_lock();
        let (temp, repo) = fixture_repo();
        let override_path = temp.path().join("custom/store.sqlite3");

        let report = initialize_store(
            &repo,
            &StoreConfig {
                path_override: Some(override_path.clone()),
            },
        )
        .unwrap();

        assert_eq!(report.database_path, override_path);
        assert!(report.database_path.exists());
    }

    #[test]
    fn initialize_store_is_idempotent_and_enables_foreign_keys() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        let config = StoreConfig {
            path_override: Some(database_path.clone()),
        };

        let first = initialize_store(&repo, &config).unwrap();
        let second = initialize_store(&repo, &config).unwrap();
        let connection = Connection::open(&database_path).unwrap();
        let foreign_keys: u32 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();

        assert_eq!(first.repo_id, second.repo_id);
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn initialize_store_creates_all_required_tables() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        initialize_store(
            &repo,
            &StoreConfig {
                path_override: Some(database_path.clone()),
            },
        )
        .unwrap();

        let report = inspect_store_schema(&database_path).unwrap();

        assert_eq!(report.compatibility, StorageCompatibility::Compatible);
        assert!(report.missing_tables.is_empty());
        for table in required_table_names() {
            assert!(
                report.present_tables.contains(&table.to_string()),
                "{table}"
            );
        }
    }

    #[test]
    fn schema_tables_do_not_use_source_bearing_columns() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        initialize_store(
            &repo,
            &StoreConfig {
                path_override: Some(database_path.clone()),
            },
        )
        .unwrap();
        let connection = Connection::open(&database_path).unwrap();
        let prohibited = [
            "source", "content", "snippet", "prompt", "secret", "raw", "subject",
        ];

        for table in required_table_names() {
            let mut statement = connection
                .prepare(&format!("PRAGMA table_info({table})"))
                .unwrap();
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            for column in columns {
                if column == "content_hash" {
                    continue;
                }
                for fragment in prohibited {
                    assert!(
                        !column.contains(fragment),
                        "column {table}.{column} contains prohibited fragment {fragment}"
                    );
                }
            }
        }
    }

    #[test]
    fn migration_history_is_idempotent() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        let config = StoreConfig {
            path_override: Some(database_path.clone()),
        };
        initialize_store(&repo, &config).unwrap();
        initialize_store(&repo, &config).unwrap();
        let connection = Connection::open(&database_path).unwrap();
        let count: u32 = connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let name: String = connection
            .query_row(
                "SELECT name FROM schema_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(name, INITIAL_MIGRATION_NAME);
    }

    #[test]
    fn metadata_created_timestamp_survives_reinitialization() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        let config = StoreConfig {
            path_override: Some(database_path.clone()),
        };
        initialize_store(&repo, &config).unwrap();
        let first = open_store_report(&database_path).unwrap();
        initialize_store(&repo, &config).unwrap();
        let second = open_store_report(&database_path).unwrap();

        let connection = Connection::open(&database_path).unwrap();
        let created_count: u32 = connection
            .query_row("SELECT COUNT(*) FROM storage_metadata", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(created_count, 1);
        assert_eq!(first.repo_id, second.repo_id);
        assert_eq!(first.schema_version, second.schema_version);
    }

    #[test]
    fn inspect_store_reports_missing_and_incompatible_schema() {
        let temp = tempfile::tempdir().unwrap();
        let missing_path = temp.path().join("missing.sqlite3");
        let missing = inspect_store_schema(&missing_path).unwrap();
        assert_eq!(missing.compatibility, StorageCompatibility::MissingTables);
        assert_eq!(missing.diagnostics[0].code, "storage_missing");

        let database_path = temp.path().join("incompatible.sqlite3");
        let connection = Connection::open(&database_path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE storage_metadata (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    repo_id TEXT NOT NULL,
                    repo_root TEXT NOT NULL,
                    schema_version INTEGER NOT NULL,
                    ctxpack_version TEXT NOT NULL,
                    ranking_version INTEGER NOT NULL,
                    compiler_version INTEGER NOT NULL,
                    privacy_mode TEXT NOT NULL,
                    created_at_unix_seconds INTEGER NOT NULL,
                    updated_at_unix_seconds INTEGER NOT NULL
                );",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO storage_metadata VALUES (1, 'repo', '/tmp/repo', 999, 'x', 1, 1, 'source_free', 1, 1)",
                [],
            )
            .unwrap();
        let incompatible = inspect_store_schema(&database_path).unwrap();
        assert_eq!(
            incompatible.compatibility,
            StorageCompatibility::MissingTables
        );
        assert!(incompatible
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "storage_missing_tables"));
    }

    #[test]
    fn inspect_store_reports_incompatible_schema_version() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        initialize_store(
            &repo,
            &StoreConfig {
                path_override: Some(database_path.clone()),
            },
        )
        .unwrap();
        let connection = Connection::open(&database_path).unwrap();
        connection
            .execute("UPDATE storage_metadata SET schema_version = 999", [])
            .unwrap();

        let report = inspect_store_schema(&database_path).unwrap();

        assert_eq!(
            report.compatibility,
            StorageCompatibility::IncompatibleSchema
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "storage_incompatible_schema"));
    }

    #[test]
    fn sqlite_database_does_not_contain_source_or_prompt_sentinels() {
        let (_guard, _temp, repo) = {
            let guard = env_lock();
            let temp = tempfile::tempdir().unwrap();
            let repo = temp.path().join("repo");
            fs::create_dir_all(repo.join(".git")).unwrap();
            fs::create_dir_all(repo.join("src")).unwrap();
            fs::write(
                repo.join("src/lib.rs"),
                "pub fn leak() { /* CTXPACK_SHOULD_NOT_STORE_SOURCE_BODY */ }\n",
            )
            .unwrap();
            std::env::set_var("CTXPACK_HOME", temp.path().join("home"));
            (guard, temp, repo)
        };
        let report = initialize_store(&repo, &StoreConfig::default()).unwrap();
        let bytes = fs::read(&report.database_path).unwrap();
        let database_text = String::from_utf8_lossy(&bytes);

        assert!(!database_text.contains("CTXPACK_SHOULD_NOT_STORE_SOURCE_BODY"));
        assert!(!database_text.contains("CTXPACK_SHOULD_NOT_STORE_PROMPT_TEXT"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn json_inventory_and_trace_fallback_remain_intact() {
        let _guard = env_lock();
        let (temp, repo) = fixture_repo();
        let home = temp.path().join("home");
        std::env::set_var("CTXPACK_HOME", &home);

        let store = initialize_store(&repo, &StoreConfig::default()).unwrap();
        let inventory_report = write_inventory(&repo, &InventoryOptions::default()).unwrap();
        let inventory = load_or_build_inventory(&repo, &InventoryOptions::default()).unwrap();
        let trace = EvalTrace {
            id: Uuid::nil(),
            repo_id: inventory.repo_id.clone(),
            task_hash: "task_hash".to_string(),
            task_type: TaskType::Feature,
            pack_id: None,
            target_agent: "generic".to_string(),
            budget: Some(PackBudget::Brief),
            recommended_files: vec!["src/lib.rs".to_string()],
            recommended_tests: Vec::new(),
            recommended_commands: Vec::new(),
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };
        append_eval_trace(&repo, &trace).unwrap();
        let traces = crate::list_eval_traces(&repo, 10).unwrap();
        let trace_path = trace_path(&inventory.repo_id);

        assert!(store.database_path.ends_with("ctxpack.sqlite3"));
        assert!(inventory_report.inventory_path.ends_with("inventory.json"));
        assert!(trace_path.ends_with("traces.jsonl"));
        assert!(inventory_report.inventory_path.exists());
        assert!(trace_path.exists());
        assert_eq!(traces.len(), 1);

        std::env::remove_var("CTXPACK_HOME");
    }
}
