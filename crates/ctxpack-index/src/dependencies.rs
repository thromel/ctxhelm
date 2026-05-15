use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{
    canonicalize, normalize_input_path, FileInventoryEntry, InventoryError, InventoryOptions,
    RepoInventory,
};
use crate::policy::{read_safe_source, SourceReadStatus, SOURCE_READ_MAX_BYTES};
use ctxpack_core::{CacheStatus, Diagnostic, DiagnosticSeverity, FileRole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const PRECISION_EDGES_SCHEMA_VERSION: u32 = 1;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEdgesReport {
    pub edges: Vec<DependencyEdge>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrecisionEdgeRecord {
    pub source_path: String,
    pub target_path: String,
    #[serde(default = "default_precision_edge_type")]
    pub edge_type: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrecisionEdgesFile {
    pub schema_version: u32,
    pub provider: String,
    #[serde(default)]
    pub edges: Vec<PrecisionEdgeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrecisionImportReport {
    pub path: String,
    pub provider: String,
    pub accepted_edges: usize,
    pub rejected_edges: usize,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

fn default_precision_edge_type() -> String {
    "references".to_string()
}

pub fn dependency_edges(
    repo_root: impl AsRef<Path>,
    options: &DependencyOptions,
) -> Result<Vec<DependencyEdge>, InventoryError> {
    Ok(dependency_edges_report(repo_root, options)?.edges)
}

pub fn dependency_edges_report(
    repo_root: impl AsRef<Path>,
    options: &DependencyOptions,
) -> Result<DependencyEdgesReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    let safe_files = safe_dependency_files(&inventory_report.inventory);
    let safe_paths = safe_files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut edges = Vec::new();

    for file in safe_files {
        let source = read_safe_source(
            &repo_root,
            &inventory_report.inventory,
            &file.path,
            SOURCE_READ_MAX_BYTES,
        )?;
        if !source.diagnostics.is_empty() {
            diagnostics.extend(source.diagnostics);
            diagnostics.push(partial_diagnostic(
                "graph_partial",
                "Dependency graph inference skipped at least one source file because it could not be safely read.",
                vec![file.path.clone()],
            ));
        }
        let SourceReadStatus::Read = source.status else {
            continue;
        };
        let content = source.text.unwrap_or_default();
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
    let overlay = load_precision_overlay(&repo_root, &safe_paths);
    diagnostics.extend(overlay.diagnostics);
    for record in overlay.edges {
        let kind = format!("precision:{}", record.edge_type);
        if seen.insert((
            record.source_path.clone(),
            record.target_path.clone(),
            kind.clone(),
        )) {
            let symbol = record
                .symbol
                .as_deref()
                .map(|symbol| format!(" for `{symbol}`"))
                .unwrap_or_default();
            let reason = record.reason.unwrap_or_else(|| {
                format!(
                    "source-free precision edge `{}`{}",
                    record.edge_type, symbol
                )
            });
            edges.push(DependencyEdge {
                source_path: record.source_path,
                target_path: record.target_path,
                kind,
                confidence: record.confidence.unwrap_or(0.95).clamp(0.0, 1.0),
                reason,
            });
        }
    }

    edges.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.target_path.cmp(&right.target_path))
    });
    edges.truncate(options.limit.max(1));
    Ok(DependencyEdgesReport {
        edges,
        diagnostics,
        cache_status,
    })
}

fn partial_diagnostic(code: &str, message: &str, paths: Vec<String>) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: message.to_string(),
        count: paths.len(),
        paths,
    }
}

pub fn related_dependency_edges(
    repo_root: impl AsRef<Path>,
    anchor_paths: &[String],
    options: &DependencyOptions,
) -> Result<Vec<DependencyEdge>, InventoryError> {
    Ok(related_dependency_edges_report(repo_root, anchor_paths, options)?.edges)
}

pub fn related_dependency_edges_report(
    repo_root: impl AsRef<Path>,
    anchor_paths: &[String],
    options: &DependencyOptions,
) -> Result<DependencyEdgesReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let anchors = anchor_paths
        .iter()
        .map(|path| normalize_input_path(&repo_root, path))
        .filter(|path| !path.is_empty())
        .collect::<BTreeSet<_>>();
    if anchors.is_empty() {
        let report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
        return Ok(DependencyEdgesReport {
            edges: Vec::new(),
            diagnostics: report.diagnostics,
            cache_status: report.cache_status,
        });
    }

    let full_report =
        dependency_edges_report(&repo_root, &DependencyOptions { limit: usize::MAX })?;
    let mut edges = full_report
        .edges
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
    Ok(DependencyEdgesReport {
        edges,
        diagnostics: full_report.diagnostics,
        cache_status: full_report.cache_status,
    })
}

pub fn precision_edges_path(repo_root: impl AsRef<Path>) -> Result<PathBuf, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    Ok(repo_root.join(".ctxpack").join("precision-edges.json"))
}

pub fn import_precision_edges(
    repo_root: impl AsRef<Path>,
    input_path: impl AsRef<Path>,
) -> Result<PrecisionImportReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let input_path = input_path.as_ref();
    let raw = fs::read_to_string(input_path).map_err(|source| InventoryError::Read {
        path: input_path.to_path_buf(),
        source,
    })?;
    let parsed: PrecisionEdgesFile =
        serde_json::from_str(&raw).map_err(|source| InventoryError::Deserialize {
            path: input_path.to_path_buf(),
            source,
        })?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_paths = safe_dependency_files(&inventory_report.inventory)
        .into_iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = inventory_report.diagnostics;
    let mut accepted = Vec::new();
    let mut rejected_edges = 0;

    for record in parsed.edges {
        let source_path = normalize_input_path(&repo_root, &record.source_path);
        let target_path = normalize_input_path(&repo_root, &record.target_path);
        if safe_paths.contains(&source_path) && safe_paths.contains(&target_path) {
            accepted.push(PrecisionEdgeRecord {
                source_path,
                target_path,
                edge_type: precision_edge_type(&record.edge_type),
                symbol: record.symbol.filter(|value| !value.trim().is_empty()),
                confidence: record.confidence.map(|value| value.clamp(0.0, 1.0)),
                reason: record.reason.filter(|value| !value.trim().is_empty()),
            });
        } else {
            rejected_edges += 1;
        }
    }

    if rejected_edges > 0 {
        diagnostics.push(Diagnostic {
            code: "precision_edges_rejected".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Precision edge import rejected records whose source or target path was not in the safe local inventory.".to_string(),
            count: rejected_edges,
            paths: Vec::new(),
        });
    }

    let output = PrecisionEdgesFile {
        schema_version: PRECISION_EDGES_SCHEMA_VERSION,
        provider: parsed.provider,
        edges: accepted,
    };
    let path = precision_edges_path(&repo_root)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(&output).map_err(InventoryError::Serialize)?;
    fs::write(&path, format!("{json}\n")).map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(PrecisionImportReport {
        path: path.display().to_string(),
        provider: output.provider,
        accepted_edges: output.edges.len(),
        rejected_edges,
        diagnostics,
    })
}

#[derive(Debug, Default)]
struct PrecisionOverlay {
    edges: Vec<PrecisionEdgeRecord>,
    diagnostics: Vec<Diagnostic>,
}

fn load_precision_overlay(repo_root: &Path, safe_paths: &BTreeSet<String>) -> PrecisionOverlay {
    let path = repo_root.join(".ctxpack").join("precision-edges.json");
    if !path.exists() {
        return PrecisionOverlay::default();
    }
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(_) => {
            return PrecisionOverlay {
                edges: Vec::new(),
                diagnostics: vec![Diagnostic {
                    code: "precision_edges_unreadable".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: "Precision edge overlay exists but could not be read; falling back to inferred dependency edges.".to_string(),
                    count: 1,
                    paths: vec![path.display().to_string()],
                }],
            };
        }
    };
    let parsed: PrecisionEdgesFile = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(_) => {
            return PrecisionOverlay {
                edges: Vec::new(),
                diagnostics: vec![Diagnostic {
                    code: "precision_edges_invalid".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: "Precision edge overlay is invalid JSON or does not match the source-free edge schema; falling back to inferred dependency edges.".to_string(),
                    count: 1,
                    paths: vec![path.display().to_string()],
                }],
            };
        }
    };
    let mut rejected_edges = 0;
    let mut edges = Vec::new();
    for record in parsed.edges {
        let source_path = normalize_input_path(repo_root, &record.source_path);
        let target_path = normalize_input_path(repo_root, &record.target_path);
        if safe_paths.contains(&source_path) && safe_paths.contains(&target_path) {
            edges.push(PrecisionEdgeRecord {
                source_path,
                target_path,
                edge_type: precision_edge_type(&record.edge_type),
                symbol: record.symbol,
                confidence: record.confidence.map(|value| value.clamp(0.0, 1.0)),
                reason: record.reason,
            });
        } else {
            rejected_edges += 1;
        }
    }
    let diagnostics = if rejected_edges > 0 {
        vec![Diagnostic {
            code: "precision_edges_rejected".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Precision edge overlay ignored records whose source or target path was not in the safe local inventory.".to_string(),
            count: rejected_edges,
            paths: Vec::new(),
        }]
    } else {
        Vec::new()
    };
    PrecisionOverlay { edges, diagnostics }
}

fn precision_edge_type(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return default_precision_edge_type();
    }
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
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
                    Some("typescript" | "javascript" | "python" | "rust" | "java" | "kotlin")
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
        Some("java") => java_imports(content),
        Some("kotlin") => kotlin_imports(content),
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

fn java_imports(content: &str) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("import ") else {
            continue;
        };
        let rest = rest.strip_prefix("static ").unwrap_or(rest);
        let Some(module) = rest.split(';').next().map(str::trim) else {
            continue;
        };
        if module.is_empty() || module.ends_with(".*") {
            continue;
        }
        imports.push(ImportRef {
            raw: module.replace('.', "/"),
            language: "java",
            confidence: 0.9,
        });
    }
    imports
}

fn kotlin_imports(content: &str) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("import ") else {
            continue;
        };
        let module = rest.split_whitespace().next().unwrap_or("").trim();
        if module.is_empty() || module.ends_with(".*") {
            continue;
        }
        imports.push(ImportRef {
            raw: module.replace('.', "/"),
            language: "kotlin",
            confidence: 0.9,
        });
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
    for extension in [
        "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rs", "go", "java", "kt", "kts",
    ] {
        candidates.push(format!("{base}.{extension}"));
    }
    for extension in ["ts", "tsx", "js", "jsx", "py", "rs", "kt", "kts"] {
        candidates.push(format!("{base}/index.{extension}"));
        candidates.push(format!("{base}/mod.{extension}"));
    }
    for candidate in candidates {
        if safe_paths.contains(&candidate) {
            return Some(candidate);
        }
        let suffix = format!("/{candidate}");
        if let Some(path) = safe_paths.iter().find(|path| path.ends_with(&suffix)) {
            return Some(path.clone());
        }
    }
    None
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
