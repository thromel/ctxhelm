use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{
    canonicalize, inventory_path, FileInventoryEntry, InventoryError, InventoryOptions,
    RepoInventory,
};
use crate::policy::{read_safe_source, SourceReadStatus, SOURCE_READ_MAX_BYTES};
use ctxpack_core::{CacheStatus, CacheStatusKind, Diagnostic, FileRole};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
        let Some((score, reason)) = score_file(&file, &content, &query_terms) else {
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

    let report = SearchReport {
        results,
        diagnostics,
        cache_status,
    };
    let _ = persist_lexical_search_cache(&cache_path, &report);
    Ok(report)
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
    key.update(b"lexical-search-cache-v1");
    key.update(query.trim().as_bytes());
    key.update(&limit.max(1).to_le_bytes());
    for term in query_terms {
        key.update(term.as_bytes());
        key.update(b"\0");
    }
    for file in &inventory.files {
        key.update(file.path.as_bytes());
        key.update(b"\0");
        key.update(file.hash.as_bytes());
        key.update(b"\0");
        key.update(format!("{:?}", file.role).as_bytes());
        key.update(b"\0");
    }
    let repo_cache_dir = inventory_path(&inventory.repo_id)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from(".ctxpack"));
    repo_cache_dir
        .join("lexical-search")
        .join(format!("{}.json", key.finalize().to_hex()))
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
