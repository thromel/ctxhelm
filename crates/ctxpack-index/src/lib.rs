use ctxpack_core::{EvalTrace, FileRole};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("failed to canonicalize {path}: {source}")]
    Canonicalize { path: PathBuf, source: io::Error },
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("failed to create directory {path}: {source}")]
    CreateDir { path: PathBuf, source: io::Error },
    #[error("failed to serialize inventory: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize inventory {path}: {source}")]
    Deserialize {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("git command failed in {repo_root}: {message}")]
    Git { repo_root: PathBuf, message: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryOptions {
    pub include_generated: bool,
    pub include_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileInventoryEntry {
    pub path: String,
    pub language: Option<String>,
    pub role: FileRole,
    pub hash: String,
    pub size_bytes: u64,
    pub generated: bool,
    pub ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepoInventory {
    pub repo_id: String,
    pub repo_root: PathBuf,
    pub files: Vec<FileInventoryEntry>,
    pub ignored_count: usize,
    pub generated_count: usize,
    pub sensitive_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryReport {
    pub repo_id: String,
    pub repo_root: PathBuf,
    pub inventory_path: PathBuf,
    pub file_count: usize,
    pub ignored_count: usize,
    pub generated_count: usize,
    pub sensitive_count: usize,
}

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
pub struct RelatedTestResult {
    pub path: String,
    pub command: Option<String>,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoChangeOptions {
    pub limit: usize,
}

impl Default for CoChangeOptions {
    fn default() -> Self {
        Self { limit: 10 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoChangeHint {
    pub path: String,
    pub commit_count: usize,
    pub confidence: f32,
    pub sample_commits: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependencyOptions {
    pub limit: usize,
}

impl Default for DependencyOptions {
    fn default() -> Self {
        Self { limit: 50 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEdge {
    pub source_path: String,
    pub target_path: String,
    pub kind: String,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolOptions {
    pub limit: usize,
}

impl Default for SymbolOptions {
    fn default() -> Self {
        Self { limit: 20 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Type,
    Constant,
    Import,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub path: String,
    pub language: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
    pub signature: String,
    pub exported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchResult {
    pub symbol: CodeSymbol,
    pub score: f32,
    pub reason: String,
}

pub fn build_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<RepoInventory, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let mut files = Vec::new();
    let mut generated_count = 0;
    let mut sensitive_count = 0;

    let mut walker = WalkBuilder::new(&repo_root);
    walker
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .parents(true)
        .ignore(false)
        .add_custom_ignore_filename(".ctxpackignore")
        .add_custom_ignore_filename(".cursorignore");

    for result in walker.build() {
        let Ok(entry) = result else {
            continue;
        };
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }

        let path = entry.path();
        let relative = normalize_relative_path(&repo_root, path);
        if relative == ".git" || relative.starts_with(".git/") {
            continue;
        }

        let role = classify_path(&relative);
        let generated = role == FileRole::Generated;
        let sensitive = role == FileRole::Sensitive;

        if generated {
            generated_count += 1;
        }
        if sensitive {
            sensitive_count += 1;
        }
        if (generated && !options.include_generated) || (sensitive && !options.include_sensitive) {
            continue;
        }

        let metadata = fs::metadata(path).map_err(|source| InventoryError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let bytes = fs::read(path).map_err(|source| InventoryError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        files.push(FileInventoryEntry {
            path: relative.clone(),
            language: language_for_path(&relative).map(str::to_string),
            role,
            hash: blake3::hash(&bytes).to_hex().to_string(),
            size_bytes: metadata.len(),
            generated,
            ignored: false,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(RepoInventory {
        repo_id: repo_id_for_path(&repo_root),
        repo_root,
        files,
        ignored_count: 0,
        generated_count,
        sensitive_count,
    })
}

pub fn write_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<InventoryReport, InventoryError> {
    let inventory = build_inventory(repo_root, options)?;
    let inventory_path = inventory_path(&inventory.repo_id);
    if let Some(parent) = inventory_path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let json = serde_json::to_string_pretty(&inventory).map_err(InventoryError::Serialize)?;
    fs::write(&inventory_path, json).map_err(|source| InventoryError::Write {
        path: inventory_path.clone(),
        source,
    })?;

    Ok(InventoryReport {
        repo_id: inventory.repo_id,
        repo_root: inventory.repo_root,
        inventory_path,
        file_count: inventory.files.len(),
        ignored_count: inventory.ignored_count,
        generated_count: inventory.generated_count,
        sensitive_count: inventory.sensitive_count,
    })
}

pub fn load_inventory(repo_id: &str) -> Result<RepoInventory, InventoryError> {
    let path = inventory_path(repo_id);
    let json = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&json).map_err(|source| InventoryError::Deserialize { path, source })
}

pub fn load_or_build_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<RepoInventory, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    match load_inventory(&repo_id) {
        Ok(inventory) => Ok(inventory),
        Err(InventoryError::Read { .. }) => {
            write_inventory(&repo_root, options)?;
            load_inventory(&repo_id)
        }
        Err(error) => Err(error),
    }
}

pub fn extract_symbols(repo_root: impl AsRef<Path>) -> Result<Vec<CodeSymbol>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory = load_or_build_inventory(&repo_root, &InventoryOptions::default())?;
    let mut symbols = Vec::new();

    for file in inventory.files {
        if file.generated || file.role == FileRole::Sensitive || file.ignored {
            continue;
        }
        if file.language.is_none() {
            continue;
        }

        let content = fs::read_to_string(repo_root.join(&file.path)).unwrap_or_default();
        symbols.extend(symbols_for_file(&file, &content));
    }

    symbols.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.start_line.cmp(&right.start_line))
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(symbols)
}

pub fn symbol_search(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SymbolOptions,
) -> Result<Vec<SymbolSearchResult>, InventoryError> {
    let query_terms = query_terms(query);
    if query_terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = extract_symbols(repo_root)?
        .into_iter()
        .filter_map(|symbol| score_symbol(symbol, &query_terms))
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.symbol.path.cmp(&right.symbol.path))
            .then_with(|| left.symbol.start_line.cmp(&right.symbol.start_line))
    });
    results.truncate(options.limit.max(1));
    Ok(results)
}

pub fn lexical_search(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let query_terms = query_terms(query);
    if query_terms.is_empty() {
        return Ok(Vec::new());
    }

    let inventory = load_or_build_inventory(&repo_root, &InventoryOptions::default())?;
    let mut results = Vec::new();

    for file in inventory.files {
        if file.generated || file.role == FileRole::Sensitive || file.ignored {
            continue;
        }

        let path = repo_root.join(&file.path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let Some((score, reason)) = score_file(&file, &content, &query_terms) else {
            continue;
        };

        results.push(SearchResult {
            path: file.path,
            role: file.role,
            language: file.language,
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

    Ok(results)
}

pub fn related_tests(
    repo_root: impl AsRef<Path>,
    source_paths: &[String],
) -> Result<Vec<RelatedTestResult>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory = load_or_build_inventory(&repo_root, &InventoryOptions::default())?;
    let source_keys = source_paths
        .iter()
        .map(|path| source_key(path))
        .collect::<Vec<_>>();
    if source_keys.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    for file in inventory.files {
        if file.role != FileRole::Test || file.generated || file.ignored {
            continue;
        }

        let path = repo_root.join(&file.path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let Some((score, reason)) = score_test_file(&file.path, &content, &source_keys) else {
            continue;
        };

        results.push(RelatedTestResult {
            path: file.path.clone(),
            command: test_command_for(&repo_root, &file.path),
            confidence: (score / 20.0).min(0.95),
            reason,
        });
    }

    results.sort_by(|left, right| {
        right
            .confidence
            .total_cmp(&left.confidence)
            .then_with(|| left.path.cmp(&right.path))
    });

    Ok(results)
}

pub fn test_map(repo_root: impl AsRef<Path>) -> Result<Vec<RelatedTestResult>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory = load_or_build_inventory(&repo_root, &InventoryOptions::default())?;
    let mut results = inventory
        .files
        .into_iter()
        .filter(|file| file.role == FileRole::Test && !file.generated && !file.ignored)
        .map(|file| RelatedTestResult {
            path: file.path.clone(),
            command: test_command_for(&repo_root, &file.path),
            confidence: 1.0,
            reason: "safe test file from inventory".to_string(),
        })
        .collect::<Vec<_>>();

    results.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(results)
}

pub fn co_change_hints(
    repo_root: impl AsRef<Path>,
    anchor_paths: &[String],
    options: &CoChangeOptions,
) -> Result<Vec<CoChangeHint>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory = load_or_build_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_paths = inventory
        .files
        .iter()
        .filter(|file| {
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
        })
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let anchors = anchor_paths
        .iter()
        .map(|path| normalize_input_path(&repo_root, path))
        .collect::<BTreeSet<_>>();
    if anchors.is_empty() {
        return Ok(Vec::new());
    }

    let commits = git_commit_file_sets(&repo_root)?;
    let mut counts: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for commit in commits {
        if !commit.files.iter().any(|path| anchors.contains(path)) {
            continue;
        }

        for path in commit.files {
            if anchors.contains(&path) || !safe_paths.contains(&path) {
                continue;
            }
            counts.entry(path).or_default().push(commit.sha.clone());
        }
    }

    let mut hints = counts
        .into_iter()
        .map(|(path, commits)| {
            let commit_count = commits.len();
            CoChangeHint {
                path: path.clone(),
                commit_count,
                confidence: (commit_count as f32 / 5.0).min(0.95),
                sample_commits: commits.into_iter().take(3).collect(),
                reason: format!("changed with requested paths in {commit_count} local commit(s)"),
            }
        })
        .collect::<Vec<_>>();

    hints.sort_by(|left, right| {
        right
            .commit_count
            .cmp(&left.commit_count)
            .then_with(|| left.path.cmp(&right.path))
    });
    hints.truncate(options.limit.max(1));

    Ok(hints)
}

pub fn dependency_edges(
    repo_root: impl AsRef<Path>,
    options: &DependencyOptions,
) -> Result<Vec<DependencyEdge>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory = load_or_build_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_files = safe_dependency_files(&inventory);
    let safe_paths = safe_files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut edges = Vec::new();

    for file in safe_files {
        let content = fs::read_to_string(repo_root.join(&file.path)).unwrap_or_default();
        for import in imports_for_file(file, &content) {
            let Some(target_path) = resolve_import_target(&file.path, &import.raw, &safe_paths)
            else {
                continue;
            };
            if target_path == file.path {
                continue;
            }
            if seen.insert((file.path.clone(), target_path.clone(), import.raw.clone())) {
                edges.push(DependencyEdge {
                    source_path: file.path.clone(),
                    target_path,
                    kind: "imports".to_string(),
                    confidence: import.confidence,
                    reason: format!("{} import `{}`", import.language, import.raw),
                });
            }
        }
    }

    edges.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.target_path.cmp(&right.target_path))
    });
    edges.truncate(options.limit.max(1));
    Ok(edges)
}

pub fn related_dependency_edges(
    repo_root: impl AsRef<Path>,
    anchor_paths: &[String],
    options: &DependencyOptions,
) -> Result<Vec<DependencyEdge>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let anchors = anchor_paths
        .iter()
        .map(|path| normalize_input_path(&repo_root, path))
        .filter(|path| !path.is_empty())
        .collect::<BTreeSet<_>>();
    if anchors.is_empty() {
        return Ok(Vec::new());
    }

    let mut edges = dependency_edges(&repo_root, &DependencyOptions { limit: usize::MAX })?
        .into_iter()
        .filter(|edge| anchors.contains(&edge.source_path) || anchors.contains(&edge.target_path))
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        edge_anchor_rank(left, &anchors)
            .cmp(&edge_anchor_rank(right, &anchors))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.target_path.cmp(&right.target_path))
    });
    edges.truncate(options.limit.max(1));
    Ok(edges)
}

pub fn append_eval_trace(
    repo_root: impl AsRef<Path>,
    trace: &EvalTrace,
) -> Result<PathBuf, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = trace_path(&repo_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
    let json = serde_json::to_string(trace).map_err(InventoryError::Serialize)?;
    writeln!(file, "{json}").map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

pub fn list_eval_traces(
    repo_root: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<EvalTrace>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = trace_path(&repo_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(InventoryError::Read {
                path: path.clone(),
                source,
            })
        }
    };

    let mut traces = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<EvalTrace>(line).map_err(|source| InventoryError::Deserialize {
                path: path.clone(),
                source,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    traces.reverse();
    traces.truncate(limit.max(1));
    Ok(traces)
}

pub fn inventory_path(repo_id: &str) -> PathBuf {
    ctxpack_home()
        .join("repos")
        .join(repo_id)
        .join("inventory.json")
}

pub fn trace_path(repo_id: &str) -> PathBuf {
    ctxpack_home()
        .join("repos")
        .join(repo_id)
        .join("traces.jsonl")
}

pub fn repo_id_for_path(repo_root: &Path) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, repo_root.to_string_lossy().as_bytes()).to_string()
}

pub fn task_hash(task: &str) -> String {
    blake3::hash(task.trim().as_bytes()).to_hex().to_string()
}

fn ctxpack_home() -> PathBuf {
    std::env::var_os("CTXPACK_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".ctxpack")))
        .unwrap_or_else(|| PathBuf::from(".ctxpack"))
}

fn canonicalize(path: &Path) -> Result<PathBuf, InventoryError> {
    fs::canonicalize(path).map_err(|source| InventoryError::Canonicalize {
        path: path.to_path_buf(),
        source,
    })
}

fn normalize_relative_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_input_path(repo_root: &Path, path: &str) -> String {
    let path = Path::new(path);
    let path = if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn safe_dependency_files(inventory: &RepoInventory) -> Vec<&FileInventoryEntry> {
    inventory
        .files
        .iter()
        .filter(|file| {
            !file.generated
                && !file.ignored
                && matches!(file.role, FileRole::Source | FileRole::Test)
                && matches!(
                    file.language.as_deref(),
                    Some("typescript" | "javascript" | "python" | "rust")
                )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
struct ImportRef {
    raw: String,
    language: &'static str,
    confidence: f32,
}

fn imports_for_file(file: &FileInventoryEntry, content: &str) -> Vec<ImportRef> {
    match file.language.as_deref() {
        Some("typescript" | "javascript") => js_imports(content),
        Some("python") => python_imports(content),
        Some("rust") => rust_imports(content),
        _ => Vec::new(),
    }
}

fn js_imports(content: &str) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("import ") {
            if let Some((_, after_from)) = trimmed.rsplit_once(" from ") {
                push_quoted_import(&mut imports, after_from, "javascript/typescript", 0.95);
            } else {
                push_quoted_import(&mut imports, trimmed, "javascript/typescript", 0.9);
            }
        }
        for marker in ["require(", "import("] {
            if let Some((_, rest)) = trimmed.split_once(marker) {
                push_quoted_import(&mut imports, rest, "javascript/typescript", 0.8);
            }
        }
    }
    imports
}

fn python_imports(content: &str) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("from ") {
            if let Some((module, _)) = rest.split_once(" import ") {
                if !module.trim().is_empty() {
                    imports.push(ImportRef {
                        raw: python_module_to_path(module.trim()),
                        language: "python",
                        confidence: 0.9,
                    });
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("import ") {
            for module in rest.split(',') {
                let module = module.split_whitespace().next().unwrap_or("");
                if !module.is_empty() {
                    imports.push(ImportRef {
                        raw: python_module_to_path(module),
                        language: "python",
                        confidence: 0.75,
                    });
                }
            }
        }
    }
    imports
}

fn rust_imports(content: &str) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let trimmed = trimmed.strip_prefix("pub ").unwrap_or(trimmed);
        if let Some(module) = trimmed
            .strip_prefix("mod ")
            .and_then(|rest| rest.split(';').next())
        {
            imports.push(ImportRef {
                raw: format!("./{}", module.trim()),
                language: "rust",
                confidence: 0.9,
            });
        } else if let Some(path) = trimmed
            .strip_prefix("use ")
            .and_then(|rest| rest.split(';').next())
            .filter(|path| path.starts_with("crate::") || path.starts_with("super::"))
        {
            imports.push(ImportRef {
                raw: path.trim().to_string(),
                language: "rust",
                confidence: 0.7,
            });
        }
    }
    imports
}

fn push_quoted_import(
    imports: &mut Vec<ImportRef>,
    text: &str,
    language: &'static str,
    confidence: f32,
) {
    if let Some(raw) = quoted_import_value(text) {
        imports.push(ImportRef {
            raw,
            language,
            confidence,
        });
    }
}

fn quoted_import_value(text: &str) -> Option<String> {
    let quote_index = text.find(['"', '\''])?;
    let quote = text.as_bytes()[quote_index] as char;
    let rest = &text[quote_index + 1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn python_module_to_path(module: &str) -> String {
    if module.starts_with('.') {
        let dots = module
            .chars()
            .take_while(|character| *character == '.')
            .count();
        let rest = module.trim_start_matches('.').replace('.', "/");
        let mut path = if dots <= 1 {
            ".".to_string()
        } else {
            std::iter::repeat_n("..", dots - 1)
                .collect::<Vec<_>>()
                .join("/")
        };
        if !rest.is_empty() {
            path.push('/');
            path.push_str(&rest);
        }
        path
    } else {
        module.replace('.', "/")
    }
}

fn resolve_import_target(
    source_path: &str,
    raw: &str,
    safe_paths: &BTreeSet<String>,
) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if raw.starts_with("crate::") {
        let base = raw.trim_start_matches("crate::").replace("::", "/");
        return first_existing(&format!("src/{base}"), safe_paths);
    }
    if raw.starts_with("super::") {
        let base = raw.trim_start_matches("super::").replace("::", "/");
        let parent = Path::new(source_path).parent().and_then(Path::parent)?;
        return first_existing(&join_normalized(parent, &base)?, safe_paths);
    }

    let base = if raw.starts_with('.') {
        let parent = Path::new(source_path)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        join_normalized(parent, raw)?
    } else {
        raw.to_string()
    };
    first_existing(&base, safe_paths)
}

fn first_existing(base: &str, safe_paths: &BTreeSet<String>) -> Option<String> {
    let mut candidates = vec![base.to_string()];
    for extension in ["ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rs", "go"] {
        candidates.push(format!("{base}.{extension}"));
    }
    for extension in ["ts", "tsx", "js", "jsx", "py", "rs"] {
        candidates.push(format!("{base}/index.{extension}"));
        candidates.push(format!("{base}/mod.{extension}"));
    }
    candidates
        .into_iter()
        .find(|candidate| safe_paths.contains(candidate))
}

fn join_normalized(parent: &Path, raw: &str) -> Option<String> {
    let mut parts = parent
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for component in Path::new(raw).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

fn edge_anchor_rank(edge: &DependencyEdge, anchors: &BTreeSet<String>) -> u8 {
    match (
        anchors.contains(&edge.source_path),
        anchors.contains(&edge.target_path),
    ) {
        (true, false) => 0,
        (false, true) => 1,
        (true, true) => 2,
        (false, false) => 3,
    }
}

fn classify_path(path: &str) -> FileRole {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(lower.as_str());

    if is_sensitive_path(&lower, name) {
        return FileRole::Sensitive;
    }
    if is_generated_path(&lower, name) {
        return FileRole::Generated;
    }
    if is_test_path(&lower, name) {
        return FileRole::Test;
    }
    if is_schema_path(&lower, name) {
        return FileRole::Schema;
    }
    if is_config_path(&lower, name) {
        return FileRole::Config;
    }
    if is_docs_path(&lower, name) {
        return FileRole::Docs;
    }
    if language_for_path(path).is_some() {
        return FileRole::Source;
    }

    FileRole::Unknown
}

fn is_sensitive_path(lower: &str, name: &str) -> bool {
    name == ".env"
        || name.starts_with(".env.")
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name.ends_with(".p12")
        || name.ends_with(".pfx")
        || name.ends_with(".crt")
        || name.ends_with(".cer")
        || name.ends_with(".dump")
        || name.ends_with(".sql.gz")
        || lower.contains("/.env.")
        || lower.contains("secret")
        || lower.contains("credentials")
}

fn is_generated_path(lower: &str, name: &str) -> bool {
    lower.contains("/node_modules/")
        || lower.contains("/target/")
        || lower.contains("/dist/")
        || lower.contains("/build/")
        || lower.contains("/coverage/")
        || lower.contains("/vendor/")
        || lower.starts_with("node_modules/")
        || lower.starts_with("target/")
        || lower.starts_with("dist/")
        || lower.starts_with("build/")
        || name.ends_with(".min.js")
        || name.ends_with(".min.css")
        || name == "package-lock.json"
        || name == "pnpm-lock.yaml"
        || name == "yarn.lock"
        || name == "cargo.lock"
}

fn is_test_path(lower: &str, name: &str) -> bool {
    lower.contains("/tests/")
        || lower.contains("/test/")
        || lower.contains("/spec/")
        || lower.starts_with("tests/")
        || lower.starts_with("test/")
        || lower.starts_with("spec/")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.ends_with("_test.go")
        || name.ends_with("_test.py")
        || name.starts_with("test_")
}

fn is_config_path(lower: &str, name: &str) -> bool {
    matches!(
        name,
        "package.json"
            | "pyproject.toml"
            | "cargo.toml"
            | "go.mod"
            | "go.sum"
            | "tsconfig.json"
            | "vite.config.ts"
            | "next.config.js"
            | "dockerfile"
            | "compose.yaml"
            | "docker-compose.yml"
            | "ctxpack.toml"
    ) || lower.ends_with(".config.js")
        || lower.ends_with(".config.ts")
        || lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
}

fn is_schema_path(lower: &str, name: &str) -> bool {
    lower.contains("/migrations/")
        || lower.contains("/schema/")
        || name.ends_with(".graphql")
        || name.ends_with(".graphqls")
        || name.ends_with(".proto")
        || name.ends_with(".prisma")
        || name.ends_with(".sql")
}

fn is_docs_path(lower: &str, name: &str) -> bool {
    lower.starts_with("docs/")
        || name == "readme.md"
        || name == "agents.md"
        || name.ends_with(".md")
        || name.ends_with(".mdx")
        || name.ends_with(".rst")
        || name.ends_with(".txt")
}

fn language_for_path(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(lower.as_str());
    let extension = name.rsplit_once('.').map(|(_, extension)| extension);

    match extension {
        Some("rs") => Some("rust"),
        Some("ts") | Some("tsx") => Some("typescript"),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Some("javascript"),
        Some("py") => Some("python"),
        Some("go") => Some("go"),
        Some("java") => Some("java"),
        Some("kt") | Some("kts") => Some("kotlin"),
        Some("scala") => Some("scala"),
        Some("cs") => Some("csharp"),
        Some("rb") => Some("ruby"),
        Some("php") => Some("php"),
        Some("dart") => Some("dart"),
        Some("c") | Some("h") => Some("c"),
        Some("cc") | Some("cpp") | Some("cxx") | Some("hpp") => Some("cpp"),
        Some("swift") => Some("swift"),
        Some("sql") => Some("sql"),
        _ => None,
    }
}

fn query_terms(query: &str) -> Vec<String> {
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
        let mut matched = false;

        if path.contains(term) {
            score += 8.0;
            matched = true;
            reasons.push(format!("path matched `{term}`"));
        }
        if file_name.contains(term) {
            score += 5.0;
            matched = true;
            reasons.push(format!("file name matched `{term}`"));
        }

        let occurrences = count_occurrences(&content, term);
        if occurrences > 0 {
            score += 2.0 + occurrences.min(10) as f32;
            matched = true;
            reasons.push(format!("content matched `{term}` {occurrences} time(s)"));
        }

        if !matched {
            score -= 1.0;
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

    reasons.sort();
    reasons.dedup();

    Some((score, reasons.join("; ")))
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

fn symbols_for_file(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    match file.language.as_deref() {
        Some("typescript" | "javascript") => symbols_for_js_like(file, content),
        Some("python") => symbols_for_python(file, content),
        Some("rust") => symbols_for_rust(file, content),
        Some("go") => symbols_for_go(file, content),
        _ => Vec::new(),
    }
}

fn symbols_for_js_like(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("import ") {
            if let Some(name) = import_name(trimmed) {
                symbols.push(code_symbol(
                    file,
                    name,
                    SymbolKind::Import,
                    line_no,
                    trimmed,
                    false,
                ));
            }
            continue;
        }

        let exported = trimmed.starts_with("export ");
        let rest = strip_modifiers(
            trimmed,
            &[
                "export",
                "default",
                "declare",
                "async",
                "public",
                "private",
                "protected",
                "static",
                "readonly",
            ],
        );
        if let Some(name) = identifier_after(rest, "function ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Function,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "class ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Class,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "interface ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Interface,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "type ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Type,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = variable_name(rest) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Constant,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = js_method_name(rest) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Method,
                line_no,
                trimmed,
                exported,
            ));
        }
    }
    symbols
}

fn symbols_for_python(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len().saturating_sub(line.trim_start().len());
        let rest = strip_modifiers(trimmed, &["async"]);
        if let Some(name) = identifier_after(rest, "def ") {
            let kind = if indent > 0 {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            };
            symbols.push(code_symbol(file, name, kind, line_no, trimmed, false));
        } else if let Some(name) = identifier_after(rest, "class ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Class,
                line_no,
                trimmed,
                false,
            ));
        }
    }
    symbols
}

fn symbols_for_rust(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#[") {
            continue;
        }
        let exported = trimmed.starts_with("pub ");
        let rest = strip_modifiers(trimmed, &["pub", "async", "unsafe"]);
        let candidates = [
            ("fn ", SymbolKind::Function),
            ("struct ", SymbolKind::Class),
            ("enum ", SymbolKind::Type),
            ("trait ", SymbolKind::Interface),
            ("type ", SymbolKind::Type),
            ("const ", SymbolKind::Constant),
            ("static ", SymbolKind::Constant),
            ("mod ", SymbolKind::Module),
        ];
        for (prefix, kind) in candidates {
            if let Some(name) = identifier_after(rest, prefix) {
                symbols.push(code_symbol(file, name, kind, line_no, trimmed, exported));
                break;
            }
        }
    }
    symbols
}

fn symbols_for_go(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(name) = go_func_name(trimmed) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Function,
                line_no,
                trimmed,
                is_exported_go(name),
            ));
        } else if let Some(name) = identifier_after(trimmed, "type ") {
            let kind = if trimmed.contains(" interface") {
                SymbolKind::Interface
            } else {
                SymbolKind::Type
            };
            symbols.push(code_symbol(
                file,
                name,
                kind,
                line_no,
                trimmed,
                is_exported_go(name),
            ));
        }
    }
    symbols
}

fn code_symbol(
    file: &FileInventoryEntry,
    name: &str,
    kind: SymbolKind,
    line_no: u32,
    signature: &str,
    exported: bool,
) -> CodeSymbol {
    CodeSymbol {
        name: name.to_string(),
        kind,
        path: file.path.clone(),
        language: file.language.clone(),
        start_line: line_no,
        end_line: line_no,
        signature: signature.chars().take(200).collect(),
        exported,
    }
}

fn score_symbol(symbol: CodeSymbol, query_terms: &[String]) -> Option<SymbolSearchResult> {
    let name = symbol.name.to_ascii_lowercase();
    let path = symbol.path.to_ascii_lowercase();
    let signature = symbol.signature.to_ascii_lowercase();
    let mut score = 0.0;
    let mut reasons = Vec::new();

    for term in query_terms {
        let mut matched = false;
        if name == *term {
            score += 24.0;
            matched = true;
            reasons.push(format!("symbol name exactly matched `{term}`"));
        } else if name.starts_with(term) {
            score += 16.0;
            matched = true;
            reasons.push(format!("symbol name starts with `{term}`"));
        } else if name.contains(term) {
            score += 12.0;
            matched = true;
            reasons.push(format!("symbol name contains `{term}`"));
        }
        if path.contains(term) {
            score += 4.0;
            matched = true;
            reasons.push(format!("path contains `{term}`"));
        }
        if signature.contains(term) {
            score += 2.0;
            matched = true;
            reasons.push(format!("signature contains `{term}`"));
        }
        if !matched {
            score -= 2.0;
        }
    }

    if symbol.exported {
        score += 1.0;
    }
    if score <= 0.0 {
        return None;
    }

    reasons.sort();
    reasons.dedup();
    Some(SymbolSearchResult {
        symbol,
        score,
        reason: reasons.join("; "),
    })
}

fn line_number(line_index: usize) -> u32 {
    u32::try_from(line_index + 1).unwrap_or(u32::MAX)
}

fn strip_modifiers<'a>(line: &'a str, modifiers: &[&str]) -> &'a str {
    let mut rest = line.trim();
    loop {
        let mut changed = false;
        for modifier in modifiers {
            if let Some(next) = rest.strip_prefix(modifier).and_then(|value| {
                value
                    .starts_with(char::is_whitespace)
                    .then_some(value.trim_start())
            }) {
                rest = next;
                changed = true;
            }
        }
        if !changed {
            return rest;
        }
    }
}

fn identifier_after<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?;
    take_identifier(rest)
}

fn take_identifier(input: &str) -> Option<&str> {
    let input = input.trim_start();
    let end = input
        .char_indices()
        .find_map(|(index, character)| {
            (!is_identifier_char(character) && index > 0).then_some(index)
        })
        .unwrap_or(input.len());
    let identifier = &input[..end];
    (!identifier.is_empty()).then_some(identifier)
}

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '$'
}

fn variable_name(line: &str) -> Option<&str> {
    ["const ", "let ", "var "]
        .into_iter()
        .find_map(|prefix| identifier_after(line, prefix))
}

fn js_method_name(line: &str) -> Option<&str> {
    let disallowed = [
        "if ",
        "for ",
        "while ",
        "switch ",
        "catch ",
        "return ",
        "function ",
    ];
    if disallowed.iter().any(|prefix| line.starts_with(prefix)) {
        return None;
    }
    let open_paren = line.find('(')?;
    let before = line[..open_paren].trim();
    if before.contains('=') || before.contains(' ') || before.is_empty() {
        return None;
    }
    take_identifier(before)
}

fn import_name(line: &str) -> Option<&str> {
    if let Some((_, module)) = line.split_once(" from ") {
        return quoted_value(module);
    }
    line.strip_prefix("import ").and_then(quoted_value)
}

fn quoted_value(input: &str) -> Option<&str> {
    let start = input.find(['"', '\''])?;
    let quote = input.as_bytes()[start] as char;
    let rest = &input[start + 1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn go_func_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("func ")?;
    let rest = if rest.starts_with('(') {
        let close = rest.find(')')?;
        rest[close + 1..].trim_start()
    } else {
        rest
    };
    take_identifier(rest)
}

fn is_exported_go(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

#[derive(Debug, Clone)]
struct SourceKey {
    path: String,
    stem: String,
    directory: String,
    identifiers: Vec<String>,
}

fn source_key(path: &str) -> SourceKey {
    let normalized = path.trim_start_matches("./").replace('\\', "/");
    let directory = normalized
        .rsplit_once('/')
        .map(|(directory, _)| directory.to_ascii_lowercase())
        .unwrap_or_default();
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    let stem = source_stem(file_name);
    let mut identifiers = query_terms(&stem);
    identifiers.extend(query_terms(&normalized));
    identifiers.sort();
    identifiers.dedup();

    SourceKey {
        path: normalized.to_ascii_lowercase(),
        stem,
        directory,
        identifiers,
    }
}

fn source_stem(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    let without_extension = lower
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(&lower);
    without_extension
        .trim_end_matches(".test")
        .trim_end_matches(".spec")
        .trim_start_matches("test_")
        .trim_end_matches("_test")
        .to_string()
}

fn score_test_file(
    test_path: &str,
    content: &str,
    source_keys: &[SourceKey],
) -> Option<(f32, String)> {
    let test_path_lower = test_path.to_ascii_lowercase();
    let test_name = test_path_lower
        .rsplit('/')
        .next()
        .unwrap_or(test_path_lower.as_str());
    let content = content.to_ascii_lowercase();
    let mut score = 0.0;
    let mut reasons = Vec::new();

    for source in source_keys {
        if !source.stem.is_empty() && test_name.contains(&source.stem) {
            score += 9.0;
            reasons.push(format!(
                "test file name matches source stem `{}`",
                source.stem
            ));
        }
        if !source.directory.is_empty() && test_path_lower.contains(&source.directory) {
            score += 4.0;
            reasons.push(format!("test path shares directory `{}`", source.directory));
        }
        if content.contains(&source.path) {
            score += 8.0;
            reasons.push(format!(
                "test content mentions source path `{}`",
                source.path
            ));
        }
        for identifier in &source.identifiers {
            if identifier.len() < 3 {
                continue;
            }
            let occurrences = count_occurrences(&content, identifier);
            if occurrences > 0 {
                score += 1.5 + occurrences.min(5) as f32;
                reasons.push(format!(
                    "test content mentions source term `{identifier}` {occurrences} time(s)"
                ));
            }
        }
    }

    if score <= 0.0 {
        return None;
    }

    reasons.sort();
    reasons.dedup();
    Some((score, reasons.join("; ")))
}

fn test_command_for(repo_root: &Path, path: &str) -> Option<String> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        Some(rust_test_command(path))
    } else if lower.ends_with(".go") {
        Some(format!("go test ./{}", package_dir(path)))
    } else if lower.ends_with(".py") {
        Some(format!("pytest {path}"))
    } else if lower.ends_with(".ts")
        || lower.ends_with(".tsx")
        || lower.ends_with(".js")
        || lower.ends_with(".jsx")
    {
        Some(javascript_test_command(repo_root, path))
    } else {
        None
    }
}

fn rust_test_command(path: &str) -> String {
    if let Some(file_name) = path
        .strip_prefix("tests/")
        .and_then(|rest| rest.strip_suffix(".rs"))
    {
        if !file_name.contains('/') {
            return format!("cargo test --test {file_name}");
        }
    }
    "cargo test".to_string()
}

fn javascript_test_command(repo_root: &Path, path: &str) -> String {
    let package_root =
        nearest_package_root(repo_root, path).unwrap_or_else(|| repo_root.to_path_buf());
    let package_manager = detect_js_package_manager(&package_root);
    let script = read_test_script(&package_root);

    if let Some(script) = script {
        let lower_script = script.to_ascii_lowercase();
        if lower_script.contains("vitest") {
            return format!("{} vitest run {path}", package_manager.command());
        }
        if lower_script.contains("jest") {
            return format!("{} jest {path}", package_manager.command());
        }
        if !is_placeholder_test_script(&lower_script) {
            return package_manager.run_test_script(path);
        }
    }

    format!("{} test {path}", package_manager.command())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsPackageManager {
    Pnpm,
    Yarn,
    Npm,
    Bun,
}

impl JsPackageManager {
    fn command(self) -> &'static str {
        match self {
            JsPackageManager::Pnpm => "pnpm",
            JsPackageManager::Yarn => "yarn",
            JsPackageManager::Npm => "npm",
            JsPackageManager::Bun => "bun",
        }
    }

    fn run_test_script(self, path: &str) -> String {
        match self {
            JsPackageManager::Pnpm => format!("pnpm test -- {path}"),
            JsPackageManager::Yarn => format!("yarn test {path}"),
            JsPackageManager::Npm => format!("npm test -- {path}"),
            JsPackageManager::Bun => format!("bun test {path}"),
        }
    }
}

fn nearest_package_root(repo_root: &Path, path: &str) -> Option<PathBuf> {
    let mut current = repo_root.join(path).parent()?.to_path_buf();
    loop {
        if current.join("package.json").is_file() {
            return Some(current);
        }
        if current == repo_root {
            break;
        }
        if !current.pop() {
            break;
        }
    }
    if repo_root.join("package.json").is_file() {
        Some(repo_root.to_path_buf())
    } else {
        None
    }
}

fn detect_js_package_manager(package_root: &Path) -> JsPackageManager {
    if package_root.join("pnpm-lock.yaml").is_file() {
        JsPackageManager::Pnpm
    } else if package_root.join("yarn.lock").is_file() {
        JsPackageManager::Yarn
    } else if package_root.join("package-lock.json").is_file()
        || package_root.join("npm-shrinkwrap.json").is_file()
    {
        JsPackageManager::Npm
    } else if package_root.join("bun.lock").is_file() || package_root.join("bun.lockb").is_file() {
        JsPackageManager::Bun
    } else {
        JsPackageManager::Pnpm
    }
}

fn read_test_script(package_root: &Path) -> Option<String> {
    let package_json = fs::read_to_string(package_root.join("package.json")).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&package_json).ok()?;
    value
        .get("scripts")?
        .get("test")?
        .as_str()
        .map(str::trim)
        .filter(|script| !script.is_empty())
        .map(str::to_string)
}

fn is_placeholder_test_script(script: &str) -> bool {
    script.contains("no test")
        || script.contains("no tests")
        || script.contains("missing script")
        || script.contains("error")
}

fn package_dir(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or(".")
        .to_string()
}

#[derive(Debug)]
struct GitCommitFiles {
    sha: String,
    files: Vec<String>,
}

fn git_commit_file_sets(repo_root: &Path) -> Result<Vec<GitCommitFiles>, InventoryError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("log")
        .arg("--name-only")
        .arg("--format=commit:%H")
        .output()
        .map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if is_empty_git_history_message(&message) {
            return Ok(Vec::new());
        }
        return Err(InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message,
        });
    }

    Ok(parse_git_log_name_only(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn is_empty_git_history_message(message: &str) -> bool {
    message.contains("does not have any commits yet")
        || message.contains("your current branch") && message.contains("does not have any commits")
}

fn parse_git_log_name_only(output: &str) -> Vec<GitCommitFiles> {
    let mut commits = Vec::new();
    let mut current_sha: Option<String> = None;
    let mut current_files = Vec::new();

    for line in output.lines() {
        if let Some(sha) = line.strip_prefix("commit:") {
            if let Some(previous_sha) = current_sha.replace(sha.to_string()) {
                commits.push(GitCommitFiles {
                    sha: previous_sha,
                    files: std::mem::take(&mut current_files),
                });
            }
            continue;
        }

        let path = line.trim();
        if path.is_empty() {
            continue;
        }
        current_files.push(path.replace('\\', "/"));
    }

    if let Some(sha) = current_sha {
        commits.push(GitCommitFiles {
            sha,
            files: current_files,
        });
    }

    commits
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
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
        fs::write(repo.join("src/lib.ts"), "export const x = 1;\n").unwrap();
        fs::write(repo.join("tests/lib.test.ts"), "test('x', () => {});\n").unwrap();
        fs::write(repo.join("README.md"), "# Repo\n").unwrap();
        fs::write(repo.join("schema.sql"), "create table users(id int);\n").unwrap();
        fs::write(repo.join("package.json"), "{}\n").unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();
        fs::write(repo.join("private.key"), "secret\n").unwrap();
        fs::write(repo.join("dist/app.min.js"), "minified\n").unwrap();
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
