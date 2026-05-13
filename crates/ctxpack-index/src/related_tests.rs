use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{canonicalize, InventoryError, InventoryOptions};
use crate::policy::{read_safe_source, SourceReadStatus, SOURCE_READ_MAX_BYTES};
use crate::search::{count_occurrences, query_terms};
use ctxpack_core::{CacheStatus, Diagnostic, DiagnosticSeverity, FileRole};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTestResult {
    pub path: String,
    pub command: Option<String>,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTestsReport {
    pub results: Vec<RelatedTestResult>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

pub fn related_tests(
    repo_root: impl AsRef<Path>,
    source_paths: &[String],
) -> Result<Vec<RelatedTestResult>, InventoryError> {
    Ok(related_tests_report(repo_root, source_paths)?.results)
}

pub fn related_tests_report(
    repo_root: impl AsRef<Path>,
    source_paths: &[String],
) -> Result<RelatedTestsReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    let source_keys = source_paths
        .iter()
        .map(|path| source_key(path))
        .collect::<Vec<_>>();
    if source_keys.is_empty() {
        return Ok(RelatedTestsReport {
            results: Vec::new(),
            diagnostics,
            cache_status,
        });
    }

    let mut results = Vec::new();
    for file in &inventory_report.inventory.files {
        if file.role != FileRole::Test || file.generated || file.ignored {
            continue;
        }

        let source = read_safe_source(
            &repo_root,
            &inventory_report.inventory,
            &file.path,
            SOURCE_READ_MAX_BYTES,
        )?;
        if !source.diagnostics.is_empty() {
            diagnostics.extend(source.diagnostics);
            diagnostics.push(partial_diagnostic(
                "test_map_partial",
                "Related-test inference skipped at least one test file because it could not be safely read.",
                vec![file.path.clone()],
            ));
        }
        let SourceReadStatus::Read = source.status else {
            continue;
        };
        let content = source.text.unwrap_or_default();
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

    Ok(RelatedTestsReport {
        results,
        diagnostics,
        cache_status,
    })
}

pub fn test_map(repo_root: impl AsRef<Path>) -> Result<Vec<RelatedTestResult>, InventoryError> {
    Ok(test_map_report(repo_root)?.results)
}

pub fn test_map_report(repo_root: impl AsRef<Path>) -> Result<RelatedTestsReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    let mut results = inventory_report
        .inventory
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
    Ok(RelatedTestsReport {
        results,
        diagnostics,
        cache_status,
    })
}

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

fn partial_diagnostic(code: &str, message: &str, paths: Vec<String>) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: message.to_string(),
        count: paths.len(),
        paths,
    }
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
