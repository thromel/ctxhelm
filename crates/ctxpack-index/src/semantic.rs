use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{canonicalize, InventoryError, InventoryOptions};
use crate::policy::{read_safe_source, SourceReadStatus, SOURCE_READ_MAX_BYTES};
use crate::search::{query_term_weight, query_terms};
use crate::storage::{
    persist_semantic_vector_records, sync_inventory_to_store, StorageError,
    StorageSemanticIndexReport, StorageSemanticVectorRecord, StoreConfig,
};
use ctxpack_core::{CacheStatus, Diagnostic, DiagnosticSeverity, FileRole, PrivacyStatus};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DEFAULT_SEMANTIC_DIMENSIONS: usize = 64;
pub const DEFAULT_SEMANTIC_PROVIDER: &str = "local_hash";
pub const DEFAULT_SEMANTIC_MODEL: &str = "ctxpack-local-hash-v1";
pub const DEFAULT_SEMANTIC_DISTANCE: &str = "cosine";
pub const LOCAL_HASH_PROVIDER_ROLE: &str = "deterministic_scaffold";
pub const LOCAL_FASTEMBED_PROVIDER: &str = "local_fastembed";
pub const LOCAL_FASTEMBED_MODEL: &str = "JinaEmbeddingsV2BaseCode";
pub const LOCAL_FASTEMBED_DIMENSIONS: usize = 768;
pub const LOCAL_FASTEMBED_PROVIDER_ROLE: &str = "production_local";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticProviderConfig {
    pub provider: String,
    pub model: String,
    pub dimensions: usize,
    pub distance_metric: String,
    #[serde(default = "default_semantic_provider_role")]
    pub provider_role: String,
    #[serde(default)]
    pub quality_backend: bool,
    #[serde(default = "default_true")]
    pub local_only: bool,
    #[serde(default = "default_true")]
    pub available: bool,
}

impl Default for SemanticProviderConfig {
    fn default() -> Self {
        Self {
            provider: DEFAULT_SEMANTIC_PROVIDER.to_string(),
            model: DEFAULT_SEMANTIC_MODEL.to_string(),
            dimensions: DEFAULT_SEMANTIC_DIMENSIONS,
            distance_metric: DEFAULT_SEMANTIC_DISTANCE.to_string(),
            provider_role: LOCAL_HASH_PROVIDER_ROLE.to_string(),
            quality_backend: false,
            local_only: true,
            available: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticOptions {
    pub limit: usize,
    pub enabled: bool,
    #[serde(default)]
    pub provider: SemanticProviderConfig,
}

impl Default for SemanticOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            enabled: false,
            provider: SemanticProviderConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchResult {
    pub path: String,
    pub role: FileRole,
    pub language: Option<String>,
    pub score: f32,
    pub reason: String,
    pub provider: SemanticProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSearchReport {
    pub results: Vec<SemanticSearchResult>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
    pub privacy_status: PrivacyStatus,
    pub provider: SemanticProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticVectorRecord {
    pub path: String,
    pub role: FileRole,
    pub language: Option<String>,
    pub safe_hash: String,
    pub vector: Vec<f32>,
    pub provider: SemanticProviderConfig,
    pub privacy_status: PrivacyStatus,
}

pub fn semantic_search(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SemanticOptions,
) -> Result<Vec<SemanticSearchResult>, InventoryError> {
    Ok(semantic_search_report(repo_root, query, options)?.results)
}

pub fn semantic_search_report(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SemanticOptions,
) -> Result<SemanticSearchReport, InventoryError> {
    let provider = normalized_provider(&options.provider);
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    if !options.enabled {
        diagnostics.push(semantic_disabled_diagnostic());
        return Ok(SemanticSearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
            privacy_status: PrivacyStatus::local_only(),
            provider,
        });
    }

    if !provider.available {
        diagnostics.push(semantic_provider_unavailable_diagnostic(&provider));
        return Ok(SemanticSearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
            privacy_status: PrivacyStatus::local_only(),
            provider,
        });
    }

    if query_terms(query).is_empty() {
        diagnostics.push(Diagnostic {
            code: "semantic_query_empty".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Semantic retrieval received no meaningful query terms.".to_string(),
            paths: Vec::new(),
            count: 0,
        });
        return Ok(SemanticSearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
            privacy_status: PrivacyStatus::local_only(),
            provider,
        });
    }

    let mut candidate_texts = Vec::new();
    let mut candidate_files = Vec::new();
    for file in &inventory_report.inventory.files {
        if file.generated || file.role == FileRole::Sensitive || file.ignored {
            continue;
        }
        let source = read_safe_source(
            &repo_root,
            &inventory_report.inventory,
            &file.path,
            SOURCE_READ_MAX_BYTES,
        )?;
        diagnostics.extend(source.diagnostics);
        let SourceReadStatus::Read = source.status else {
            continue;
        };
        candidate_texts.push(format!(
            "{}\n{}",
            file.path,
            source.text.unwrap_or_default()
        ));
        candidate_files.push(file);
    }

    let mut texts = Vec::with_capacity(candidate_texts.len() + 1);
    texts.push(query.to_string());
    texts.extend(candidate_texts);
    let vectors = match embed_texts(&texts, &provider) {
        Ok(vectors) => vectors,
        Err(message) => {
            diagnostics.push(Diagnostic {
                code: "semantic_provider_unavailable".to_string(),
                severity: DiagnosticSeverity::Warning,
                message,
                paths: Vec::new(),
                count: 0,
            });
            return Ok(SemanticSearchReport {
                results: Vec::new(),
                diagnostics,
                cache_status,
                privacy_status: PrivacyStatus::local_only(),
                provider,
            });
        }
    };
    let Some((query_vector, file_vectors)) = vectors.split_first() else {
        return Ok(SemanticSearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
            privacy_status: PrivacyStatus::local_only(),
            provider,
        });
    };
    if is_zero_vector(query_vector) {
        diagnostics.push(Diagnostic {
            code: "semantic_query_empty".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Semantic retrieval received no meaningful query terms.".to_string(),
            paths: Vec::new(),
            count: 0,
        });
        return Ok(SemanticSearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
            privacy_status: PrivacyStatus::local_only(),
            provider,
        });
    }

    let mut results = Vec::new();
    for (file, file_vector) in candidate_files.into_iter().zip(file_vectors.iter()) {
        let score = cosine_similarity(&query_vector, &file_vector);
        if score < 0.08 {
            continue;
        }
        results.push(SemanticSearchResult {
            path: file.path.clone(),
            role: file.role.clone(),
            language: file.language.clone(),
            score,
            reason: format!(
                "local semantic similarity via {} {}",
                provider.provider, provider.model
            ),
            provider: provider.clone(),
        });
    }

    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    results.truncate(options.limit.max(1));

    Ok(SemanticSearchReport {
        results,
        diagnostics,
        cache_status,
        privacy_status: PrivacyStatus::local_only(),
        provider,
    })
}

pub fn semantic_vector_records(
    repo_root: impl AsRef<Path>,
    options: &SemanticOptions,
) -> Result<Vec<SemanticVectorRecord>, InventoryError> {
    let provider = normalized_provider(&options.provider);
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    if !options.enabled || !provider.available {
        return Ok(Vec::new());
    }
    let mut inputs = Vec::new();
    let mut files = Vec::new();
    let mut hashes = Vec::new();
    for file in &inventory_report.inventory.files {
        if file.generated || file.role == FileRole::Sensitive || file.ignored {
            continue;
        }
        let source = read_safe_source(
            &repo_root,
            &inventory_report.inventory,
            &file.path,
            SOURCE_READ_MAX_BYTES,
        )?;
        let SourceReadStatus::Read = source.status else {
            continue;
        };
        inputs.push(format!(
            "{}\n{}",
            file.path,
            source.text.unwrap_or_default()
        ));
        files.push(file);
        hashes.push(file.hash.clone());
    }
    let vectors = embed_texts(&inputs, &provider).unwrap_or_default();
    let mut records = Vec::new();
    for ((file, safe_hash), vector) in files.into_iter().zip(hashes).zip(vectors) {
        records.push(SemanticVectorRecord {
            path: file.path.clone(),
            role: file.role.clone(),
            language: file.language.clone(),
            safe_hash,
            vector,
            provider: provider.clone(),
            privacy_status: PrivacyStatus::local_only(),
        });
    }
    Ok(records)
}

pub fn sync_semantic_index_to_store(
    repo_root: impl AsRef<Path>,
    options: &SemanticOptions,
    config: &StoreConfig,
) -> Result<StorageSemanticIndexReport, StorageError> {
    let repo_root = repo_root.as_ref();
    sync_inventory_to_store(repo_root, &InventoryOptions::default(), config)?;
    let records = semantic_vector_records(repo_root, options)
        .map_err(|error| semantic_inventory_error(repo_root, error))?;
    let storage_records = records
        .into_iter()
        .map(|record| StorageSemanticVectorRecord {
            path: record.path,
            safe_hash: record.safe_hash,
            provider: record.provider.provider,
            model: record.provider.model,
            dimensions: record.provider.dimensions,
            distance_metric: record.provider.distance_metric,
            vector: record.vector,
            privacy_status: serde_json::to_string(&record.privacy_status)
                .unwrap_or_else(|_| "local_only".to_string()),
        })
        .collect::<Vec<_>>();
    persist_semantic_vector_records(repo_root, config, &storage_records)
}

fn semantic_inventory_error(path: &Path, error: InventoryError) -> StorageError {
    StorageError::Canonicalize {
        path: path.to_path_buf(),
        source: std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
    }
}

pub(crate) fn vectorize_text(text: &str, dimensions: usize) -> Vec<f32> {
    let dimensions = dimensions.clamp(8, 4096);
    let mut vector = vec![0.0; dimensions];
    for term in query_terms(text) {
        let weight = query_term_weight(&term);
        if weight == 0.0 {
            continue;
        }
        let hash = blake3::hash(term.as_bytes());
        let bytes = hash.as_bytes();
        let index = u64::from_le_bytes(bytes[0..8].try_into().unwrap()) as usize % dimensions;
        let sign = if bytes[8] & 1 == 0 { 1.0 } else { -1.0 };
        vector[index] += sign * weight;
    }
    normalize_vector(&mut vector);
    vector
}

pub fn normalized_provider(provider: &SemanticProviderConfig) -> SemanticProviderConfig {
    let default = SemanticProviderConfig::default();
    let provider_id = empty_to_default(&provider.provider, &default.provider);
    if provider_id == LOCAL_FASTEMBED_PROVIDER {
        return SemanticProviderConfig {
            provider: provider_id,
            model: if provider.model.trim().is_empty() || provider.model == DEFAULT_SEMANTIC_MODEL {
                LOCAL_FASTEMBED_MODEL.to_string()
            } else {
                provider.model.trim().to_string()
            },
            dimensions: if provider.dimensions == DEFAULT_SEMANTIC_DIMENSIONS {
                LOCAL_FASTEMBED_DIMENSIONS
            } else {
                provider.dimensions.clamp(8, 4096)
            },
            distance_metric: empty_to_default(&provider.distance_metric, &default.distance_metric),
            provider_role: LOCAL_FASTEMBED_PROVIDER_ROLE.to_string(),
            quality_backend: true,
            local_only: true,
            available: local_fastembed_available(),
        };
    }
    if provider_id != DEFAULT_SEMANTIC_PROVIDER {
        return SemanticProviderConfig {
            provider: provider_id,
            model: empty_to_default(&provider.model, &default.model),
            dimensions: provider.dimensions.clamp(8, 4096),
            distance_metric: empty_to_default(&provider.distance_metric, &default.distance_metric),
            provider_role: "unsupported".to_string(),
            quality_backend: false,
            local_only: true,
            available: false,
        };
    }
    SemanticProviderConfig {
        provider: provider_id,
        model: empty_to_default(&provider.model, &default.model),
        dimensions: provider.dimensions.clamp(8, 4096),
        distance_metric: empty_to_default(&provider.distance_metric, &default.distance_metric),
        provider_role: LOCAL_HASH_PROVIDER_ROLE.to_string(),
        quality_backend: false,
        local_only: true,
        available: true,
    }
}

fn default_semantic_provider_role() -> String {
    LOCAL_HASH_PROVIDER_ROLE.to_string()
}

fn default_true() -> bool {
    true
}

fn local_fastembed_available() -> bool {
    cfg!(feature = "local-embeddings")
}

fn semantic_provider_unavailable_diagnostic(provider: &SemanticProviderConfig) -> Diagnostic {
    let message = if provider.provider == LOCAL_FASTEMBED_PROVIDER {
        "Semantic provider local_fastembed requires the ctxpack-index local-embeddings feature and remains opt-in; no cloud provider was used."
    } else {
        "Semantic provider is unsupported by this ctxpack build; no cloud provider was used."
    };
    Diagnostic {
        code: "semantic_provider_unavailable".to_string(),
        severity: DiagnosticSeverity::Warning,
        message: message.to_string(),
        paths: Vec::new(),
        count: 0,
    }
}

fn embed_texts(
    texts: &[String],
    provider: &SemanticProviderConfig,
) -> Result<Vec<Vec<f32>>, String> {
    if provider.provider == DEFAULT_SEMANTIC_PROVIDER {
        return Ok(texts
            .iter()
            .map(|text| vectorize_text(text, provider.dimensions))
            .collect());
    }
    if provider.provider == LOCAL_FASTEMBED_PROVIDER {
        return local_fastembed_vectors(texts, provider);
    }
    Err(format!(
        "Semantic provider {} is unsupported by this ctxpack build; no cloud provider was used.",
        provider.provider
    ))
}

#[cfg(feature = "local-embeddings")]
fn local_fastembed_vectors(
    texts: &[String],
    provider: &SemanticProviderConfig,
) -> Result<Vec<Vec<f32>>, String> {
    use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
    use std::str::FromStr;

    let embedding_model = if provider.model == LOCAL_FASTEMBED_MODEL {
        EmbeddingModel::JinaEmbeddingsV2BaseCode
    } else {
        EmbeddingModel::from_str(&provider.model).map_err(|error| {
            format!(
                "Semantic provider {} could not parse model {}: {}",
                provider.provider, provider.model, error
            )
        })?
    };
    let model = TextEmbedding::try_new(
        InitOptions::new(embedding_model).with_show_download_progress(false),
    )
    .map_err(|error| {
        format!(
            "Semantic provider {} failed to initialize local model {}: {}",
            provider.provider, provider.model, error
        )
    })?;
    model.embed(texts.to_vec(), None).map_err(|error| {
        format!(
            "Semantic provider {} failed to embed with local model {}: {}",
            provider.provider, provider.model, error
        )
    })
}

#[cfg(not(feature = "local-embeddings"))]
fn local_fastembed_vectors(
    _texts: &[String],
    provider: &SemanticProviderConfig,
) -> Result<Vec<Vec<f32>>, String> {
    Err(format!(
        "Semantic provider {} requires the ctxpack-index local-embeddings feature; no cloud provider was used.",
        provider.provider
    ))
}

fn empty_to_default(value: &str, default: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        default.to_string()
    } else {
        value.to_string()
    }
}

fn normalize_vector(vector: &mut [f32]) {
    let magnitude = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude == 0.0 {
        return;
    }
    for value in vector {
        *value /= magnitude;
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum::<f32>()
        .max(0.0)
}

fn is_zero_vector(vector: &[f32]) -> bool {
    vector.iter().all(|value| *value == 0.0)
}

fn semantic_disabled_diagnostic() -> Diagnostic {
    Diagnostic {
        code: "semantic_disabled".to_string(),
        severity: DiagnosticSeverity::Info,
        message: "Local semantic retrieval is disabled; pass an explicit semantic flag to enable the local provider.".to_string(),
        paths: Vec::new(),
        count: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn semantic_search_is_disabled_by_default() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::write(repo.join("README.md"), "payment webhook validation\n").unwrap();

        let report =
            semantic_search_report(repo, "payment webhook", &SemanticOptions::default()).unwrap();

        assert!(report.results.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "semantic_disabled"));
        assert!(report.privacy_status.local_only);
        assert!(!report.privacy_status.remote_embeddings_used);
    }

    #[test]
    fn semantic_search_finds_conceptual_safe_files() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/webhooks.ts"),
            "export function verifyStripeSignature() { return 'payment event validation'; }\n",
        )
        .unwrap();
        fs::write(repo.join(".env"), "SECRET=payment webhook\n").unwrap();

        let report = semantic_search_report(
            repo,
            "payment webhook validation",
            &SemanticOptions {
                enabled: true,
                limit: 5,
                provider: SemanticProviderConfig::default(),
            },
        )
        .unwrap();

        assert_eq!(report.results[0].path, "src/webhooks.ts");
        assert!(report.results.iter().all(|result| result.path != ".env"));
        assert_eq!(report.provider.provider, DEFAULT_SEMANTIC_PROVIDER);
        assert_eq!(report.provider.provider_role, LOCAL_HASH_PROVIDER_ROLE);
        assert!(!report.provider.quality_backend);
    }

    #[cfg(not(feature = "local-embeddings"))]
    #[test]
    fn semantic_provider_unavailable_without_feature() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn semantic_backend() {}\n").unwrap();

        let report = semantic_search_report(
            repo,
            "semantic backend",
            &SemanticOptions {
                enabled: true,
                limit: 5,
                provider: SemanticProviderConfig {
                    provider: LOCAL_FASTEMBED_PROVIDER.to_string(),
                    ..SemanticProviderConfig::default()
                },
            },
        )
        .unwrap();

        assert!(report.results.is_empty());
        assert_eq!(report.provider.provider, LOCAL_FASTEMBED_PROVIDER);
        assert_eq!(report.provider.model, LOCAL_FASTEMBED_MODEL);
        assert_eq!(report.provider.provider_role, LOCAL_FASTEMBED_PROVIDER_ROLE);
        assert!(report.provider.quality_backend);
        assert!(report.provider.local_only);
        assert!(!report.provider.available);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "semantic_provider_unavailable"));
        assert!(report.privacy_status.local_only);
        assert!(!report.privacy_status.remote_embeddings_used);
    }
}
