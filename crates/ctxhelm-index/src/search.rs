use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{
    canonicalize, inventory_content_fingerprint, inventory_path, FileInventoryEntry,
    InventoryError, InventoryOptions, RepoInventory,
};
use crate::policy::{read_safe_source, SourceReadStatus, SOURCE_READ_MAX_BYTES};
use crate::symbols::extract_symbols_report;
use ctxhelm_core::{CacheStatus, CacheStatusKind, Diagnostic, FileRole};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, TantivyDocument, Value, STORED, STRING, TEXT};
use tantivy::{doc, Index};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    pub limit: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self { limit: 10 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub path: String,
    pub role: FileRole,
    pub language: Option<String>,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchReport {
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

pub fn lexical_search(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>, InventoryError> {
    Ok(lexical_search_report(repo_root, query, options)?.results)
}

pub fn lexical_search_report(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SearchOptions,
) -> Result<SearchReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let query_terms = query_terms(query);
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    if query_terms.is_empty() {
        return Ok(SearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
        });
    }

    let cache_path = lexical_search_cache_path(
        &inventory_report.inventory,
        query,
        options.limit,
        &query_terms,
    );
    if let Ok(json) = fs::read_to_string(&cache_path) {
        if let Ok(mut cached) = serde_json::from_str::<SearchReport>(&json) {
            cached.cache_status = CacheStatus {
                status: CacheStatusKind::Hit,
                path: Some(cache_path.to_string_lossy().to_string()),
                diagnostics: Vec::new(),
            };
            return Ok(cached);
        }
    }

    let (mut results, bm25_diagnostics) = bm25_lexical_search(
        &repo_root,
        &inventory_report.inventory,
        &query_terms,
        options,
    )?;
    diagnostics.extend(bm25_diagnostics);
    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    results.truncate(options.limit.max(1));

    let report = SearchReport {
        results,
        diagnostics,
        cache_status,
    };
    let _ = persist_lexical_search_cache(&cache_path, &report);
    Ok(report)
}

pub fn legacy_lexical_search_report(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SearchOptions,
) -> Result<SearchReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let query_terms = query_terms(query);
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    if query_terms.is_empty() {
        return Ok(SearchReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
        });
    }

    let mut results = Vec::new();
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
        let content = source.text.unwrap_or_default();
        let Some((score, reason)) = score_file(file, &content, &query_terms) else {
            continue;
        };

        results.push(SearchResult {
            path: file.path.clone(),
            role: file.role.clone(),
            language: file.language.clone(),
            score,
            reason,
        });
    }

    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    results.truncate(options.limit.max(1));

    Ok(SearchReport {
        results,
        diagnostics,
        cache_status,
    })
}

fn persist_lexical_search_cache(path: &Path, report: &SearchReport) -> Result<(), InventoryError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string(report).map_err(InventoryError::Serialize)?;
    fs::write(path, json).map_err(|source| InventoryError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn lexical_search_cache_path(
    inventory: &RepoInventory,
    query: &str,
    limit: usize,
    query_terms: &[String],
) -> std::path::PathBuf {
    let mut key = blake3::Hasher::new();
    key.update(b"lexical-search-cache-v6");
    key.update(query.trim().as_bytes());
    key.update(&limit.max(1).to_le_bytes());
    for term in query_terms {
        key.update(term.as_bytes());
        key.update(b"\0");
    }
    key.update(inventory_content_fingerprint(inventory).as_bytes());
    let repo_cache_dir = inventory_path(&inventory.repo_id)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from(".ctxhelm"));
    repo_cache_dir
        .join("lexical-search")
        .join(format!("{}.json", key.finalize().to_hex()))
}

#[derive(Debug, Clone)]
struct IndexedFileEvidence {
    file: FileInventoryEntry,
    exact_score: f32,
    exact_reason: String,
}

#[derive(Debug, Clone)]
struct CachedIndexedDocument {
    file: FileInventoryEntry,
    filename: String,
    content: String,
}

#[derive(Debug, Clone, Copy)]
struct LexicalFields {
    path: tantivy::schema::Field,
    filename: tantivy::schema::Field,
    role: tantivy::schema::Field,
    language: tantivy::schema::Field,
    symbols: tantivy::schema::Field,
    content: tantivy::schema::Field,
}

struct CachedBm25Index {
    index: Index,
    fields: LexicalFields,
    documents: Vec<CachedIndexedDocument>,
    symbols_by_path: BTreeMap<String, Vec<String>>,
    diagnostics: Vec<Diagnostic>,
}

const EXACT_LEXICAL_SCORE_MULTIPLIER: f32 = 100.0;
const EXACT_LEXICAL_DOMINANT_MIN_RESULTS: usize = 5;
const BM25_INDEX_CACHE_LIMIT: usize = 2;

type Bm25IndexCacheEntry = (String, Arc<CachedBm25Index>);
type Bm25IndexCache = Mutex<VecDeque<Bm25IndexCacheEntry>>;

static BM25_INDEX_CACHE: OnceLock<Bm25IndexCache> = OnceLock::new();

fn bm25_lexical_search(
    repo_root: &Path,
    inventory: &RepoInventory,
    query_terms: &[String],
    options: &SearchOptions,
) -> Result<(Vec<SearchResult>, Vec<Diagnostic>), InventoryError> {
    let mut diagnostics = Vec::new();
    let weighted_query_terms = query_terms
        .iter()
        .filter(|term| query_term_weight(term) > 0.0)
        .cloned()
        .collect::<Vec<_>>();
    if weighted_query_terms.is_empty() {
        return Ok((Vec::new(), diagnostics));
    }
    let limit = options.limit.max(1);
    let index_cache_key = format!(
        "bm25-fielded-index-v1:{}",
        inventory_content_fingerprint(inventory)
    );
    if let Some(cached) = get_cached_bm25_index(&index_cache_key) {
        return search_cached_bm25_index(&cached, query_terms, &weighted_query_terms, limit)
            .map(|results| (results, cached.diagnostics.clone()));
    }

    let mut documents = Vec::new();
    for file in &inventory.files {
        if file.generated || file.role == FileRole::Sensitive || file.ignored {
            continue;
        }

        let source = read_safe_source(repo_root, inventory, &file.path, SOURCE_READ_MAX_BYTES)?;
        diagnostics.extend(source.diagnostics);
        let SourceReadStatus::Read = source.status else {
            continue;
        };
        let content = source.text.unwrap_or_default();
        let filename = file
            .path
            .rsplit('/')
            .next()
            .unwrap_or(file.path.as_str())
            .to_string();
        documents.push(CachedIndexedDocument {
            file: file.clone(),
            filename,
            content,
        });
    }

    let exact_evidence_by_path = exact_evidence_by_path(&documents, query_terms);
    if should_use_exact_primary_path(exact_evidence_by_path.len(), limit, &weighted_query_terms) {
        return Ok((
            exact_primary_results(
                &exact_evidence_by_path,
                limit,
                exact_primary_reason(exact_evidence_by_path.len(), limit, &weighted_query_terms),
            ),
            diagnostics,
        ));
    }

    let symbol_report = extract_symbols_report(repo_root)?;
    diagnostics.extend(symbol_report.diagnostics);
    let mut symbols_by_path: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for symbol in symbol_report.symbols {
        symbols_by_path
            .entry(symbol.path)
            .or_default()
            .push(symbol.name);
    }

    let mut schema_builder = Schema::builder();
    let fields = LexicalFields {
        path: schema_builder.add_text_field("path", TEXT | STORED),
        filename: schema_builder.add_text_field("filename", TEXT),
        role: schema_builder.add_text_field("role", STRING | STORED),
        language: schema_builder.add_text_field("language", STRING | STORED),
        symbols: schema_builder.add_text_field("symbols", TEXT),
        content: schema_builder.add_text_field("content", TEXT),
    };
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    let mut writer = index
        .writer(64_000_000)
        .map_err(|source| InventoryError::InvalidInput(format!("bm25 index writer: {source}")))?;
    for document in &documents {
        let symbols = symbols_by_path
            .get(&document.file.path)
            .map(|names| names.join(" "))
            .unwrap_or_default();
        writer
            .add_document(doc!(
                fields.path => document.file.path.as_str(),
                fields.filename => document.filename.as_str(),
                fields.role => format!("{:?}", document.file.role),
                fields.language => document.file.language.clone().unwrap_or_default(),
                fields.symbols => symbols,
                fields.content => document.content.as_str(),
            ))
            .map_err(|source| {
                InventoryError::InvalidInput(format!("bm25 add document: {source}"))
            })?;
    }

    writer
        .commit()
        .map_err(|source| InventoryError::InvalidInput(format!("bm25 commit: {source}")))?;
    let cached = Arc::new(CachedBm25Index {
        index,
        fields,
        documents,
        symbols_by_path,
        diagnostics: diagnostics.clone(),
    });
    remember_cached_bm25_index(index_cache_key, cached.clone());
    search_cached_bm25_index(&cached, query_terms, &weighted_query_terms, limit)
        .map(|results| (results, diagnostics))
}

fn get_cached_bm25_index(key: &str) -> Option<Arc<CachedBm25Index>> {
    let cache = BM25_INDEX_CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    let mut cache = cache.lock().ok()?;
    let position = cache.iter().position(|(cached_key, _)| cached_key == key)?;
    let entry = cache.remove(position)?;
    let index = entry.1.clone();
    cache.push_back(entry);
    Some(index)
}

fn remember_cached_bm25_index(key: String, index: Arc<CachedBm25Index>) {
    let cache = BM25_INDEX_CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    let Ok(mut cache) = cache.lock() else {
        return;
    };
    if let Some(position) = cache
        .iter()
        .position(|(cached_key, _)| cached_key == key.as_str())
    {
        cache.remove(position);
    }
    cache.push_back((key, index));
    while cache.len() > BM25_INDEX_CACHE_LIMIT {
        cache.pop_front();
    }
}

fn exact_evidence_by_path(
    documents: &[CachedIndexedDocument],
    query_terms: &[String],
) -> BTreeMap<String, IndexedFileEvidence> {
    documents
        .iter()
        .filter_map(|document| {
            let (exact_score, exact_reason) =
                score_file(&document.file, &document.content, query_terms)?;
            Some((
                document.file.path.clone(),
                IndexedFileEvidence {
                    file: document.file.clone(),
                    exact_score,
                    exact_reason,
                },
            ))
        })
        .collect()
}

fn indexed_evidence_by_path(
    documents: &[CachedIndexedDocument],
    query_terms: &[String],
) -> BTreeMap<String, IndexedFileEvidence> {
    documents
        .iter()
        .map(|document| {
            let (exact_score, exact_reason) =
                score_file(&document.file, &document.content, query_terms)
                    .unwrap_or_else(|| (0.0, "no exact field match".to_string()));
            (
                document.file.path.clone(),
                IndexedFileEvidence {
                    file: document.file.clone(),
                    exact_score,
                    exact_reason,
                },
            )
        })
        .collect()
}

fn should_use_exact_primary_path(
    exact_count: usize,
    limit: usize,
    weighted_query_terms: &[String],
) -> bool {
    exact_count >= limit
        || exact_count >= limit.min(EXACT_LEXICAL_DOMINANT_MIN_RESULTS)
        || (exact_count > 0 && is_single_identifier_query(weighted_query_terms))
}

fn exact_primary_reason(
    exact_count: usize,
    limit: usize,
    weighted_query_terms: &[String],
) -> &'static str {
    if exact_count >= limit {
        "exact lexical saturated budget"
    } else if is_single_identifier_query(weighted_query_terms) {
        "exact lexical identifier budget"
    } else {
        "exact lexical dominant budget"
    }
}

fn is_single_identifier_query(weighted_query_terms: &[String]) -> bool {
    weighted_query_terms.len() == 1
        && weighted_query_terms[0].len() >= 8
        && weighted_query_terms[0]
            .chars()
            .any(|character| character.is_ascii_alphabetic())
}

fn exact_primary_results(
    evidence_by_path: &BTreeMap<String, IndexedFileEvidence>,
    limit: usize,
    reason_label: &str,
) -> Vec<SearchResult> {
    let mut results = evidence_by_path
        .values()
        .map(|evidence| {
            let mut reasons = vec![reason_label.to_string(), evidence.exact_reason.clone()];
            reasons.sort();
            reasons.dedup();
            SearchResult {
                path: evidence.file.path.clone(),
                role: evidence.file.role.clone(),
                language: evidence.file.language.clone(),
                score: evidence.exact_score * EXACT_LEXICAL_SCORE_MULTIPLIER,
                reason: reasons.join("; "),
            }
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    results.truncate(limit);
    results
}

fn search_cached_bm25_index(
    cached: &CachedBm25Index,
    query_terms: &[String],
    weighted_query_terms: &[String],
    limit: usize,
) -> Result<Vec<SearchResult>, InventoryError> {
    let exact_evidence_by_path = exact_evidence_by_path(&cached.documents, query_terms);
    if should_use_exact_primary_path(exact_evidence_by_path.len(), limit, weighted_query_terms) {
        return Ok(exact_primary_results(
            &exact_evidence_by_path,
            limit,
            exact_primary_reason(exact_evidence_by_path.len(), limit, weighted_query_terms),
        ));
    }

    let evidence_by_path = indexed_evidence_by_path(&cached.documents, query_terms);
    let reader = cached
        .index
        .reader()
        .map_err(|source| InventoryError::InvalidInput(format!("bm25 reader: {source}")))?;
    let searcher = reader.searcher();
    let fields = cached.fields;
    let mut parser = QueryParser::for_index(
        &cached.index,
        vec![
            fields.path,
            fields.filename,
            fields.symbols,
            fields.content,
            fields.role,
            fields.language,
        ],
    );
    parser.set_field_boost(fields.path, 3.5);
    parser.set_field_boost(fields.filename, 4.0);
    parser.set_field_boost(fields.symbols, 6.0);
    parser.set_field_boost(fields.content, 1.0);
    parser.set_field_boost(fields.role, 0.4);
    parser.set_field_boost(fields.language, 0.3);

    let query_text = weighted_query_terms.join(" ");
    let query = parser
        .parse_query(&query_text)
        .map_err(|source| InventoryError::InvalidInput(format!("bm25 query parse: {source}")))?;
    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(limit.saturating_mul(8)))
        .map_err(|source| InventoryError::InvalidInput(format!("bm25 search: {source}")))?;

    let mut results = Vec::new();
    let mut seen_paths = BTreeSet::new();
    for (bm25_score, doc_address) in top_docs {
        let document = searcher
            .doc::<TantivyDocument>(doc_address)
            .map_err(|source| {
                InventoryError::InvalidInput(format!("bm25 document read: {source}"))
            })?;
        let Some(path) = document
            .get_first(fields.path)
            .and_then(|value| value.as_str())
            .map(str::to_string)
        else {
            continue;
        };
        let Some(evidence) = evidence_by_path.get(&path) else {
            continue;
        };
        let mut score = if evidence.exact_score > 0.0 {
            evidence.exact_score * EXACT_LEXICAL_SCORE_MULTIPLIER
        } else {
            bm25_score
        };
        let mut reasons = vec![format!("bm25 fielded score {bm25_score:.3}")];
        if evidence.exact_score > 0.0 {
            reasons.push(evidence.exact_reason.clone());
        }
        if cached.symbols_by_path.contains_key(&path) {
            reasons.push("exact symbol index available".to_string());
        }
        if is_archive_context_artifact(&path.to_ascii_lowercase()) {
            score *= 0.35;
            reasons.push("archive context artifact dampened".to_string());
        }
        reasons.sort();
        reasons.dedup();
        results.push(SearchResult {
            path: path.clone(),
            role: evidence.file.role.clone(),
            language: evidence.file.language.clone(),
            score,
            reason: reasons.join("; "),
        });
        seen_paths.insert(path);
    }

    for (path, evidence) in &evidence_by_path {
        if seen_paths.contains(path) || evidence.exact_score <= 0.0 {
            continue;
        }
        let mut score = evidence.exact_score * EXACT_LEXICAL_SCORE_MULTIPLIER;
        let mut reasons = vec![
            "exact lexical reserve".to_string(),
            evidence.exact_reason.clone(),
        ];
        if cached.symbols_by_path.contains_key(path) {
            reasons.push("exact symbol index available".to_string());
        }
        if is_archive_context_artifact(&path.to_ascii_lowercase()) {
            score *= 0.35;
            reasons.push("archive context artifact dampened".to_string());
        }
        reasons.sort();
        reasons.dedup();
        results.push(SearchResult {
            path: path.clone(),
            role: evidence.file.role.clone(),
            language: evidence.file.language.clone(),
            score,
            reason: reasons.join("; "),
        });
    }
    Ok(results)
}

pub(crate) fn query_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|term| !term.is_empty())
        .map(|term| term.to_ascii_lowercase())
        .collect()
}

fn score_file(
    file: &FileInventoryEntry,
    content: &str,
    query_terms: &[String],
) -> Option<(f32, String)> {
    let path = file.path.to_ascii_lowercase();
    let file_name = path.rsplit('/').next().unwrap_or(path.as_str());
    let content = content.to_ascii_lowercase();
    let mut score = 0.0;
    let mut reasons = Vec::new();

    for term in query_terms {
        let weight = query_term_weight(term);
        if weight == 0.0 {
            continue;
        }
        let mut matched = false;

        if path.contains(term) {
            score += 8.0 * weight;
            matched = true;
            reasons.push(format!("path matched `{term}`"));
        }
        if file_name.contains(term) {
            score += 5.0 * weight;
            matched = true;
            reasons.push(format!("file name matched `{term}`"));
        }

        let occurrences = count_occurrences(&content, term);
        if occurrences > 0 {
            score += (2.0 + occurrences.min(10) as f32) * weight;
            matched = true;
            reasons.push(format!("content matched `{term}` {occurrences} time(s)"));
        }

        if !matched {
            score -= 1.0 * weight;
        }
    }

    if score <= 0.0 {
        return None;
    }

    score += match file.role {
        FileRole::Source => 1.0,
        FileRole::Test => 0.7,
        FileRole::Config | FileRole::Schema | FileRole::Docs => 0.4,
        FileRole::Generated | FileRole::Sensitive | FileRole::Unknown => 0.0,
    };
    if is_archive_context_artifact(&path) {
        score *= 0.35;
        reasons.push("archive context artifact dampened".to_string());
    }

    reasons.sort();
    reasons.dedup();

    Some((score, reasons.join("; ")))
}

fn is_archive_context_artifact(path: &str) -> bool {
    path.starts_with(".planning/milestones/")
        || (path.starts_with(".planning/e2e/") && path.ends_with(".json"))
}

pub(crate) fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

pub(crate) fn query_term_weight(term: &str) -> f32 {
    if matches!(
        term,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "be"
            | "by"
            | "change"
            | "changed"
            | "changes"
            | "default"
            | "fix"
            | "fixed"
            | "fixes"
            | "for"
            | "from"
            | "handle"
            | "improve"
            | "improved"
            | "improvement"
            | "improvements"
            | "in"
            | "is"
            | "of"
            | "on"
            | "or"
            | "support"
            | "supports"
            | "the"
            | "to"
            | "with"
    ) {
        return 0.0;
    }

    if matches!(
        term,
        "csharp"
            | "go"
            | "java"
            | "javascript"
            | "js"
            | "kotlin"
            | "python"
            | "rust"
            | "scala"
            | "typescript"
            | "ts"
    ) {
        return 0.25;
    }

    if matches!(term, "node" | "nodes") {
        return 0.10;
    }

    1.0
}
