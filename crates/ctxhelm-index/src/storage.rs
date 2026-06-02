use crate::inventory::{build_inventory, InventoryOptions};
use crate::inventory::{canonicalize, ctxhelm_home, repo_id_for_path};
use ctxhelm_core::{
    Diagnostic, DiagnosticSeverity, FileRole, MemoryCard, MemoryCardKind, MemoryFreshness,
    MemoryReviewStatus, PrivacyStatus,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const STORAGE_SCHEMA_VERSION: u32 = 3;
pub const RANKING_STORAGE_VERSION: u32 = 1;
pub const COMPILER_STORAGE_VERSION: u32 = 1;

const INITIAL_MIGRATION_NAME: &str = "initial_source_free_storage_schema";
const INITIAL_MIGRATION_CHECKSUM: &str = "ctxhelm-storage-v1-source-free";
const SEMANTIC_MIGRATION_NAME: &str = "local_semantic_vector_metadata";
const SEMANTIC_MIGRATION_CHECKSUM: &str = "ctxhelm-storage-v2-local-semantic";
const MEMORY_MIGRATION_NAME: &str = "repo_memory_cards";
const MEMORY_MIGRATION_CHECKSUM: &str = "ctxhelm-storage-v3-source-free-memory";

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
    "semantic_vectors",
    "memory_cards",
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
    pub ctxhelm_version: String,
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
    pub ctxhelm_version: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageIndexReport {
    pub repo_id: String,
    pub repo_root: PathBuf,
    pub database_path: PathBuf,
    pub schema_version: u32,
    pub reused_records: usize,
    pub created_records: usize,
    pub updated_records: usize,
    pub deleted_records: usize,
    pub skipped_files: usize,
    pub ignored_paths: usize,
    pub generated_paths: usize,
    pub sensitive_paths: usize,
    pub compatibility: StorageCompatibility,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatusReport {
    pub repo_id: Option<String>,
    pub repo_root: Option<PathBuf>,
    pub database_path: PathBuf,
    pub schema_version: Option<u32>,
    pub file_records: usize,
    pub symbol_records: usize,
    pub context_pack_records: usize,
    pub benchmark_run_records: usize,
    pub proof_report_records: usize,
    pub semantic_vector_records: usize,
    pub memory_card_records: usize,
    pub compatibility: StorageCompatibility,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageSemanticIndexReport {
    pub repo_id: String,
    pub repo_root: PathBuf,
    pub database_path: PathBuf,
    pub schema_version: u32,
    pub reused_records: usize,
    pub created_records: usize,
    pub updated_records: usize,
    pub deleted_records: usize,
    pub semantic_vector_records: usize,
    pub compatibility: StorageCompatibility,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageMetricRecord {
    pub name: String,
    pub value: f32,
    pub budget: Option<String>,
    pub target_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageGapRecord {
    pub family: String,
    pub recommendation_area: Option<String>,
    pub target_status: Option<String>,
    pub safe_path: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageBenchmarkRunRecord {
    pub run_id: String,
    pub suite_id: String,
    pub revision_id: Option<String>,
    pub budget: Option<String>,
    pub privacy_status: String,
    pub metrics: Vec<StorageMetricRecord>,
    pub gaps: Vec<StorageGapRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageContextPackRecord {
    pub pack_id: String,
    pub task_hash: String,
    pub budget: String,
    pub target_agent: String,
    pub confidence: f32,
    pub selected_candidate_ids: Vec<String>,
    pub warnings: Vec<String>,
    pub privacy_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageProofReportRecord {
    pub proof_id: String,
    pub run_id: Option<String>,
    pub headline_metrics_json: String,
    pub limitations_json: String,
    pub privacy_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageSemanticVectorRecord {
    pub path: String,
    pub safe_hash: String,
    pub provider: String,
    pub model: String,
    pub dimensions: usize,
    pub distance_metric: String,
    pub vector: Vec<f32>,
    pub privacy_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageMemoryCardRecord {
    pub card: MemoryCard,
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
    insert_semantic_migration(&connection, &paths.database_path)?;
    insert_memory_migration(&connection, &paths.database_path)?;
    let schema_report = inspect_connection_schema(&connection, &paths.database_path)?;

    Ok(StorageReport {
        repo_id,
        repo_root,
        database_path: paths.database_path,
        schema_version: metadata.schema_version,
        ctxhelm_version: metadata.ctxhelm_version,
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
        ctxhelm_version: metadata.ctxhelm_version,
        ranking_version: metadata.ranking_version,
        compiler_version: metadata.compiler_version,
        privacy_mode: metadata.privacy_mode,
        compatibility: schema_report.compatibility,
        diagnostics: schema_report.diagnostics,
    })
}

pub fn sync_inventory_to_store(
    repo_root: impl AsRef<Path>,
    inventory_options: &InventoryOptions,
    config: &StoreConfig,
) -> Result<StorageIndexReport, StorageError> {
    let repo_root = canonicalize_storage(repo_root.as_ref())?;
    let storage = initialize_store(&repo_root, config)?;
    let inventory = build_inventory(&repo_root, inventory_options).map_err(|error| {
        StorageError::Canonicalize {
            path: repo_root.clone(),
            source: std::io::Error::other(error.to_string()),
        }
    })?;
    let connection = open_connection(&storage.database_path)?;
    enable_foreign_keys(&connection, &storage.database_path)?;

    let existing = existing_file_records(&connection, &storage.database_path, &storage.repo_id)?;
    let mut seen_paths = BTreeSet::new();
    let mut reused_records = 0;
    let mut created_records = 0;
    let mut updated_records = 0;

    for file in &inventory.files {
        seen_paths.insert(file.path.clone());
        let file_id = file_id_for_path(&storage.repo_id, &file.path);
        let role = role_name(&file.role);
        let stored = existing.get(&file.path);
        let unchanged = stored.is_some_and(|stored| {
            stored.file_id == file_id
                && stored.language == file.language
                && stored.role == role
                && stored.content_hash == file.hash
                && stored.size_bytes == file.size_bytes
                && stored.generated == file.generated
                && stored.ignored == file.ignored
        });
        if unchanged {
            reused_records += 1;
            continue;
        }

        sqlite(
            &storage.database_path,
            connection.execute(
                r#"
INSERT INTO files (
  file_id, repo_id, path, language, role, content_hash, size_bytes,
  generated, ignored, updated_at_unix_seconds
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
ON CONFLICT(repo_id, path) DO UPDATE SET
  file_id = excluded.file_id,
  language = excluded.language,
  role = excluded.role,
  content_hash = excluded.content_hash,
  size_bytes = excluded.size_bytes,
  generated = excluded.generated,
  ignored = excluded.ignored,
  updated_at_unix_seconds = excluded.updated_at_unix_seconds
"#,
                params![
                    file_id,
                    storage.repo_id,
                    file.path,
                    file.language,
                    role,
                    file.hash,
                    as_i64(file.size_bytes),
                    bool_as_i64(file.generated),
                    bool_as_i64(file.ignored),
                    as_i64(current_unix_seconds())
                ],
            ),
        )?;
        if stored.is_some() {
            updated_records += 1;
        } else {
            created_records += 1;
        }
    }

    let mut deleted_records = 0;
    for path in existing.keys().filter(|path| !seen_paths.contains(*path)) {
        deleted_records += sqlite(
            &storage.database_path,
            connection.execute(
                "DELETE FROM files WHERE repo_id = ?1 AND path = ?2",
                params![storage.repo_id, path],
            ),
        )?;
    }

    Ok(StorageIndexReport {
        repo_id: storage.repo_id,
        repo_root,
        database_path: storage.database_path,
        schema_version: storage.schema_version,
        reused_records,
        created_records,
        updated_records,
        deleted_records,
        skipped_files: inventory.ignored_count,
        ignored_paths: inventory.ignored_count,
        generated_paths: inventory.generated_count,
        sensitive_paths: inventory.sensitive_count,
        compatibility: storage.compatibility,
        diagnostics: storage.diagnostics,
    })
}

pub fn storage_status_for_repo(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
) -> Result<StorageStatusReport, StorageError> {
    let repo_root = canonicalize_storage(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let paths = store_paths_for_repo_id(&repo_id, config);
    storage_status_for_path(&paths.database_path)
}

pub fn storage_status_for_path(
    database_path: impl AsRef<Path>,
) -> Result<StorageStatusReport, StorageError> {
    let database_path = database_path.as_ref().to_path_buf();
    let schema = inspect_store_schema(&database_path)?;
    if !database_path.exists() || schema.compatibility != StorageCompatibility::Compatible {
        return Ok(StorageStatusReport {
            repo_id: None,
            repo_root: None,
            database_path,
            schema_version: schema.schema_version,
            file_records: 0,
            symbol_records: 0,
            context_pack_records: 0,
            benchmark_run_records: 0,
            proof_report_records: 0,
            semantic_vector_records: 0,
            memory_card_records: 0,
            compatibility: schema.compatibility,
            diagnostics: schema.diagnostics,
        });
    }

    let connection = open_connection(&database_path)?;
    let repo = read_repo_identity(&connection, &database_path).ok();
    Ok(StorageStatusReport {
        repo_id: repo.as_ref().map(|repo| repo.0.clone()),
        repo_root: repo.map(|repo| repo.1),
        database_path: database_path.clone(),
        schema_version: schema.schema_version,
        file_records: count_rows(&connection, &database_path, "files")?,
        symbol_records: count_rows(&connection, &database_path, "symbols")?,
        context_pack_records: count_rows(&connection, &database_path, "context_packs")?,
        benchmark_run_records: count_rows(&connection, &database_path, "benchmark_runs")?,
        proof_report_records: count_rows(&connection, &database_path, "proof_reports")?,
        semantic_vector_records: count_rows(&connection, &database_path, "semantic_vectors")?,
        memory_card_records: count_rows(&connection, &database_path, "memory_cards")?,
        compatibility: schema.compatibility,
        diagnostics: schema.diagnostics,
    })
}

pub fn persist_semantic_vector_records(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    records: &[StorageSemanticVectorRecord],
) -> Result<StorageSemanticIndexReport, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    enable_foreign_keys(&connection, &storage.database_path)?;
    let existing =
        existing_semantic_records(&connection, &storage.database_path, &storage.repo_id)?;
    let existing_by_unique_key = existing
        .iter()
        .map(|(vector_id, record)| (record.unique_key(), vector_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut retained_vector_ids = BTreeSet::new();
    let mut reused_records = 0;
    let mut created_records = 0;
    let mut updated_records = 0;
    for record in records {
        let file_id = file_id_for_path(&storage.repo_id, &record.path);
        let vector_id = semantic_vector_id(&storage.repo_id, record);
        retained_vector_ids.insert(vector_id.clone());
        let vector_json = json_string(&record.vector)?;
        let existing_record = existing_by_unique_key
            .get(&semantic_record_unique_key(
                &record.path,
                &record.provider,
                &record.model,
            ))
            .and_then(|vector_id| existing.get(vector_id));
        if existing_record.is_some_and(|existing| existing.matches(record, &vector_json)) {
            reused_records += 1;
            continue;
        }
        if existing_record.is_some() {
            updated_records += 1;
        } else {
            created_records += 1;
        }
        sqlite(
            &storage.database_path,
            connection.execute(
                r#"
INSERT INTO semantic_vectors (
  vector_id, repo_id, file_id, path, safe_hash, provider, model, dimensions,
  distance_metric, vector_json, privacy_status, updated_at_unix_seconds
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
ON CONFLICT(repo_id, path, provider, model) DO UPDATE SET
  vector_id = excluded.vector_id,
  file_id = excluded.file_id,
  safe_hash = excluded.safe_hash,
  provider = excluded.provider,
  model = excluded.model,
  dimensions = excluded.dimensions,
  distance_metric = excluded.distance_metric,
  vector_json = excluded.vector_json,
  privacy_status = excluded.privacy_status,
  updated_at_unix_seconds = excluded.updated_at_unix_seconds
"#,
                params![
                    vector_id,
                    storage.repo_id,
                    file_id,
                    record.path,
                    record.safe_hash,
                    record.provider,
                    record.model,
                    as_i64(record.dimensions as u64),
                    record.distance_metric,
                    vector_json,
                    record.privacy_status,
                    as_i64(current_unix_seconds()),
                ],
            ),
        )?;
    }
    let mut deleted_records = 0;
    for vector_id in existing
        .keys()
        .filter(|id| !retained_vector_ids.contains(*id))
    {
        deleted_records += sqlite(
            &storage.database_path,
            connection.execute(
                "DELETE FROM semantic_vectors WHERE repo_id = ?1 AND vector_id = ?2",
                params![storage.repo_id, vector_id],
            ),
        )?;
    }
    let status = storage_status_for_path(&storage.database_path)?;
    Ok(StorageSemanticIndexReport {
        repo_id: storage.repo_id,
        repo_root: storage.repo_root,
        database_path: storage.database_path,
        schema_version: storage.schema_version,
        reused_records,
        created_records,
        updated_records,
        deleted_records,
        semantic_vector_records: status.semantic_vector_records,
        compatibility: status.compatibility,
        diagnostics: status.diagnostics,
    })
}

pub fn load_semantic_vector_records(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    provider: &str,
    model: &str,
    dimensions: usize,
    distance_metric: &str,
) -> Result<Vec<StorageSemanticVectorRecord>, StorageError> {
    let repo_root =
        fs::canonicalize(repo_root.as_ref()).map_err(|source| StorageError::Canonicalize {
            path: repo_root.as_ref().to_path_buf(),
            source,
        })?;
    let repo_id = repo_id_for_path(&repo_root);
    let paths = store_paths_for_repo_id(&repo_id, config);
    if !paths.database_path.exists() {
        return Ok(Vec::new());
    }
    let connection = open_connection(&paths.database_path)?;
    let mut statement = sqlite(
        &paths.database_path,
        connection.prepare(
            r#"
SELECT path, safe_hash, provider, model, dimensions, distance_metric, vector_json, privacy_status
FROM semantic_vectors
WHERE repo_id = ?1
  AND provider = ?2
  AND model = ?3
  AND dimensions = ?4
  AND distance_metric = ?5
ORDER BY path
"#,
        ),
    )?;
    let rows = sqlite(
        &paths.database_path,
        statement.query_map(
            params![
                repo_id,
                provider,
                model,
                as_i64(dimensions as u64),
                distance_metric
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    as_u64(row.get::<_, i64>(4)?) as usize,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        ),
    )?;
    let mut records = Vec::new();
    for row in rows {
        let (
            path,
            safe_hash,
            provider,
            model,
            dimensions,
            distance_metric,
            vector_json,
            privacy_status,
        ) = sqlite(&paths.database_path, row)?;
        let vector = serde_json::from_str::<Vec<f32>>(&vector_json).map_err(|source| {
            StorageError::Sqlite {
                path: paths.database_path.clone(),
                source: rusqlite::Error::ToSqlConversionFailure(Box::new(source)),
            }
        })?;
        records.push(StorageSemanticVectorRecord {
            path,
            safe_hash,
            provider,
            model,
            dimensions,
            distance_metric,
            vector,
            privacy_status,
        });
    }
    Ok(records)
}

pub fn persist_memory_card_records(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    records: &[StorageMemoryCardRecord],
) -> Result<StorageStatusReport, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    enable_foreign_keys(&connection, &storage.database_path)?;
    for record in records {
        let card = &record.card;
        sqlite(
            &storage.database_path,
            connection.execute(
                r#"
INSERT INTO memory_cards (
  card_id, repo_id, kind, title, summary_text, link_paths_json, input_hashes_json,
  freshness, review_status, disabled, confidence, reason, privacy_status,
  created_at_unix_seconds, updated_at_unix_seconds
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)
ON CONFLICT(card_id) DO UPDATE SET
  kind = excluded.kind,
  title = excluded.title,
  summary_text = excluded.summary_text,
  link_paths_json = excluded.link_paths_json,
  input_hashes_json = excluded.input_hashes_json,
  freshness = excluded.freshness,
  review_status = excluded.review_status,
  disabled = excluded.disabled,
  confidence = excluded.confidence,
  reason = excluded.reason,
  privacy_status = excluded.privacy_status,
  updated_at_unix_seconds = excluded.updated_at_unix_seconds
"#,
                params![
                    card.id,
                    storage.repo_id,
                    memory_kind_label(&card.kind),
                    card.title,
                    card.summary,
                    json_string(&card.source_links)?,
                    json_string(&card.input_hashes)?,
                    memory_freshness_label(&card.freshness),
                    memory_review_label(&card.review_status),
                    bool_as_i64(card.disabled),
                    card.confidence,
                    card.reason,
                    json_string(&card.privacy_status)?,
                    as_i64(current_unix_seconds()),
                ],
            ),
        )?;
    }
    storage_status_for_path(&storage.database_path)
}

pub fn list_memory_cards(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    include_disabled: bool,
) -> Result<Vec<MemoryCard>, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    let where_clause = if include_disabled {
        "repo_id = ?1"
    } else {
        "repo_id = ?1 AND disabled = 0"
    };
    let mut statement = sqlite(
        &storage.database_path,
        connection.prepare(&format!(
            r#"
SELECT card_id, kind, title, summary_text, link_paths_json, input_hashes_json,
       freshness, review_status, disabled, confidence, reason, privacy_status
FROM memory_cards WHERE {where_clause}
ORDER BY kind, title, card_id
"#
        )),
    )?;
    let rows = sqlite(
        &storage.database_path,
        statement.query_map(params![storage.repo_id], |row| {
            let privacy_json: String = row.get(11)?;
            let privacy_status = serde_json::from_str::<PrivacyStatus>(&privacy_json)
                .unwrap_or_else(|_| PrivacyStatus::local_only());
            Ok(MemoryCard {
                id: row.get(0)?,
                kind: memory_kind_from_label(&row.get::<_, String>(1)?),
                title: row.get(2)?,
                summary: row.get(3)?,
                source_links: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                input_hashes: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                freshness: memory_freshness_from_label(&row.get::<_, String>(6)?),
                review_status: memory_review_from_label(&row.get::<_, String>(7)?),
                disabled: row.get::<_, i64>(8)? != 0,
                confidence: row.get(9)?,
                reason: row.get(10)?,
                privacy_status,
            })
        }),
    )?;
    let mut cards = Vec::new();
    for row in rows {
        cards.push(sqlite(&storage.database_path, row)?);
    }
    Ok(cards)
}

pub fn update_memory_card_review_status(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    card_id: &str,
    review_status: MemoryReviewStatus,
    disabled: bool,
) -> Result<StorageStatusReport, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    sqlite(
        &storage.database_path,
        connection.execute(
            r#"
UPDATE memory_cards
SET review_status = ?1, disabled = ?2, updated_at_unix_seconds = ?3
WHERE repo_id = ?4 AND card_id = ?5
"#,
            params![
                memory_review_label(&review_status),
                bool_as_i64(disabled),
                as_i64(current_unix_seconds()),
                storage.repo_id,
                card_id,
            ],
        ),
    )?;
    storage_status_for_path(&storage.database_path)
}

pub fn vacuum_store(database_path: impl AsRef<Path>) -> Result<(), StorageError> {
    let database_path = database_path.as_ref().to_path_buf();
    let connection = open_connection(&database_path)?;
    sqlite(&database_path, connection.execute_batch("VACUUM"))
}

pub fn persist_context_pack_record(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    record: &StorageContextPackRecord,
) -> Result<StorageStatusReport, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    sqlite(
        &storage.database_path,
        connection.execute(
            r#"
INSERT INTO context_packs (
  pack_id, repo_id, task_hash, budget, target_agent, confidence,
  selected_candidate_ids_json, warnings_json, privacy_status, created_at_unix_seconds
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
ON CONFLICT(pack_id) DO UPDATE SET
  task_hash = excluded.task_hash,
  budget = excluded.budget,
  target_agent = excluded.target_agent,
  confidence = excluded.confidence,
  selected_candidate_ids_json = excluded.selected_candidate_ids_json,
  warnings_json = excluded.warnings_json,
  privacy_status = excluded.privacy_status
"#,
            params![
                record.pack_id,
                storage.repo_id,
                record.task_hash,
                record.budget,
                record.target_agent,
                record.confidence,
                json_string(&record.selected_candidate_ids)?,
                json_string(&record.warnings)?,
                record.privacy_status,
                as_i64(current_unix_seconds())
            ],
        ),
    )?;
    storage_status_for_path(&storage.database_path)
}

pub fn persist_benchmark_run_record(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    record: &StorageBenchmarkRunRecord,
) -> Result<StorageStatusReport, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    sqlite(
        &storage.database_path,
        connection.execute(
            r#"
INSERT INTO benchmark_runs (
  run_id, repo_id, suite_id, revision_id, budget, privacy_status, created_at_unix_seconds
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT(run_id) DO UPDATE SET
  suite_id = excluded.suite_id,
  revision_id = excluded.revision_id,
  budget = excluded.budget,
  privacy_status = excluded.privacy_status
"#,
            params![
                record.run_id,
                storage.repo_id,
                record.suite_id,
                record.revision_id,
                record.budget,
                record.privacy_status,
                as_i64(current_unix_seconds())
            ],
        ),
    )?;
    sqlite(
        &storage.database_path,
        connection.execute(
            "DELETE FROM benchmark_metrics WHERE run_id = ?1",
            params![record.run_id],
        ),
    )?;
    sqlite(
        &storage.database_path,
        connection.execute(
            "DELETE FROM retrieval_gaps WHERE run_id = ?1",
            params![record.run_id],
        ),
    )?;
    for metric in &record.metrics {
        sqlite(
            &storage.database_path,
            connection.execute(
                r#"
INSERT INTO benchmark_metrics (metric_id, run_id, metric_name, metric_value, budget, target_kind)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)
"#,
                params![
                    metric_id(&record.run_id, &metric.name, metric.budget.as_deref()),
                    record.run_id,
                    metric.name,
                    metric.value,
                    metric.budget,
                    metric.target_kind
                ],
            ),
        )?;
    }
    for gap in &record.gaps {
        sqlite(
            &storage.database_path,
            connection.execute(
                r#"
INSERT INTO retrieval_gaps (
  gap_id, run_id, family, recommendation_area, target_status, safe_path, count
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#,
                params![
                    gap_id(&record.run_id, gap),
                    record.run_id,
                    gap.family,
                    gap.recommendation_area,
                    gap.target_status,
                    gap.safe_path,
                    as_i64(gap.count as u64)
                ],
            ),
        )?;
    }
    storage_status_for_path(&storage.database_path)
}

pub fn persist_proof_report_record(
    repo_root: impl AsRef<Path>,
    config: &StoreConfig,
    record: &StorageProofReportRecord,
) -> Result<StorageStatusReport, StorageError> {
    let storage = initialize_store(repo_root, config)?;
    let connection = open_connection(&storage.database_path)?;
    sqlite(
        &storage.database_path,
        connection.execute(
            r#"
INSERT INTO proof_reports (
  proof_id, repo_id, run_id, headline_metrics_json, limitations_json,
  privacy_status, created_at_unix_seconds
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT(proof_id) DO UPDATE SET
  run_id = excluded.run_id,
  headline_metrics_json = excluded.headline_metrics_json,
  limitations_json = excluded.limitations_json,
  privacy_status = excluded.privacy_status
"#,
            params![
                record.proof_id,
                storage.repo_id,
                record.run_id,
                record.headline_metrics_json,
                record.limitations_json,
                record.privacy_status,
                as_i64(current_unix_seconds())
            ],
        ),
    )?;
    storage_status_for_path(&storage.database_path)
}

fn store_paths_for_repo_id(repo_id: &str, config: &StoreConfig) -> StorePaths {
    let database_path = config.path_override.clone().unwrap_or_else(|| {
        ctxhelm_home()
            .join("repos")
            .join(repo_id)
            .join("ctxhelm.sqlite3")
    });
    StorePaths {
        repo_id: repo_id.to_string(),
        database_path,
    }
}

#[derive(Debug, Clone)]
struct StoredFileRecord {
    file_id: String,
    language: Option<String>,
    role: String,
    content_hash: String,
    size_bytes: u64,
    generated: bool,
    ignored: bool,
}

fn existing_file_records(
    connection: &Connection,
    path: &Path,
    repo_id: &str,
) -> Result<BTreeMap<String, StoredFileRecord>, StorageError> {
    let mut statement = sqlite(
        path,
        connection.prepare(
            r#"
SELECT path, file_id, language, role, content_hash, size_bytes, generated, ignored
FROM files WHERE repo_id = ?1
"#,
        ),
    )?;
    let rows = sqlite(
        path,
        statement.query_map(params![repo_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                StoredFileRecord {
                    file_id: row.get(1)?,
                    language: row.get(2)?,
                    role: row.get(3)?,
                    content_hash: row.get(4)?,
                    size_bytes: as_u64(row.get::<_, i64>(5)?),
                    generated: row.get::<_, i64>(6)? != 0,
                    ignored: row.get::<_, i64>(7)? != 0,
                },
            ))
        }),
    )?;
    let mut records = BTreeMap::new();
    for row in rows {
        let (path, record) = sqlite(path, row)?;
        records.insert(path, record);
    }
    Ok(records)
}

#[derive(Debug, Clone)]
struct StoredSemanticRecord {
    path: String,
    safe_hash: String,
    provider: String,
    model: String,
    dimensions: usize,
    distance_metric: String,
    vector_json: String,
    privacy_status: String,
}

impl StoredSemanticRecord {
    fn matches(&self, record: &StorageSemanticVectorRecord, vector_json: &str) -> bool {
        self.safe_hash == record.safe_hash
            && self.dimensions == record.dimensions
            && self.distance_metric == record.distance_metric
            && self.vector_json == vector_json
            && self.privacy_status == record.privacy_status
    }

    fn unique_key(&self) -> String {
        semantic_record_unique_key(&self.path, &self.provider, &self.model)
    }
}

fn existing_semantic_records(
    connection: &Connection,
    path: &Path,
    repo_id: &str,
) -> Result<BTreeMap<String, StoredSemanticRecord>, StorageError> {
    let mut statement = sqlite(
        path,
        connection.prepare(
            r#"
SELECT vector_id, path, safe_hash, provider, model, dimensions, distance_metric, vector_json, privacy_status
FROM semantic_vectors WHERE repo_id = ?1
"#,
        ),
    )?;
    let rows = sqlite(
        path,
        statement.query_map(params![repo_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                StoredSemanticRecord {
                    path: row.get(1)?,
                    safe_hash: row.get(2)?,
                    provider: row.get(3)?,
                    model: row.get(4)?,
                    dimensions: as_u64(row.get::<_, i64>(5)?) as usize,
                    distance_metric: row.get(6)?,
                    vector_json: row.get(7)?,
                    privacy_status: row.get(8)?,
                },
            ))
        }),
    )?;
    let mut records = BTreeMap::new();
    for row in rows {
        let (vector_id, record) = sqlite(path, row)?;
        records.insert(vector_id, record);
    }
    Ok(records)
}

fn semantic_record_unique_key(path: &str, provider: &str, model: &str) -> String {
    format!("{path}\n{provider}\n{model}")
}

fn count_rows(connection: &Connection, path: &Path, table: &str) -> Result<usize, StorageError> {
    let query = format!("SELECT COUNT(*) FROM {table}");
    let count = sqlite(
        path,
        connection.query_row(&query, [], |row| row.get::<_, i64>(0)),
    )?;
    Ok(as_u64(count) as usize)
}

fn file_id_for_path(repo_id: &str, path: &str) -> String {
    let hash = blake3::hash(format!("{repo_id}:{path}").as_bytes())
        .to_hex()
        .to_string();
    format!("file:{hash}")
}

fn semantic_vector_id(repo_id: &str, record: &StorageSemanticVectorRecord) -> String {
    let hash = blake3::hash(
        format!(
            "{}:{}:{}:{}:{}",
            repo_id, record.path, record.safe_hash, record.provider, record.model
        )
        .as_bytes(),
    )
    .to_hex()
    .to_string();
    format!("semantic:{hash}")
}

fn memory_kind_label(kind: &MemoryCardKind) -> &'static str {
    match kind {
        MemoryCardKind::Domain => "domain",
        MemoryCardKind::Experience => "experience",
    }
}

fn memory_kind_from_label(label: &str) -> MemoryCardKind {
    match label {
        "experience" => MemoryCardKind::Experience,
        _ => MemoryCardKind::Domain,
    }
}

fn memory_freshness_label(freshness: &MemoryFreshness) -> &'static str {
    match freshness {
        MemoryFreshness::Fresh => "fresh",
        MemoryFreshness::Stale => "stale",
        MemoryFreshness::Degraded => "degraded",
    }
}

fn memory_freshness_from_label(label: &str) -> MemoryFreshness {
    match label {
        "stale" => MemoryFreshness::Stale,
        "degraded" => MemoryFreshness::Degraded,
        _ => MemoryFreshness::Fresh,
    }
}

fn memory_review_label(status: &MemoryReviewStatus) -> &'static str {
    match status {
        MemoryReviewStatus::Deterministic => "deterministic",
        MemoryReviewStatus::Pending => "pending",
        MemoryReviewStatus::Approved => "approved",
        MemoryReviewStatus::Rejected => "rejected",
        MemoryReviewStatus::Disabled => "disabled",
    }
}

fn memory_review_from_label(label: &str) -> MemoryReviewStatus {
    match label {
        "approved" => MemoryReviewStatus::Approved,
        "rejected" => MemoryReviewStatus::Rejected,
        "disabled" => MemoryReviewStatus::Disabled,
        "pending" => MemoryReviewStatus::Pending,
        _ => MemoryReviewStatus::Deterministic,
    }
}

fn role_name(role: &FileRole) -> String {
    format!("{:?}", role).to_lowercase()
}

fn json_string<T: Serialize>(value: &T) -> Result<String, StorageError> {
    serde_json::to_string(value).map_err(|source| StorageError::Sqlite {
        path: PathBuf::from("serde_json"),
        source: rusqlite::Error::ToSqlConversionFailure(Box::new(source)),
    })
}

fn metric_id(run_id: &str, name: &str, budget: Option<&str>) -> String {
    let hash = blake3::hash(format!("{run_id}:{name}:{}", budget.unwrap_or("")).as_bytes())
        .to_hex()
        .to_string();
    format!("metric:{hash}")
}

fn gap_id(run_id: &str, gap: &StorageGapRecord) -> String {
    let hash = blake3::hash(
        format!(
            "{}:{}:{}:{}:{}",
            run_id,
            gap.family,
            gap.recommendation_area.as_deref().unwrap_or(""),
            gap.target_status.as_deref().unwrap_or(""),
            gap.safe_path.as_deref().unwrap_or("")
        )
        .as_bytes(),
    )
    .to_hex()
    .to_string();
    format!("gap:{hash}")
}

fn canonicalize_storage(path: &Path) -> Result<PathBuf, StorageError> {
    canonicalize(path).map_err(|error| match error {
        crate::InventoryError::Canonicalize { path, source } => {
            StorageError::Canonicalize { path, source }
        }
        _ => StorageError::Canonicalize {
            path: path.to_path_buf(),
            source: std::io::Error::other(error.to_string()),
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
  ctxhelm_version TEXT NOT NULL,
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

CREATE TABLE IF NOT EXISTS semantic_vectors (
  vector_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  file_id TEXT REFERENCES files(file_id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  safe_hash TEXT NOT NULL,
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  dimensions INTEGER NOT NULL,
  distance_metric TEXT NOT NULL,
  vector_json TEXT NOT NULL,
  privacy_status TEXT NOT NULL,
  updated_at_unix_seconds INTEGER NOT NULL,
  UNIQUE(repo_id, path, provider, model)
);

CREATE TABLE IF NOT EXISTS memory_cards (
  card_id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL REFERENCES repos(repo_id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  title TEXT NOT NULL,
  summary_text TEXT NOT NULL,
  link_paths_json TEXT NOT NULL,
  input_hashes_json TEXT NOT NULL,
  freshness TEXT NOT NULL,
  review_status TEXT NOT NULL,
  disabled INTEGER NOT NULL DEFAULT 0,
  confidence REAL NOT NULL DEFAULT 0.0,
  reason TEXT NOT NULL,
  privacy_status TEXT NOT NULL,
  created_at_unix_seconds INTEGER NOT NULL,
  updated_at_unix_seconds INTEGER NOT NULL
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
  id, repo_id, repo_root, schema_version, ctxhelm_version, ranking_version,
  compiler_version, privacy_mode, created_at_unix_seconds, updated_at_unix_seconds
)
SELECT 1, repo_id, repo_root, ?2, ?3, ?4, ?5, ?6, ?7, ?8
FROM repos WHERE repo_id = ?1
ON CONFLICT(id) DO UPDATE SET
  repo_id = excluded.repo_id,
  repo_root = excluded.repo_root,
  schema_version = excluded.schema_version,
  ctxhelm_version = excluded.ctxhelm_version,
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
                1_u32,
                INITIAL_MIGRATION_NAME,
                as_i64(current_unix_seconds()),
                INITIAL_MIGRATION_CHECKSUM
            ],
        ),
    )?;
    Ok(())
}

fn insert_semantic_migration(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    sqlite(
        path,
        connection.execute(
            r#"
INSERT INTO schema_migrations (version, name, applied_at_unix_seconds, checksum)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(version) DO NOTHING
"#,
            params![
                2_u32,
                SEMANTIC_MIGRATION_NAME,
                as_i64(current_unix_seconds()),
                SEMANTIC_MIGRATION_CHECKSUM
            ],
        ),
    )?;
    Ok(())
}

fn insert_memory_migration(connection: &Connection, path: &Path) -> Result<(), StorageError> {
    sqlite(
        path,
        connection.execute(
            r#"
INSERT INTO schema_migrations (version, name, applied_at_unix_seconds, checksum)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(version) DO NOTHING
"#,
            params![
                3_u32,
                MEMORY_MIGRATION_NAME,
                as_i64(current_unix_seconds()),
                MEMORY_MIGRATION_CHECKSUM
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
SELECT schema_version, ctxhelm_version, ranking_version, compiler_version,
       privacy_mode, created_at_unix_seconds, updated_at_unix_seconds
FROM storage_metadata WHERE id = 1
"#,
                [],
                |row| {
                    Ok(StorageMetadata {
                        schema_version: row.get::<_, u32>(0)?,
                        ctxhelm_version: row.get(1)?,
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

fn bool_as_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        append_eval_trace, load_or_build_inventory, trace_path, write_inventory, InventoryOptions,
    };
    use ctxhelm_core::{EvalTrace, PackBudget, TaskType};
    use std::fs;
    use uuid::Uuid;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        crate::test_env_lock()
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
    fn initialize_store_uses_ctxhelm_home_default_path() {
        let _guard = env_lock();
        let (temp, repo) = fixture_repo();
        let home = temp.path().join("home");
        std::env::set_var("CTXHELM_HOME", &home);

        let report = initialize_store(&repo, &StoreConfig::default()).unwrap();

        assert!(report.database_path.starts_with(&home));
        assert!(report.database_path.ends_with(Path::new("ctxhelm.sqlite3")));
        assert_eq!(report.schema_version, STORAGE_SCHEMA_VERSION);
        assert_eq!(report.privacy_mode, StoragePrivacyMode::SourceFree);

        std::env::remove_var("CTXHELM_HOME");
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
                    ctxhelm_version TEXT NOT NULL,
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
                "pub fn leak() { /* CTXHELM_SHOULD_NOT_STORE_SOURCE_BODY */ }\n",
            )
            .unwrap();
            std::env::set_var("CTXHELM_HOME", temp.path().join("home"));
            (guard, temp, repo)
        };
        let report = initialize_store(&repo, &StoreConfig::default()).unwrap();
        let bytes = fs::read(&report.database_path).unwrap();
        let database_text = String::from_utf8_lossy(&bytes);

        assert!(!database_text.contains("CTXHELM_SHOULD_NOT_STORE_SOURCE_BODY"));
        assert!(!database_text.contains("CTXHELM_SHOULD_NOT_STORE_PROMPT_TEXT"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn sync_inventory_reports_reused_updated_created_and_deleted_records() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let config = StoreConfig {
            path_override: Some(temp.path().join("store.sqlite3")),
        };
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn demo() {}\n").unwrap();
        fs::write(repo.join("src/old.rs"), "pub fn old() {}\n").unwrap();

        let first = sync_inventory_to_store(&repo, &InventoryOptions::default(), &config).unwrap();
        assert_eq!(first.created_records, 2);
        assert_eq!(first.reused_records, 0);
        assert_eq!(first.updated_records, 0);
        assert_eq!(first.deleted_records, 0);

        let second = sync_inventory_to_store(&repo, &InventoryOptions::default(), &config).unwrap();
        assert_eq!(second.reused_records, 2);
        assert_eq!(second.created_records, 0);
        assert_eq!(second.updated_records, 0);
        assert_eq!(second.deleted_records, 0);

        fs::write(repo.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
        fs::remove_file(repo.join("src/old.rs")).unwrap();
        fs::write(repo.join("src/new.rs"), "pub fn new() {}\n").unwrap();
        let third = sync_inventory_to_store(&repo, &InventoryOptions::default(), &config).unwrap();
        assert_eq!(third.reused_records, 0);
        assert_eq!(third.created_records, 1);
        assert_eq!(third.updated_records, 1);
        assert_eq!(third.deleted_records, 1);

        let status = storage_status_for_repo(&repo, &config).unwrap();
        assert_eq!(status.file_records, 2);
        assert_eq!(status.compatibility, StorageCompatibility::Compatible);

        drop(temp);
    }

    #[test]
    fn sync_inventory_keeps_source_content_out_of_storage() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let config = StoreConfig {
            path_override: Some(temp.path().join("store.sqlite3")),
        };
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/lib.rs"),
            "pub fn leak() { /* CTXHELM_INCREMENTAL_SOURCE_SENTINEL */ }\n",
        )
        .unwrap();

        let report = sync_inventory_to_store(&repo, &InventoryOptions::default(), &config).unwrap();
        let bytes = fs::read(&report.database_path).unwrap();
        let database_text = String::from_utf8_lossy(&bytes);

        assert!(!database_text.contains("CTXHELM_INCREMENTAL_SOURCE_SENTINEL"));

        drop(temp);
    }

    #[test]
    fn persists_semantic_vectors_without_source_text() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let config = StoreConfig {
            path_override: Some(temp.path().join("store.sqlite3")),
        };
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/lib.rs"),
            "pub fn leak() { /* CTXHELM_SEMANTIC_SOURCE_SENTINEL */ }\n",
        )
        .unwrap();

        sync_inventory_to_store(&repo, &InventoryOptions::default(), &config).unwrap();
        let status = persist_semantic_vector_records(
            &repo,
            &config,
            &[StorageSemanticVectorRecord {
                path: "src/lib.rs".to_string(),
                safe_hash: "safe-hash".to_string(),
                provider: "local_hash".to_string(),
                model: "ctxhelm-local-hash-v1".to_string(),
                dimensions: 3,
                distance_metric: "cosine".to_string(),
                vector: vec![0.1, 0.2, 0.3],
                privacy_status: "local_only".to_string(),
            }],
        )
        .unwrap();

        assert_eq!(status.semantic_vector_records, 1);
        assert_eq!(status.created_records, 1);
        assert_eq!(status.reused_records, 0);
        let reused = persist_semantic_vector_records(
            &repo,
            &config,
            &[StorageSemanticVectorRecord {
                path: "src/lib.rs".to_string(),
                safe_hash: "safe-hash".to_string(),
                provider: "local_hash".to_string(),
                model: "ctxhelm-local-hash-v1".to_string(),
                dimensions: 3,
                distance_metric: "cosine".to_string(),
                vector: vec![0.1, 0.2, 0.3],
                privacy_status: "local_only".to_string(),
            }],
        )
        .unwrap();
        assert_eq!(reused.semantic_vector_records, 1);
        assert_eq!(reused.created_records, 0);
        assert_eq!(reused.updated_records, 0);
        assert_eq!(reused.reused_records, 1);
        let second_provider = persist_semantic_vector_records(
            &repo,
            &config,
            &[StorageSemanticVectorRecord {
                path: "src/lib.rs".to_string(),
                safe_hash: "safe-hash".to_string(),
                provider: "local_fastembed".to_string(),
                model: "JinaEmbeddingsV2BaseCode".to_string(),
                dimensions: 768,
                distance_metric: "cosine".to_string(),
                vector: vec![0.4, 0.5, 0.6],
                privacy_status: "local_only".to_string(),
            }],
        )
        .unwrap();
        assert_eq!(second_provider.semantic_vector_records, 1);
        assert_eq!(second_provider.created_records, 1);
        let loaded = load_semantic_vector_records(
            &repo,
            &config,
            "local_fastembed",
            "JinaEmbeddingsV2BaseCode",
            768,
            "cosine",
        )
        .unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].path, "src/lib.rs");
        assert_eq!(loaded[0].safe_hash, "safe-hash");
        assert_eq!(loaded[0].vector, vec![0.4, 0.5, 0.6]);
        let changed_hash = persist_semantic_vector_records(
            &repo,
            &config,
            &[StorageSemanticVectorRecord {
                path: "src/lib.rs".to_string(),
                safe_hash: "new-safe-hash".to_string(),
                provider: "local_fastembed".to_string(),
                model: "JinaEmbeddingsV2BaseCode".to_string(),
                dimensions: 768,
                distance_metric: "cosine".to_string(),
                vector: vec![0.7, 0.8, 0.9],
                privacy_status: "local_only".to_string(),
            }],
        )
        .unwrap();
        assert_eq!(changed_hash.semantic_vector_records, 1);
        assert_eq!(changed_hash.updated_records, 1);
        let reloaded = load_semantic_vector_records(
            &repo,
            &config,
            "local_fastembed",
            "JinaEmbeddingsV2BaseCode",
            768,
            "cosine",
        )
        .unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].safe_hash, "new-safe-hash");
        assert_eq!(reloaded[0].vector, vec![0.7, 0.8, 0.9]);
        let bytes = fs::read(&status.database_path).unwrap();
        let database_text = String::from_utf8_lossy(&bytes);
        assert!(!database_text.contains("CTXHELM_SEMANTIC_SOURCE_SENTINEL"));
        assert!(database_text.contains("local_fastembed"));
        assert!(database_text.contains("JinaEmbeddingsV2BaseCode"));

        drop(temp);
    }

    #[test]
    fn persists_and_reviews_memory_cards_without_source_text() {
        let (_temp, repo) = fixture_repo();
        let database_path = repo.join("store.sqlite3");
        let config = StoreConfig {
            path_override: Some(database_path.clone()),
        };
        let card = MemoryCard {
            id: "domain:auth".to_string(),
            kind: MemoryCardKind::Domain,
            title: "Auth".to_string(),
            summary: "Auth uses session and middleware metadata.".to_string(),
            source_links: vec!["src/lib.rs".to_string()],
            input_hashes: vec!["hash-1".to_string()],
            freshness: MemoryFreshness::Fresh,
            review_status: MemoryReviewStatus::Pending,
            disabled: false,
            confidence: 0.7,
            reason: "test metadata".to_string(),
            privacy_status: PrivacyStatus::local_only(),
        };

        let status = persist_memory_card_records(
            &repo,
            &config,
            &[StorageMemoryCardRecord { card: card.clone() }],
        )
        .unwrap();
        assert_eq!(status.memory_card_records, 1);
        let cards = list_memory_cards(&repo, &config, false).unwrap();
        assert_eq!(cards[0].id, "domain:auth");
        assert_eq!(cards[0].review_status, MemoryReviewStatus::Pending);

        let status = update_memory_card_review_status(
            &repo,
            &config,
            "domain:auth",
            MemoryReviewStatus::Approved,
            false,
        )
        .unwrap();
        assert_eq!(status.memory_card_records, 1);
        let cards = list_memory_cards(&repo, &config, false).unwrap();
        assert_eq!(cards[0].review_status, MemoryReviewStatus::Approved);

        update_memory_card_review_status(
            &repo,
            &config,
            "domain:auth",
            MemoryReviewStatus::Rejected,
            true,
        )
        .unwrap();
        assert!(list_memory_cards(&repo, &config, false).unwrap().is_empty());
        assert_eq!(list_memory_cards(&repo, &config, true).unwrap().len(), 1);

        let bytes = fs::read(&database_path).unwrap();
        let database_text = String::from_utf8_lossy(&bytes);
        assert!(!database_text.contains("pub fn demo"));
        assert!(!database_text.contains("prompt"));
    }

    #[test]
    fn persists_pack_benchmark_and_proof_metadata_without_source_text() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join("README.md"), "# Repo\n").unwrap();
        let config = StoreConfig {
            path_override: Some(temp.path().join("store.sqlite3")),
        };

        let pack_status = persist_context_pack_record(
            &repo,
            &config,
            &StorageContextPackRecord {
                pack_id: "pack-1".to_string(),
                task_hash: "task-hash".to_string(),
                budget: "brief".to_string(),
                target_agent: "codex".to_string(),
                confidence: 0.7,
                selected_candidate_ids: vec!["file:src/lib.rs".to_string()],
                warnings: vec!["CTXHELM_SHOULD_NOT_STORE_PROMPT_TEXT".to_string()],
                privacy_status: "local_only".to_string(),
            },
        )
        .unwrap();
        assert_eq!(pack_status.context_pack_records, 1);

        let benchmark_status = persist_benchmark_run_record(
            &repo,
            &config,
            &StorageBenchmarkRunRecord {
                run_id: "run-1".to_string(),
                suite_id: "suite-1".to_string(),
                revision_id: Some("rev".to_string()),
                budget: Some("10".to_string()),
                privacy_status: "local_only".to_string(),
                metrics: vec![StorageMetricRecord {
                    name: "fileRecallAt10".to_string(),
                    value: 0.9,
                    budget: Some("10".to_string()),
                    target_kind: Some("file".to_string()),
                }],
                gaps: vec![StorageGapRecord {
                    family: "docs".to_string(),
                    recommendation_area: Some("storage".to_string()),
                    target_status: Some("currentReachable".to_string()),
                    safe_path: Some("README.md".to_string()),
                    count: 1,
                }],
            },
        )
        .unwrap();
        assert_eq!(benchmark_status.benchmark_run_records, 1);

        let proof_status = persist_proof_report_record(
            &repo,
            &config,
            &StorageProofReportRecord {
                proof_id: "proof-1".to_string(),
                run_id: Some("run-1".to_string()),
                headline_metrics_json: r#"[{"label":"Recall","value":0.9}]"#.to_string(),
                limitations_json: r#"["large repos need more coverage"]"#.to_string(),
                privacy_status: "local_only".to_string(),
            },
        )
        .unwrap();
        assert_eq!(proof_status.proof_report_records, 1);

        let bytes = fs::read(&proof_status.database_path).unwrap();
        let database_text = String::from_utf8_lossy(&bytes);
        assert!(!database_text.contains("CTXHELM_SHOULD_NOT_STORE_SOURCE_BODY"));

        drop(temp);
    }

    #[test]
    fn json_inventory_and_trace_fallback_remain_intact() {
        let _guard = env_lock();
        let (temp, repo) = fixture_repo();
        let home = temp.path().join("home");
        std::env::set_var("CTXHELM_HOME", &home);

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

        assert!(store.database_path.ends_with("ctxhelm.sqlite3"));
        assert!(inventory_report.inventory_path.ends_with("inventory.json"));
        assert!(trace_path.ends_with("traces.jsonl"));
        assert!(inventory_report.inventory_path.exists());
        assert!(trace_path.exists());
        assert_eq!(traces.len(), 1);

        std::env::remove_var("CTXHELM_HOME");
    }
}
