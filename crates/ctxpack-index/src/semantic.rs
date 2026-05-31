use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{canonicalize, FileInventoryEntry, InventoryError, InventoryOptions};
use crate::search::{query_term_weight, query_terms};
use crate::storage::{
    persist_semantic_vector_records, sync_inventory_to_store, StorageError,
    StorageSemanticIndexReport, StorageSemanticVectorRecord, StoreConfig,
};
use crate::{
    dependency_edges_report, extract_symbols_report, precision_edges_path, related_tests_report,
    DependencyEdge, DependencyOptions, PrecisionEdgesFile,
};
use ctxpack_core::{
    CacheStatus, Diagnostic, DiagnosticSeverity, FileRole, LineRange, PrecisionStatus,
    PrecisionStatusReport, PrivacyStatus, SemanticDocument, SemanticDocumentFacet,
    SemanticDocumentFacetKind, SemanticDocumentReport,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
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
#[cfg(feature = "local-embeddings")]
const LOCAL_FASTEMBED_VECTOR_CACHE_LIMIT: usize = 10_000;
const DEFAULT_LOCAL_FASTEMBED_DOCUMENT_PREFILTER_LIMIT: usize = 128;

#[cfg(feature = "local-embeddings")]
struct CachedFastembedModel {
    model_id: String,
    model: fastembed::TextEmbedding,
}

#[cfg(feature = "local-embeddings")]
thread_local! {
    static LOCAL_FASTEMBED_MODEL_CACHE: std::cell::RefCell<Option<CachedFastembedModel>> = const { std::cell::RefCell::new(None) };
    static LOCAL_FASTEMBED_VECTOR_CACHE: std::cell::RefCell<BTreeMap<String, Vec<f32>>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDocumentOptions {
    pub limit: usize,
}

impl Default for SemanticDocumentOptions {
    fn default() -> Self {
        Self { limit: 500 }
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
    pub document_id: Option<String>,
    #[serde(default)]
    pub matched_facets: Vec<SemanticDocumentFacet>,
    pub precision_status: Option<PrecisionStatus>,
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
    pub document_id: Option<String>,
    #[serde(default)]
    pub facet_count: usize,
}

pub fn semantic_document_report(
    repo_root: impl AsRef<Path>,
    options: &SemanticDocumentOptions,
) -> Result<SemanticDocumentReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    let inventory = inventory_report.inventory;

    let symbol_report = extract_symbols_report(&repo_root)?;
    diagnostics.extend(symbol_report.diagnostics);
    let mut symbols_by_path = BTreeMap::<String, Vec<_>>::new();
    for symbol in symbol_report.symbols {
        symbols_by_path
            .entry(symbol.path.clone())
            .or_default()
            .push(symbol);
    }

    let dependency_report =
        dependency_edges_report(&repo_root, &DependencyOptions { limit: usize::MAX })?;
    let precision_status = precision_status_from_dependencies(
        &repo_root,
        &dependency_report.edges,
        &dependency_report.diagnostics,
    );
    diagnostics.extend(dependency_report.diagnostics.clone());
    let mut edges_by_path = BTreeMap::<String, Vec<DependencyEdge>>::new();
    for edge in dependency_report.edges {
        edges_by_path
            .entry(edge.source_path.clone())
            .or_default()
            .push(edge.clone());
        edges_by_path
            .entry(edge.target_path.clone())
            .or_default()
            .push(edge);
    }

    let mut documents = Vec::new();
    for file in inventory
        .files
        .iter()
        .filter(|file| semantic_document_file(file))
    {
        let mut facets = Vec::new();
        facets.push(SemanticDocumentFacet {
            kind: SemanticDocumentFacetKind::Metadata,
            label: "role".to_string(),
            value: format!("{:?}", file.role).to_ascii_lowercase(),
            path: None,
            line_range: None,
            weight: 0.4,
        });
        if let Some(language) = &file.language {
            facets.push(SemanticDocumentFacet {
                kind: SemanticDocumentFacetKind::Metadata,
                label: "language".to_string(),
                value: language.clone(),
                path: None,
                line_range: None,
                weight: 0.4,
            });
        }

        for symbol in symbols_by_path
            .get(&file.path)
            .into_iter()
            .flatten()
            .take(12)
        {
            let value = safe_symbol_signature(&symbol.name, &symbol.signature);
            facets.push(SemanticDocumentFacet {
                kind: SemanticDocumentFacetKind::Symbol,
                label: format!("{:?}", symbol.kind).to_ascii_lowercase(),
                value,
                path: Some(symbol.path.clone()),
                line_range: Some(LineRange {
                    start: symbol.start_line,
                    end: symbol.end_line.max(symbol.start_line),
                }),
                weight: if symbol.exported { 1.0 } else { 0.8 },
            });
        }

        for edge in edges_by_path.get(&file.path).into_iter().flatten().take(12) {
            let other_path = if edge.source_path == file.path {
                edge.target_path.clone()
            } else {
                edge.source_path.clone()
            };
            let is_precision = edge.kind.starts_with("precision:");
            facets.push(SemanticDocumentFacet {
                kind: if is_precision {
                    SemanticDocumentFacetKind::Precision
                } else {
                    SemanticDocumentFacetKind::Dependency
                },
                label: edge.kind.clone(),
                value: edge.reason.clone(),
                path: Some(other_path),
                line_range: None,
                weight: edge.confidence.clamp(0.0, 1.0),
            });
        }

        if matches!(file.role, FileRole::Source) {
            match related_tests_report(&repo_root, std::slice::from_ref(&file.path)) {
                Ok(test_report) => {
                    diagnostics.extend(test_report.diagnostics);
                    for test in test_report.results.into_iter().take(3) {
                        facets.push(SemanticDocumentFacet {
                            kind: SemanticDocumentFacetKind::RelatedTest,
                            label: "related_test".to_string(),
                            value: test.reason,
                            path: Some(test.path),
                            line_range: None,
                            weight: test.confidence.clamp(0.0, 1.0),
                        });
                    }
                }
                Err(error) => diagnostics.push(Diagnostic {
                    code: "semantic_document_tests_degraded".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: format!(
                        "Semantic document related-test enrichment failed for {}: {}",
                        file.path, error
                    ),
                    paths: vec![file.path.clone()],
                    count: 1,
                }),
            }
        }

        if matches!(file.role, FileRole::Docs) {
            facets.push(SemanticDocumentFacet {
                kind: SemanticDocumentFacetKind::Doc,
                label: "doc_file".to_string(),
                value: file.path.clone(),
                path: Some(file.path.clone()),
                line_range: None,
                weight: 0.7,
            });
        }

        documents.push(SemanticDocument {
            id: semantic_document_id(&file.path, &file.hash),
            path: file.path.clone(),
            role: file.role.clone(),
            language: file.language.clone(),
            safe_hash: file.hash.clone(),
            summary: semantic_document_summary(file, facets.len()),
            facets,
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        });
    }

    documents.sort_by(|left, right| left.path.cmp(&right.path));
    documents.truncate(options.limit.max(1));
    let facet_count = documents.iter().map(|document| document.facets.len()).sum();

    Ok(SemanticDocumentReport {
        document_count: documents.len(),
        facet_count,
        documents,
        diagnostics,
        cache_status,
        precision_status,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
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

    let document_report =
        semantic_document_report(&repo_root, &SemanticDocumentOptions { limit: usize::MAX })?;
    diagnostics.extend(document_report.diagnostics);
    let original_document_count = document_report.documents.len();
    let candidate_documents =
        prefilter_semantic_documents(query, document_report.documents, &provider);
    if candidate_documents.len() < original_document_count {
        diagnostics.push(Diagnostic {
            code: "semantic_document_prefiltered".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Local production semantic retrieval bounded source-free candidate documents from {} to {} before embedding.",
                original_document_count,
                candidate_documents.len()
            ),
            paths: Vec::new(),
            count: original_document_count - candidate_documents.len(),
        });
    }
    let candidate_texts = candidate_documents
        .iter()
        .map(render_semantic_document_text)
        .collect::<Vec<_>>();

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
    for (document, file_vector) in candidate_documents.into_iter().zip(file_vectors.iter()) {
        let score = cosine_similarity(query_vector, file_vector);
        if score < 0.08 {
            continue;
        }
        let matched_facets = matched_semantic_facets(query, &document);
        results.push(SemanticSearchResult {
            path: document.path.clone(),
            role: document.role.clone(),
            language: document.language.clone(),
            score,
            reason: semantic_match_reason(&provider, &matched_facets),
            provider: provider.clone(),
            document_id: Some(document.id),
            matched_facets,
            precision_status: Some(document_report.precision_status.status.clone()),
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
    if !options.enabled || !provider.available {
        return Ok(Vec::new());
    }
    let document_report =
        semantic_document_report(&repo_root, &SemanticDocumentOptions { limit: usize::MAX })?;
    let inputs = document_report
        .documents
        .iter()
        .map(render_semantic_document_text)
        .collect::<Vec<_>>();
    let vectors = embed_texts(&inputs, &provider).unwrap_or_default();
    let mut records = Vec::new();
    for (document, vector) in document_report.documents.into_iter().zip(vectors) {
        records.push(SemanticVectorRecord {
            path: document.path,
            role: document.role,
            language: document.language,
            safe_hash: document.safe_hash,
            vector,
            provider: provider.clone(),
            privacy_status: PrivacyStatus::local_only(),
            document_id: Some(document.id),
            facet_count: document.facets.len(),
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
        source: std::io::Error::other(error.to_string()),
    }
}

fn semantic_document_file(file: &FileInventoryEntry) -> bool {
    !file.generated
        && !file.ignored
        && matches!(
            file.role,
            FileRole::Source
                | FileRole::Test
                | FileRole::Config
                | FileRole::Schema
                | FileRole::Docs
        )
}

fn precision_status_from_dependencies(
    repo_root: &Path,
    edges: &[DependencyEdge],
    diagnostics: &[Diagnostic],
) -> PrecisionStatusReport {
    let overlay_path = precision_edges_path(repo_root).ok();
    let overlay_path_string = overlay_path.as_ref().map(|path| path.display().to_string());
    let precision_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.starts_with("precision_edges_"))
        .cloned()
        .collect::<Vec<_>>();
    let rejected_edge_count = precision_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "precision_edges_rejected")
        .map(|diagnostic| diagnostic.count)
        .sum::<usize>();
    let edge_count = edges
        .iter()
        .filter(|edge| edge.kind.starts_with("precision:"))
        .count();

    let Some(path) = overlay_path else {
        return PrecisionStatusReport {
            status: PrecisionStatus::Unavailable,
            provider: None,
            overlay_path: None,
            edge_count,
            rejected_edge_count,
            stale: false,
            degraded: false,
            diagnostics: precision_diagnostics,
        };
    };

    if !path.exists() {
        return PrecisionStatusReport {
            status: PrecisionStatus::Unavailable,
            provider: None,
            overlay_path: overlay_path_string,
            edge_count,
            rejected_edge_count,
            stale: false,
            degraded: false,
            diagnostics: precision_diagnostics,
        };
    }

    let parsed = fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<PrecisionEdgesFile>(&raw).ok());
    let provider = parsed.as_ref().map(|file| file.provider.clone());
    let invalid = parsed.is_none()
        || precision_diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.code.as_str(),
                "precision_edges_invalid" | "precision_edges_unreadable"
            )
        });
    let degraded = rejected_edge_count > 0;
    let status = if invalid {
        PrecisionStatus::Invalid
    } else if degraded {
        PrecisionStatus::Degraded
    } else {
        PrecisionStatus::Available
    };

    PrecisionStatusReport {
        status,
        provider,
        overlay_path: overlay_path_string,
        edge_count,
        rejected_edge_count,
        stale: false,
        degraded,
        diagnostics: precision_diagnostics,
    }
}

fn semantic_document_id(path: &str, safe_hash: &str) -> String {
    let digest = blake3::hash(format!("{path}:{safe_hash}").as_bytes());
    format!("sem_doc_{}", digest.to_hex())
}

fn semantic_document_summary(file: &FileInventoryEntry, facet_count: usize) -> String {
    let language = file.language.as_deref().unwrap_or("unknown");
    format!(
        "{:?} file `{}` in {language} with {facet_count} source-free semantic facet(s)",
        file.role, file.path
    )
}

fn safe_symbol_signature(name: &str, signature: &str) -> String {
    let mut value = signature.trim().to_string();
    for marker in ["{", "=>", "="] {
        if let Some(index) = value.find(marker) {
            value.truncate(index);
        }
    }
    value = value.trim().trim_end_matches(';').trim().to_string();
    if value.is_empty() {
        value = name.to_string();
    }
    if value.len() > 180 {
        value.truncate(180);
    }
    value
}

fn render_semantic_document_text(document: &SemanticDocument) -> String {
    let mut lines = vec![
        format!("path {}", document.path),
        format!("role {:?}", document.role),
        document.summary.clone(),
    ];
    if let Some(language) = &document.language {
        lines.push(format!("language {language}"));
    }
    for facet in &document.facets {
        let path = facet
            .path
            .as_ref()
            .map(|path| format!(" path {path}"))
            .unwrap_or_default();
        lines.push(format!(
            "facet {:?} {} {}{}",
            facet.kind, facet.label, facet.value, path
        ));
    }
    lines.join("\n")
}

fn prefilter_semantic_documents(
    query: &str,
    mut documents: Vec<SemanticDocument>,
    provider: &SemanticProviderConfig,
) -> Vec<SemanticDocument> {
    if provider.provider != LOCAL_FASTEMBED_PROVIDER
        || documents.len() <= local_fastembed_document_prefilter_limit()
    {
        return documents;
    }
    let terms = query_terms(query);
    documents.sort_by(|left, right| {
        semantic_prefilter_score(right, &terms)
            .total_cmp(&semantic_prefilter_score(left, &terms))
            .then_with(|| left.path.cmp(&right.path))
    });
    documents.truncate(local_fastembed_document_prefilter_limit());
    documents
}

fn local_fastembed_document_prefilter_limit() -> usize {
    std::env::var("CTXPACK_FASTEMBED_DOCUMENT_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(DEFAULT_LOCAL_FASTEMBED_DOCUMENT_PREFILTER_LIMIT)
}

fn semantic_prefilter_score(document: &SemanticDocument, terms: &[String]) -> f32 {
    if terms.is_empty() {
        return 0.0;
    }
    let path = document.path.to_ascii_lowercase();
    let summary = document.summary.to_ascii_lowercase();
    let mut score = 0.0;
    for term in terms {
        if path.contains(term) {
            score += 3.0;
        }
        if summary.contains(term) {
            score += 0.5;
        }
        for facet in &document.facets {
            let facet_path = facet.path.as_deref().unwrap_or("");
            let text =
                format!("{} {} {}", facet.label, facet.value, facet_path).to_ascii_lowercase();
            if text.contains(term) {
                score += facet.weight.max(0.1);
            }
        }
    }
    score
}

fn matched_semantic_facets(query: &str, document: &SemanticDocument) -> Vec<SemanticDocumentFacet> {
    let terms = query_terms(query).into_iter().collect::<BTreeSet<_>>();
    let mut matched = document
        .facets
        .iter()
        .filter(|facet| {
            let text = format!(
                "{} {} {}",
                facet.label,
                facet.value,
                facet.path.as_deref().unwrap_or("")
            )
            .to_ascii_lowercase();
            terms.iter().any(|term| text.contains(term))
        })
        .cloned()
        .collect::<Vec<_>>();
    matched.sort_by(|left, right| {
        right
            .weight
            .total_cmp(&left.weight)
            .then_with(|| left.label.cmp(&right.label))
    });
    if matched.is_empty() {
        matched = document.facets.iter().take(3).cloned().collect();
    }
    matched.truncate(5);
    matched
}

fn semantic_match_reason(
    provider: &SemanticProviderConfig,
    facets: &[SemanticDocumentFacet],
) -> String {
    let facet_summary = if facets.is_empty() {
        "metadata facets".to_string()
    } else {
        facets
            .iter()
            .map(|facet| facet.label.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .take(4)
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "local semantic similarity via {} {} over source-free facets: {}",
        provider.provider, provider.model, facet_summary
    )
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
        let model = if provider.model.trim().is_empty() || provider.model == DEFAULT_SEMANTIC_MODEL
        {
            LOCAL_FASTEMBED_MODEL.to_string()
        } else {
            provider.model.trim().to_string()
        };
        return SemanticProviderConfig {
            provider: provider_id,
            dimensions: if provider.dimensions == DEFAULT_SEMANTIC_DIMENSIONS {
                local_fastembed_model_dimensions(&model)
            } else {
                provider.dimensions.clamp(8, 4096)
            },
            model,
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
            dimensions: if provider.dimensions == DEFAULT_SEMANTIC_DIMENSIONS {
                DEFAULT_SEMANTIC_DIMENSIONS
            } else {
                provider.dimensions.clamp(8, 4096)
            },
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

fn local_fastembed_model_dimensions(model: &str) -> usize {
    match model {
        "AllMiniLML6V2" | "AllMiniLML6V2Q" | "AllMiniLML12V2" | "AllMiniLML12V2Q" => 384,
        _ => LOCAL_FASTEMBED_DIMENSIONS,
    }
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

    let keys = texts
        .iter()
        .map(|text| local_fastembed_vector_cache_key(provider, text))
        .collect::<Vec<_>>();
    let mut vectors = LOCAL_FASTEMBED_VECTOR_CACHE.with(|cache| {
        let cache = cache.borrow();
        keys.iter()
            .map(|key| cache.get(key).cloned())
            .collect::<Vec<_>>()
    });
    let mut missing_texts = Vec::new();
    let mut missing_keys = Vec::new();
    for (index, (key, vector)) in keys.iter().zip(vectors.iter()).enumerate() {
        if vector.is_none() {
            missing_texts.push(texts[index].clone());
            missing_keys.push((index, key.clone()));
        }
    }

    if !missing_texts.is_empty() {
        let embedded_vectors = LOCAL_FASTEMBED_MODEL_CACHE.with(|cache| {
            let mut cached = cache.borrow_mut();
            let cache_miss = cached
                .as_ref()
                .is_none_or(|cached| cached.model_id != provider.model);
            if cache_miss {
                let cache_dir = local_fastembed_model_cache_dir();
                fs::create_dir_all(&cache_dir).map_err(|error| {
                    format!(
                        "Semantic provider {} could not create local model cache {}: {}",
                        provider.provider,
                        cache_dir.display(),
                        error
                    )
                })?;
                let model = TextEmbedding::try_new(
                    InitOptions::new(embedding_model)
                        .with_cache_dir(cache_dir)
                        .with_show_download_progress(false),
                )
                .map_err(|error| {
                    format!(
                        "Semantic provider {} failed to initialize local model {}: {}",
                        provider.provider, provider.model, error
                    )
                })?;
                *cached = Some(CachedFastembedModel {
                    model_id: provider.model.clone(),
                    model,
                });
            }
            let model = &cached
                .as_ref()
                .expect("local fastembed model cache initialized")
                .model;
            model.embed(missing_texts, None).map_err(|error| {
                format!(
                    "Semantic provider {} failed to embed with local model {}: {}",
                    provider.provider, provider.model, error
                )
            })
        })?;
        if embedded_vectors.len() != missing_keys.len() {
            return Err(format!(
                "Semantic provider {} returned {} vectors for {} inputs.",
                provider.provider,
                embedded_vectors.len(),
                missing_keys.len()
            ));
        }
        LOCAL_FASTEMBED_VECTOR_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            for ((index, key), vector) in missing_keys.into_iter().zip(embedded_vectors) {
                vectors[index] = Some(vector.clone());
                cache.insert(key, vector);
            }
            while cache.len() > LOCAL_FASTEMBED_VECTOR_CACHE_LIMIT {
                let Some(first_key) = cache.keys().next().cloned() else {
                    break;
                };
                cache.remove(&first_key);
            }
        });
    }

    vectors
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            format!(
                "Semantic provider {} failed to resolve all local embedding vectors.",
                provider.provider
            )
        })
}

#[cfg(feature = "local-embeddings")]
fn local_fastembed_vector_cache_key(provider: &SemanticProviderConfig, text: &str) -> String {
    let digest = blake3::hash(text.as_bytes());
    format!(
        "{}:{}:{}:{}",
        provider.provider,
        provider.model,
        provider.dimensions,
        digest.to_hex()
    )
}

#[cfg(feature = "local-embeddings")]
fn local_fastembed_model_cache_dir() -> std::path::PathBuf {
    std::env::var("CTXPACK_FASTEMBED_CACHE_DIR")
        .or_else(|_| std::env::var("FASTEMBED_CACHE_DIR"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| nearest_git_root(&cwd))
                .map(|root| root.join(".ctxpack").join("cache").join("fastembed"))
                .unwrap_or_else(|| {
                    crate::inventory::ctxpack_home()
                        .join("cache")
                        .join("fastembed")
                })
        })
}

#[cfg(feature = "local-embeddings")]
fn nearest_git_root(start: &Path) -> Option<std::path::PathBuf> {
    start
        .ancestors()
        .find(|path| path.join(".git").exists())
        .map(Path::to_path_buf)
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
    fn semantic_search_finds_structural_safe_files() {
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
            "verifyStripeSignature webhooks",
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
        assert!(report.results[0].document_id.is_some());
        assert!(!report.results[0].matched_facets.is_empty());
    }

    #[test]
    fn local_fastembed_provider_resolves_model_dimensions_and_role() {
        let provider = normalized_provider(&SemanticProviderConfig {
            provider: LOCAL_FASTEMBED_PROVIDER.to_string(),
            model: "AllMiniLML6V2Q".to_string(),
            ..SemanticProviderConfig::default()
        });

        assert_eq!(provider.provider, LOCAL_FASTEMBED_PROVIDER);
        assert_eq!(provider.model, "AllMiniLML6V2Q");
        assert_eq!(provider.dimensions, 384);
        assert_eq!(provider.provider_role, LOCAL_FASTEMBED_PROVIDER_ROLE);
        assert!(provider.quality_backend);
        assert!(provider.local_only);
    }

    #[test]
    fn semantic_documents_are_source_free_and_precision_enriched() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::create_dir_all(repo.join(".ctxpack")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function getSession() { return 'BODY_LITERAL_SHOULD_NOT_LEAK'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "test('getSession', () => getSession());\n",
        )
        .unwrap();
        fs::write(
            repo.join(".ctxpack/precision-edges.json"),
            r#"{
  "schemaVersion": 1,
  "provider": "fixture_scip",
  "edges": [{
    "sourcePath": "src/auth/session.ts",
    "targetPath": "tests/auth/session.test.ts",
    "edgeType": "references",
    "symbol": "getSession",
    "confidence": 0.98
  }]
}
"#,
        )
        .unwrap();

        let report =
            semantic_document_report(repo, &SemanticDocumentOptions { limit: 20 }).unwrap();
        let document = report
            .documents
            .iter()
            .find(|document| document.path == "src/auth/session.ts")
            .expect("source semantic document");
        let json = serde_json::to_string(&report).unwrap();

        assert!(!report.source_text_logged);
        assert!(report.privacy_status.local_only);
        assert_eq!(report.precision_status.status, PrecisionStatus::Available);
        assert_eq!(
            report.precision_status.provider.as_deref(),
            Some("fixture_scip")
        );
        assert_eq!(report.precision_status.edge_count, 1);
        assert!(document
            .facets
            .iter()
            .any(|facet| facet.kind == SemanticDocumentFacetKind::Symbol));
        assert!(document
            .facets
            .iter()
            .any(|facet| facet.kind == SemanticDocumentFacetKind::RelatedTest));
        assert!(document
            .facets
            .iter()
            .any(|facet| facet.kind == SemanticDocumentFacetKind::Precision));
        assert!(!json.contains("BODY_LITERAL_SHOULD_NOT_LEAK"));
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
