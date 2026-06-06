use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{
    canonicalize, normalize_input_path, FileInventoryEntry, InventoryError, InventoryOptions,
};
use ctxhelm_core::{CacheStatus, Diagnostic, DiagnosticSeverity, FileRole};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const REV_LIST_TIMEOUT: Duration = Duration::from_secs(10);
const HISTORY_METADATA_TIMEOUT: Duration = Duration::from_secs(1);
const HISTORY_DIFF_TIMEOUT: Duration = Duration::from_secs(1);
const MIN_HISTORY_DIFF_TIMEOUT: Duration = Duration::from_millis(250);

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoChangeReport {
    pub hints: Vec<CoChangeHint>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

const EVAL_HISTORY_SIDECAR_SCHEMA_VERSION: u32 = 1;
const EVAL_HISTORY_SIDECAR_PATH: &str = ".ctxhelm/eval-history.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct EvalHistorySidecar {
    schema_version: u32,
    revision: String,
    commits: Vec<GitCommitFiles>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDiffOptions {
    pub include_untracked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDiffExcluded {
    pub unstaged: usize,
    pub staged: usize,
    pub untracked: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDiffPrivacyStatus {
    pub local_only: bool,
    pub source_text_returned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDiffSummary {
    pub unstaged: Vec<String>,
    pub staged: Vec<String>,
    pub untracked: Vec<String>,
    pub excluded: CurrentDiffExcluded,
    pub privacy_status: CurrentDiffPrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDiffReport {
    pub summary: CurrentDiffSummary,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalCommitOptions {
    pub limit: usize,
    pub base: Option<String>,
    pub head: Option<String>,
}

impl Default for HistoricalCommitOptions {
    fn default() -> Self {
        Self {
            limit: 20,
            base: None,
            head: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum LabelScope {
    Safe,
    Generated,
    Sensitive,
    HistoricalOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum HistoricalPathExclusionReason {
    Generated,
    Sensitive,
    HistoricalOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalChangedPath {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    pub change_kind: ChangeKind,
    pub role: FileRole,
    pub label_scope: LabelScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded_reason: Option<HistoricalPathExclusionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalCommitSample {
    pub sha: String,
    pub parent_sha: Option<String>,
    #[serde(default, skip_serializing)]
    pub title: String,
    #[serde(default)]
    pub changed_paths: Vec<HistoricalChangedPath>,
    pub safe_changed_files: Vec<String>,
    pub excluded_changed_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalCommitReport {
    pub samples: Vec<HistoricalCommitSample>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MentionedCommitChangedPathsReport {
    pub safe_changed_files: Vec<String>,
    pub excluded_changed_file_count: usize,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

pub fn co_change_hints(
    repo_root: impl AsRef<Path>,
    anchor_paths: &[String],
    options: &CoChangeOptions,
) -> Result<Vec<CoChangeHint>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_paths = safe_history_paths(&inventory_report.inventory.files);
    let anchors = anchor_paths
        .iter()
        .map(|path| normalize_input_path(&repo_root, path))
        .collect::<BTreeSet<_>>();
    if anchors.is_empty() {
        return Ok(Vec::new());
    }

    let commits = git_commit_file_sets_or_sidecar(&repo_root)?;
    Ok(rank_co_change_hints(
        commits,
        &safe_paths,
        &anchors,
        options.limit,
    ))
}

pub fn co_change_hints_report(
    repo_root: impl AsRef<Path>,
    anchor_paths: &[String],
    options: &CoChangeOptions,
) -> Result<CoChangeReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    let safe_paths = safe_history_paths(&inventory_report.inventory.files);
    let anchors = anchor_paths
        .iter()
        .map(|path| normalize_input_path(&repo_root, path))
        .collect::<BTreeSet<_>>();
    if anchors.is_empty() {
        return Ok(CoChangeReport {
            hints: Vec::new(),
            diagnostics,
            cache_status,
        });
    }

    let commits = match git_commit_file_sets_or_sidecar(&repo_root) {
        Ok(commits) => commits,
        Err(error) => {
            let mut diagnostics = diagnostics;
            diagnostics.extend(git_partial_diagnostics(&error, "history_partial"));
            return Ok(CoChangeReport {
                hints: Vec::new(),
                diagnostics,
                cache_status,
            });
        }
    };
    let hints = rank_co_change_hints(commits, &safe_paths, &anchors, options.limit);

    Ok(CoChangeReport {
        hints,
        diagnostics,
        cache_status,
    })
}

pub fn write_eval_history_sidecar(
    source_repo: impl AsRef<Path>,
    revision: &str,
    snapshot_repo: impl AsRef<Path>,
) -> Result<(), InventoryError> {
    let source_repo = canonicalize(source_repo.as_ref())?;
    let snapshot_repo = canonicalize(snapshot_repo.as_ref())?;
    let commits = git_commit_file_sets_for_ref(&source_repo, revision, 50)?;
    let sidecar = EvalHistorySidecar {
        schema_version: EVAL_HISTORY_SIDECAR_SCHEMA_VERSION,
        revision: revision.to_string(),
        commits,
    };
    let path = snapshot_repo.join(EVAL_HISTORY_SIDECAR_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(&sidecar).map_err(InventoryError::Serialize)?;
    fs::write(&path, json).map_err(|source| InventoryError::Write { path, source })
}

fn safe_history_paths(files: &[FileInventoryEntry]) -> BTreeSet<String> {
    files
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
        .collect()
}

fn rank_co_change_hints(
    commits: Vec<GitCommitFiles>,
    safe_paths: &BTreeSet<String>,
    anchors: &BTreeSet<String>,
    limit: usize,
) -> Vec<CoChangeHint> {
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
    hints.truncate(limit.max(1));
    hints
}

pub fn current_diff_summary(
    repo_root: impl AsRef<Path>,
    options: &CurrentDiffOptions,
) -> Result<CurrentDiffSummary, InventoryError> {
    Ok(current_diff_summary_report(repo_root, options)?.summary)
}

pub fn current_diff_summary_report(
    repo_root: impl AsRef<Path>,
    options: &CurrentDiffOptions,
) -> Result<CurrentDiffReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let mut diagnostics = Vec::new();
    let unstaged = match git_name_only(&repo_root, &["diff", "--name-only"]) {
        Ok(paths) => paths,
        Err(error) => {
            diagnostics.extend(git_partial_diagnostics(&error, "current_diff_partial"));
            Vec::new()
        }
    };
    let staged = match git_name_only(&repo_root, &["diff", "--cached", "--name-only"]) {
        Ok(paths) => paths,
        Err(error) => {
            diagnostics.extend(git_partial_diagnostics(&error, "current_diff_partial"));
            Vec::new()
        }
    };
    let untracked = if options.include_untracked {
        match git_name_only(&repo_root, &["ls-files", "--others", "--exclude-standard"]) {
            Ok(paths) => paths,
            Err(error) => {
                diagnostics.extend(git_partial_diagnostics(&error, "current_diff_partial"));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_paths = inventory_report
        .inventory
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let (unstaged, excluded_unstaged) = safe_changed_paths(&repo_root, unstaged, &safe_paths);
    let (staged, excluded_staged) = safe_changed_paths(&repo_root, staged, &safe_paths);
    let (untracked, excluded_untracked) = safe_changed_paths(&repo_root, untracked, &safe_paths);

    Ok(CurrentDiffReport {
        summary: CurrentDiffSummary {
            unstaged,
            staged,
            untracked,
            excluded: CurrentDiffExcluded {
                unstaged: excluded_unstaged,
                staged: excluded_staged,
                untracked: excluded_untracked,
                reason: "paths excluded by safe inventory policy; source content was not returned"
                    .to_string(),
            },
            privacy_status: CurrentDiffPrivacyStatus {
                local_only: true,
                source_text_returned: false,
            },
        },
        diagnostics: {
            diagnostics.extend(inventory_report.diagnostics);
            diagnostics
        },
        cache_status: inventory_report.cache_status,
    })
}

pub fn historical_commit_samples(
    repo_root: impl AsRef<Path>,
    options: &HistoricalCommitOptions,
) -> Result<Vec<HistoricalCommitSample>, InventoryError> {
    Ok(historical_commit_samples_report(repo_root, options)?.samples)
}

pub fn historical_commit_samples_with_safe_paths(
    repo_root: impl AsRef<Path>,
    options: &HistoricalCommitOptions,
    safe_paths: &BTreeSet<String>,
) -> Result<Vec<HistoricalCommitSample>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let commits = git_commit_subject_file_sets(
        &repo_root,
        options.limit.max(1),
        options.base.as_deref(),
        options.head.as_deref(),
    )?;
    Ok(label_historical_commit_samples(
        &repo_root,
        options.limit,
        safe_paths,
        commits,
    ))
}

pub fn historical_commit_samples_report(
    repo_root: impl AsRef<Path>,
    options: &HistoricalCommitOptions,
) -> Result<HistoricalCommitReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_paths = inventory_report
        .inventory
        .files
        .into_iter()
        .map(|file| file.path)
        .collect::<BTreeSet<_>>();
    let commits = match git_commit_subject_file_sets(
        &repo_root,
        options.limit.max(1),
        options.base.as_deref(),
        options.head.as_deref(),
    ) {
        Ok(commits) => commits,
        Err(error) => {
            let mut diagnostics = inventory_report.diagnostics;
            diagnostics.extend(git_partial_diagnostics(&error, "history_partial"));
            return Ok(HistoricalCommitReport {
                samples: Vec::new(),
                diagnostics,
                cache_status: inventory_report.cache_status,
            });
        }
    };
    let samples = label_historical_commit_samples(&repo_root, options.limit, &safe_paths, commits);
    Ok(HistoricalCommitReport {
        samples,
        diagnostics: inventory_report.diagnostics,
        cache_status: inventory_report.cache_status,
    })
}

pub fn mentioned_commit_changed_paths_report(
    repo_root: impl AsRef<Path>,
    shas: &[String],
    limit: usize,
) -> Result<MentionedCommitChangedPathsReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let safe_paths = inventory_report
        .inventory
        .files
        .into_iter()
        .map(|file| file.path)
        .collect::<BTreeSet<_>>();
    let mut diagnostics = inventory_report.diagnostics;
    let mut safe_changed_files = Vec::new();
    let mut excluded_changed_file_count = 0;
    let mut observed_commit_count = 0;

    for sha in shas.iter().filter(|sha| is_hex_revision_like(sha)) {
        let output = match git_diff_tree_name_status_z_without_renames(
            &repo_root,
            sha,
            HISTORY_DIFF_TIMEOUT,
        ) {
            Ok(output) => output,
            Err(error) => {
                diagnostics.extend(git_partial_diagnostics(&error, "mentioned_commit_partial"));
                continue;
            }
        };
        observed_commit_count += 1;
        let changed_paths = label_historical_changed_paths(
            &repo_root,
            &safe_paths,
            parse_git_name_status_z(&output),
        );
        excluded_changed_file_count += changed_paths
            .iter()
            .filter(|label| label.label_scope != LabelScope::Safe)
            .count();
        safe_changed_files.extend(
            changed_paths
                .into_iter()
                .filter(|label| label.label_scope == LabelScope::Safe)
                .map(|label| label.path),
        );
    }

    safe_changed_files.sort();
    safe_changed_files.dedup();
    let limit = limit.max(1);
    if safe_changed_files.len() > limit {
        excluded_changed_file_count += safe_changed_files.len() - limit;
        safe_changed_files.truncate(limit);
    }
    if !safe_changed_files.is_empty() {
        diagnostics.push(Diagnostic {
            code: "mentioned_commit_changed_paths".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Added {} safe changed path anchor(s) from {} task-mentioned commit(s).",
                safe_changed_files.len(),
                observed_commit_count
            ),
            paths: safe_changed_files.clone(),
            count: safe_changed_files.len(),
        });
    }

    Ok(MentionedCommitChangedPathsReport {
        safe_changed_files,
        excluded_changed_file_count,
        diagnostics,
        cache_status: inventory_report.cache_status,
    })
}

fn label_historical_commit_samples(
    repo_root: &Path,
    limit: usize,
    safe_paths: &BTreeSet<String>,
    commits: Vec<GitCommitSubjectFiles>,
) -> Vec<HistoricalCommitSample> {
    let mut samples = commits
        .into_iter()
        .filter_map(|commit| {
            let changed_paths = label_historical_changed_paths(repo_root, safe_paths, commit.files);
            let mut safe_changed_files = changed_paths
                .iter()
                .filter(|label| label.label_scope == LabelScope::Safe)
                .map(|label| label.path.clone())
                .collect::<Vec<_>>();
            safe_changed_files.sort();
            safe_changed_files.dedup();
            if changed_paths.is_empty() {
                return None;
            }
            let excluded_changed_file_count = changed_paths
                .iter()
                .filter(|label| label.label_scope != LabelScope::Safe)
                .count();
            Some(HistoricalCommitSample {
                sha: commit.sha,
                parent_sha: commit.parent_sha,
                title: commit.title,
                changed_paths,
                safe_changed_files,
                excluded_changed_file_count,
            })
        })
        .collect::<Vec<_>>();

    samples.truncate(limit.max(1));
    samples
}

fn git_partial_diagnostics(error: &InventoryError, partial_code: &str) -> Vec<Diagnostic> {
    let (code, message) = match error {
        InventoryError::Git { message, .. } if message.contains("timed out") => (
            "git_timeout",
            "Git command timed out; external history signal is partial.",
        ),
        InventoryError::Git { message, .. }
            if message.contains("No such file or directory")
                || message.contains("not found")
                || message.contains("os error 2") =>
        {
            (
                "git_missing",
                "Git executable was unavailable; external history signal is partial.",
            )
        }
        InventoryError::Git { .. } => (
            "git_unavailable",
            "Git command failed; external history signal is partial.",
        ),
        _ => (
            "git_unavailable",
            "Git signal failed; external history signal is partial.",
        ),
    };
    vec![
        Diagnostic {
            code: code.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: message.to_string(),
            paths: Vec::new(),
            count: 0,
        },
        Diagnostic {
            code: partial_code.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "External git-derived signal returned partial coverage.".to_string(),
            paths: Vec::new(),
            count: 0,
        },
    ]
}

fn is_hex_revision_like(value: &str) -> bool {
    (7..=40).contains(&value.len()) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn safe_changed_paths(
    repo_root: &Path,
    paths: Vec<String>,
    safe_paths: &BTreeSet<String>,
) -> (Vec<String>, usize) {
    if paths.is_empty() {
        return (Vec::new(), 0);
    }
    let mut safe = Vec::new();
    let mut excluded = 0;
    for path in paths {
        let path = normalize_input_path(repo_root, &path);
        if safe_paths.contains(&path) {
            safe.push(path);
        } else {
            excluded += 1;
        }
    }
    safe.sort();
    safe.dedup();
    (safe, excluded)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitCommitFiles {
    pub(crate) sha: String,
    pub(crate) files: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct GitCommitSubjectFiles {
    pub(crate) sha: String,
    pub(crate) parent_sha: Option<String>,
    pub(crate) title: String,
    pub(crate) files: Vec<GitChangedPath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitChangedPath {
    pub(crate) path: String,
    pub(crate) old_path: Option<String>,
    pub(crate) change_kind: ChangeKind,
}

fn label_historical_changed_paths(
    repo_root: &Path,
    safe_paths: &BTreeSet<String>,
    files: Vec<GitChangedPath>,
) -> Vec<HistoricalChangedPath> {
    let mut labels = files
        .into_iter()
        .map(|file| {
            let path = normalize_input_path(repo_root, &file.path);
            let old_path = file
                .old_path
                .as_deref()
                .map(|path| normalize_input_path(repo_root, path));
            let role = crate::policy::classify_path(&path);
            let (label_scope, excluded_reason) = if safe_paths.contains(&path) {
                (LabelScope::Safe, None)
            } else if role == FileRole::Generated {
                (
                    LabelScope::Generated,
                    Some(HistoricalPathExclusionReason::Generated),
                )
            } else if role == FileRole::Sensitive {
                (
                    LabelScope::Sensitive,
                    Some(HistoricalPathExclusionReason::Sensitive),
                )
            } else {
                (
                    LabelScope::HistoricalOnly,
                    Some(HistoricalPathExclusionReason::HistoricalOnly),
                )
            };
            HistoricalChangedPath {
                path,
                old_path,
                change_kind: file.change_kind,
                role,
                label_scope,
                excluded_reason,
            }
        })
        .collect::<Vec<_>>();
    labels.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.old_path.cmp(&right.old_path))
            .then_with(|| left.change_kind.cmp(&right.change_kind))
    });
    labels.dedup();
    labels
}

fn git_commit_file_sets(repo_root: &Path) -> Result<Vec<GitCommitFiles>, InventoryError> {
    git_commit_file_sets_for_ref(repo_root, "HEAD", 50)
}

fn git_commit_file_sets_or_sidecar(
    repo_root: &Path,
) -> Result<Vec<GitCommitFiles>, InventoryError> {
    match git_commit_file_sets(repo_root) {
        Ok(commits) => Ok(commits),
        Err(git_error) => match read_eval_history_sidecar(repo_root) {
            Ok(commits) => Ok(commits),
            Err(_) => Err(git_error),
        },
    }
}

fn read_eval_history_sidecar(repo_root: &Path) -> Result<Vec<GitCommitFiles>, InventoryError> {
    let path = repo_root.join(EVAL_HISTORY_SIDECAR_PATH);
    let json = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
        path: path.clone(),
        source,
    })?;
    let sidecar = serde_json::from_str::<EvalHistorySidecar>(&json).map_err(|source| {
        InventoryError::Deserialize {
            path: path.clone(),
            source,
        }
    })?;
    if sidecar.schema_version != EVAL_HISTORY_SIDECAR_SCHEMA_VERSION {
        return Err(InventoryError::InvalidInput(format!(
            "unsupported eval history sidecar schema version {}",
            sidecar.schema_version
        )));
    }
    Ok(sidecar.commits)
}

fn git_commit_file_sets_for_ref(
    repo_root: &Path,
    revision: &str,
    max_count: usize,
) -> Result<Vec<GitCommitFiles>, InventoryError> {
    let max_count = max_count.max(1).to_string();
    let shas = git_stdout_with_timeout(
        repo_root,
        &["rev-list", "--max-count", &max_count, revision],
        REV_LIST_TIMEOUT,
    )?;
    let mut commits = Vec::new();

    for sha in shas.lines().map(str::trim).filter(|sha| !sha.is_empty()) {
        let Ok(output) =
            git_diff_tree_name_status_z_without_renames(repo_root, sha, HISTORY_DIFF_TIMEOUT)
        else {
            continue;
        };
        let files = parse_git_name_status_z(&output)
            .into_iter()
            .map(|file| file.path)
            .collect::<Vec<_>>();
        commits.push(GitCommitFiles {
            sha: sha.to_string(),
            files,
        });
    }

    Ok(commits)
}

fn git_diff_tree_name_status_z_without_renames(
    repo_root: &Path,
    sha: &str,
    diff_timeout: Duration,
) -> Result<Vec<u8>, InventoryError> {
    let args = [
        "diff-tree",
        "--root",
        "--no-commit-id",
        "--name-status",
        "-z",
        "-r",
        "--diff-filter=ACDMRT",
        sha,
    ];
    git_stdout_bytes_with_timeout(repo_root, &args, diff_timeout)
}

fn git_commit_subject_file_sets(
    repo_root: &Path,
    max_count: usize,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<Vec<GitCommitSubjectFiles>, InventoryError> {
    git_commit_subject_file_sets_with_timeouts(
        repo_root,
        max_count,
        base,
        head,
        HISTORY_METADATA_TIMEOUT,
        HISTORY_DIFF_TIMEOUT,
    )
}

pub(crate) fn git_commit_subject_file_sets_with_timeouts(
    repo_root: &Path,
    max_count: usize,
    base: Option<&str>,
    head: Option<&str>,
    metadata_timeout: Duration,
    diff_timeout: Duration,
) -> Result<Vec<GitCommitSubjectFiles>, InventoryError> {
    let max_count = format!("--max-count={}", max_count.max(1));
    let rev = commit_range(base, head);
    let mut rev_list_args = vec!["rev-list", &max_count];
    rev_list_args.push(rev.as_deref().unwrap_or("HEAD"));
    let shas = git_stdout_with_timeout(repo_root, &rev_list_args, REV_LIST_TIMEOUT)?;
    let mut commits = Vec::new();

    for sha in shas.lines().map(str::trim).filter(|sha| !sha.is_empty()) {
        let Ok(title) = git_stdout_with_timeout(
            repo_root,
            &["log", "-1", "--format=%s", sha],
            metadata_timeout,
        ) else {
            continue;
        };
        let parent_sha = git_parent_sha_with_timeout(repo_root, sha, metadata_timeout)
            .ok()
            .flatten();
        let Ok(output) = git_diff_tree_name_status_z_without_renames(
            repo_root,
            sha,
            diff_timeout.max(MIN_HISTORY_DIFF_TIMEOUT),
        ) else {
            continue;
        };
        let files = parse_git_name_status_z(&output);
        commits.push(GitCommitSubjectFiles {
            sha: sha.to_string(),
            parent_sha,
            title: title.trim().to_string(),
            files,
        });
    }

    Ok(commits)
}

fn git_parent_sha_with_timeout(
    repo_root: &Path,
    sha: &str,
    timeout: Duration,
) -> Result<Option<String>, InventoryError> {
    let output = git_stdout_with_timeout(
        repo_root,
        &["rev-list", "--parents", "-n", "1", sha],
        timeout,
    )?;
    Ok(output
        .split_whitespace()
        .nth(1)
        .filter(|parent| !parent.is_empty())
        .map(str::to_string))
}

fn commit_range(base: Option<&str>, head: Option<&str>) -> Option<String> {
    match (
        base.filter(|value| !value.trim().is_empty()),
        head.filter(|value| !value.trim().is_empty()),
    ) {
        (Some(base), Some(head)) => Some(format!("{}..{}", base.trim(), head.trim())),
        (Some(base), None) => Some(format!("{}..HEAD", base.trim())),
        (None, Some(head)) => Some(head.trim().to_string()),
        (None, None) => None,
    }
}

fn git_name_only(repo_root: &Path, args: &[&str]) -> Result<Vec<String>, InventoryError> {
    Ok(git_stdout(repo_root, args)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect())
}

fn git_stdout(repo_root: &Path, args: &[&str]) -> Result<String, InventoryError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .output()
        .map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if matches!(args.first(), Some(&"log") | Some(&"rev-list"))
        && is_empty_git_history_message(&message)
    {
        return Ok(String::new());
    }
    Err(InventoryError::Git {
        repo_root: repo_root.to_path_buf(),
        message,
    })
}

pub(crate) fn git_stdout_with_timeout(
    repo_root: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<String, InventoryError> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait().map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })? {
            Some(_) => break,
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(InventoryError::Git {
                    repo_root: repo_root.to_path_buf(),
                    message: format!("git {:?} timed out after {:?}", args, timeout),
                });
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if matches!(args.first(), Some(&"log") | Some(&"rev-list"))
        && is_empty_git_history_message(&message)
    {
        return Ok(String::new());
    }
    Err(InventoryError::Git {
        repo_root: repo_root.to_path_buf(),
        message,
    })
}

fn git_stdout_bytes_with_timeout(
    repo_root: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<Vec<u8>, InventoryError> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["-c", "core.fsmonitor=false"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait().map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })? {
            Some(_) => break,
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(InventoryError::Git {
                    repo_root: repo_root.to_path_buf(),
                    message: format!("git {:?} timed out after {:?}", args, timeout),
                });
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    Err(InventoryError::Git {
        repo_root: repo_root.to_path_buf(),
        message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

fn parse_git_name_status_z(output: &[u8]) -> Vec<GitChangedPath> {
    let parts = output
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).replace('\\', "/"))
        .collect::<Vec<_>>();
    let mut files = Vec::new();
    let mut index = 0;
    while index < parts.len() {
        let status = parts[index].as_str();
        index += 1;
        let Some(first_path) = parts.get(index).cloned() else {
            break;
        };
        index += 1;
        let status_kind = status.chars().next().unwrap_or('U');
        if matches!(status_kind, 'R' | 'C') {
            let Some(new_path) = parts.get(index).cloned() else {
                break;
            };
            index += 1;
            files.push(GitChangedPath {
                path: new_path,
                old_path: Some(first_path),
                change_kind: change_kind_from_status(status_kind),
            });
        } else {
            files.push(GitChangedPath {
                path: first_path,
                old_path: None,
                change_kind: change_kind_from_status(status_kind),
            });
        }
    }
    files
}

fn change_kind_from_status(status: char) -> ChangeKind {
    match status {
        'A' => ChangeKind::Added,
        'M' => ChangeKind::Modified,
        'D' => ChangeKind::Deleted,
        'R' => ChangeKind::Renamed,
        'C' => ChangeKind::Copied,
        'T' => ChangeKind::TypeChanged,
        _ => ChangeKind::Unknown,
    }
}

fn is_empty_git_history_message(message: &str) -> bool {
    message.contains("does not have any commits yet")
        || message.contains("your current branch") && message.contains("does not have any commits")
        || message.contains("ambiguous argument 'HEAD'") && message.contains("unknown revision")
}

#[cfg(test)]
pub(crate) fn parse_git_log_name_only(output: &str) -> Vec<GitCommitFiles> {
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
pub(crate) fn parse_git_log_subject_name_only(output: &str) -> Vec<GitCommitSubjectFiles> {
    let mut commits = Vec::new();
    let mut current_sha: Option<String> = None;
    let mut current_title = String::new();
    let mut current_files = Vec::new();

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("commit:") {
            if let Some(previous_sha) = current_sha.take() {
                commits.push(GitCommitSubjectFiles {
                    sha: previous_sha,
                    parent_sha: None,
                    title: std::mem::take(&mut current_title),
                    files: std::mem::take(&mut current_files),
                });
            }
            let (sha, title) = rest.split_once('\0').unwrap_or((rest, ""));
            current_sha = Some(sha.to_string());
            current_title = title.trim().to_string();
            continue;
        }

        let path = line.trim();
        if path.is_empty() {
            continue;
        }
        current_files.push(GitChangedPath {
            path: path.replace('\\', "/"),
            old_path: None,
            change_kind: ChangeKind::Modified,
        });
    }

    if let Some(sha) = current_sha {
        commits.push(GitCommitSubjectFiles {
            sha,
            parent_sha: None,
            title: current_title,
            files: current_files,
        });
    }

    commits
}
