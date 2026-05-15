use crate::packs::pack_repo_id;
use crate::planning::{
    is_low_information_task, normalized_target_agent,
    prepare_context_plan_with_paths_history_and_semantic,
};
use ctxpack_core::{
    ContextPack, ContextPlan, EvalTrace, FileRole, PackBudget, PrivacyStatus, RetrievalSignalKind,
    TaskType,
};
use ctxpack_index::{
    historical_commit_samples, lexical_search, load_or_build_inventory, repo_id_for_path,
    task_hash, HistoricalChangedPath, HistoricalCommitOptions, InventoryError, InventoryOptions,
    LabelScope, SearchOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalEvalOptions {
    pub limit: usize,
    pub ranking_budget: usize,
    pub task_type: TaskType,
    pub target_agent: String,
    pub base: Option<String>,
    pub head: Option<String>,
    #[serde(default)]
    pub semantic_enabled: bool,
}

pub type HistoricalChangedPathLabel = HistoricalChangedPath;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalEvalRefs {
    pub base: Option<String>,
    pub head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalEvalEffectiveFilters {
    pub limit: usize,
    pub ranking_budget: usize,
    pub mode: TaskType,
    pub target_agent: String,
    pub budget: PackBudget,
    pub semantic_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvalComparison {
    pub k: usize,
    pub combined: RankingMetrics,
    pub lexical_baseline: RankingMetrics,
    pub no_context_baseline: RankingMetrics,
    pub recall_lift_at_k: f32,
    pub precision_lift_at_k: f32,
    pub mrr_lift_at_k: f32,
    pub recall_lift_vs_no_context_at_k: f32,
    pub precision_lift_vs_no_context_at_k: f32,
    pub mrr_lift_vs_no_context_at_k: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankingMetrics {
    pub k: usize,
    pub recall_at_k: f32,
    pub precision_at_k: f32,
    pub mrr_at_k: f32,
    pub role_recall: Vec<RoleRecallMetric>,
    pub test_recommendation_rate: f32,
    pub average_recommended_context_files: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoleRecallMetric {
    pub role: FileRole,
    pub recall_at_k: f32,
    pub changed_file_count: usize,
    pub hit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SignalAblationResult {
    pub eval_range_id: String,
    pub disabled_signal: RetrievalSignalKind,
    pub evaluated_commits: usize,
    pub metrics: RankingMetrics,
    pub recall_lift_vs_lexical_at_k: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenRoiMetric {
    pub budget: PackBudget,
    pub ranking_cutoff: usize,
    pub estimated_tokens: usize,
    pub useful_targets: usize,
    pub safe_targets: usize,
    pub useful_targets_per_1k_tokens: f32,
    pub recall_at_cutoff: f32,
    pub marginal_useful_targets_vs_previous_budget: isize,
    pub larger_pack_adds_little_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalGapSummary {
    pub role: FileRole,
    pub signal_gap: String,
    pub package: String,
    pub path_family: String,
    pub target_status: RetrievalGapTargetStatus,
    pub recommendation_area: RetrievalGapRecommendationArea,
    pub missed_count: usize,
    pub example_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum RetrievalGapTargetStatus {
    CurrentReachable,
    HistoricalRenamed,
    HistoricalDeleted,
    PolicyExcluded,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum RetrievalGapRecommendationArea {
    Storage,
    SemanticRetrieval,
    ParserPrecision,
    TestMapping,
    HistoryRanking,
    PolicyExclusion,
    LexicalRanking,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkComparisonReport {
    pub base_suite_id: String,
    pub head_suite_id: String,
    pub repository_count: usize,
    pub metric_deltas: Vec<BenchmarkMetricDelta>,
    pub gap_family_deltas: Vec<BenchmarkGapFamilyDelta>,
    pub threshold_checks: Vec<BenchmarkThresholdCheck>,
    pub passed: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkMetricDelta {
    pub repository: String,
    pub metric: String,
    pub base_value: f32,
    pub head_value: f32,
    pub delta: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkGapFamilyDelta {
    pub repository: String,
    pub role: FileRole,
    pub signal_gap: String,
    pub package: String,
    pub path_family: String,
    pub target_status: RetrievalGapTargetStatus,
    pub recommendation_area: RetrievalGapRecommendationArea,
    pub base_missed_count: usize,
    pub head_missed_count: usize,
    pub delta: isize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRegressionThreshold {
    pub metric: String,
    pub max_drop: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkThresholdCheck {
    pub repository: String,
    pub metric: String,
    pub max_drop: f32,
    pub delta: f32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofReport {
    pub suite_name: String,
    pub suite_id: String,
    pub evaluated_repository_count: usize,
    pub evaluated_commit_count: usize,
    pub headline_metrics: Vec<ProductProofMetric>,
    pub limitations: Vec<String>,
    pub helps_when: Vec<String>,
    pub does_not_help_when: Vec<String>,
    pub future_work: Vec<String>,
    pub benchmark_report: BenchmarkSuiteReport,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofMetric {
    pub label: String,
    pub value: f32,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalEvalReport {
    pub eval_range_id: String,
    pub repo_id: String,
    pub evaluated_commits: usize,
    pub budget: PackBudget,
    pub effective_filters: HistoricalEvalEffectiveFilters,
    pub refs: HistoricalEvalRefs,
    pub base: Option<String>,
    pub head: Option<String>,
    pub ranking_comparison: EvalComparison,
    pub signal_ablations: Vec<SignalAblationResult>,
    pub token_roi: Vec<TokenRoiMetric>,
    pub retrieval_gap_summaries: Vec<RetrievalGapSummary>,
    pub low_information_commit_count: usize,
    pub file_recall_at_5: f32,
    pub file_recall_at_10: f32,
    pub lexical_baseline_recall_at_5: f32,
    pub lexical_baseline_recall_at_10: f32,
    pub ctxpack_lift_at_5: f32,
    pub ctxpack_lift_at_10: f32,
    pub source_recall_at_5: f32,
    pub source_recall_at_10: f32,
    pub test_recall_at_5: f32,
    pub test_recall_at_10: f32,
    pub test_recommendation_rate: f32,
    pub average_recommended_context_files: f32,
    pub top_missing_files: Vec<HistoricalMissingFileSummary>,
    pub commits: Vec<HistoricalCommitEval>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteConfig {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub defaults: BenchmarkDefaults,
    pub repositories: Vec<BenchmarkRepoConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkDefaults {
    #[serde(default = "default_benchmark_limit")]
    pub limit: usize,
    #[serde(default = "default_benchmark_ranking_budget")]
    pub ranking_budget: usize,
    #[serde(default = "default_benchmark_task_type")]
    pub mode: TaskType,
    #[serde(default = "default_benchmark_target_agent")]
    pub target_agent: String,
    #[serde(default)]
    pub semantic_enabled: bool,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
}

impl Default for BenchmarkDefaults {
    fn default() -> Self {
        Self {
            limit: default_benchmark_limit(),
            ranking_budget: default_benchmark_ranking_budget(),
            mode: default_benchmark_task_type(),
            target_agent: default_benchmark_target_agent(),
            semantic_enabled: false,
            role_filters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoConfig {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub head: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub ranking_budget: Option<usize>,
    #[serde(default)]
    pub mode: Option<TaskType>,
    #[serde(default)]
    pub target_agent: Option<String>,
    #[serde(default)]
    pub semantic_enabled: Option<bool>,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteReport {
    pub suite_name: String,
    pub suite_id: String,
    pub description: Option<String>,
    pub generated_at_unix_seconds: u64,
    pub repository_count: usize,
    pub evaluated_repository_count: usize,
    pub evaluated_commit_count: usize,
    pub repositories: Vec<BenchmarkRepoReport>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoReport {
    pub name: String,
    pub repo_id: Option<String>,
    pub effective_config: BenchmarkRepoEffectiveConfig,
    pub evaluated_commits: usize,
    pub excluded_changed_file_count: usize,
    pub skipped_path_count: usize,
    pub report: Option<HistoricalEvalReport>,
    pub error: Option<String>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoEffectiveConfig {
    pub base: Option<String>,
    pub head: Option<String>,
    pub limit: usize,
    pub ranking_budget: usize,
    pub mode: TaskType,
    pub target_agent: String,
    pub semantic_enabled: bool,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalMissingFileSummary {
    pub path: String,
    pub role: FileRole,
    pub missed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalCommitEval {
    pub sha: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub target_agent: String,
    pub changed_path_labels: Vec<HistoricalChangedPathLabel>,
    pub safe_changed_files: Vec<String>,
    pub excluded_changed_file_count: usize,
    pub recommended_files: Vec<String>,
    pub recommended_tests: Vec<String>,
    pub recommended_context_files: Vec<String>,
    pub recommended_commands: Vec<String>,
    pub lexical_baseline_files: Vec<String>,
    pub file_hits_at_5: Vec<String>,
    pub file_hits_at_10: Vec<String>,
    pub lexical_baseline_hits_at_5: Vec<String>,
    pub lexical_baseline_hits_at_10: Vec<String>,
    pub missing_files_at_10: Vec<String>,
    pub source_files_changed: usize,
    pub source_hits_at_5: usize,
    pub source_hits_at_10: usize,
    pub test_files_changed: usize,
    pub test_hits_at_5: usize,
    pub test_hits_at_10: usize,
    pub low_information_task: bool,
    pub confidence: f32,
    pub source_text_logged: bool,
}

pub fn load_benchmark_suite_config(
    config_path: impl AsRef<Path>,
) -> Result<BenchmarkSuiteConfig, InventoryError> {
    let config_path = config_path.as_ref();
    let content = fs::read_to_string(config_path).map_err(|source| InventoryError::Read {
        path: config_path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| InventoryError::Deserialize {
        path: config_path.to_path_buf(),
        source,
    })
}

pub fn load_benchmark_suite_report(
    report_path: impl AsRef<Path>,
) -> Result<BenchmarkSuiteReport, InventoryError> {
    let report_path = report_path.as_ref();
    let content = fs::read_to_string(report_path).map_err(|source| InventoryError::Read {
        path: report_path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| InventoryError::Deserialize {
        path: report_path.to_path_buf(),
        source,
    })
}

pub fn run_benchmark_suite(
    config_path: impl AsRef<Path>,
) -> Result<BenchmarkSuiteReport, InventoryError> {
    let config_path = config_path.as_ref();
    let config = load_benchmark_suite_config(config_path)?;
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    run_benchmark_suite_config(&config, config_dir)
}

pub fn compare_benchmark_suite_reports(
    base: &BenchmarkSuiteReport,
    head: &BenchmarkSuiteReport,
    thresholds: &[BenchmarkRegressionThreshold],
) -> BenchmarkComparisonReport {
    let mut metric_deltas = Vec::new();
    let mut gap_family_deltas = Vec::new();
    let mut threshold_checks = Vec::new();

    for head_repo in &head.repositories {
        let Some(base_repo) = base
            .repositories
            .iter()
            .find(|repo| repo.name == head_repo.name)
        else {
            continue;
        };
        metric_deltas.extend(repo_metric_deltas(base_repo, head_repo));
        gap_family_deltas.extend(repo_gap_family_deltas(base_repo, head_repo));
    }

    for threshold in thresholds {
        for delta in metric_deltas
            .iter()
            .filter(|delta| delta.metric == threshold.metric)
        {
            threshold_checks.push(BenchmarkThresholdCheck {
                repository: delta.repository.clone(),
                metric: threshold.metric.clone(),
                max_drop: threshold.max_drop,
                delta: delta.delta,
                passed: delta.delta >= -threshold.max_drop,
            });
        }
    }

    let passed = threshold_checks.iter().all(|check| check.passed);
    BenchmarkComparisonReport {
        base_suite_id: base.suite_id.clone(),
        head_suite_id: head.suite_id.clone(),
        repository_count: head.repositories.len(),
        metric_deltas,
        gap_family_deltas,
        threshold_checks,
        passed,
        privacy_status: PrivacyStatus::local_only(),
    }
}

pub fn build_product_proof_report(benchmark_report: BenchmarkSuiteReport) -> ProductProofReport {
    let evaluated_reports = benchmark_report
        .repositories
        .iter()
        .filter_map(|repo| repo.report.as_ref())
        .collect::<Vec<_>>();
    let repo_count = evaluated_reports.len().max(1) as f32;
    let average_file_recall_at_10 = evaluated_reports
        .iter()
        .map(|report| report.file_recall_at_10)
        .sum::<f32>()
        / repo_count;
    let average_lexical_recall_at_10 = evaluated_reports
        .iter()
        .map(|report| report.lexical_baseline_recall_at_10)
        .sum::<f32>()
        / repo_count;
    let average_lift_at_10 = evaluated_reports
        .iter()
        .map(|report| report.ctxpack_lift_at_10)
        .sum::<f32>()
        / repo_count;
    let average_test_recall_at_10 = evaluated_reports
        .iter()
        .map(|report| report.test_recall_at_10)
        .sum::<f32>()
        / repo_count;
    let average_brief_token_roi = evaluated_reports
        .iter()
        .map(|report| token_roi_value(report, &PackBudget::Brief))
        .sum::<f32>()
        / repo_count;

    ProductProofReport {
        suite_name: benchmark_report.suite_name.clone(),
        suite_id: benchmark_report.suite_id.clone(),
        evaluated_repository_count: benchmark_report.evaluated_repository_count,
        evaluated_commit_count: benchmark_report.evaluated_commit_count,
        headline_metrics: vec![
            ProductProofMetric {
                label: "averageFileRecallAt10".to_string(),
                value: average_file_recall_at_10,
                unit: "ratio".to_string(),
            },
            ProductProofMetric {
                label: "averageLexicalBaselineRecallAt10".to_string(),
                value: average_lexical_recall_at_10,
                unit: "ratio".to_string(),
            },
            ProductProofMetric {
                label: "averageCtxpackLiftAt10".to_string(),
                value: average_lift_at_10,
                unit: "ratio".to_string(),
            },
            ProductProofMetric {
                label: "averageTestRecallAt10".to_string(),
                value: average_test_recall_at_10,
                unit: "ratio".to_string(),
            },
            ProductProofMetric {
                label: "averageBriefTokenRoi".to_string(),
                value: average_brief_token_roi,
                unit: "useful_targets_per_1k_tokens".to_string(),
            },
        ],
        limitations: vec![
            "Historical commit subjects are only proxies for real developer prompts.".to_string(),
            "No-context baseline is zero-file until editor anchor traces are available."
                .to_string(),
            "Token ROI is estimated from budget presets, not measured model billing.".to_string(),
        ],
        helps_when: vec![
            "Tasks require choosing target files and tests across a repository.".to_string(),
            "Exact identifiers, related tests, history, and graph signals can reinforce each other."
                .to_string(),
            "Maintainers need source-free evidence for context quality over time.".to_string(),
        ],
        does_not_help_when: vec![
            "The task is a trivial single-file edit with an explicit path.".to_string(),
            "The right target file is absent from the safe local inventory.".to_string(),
            "The benchmark range contains low-information commit messages.".to_string(),
        ],
        future_work: vec![
            "v1.3: persist benchmark results in local storage for trend history.".to_string(),
            "v1.4: add optional local semantic retrieval where gaps justify it.".to_string(),
            "v1.5: add parser and SCIP/LSP precision where gap families point to it.".to_string(),
        ],
        benchmark_report,
        privacy_status: PrivacyStatus::local_only(),
    }
}

pub fn run_benchmark_suite_config(
    config: &BenchmarkSuiteConfig,
    config_dir: &Path,
) -> Result<BenchmarkSuiteReport, InventoryError> {
    let mut repositories = Vec::new();

    for repo_config in &config.repositories {
        repositories.push(run_benchmark_repo(
            config_dir,
            &config.defaults,
            repo_config,
        ));
    }

    let evaluated_repository_count = repositories
        .iter()
        .filter(|repo| repo.error.is_none())
        .count();
    let evaluated_commit_count = repositories
        .iter()
        .map(|repo| repo.evaluated_commits)
        .sum::<usize>();
    let suite_id = benchmark_suite_id(config, &repositories);

    Ok(BenchmarkSuiteReport {
        suite_name: config.name.clone(),
        suite_id,
        description: config.description.clone(),
        generated_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        repository_count: config.repositories.len(),
        evaluated_repository_count,
        evaluated_commit_count,
        repositories,
        privacy_status: PrivacyStatus::local_only(),
    })
}

fn run_benchmark_repo(
    config_dir: &Path,
    defaults: &BenchmarkDefaults,
    repo_config: &BenchmarkRepoConfig,
) -> BenchmarkRepoReport {
    let effective_config = BenchmarkRepoEffectiveConfig {
        base: repo_config.base.clone(),
        head: repo_config.head.clone(),
        limit: repo_config.limit.unwrap_or(defaults.limit).max(1),
        ranking_budget: repo_config
            .ranking_budget
            .unwrap_or(defaults.ranking_budget)
            .max(1),
        mode: repo_config
            .mode
            .clone()
            .unwrap_or_else(|| defaults.mode.clone()),
        target_agent: repo_config
            .target_agent
            .clone()
            .unwrap_or_else(|| defaults.target_agent.clone()),
        semantic_enabled: repo_config
            .semantic_enabled
            .unwrap_or(defaults.semantic_enabled),
        role_filters: if repo_config.role_filters.is_empty() {
            defaults.role_filters.clone()
        } else {
            repo_config.role_filters.clone()
        },
    };
    let repo_path = resolve_benchmark_repo_path(config_dir, &repo_config.path);
    let options = HistoricalEvalOptions {
        limit: effective_config.limit,
        ranking_budget: effective_config.ranking_budget,
        task_type: effective_config.mode.clone(),
        target_agent: effective_config.target_agent.clone(),
        base: effective_config.base.clone(),
        head: effective_config.head.clone(),
        semantic_enabled: effective_config.semantic_enabled,
    };

    match evaluate_historical_commits(&repo_path, &options) {
        Ok(report) => {
            let excluded_changed_file_count = report
                .commits
                .iter()
                .map(|commit| commit.excluded_changed_file_count)
                .sum::<usize>();
            let skipped_path_count = report
                .commits
                .iter()
                .flat_map(|commit| commit.changed_path_labels.iter())
                .filter(|label| label.label_scope != LabelScope::Safe)
                .count();
            BenchmarkRepoReport {
                name: repo_config.name.clone(),
                repo_id: Some(report.repo_id.clone()),
                effective_config,
                evaluated_commits: report.evaluated_commits,
                excluded_changed_file_count,
                skipped_path_count,
                report: Some(report),
                error: None,
                privacy_status: PrivacyStatus::local_only(),
            }
        }
        Err(error) => BenchmarkRepoReport {
            name: repo_config.name.clone(),
            repo_id: None,
            effective_config,
            evaluated_commits: 0,
            excluded_changed_file_count: 0,
            skipped_path_count: 0,
            report: None,
            error: Some(error.to_string()),
            privacy_status: PrivacyStatus::local_only(),
        },
    }
}

fn repo_metric_deltas(
    base_repo: &BenchmarkRepoReport,
    head_repo: &BenchmarkRepoReport,
) -> Vec<BenchmarkMetricDelta> {
    let repository = head_repo.name.clone();
    let mut deltas = Vec::new();
    deltas.push(metric_delta(
        &repository,
        "skippedPathCount",
        base_repo.skipped_path_count as f32,
        head_repo.skipped_path_count as f32,
    ));
    deltas.push(metric_delta(
        &repository,
        "excludedChangedFileCount",
        base_repo.excluded_changed_file_count as f32,
        head_repo.excluded_changed_file_count as f32,
    ));

    let (Some(base), Some(head)) = (&base_repo.report, &head_repo.report) else {
        return deltas;
    };
    for (metric, base_value, head_value) in [
        (
            "fileRecallAt10",
            base.file_recall_at_10,
            head.file_recall_at_10,
        ),
        (
            "testRecallAt10",
            base.test_recall_at_10,
            head.test_recall_at_10,
        ),
        (
            "ctxpackLiftAt10",
            base.ctxpack_lift_at_10,
            head.ctxpack_lift_at_10,
        ),
        (
            "recallLiftVsNoContextAtK",
            base.ranking_comparison.recall_lift_vs_no_context_at_k,
            head.ranking_comparison.recall_lift_vs_no_context_at_k,
        ),
    ] {
        deltas.push(metric_delta(&repository, metric, base_value, head_value));
    }
    for budget in [PackBudget::Brief, PackBudget::Standard, PackBudget::Deep] {
        let metric = format!("tokenRoi{:?}", budget);
        deltas.push(metric_delta(
            &repository,
            &metric,
            token_roi_value(base, &budget),
            token_roi_value(head, &budget),
        ));
    }
    deltas
}

fn metric_delta(
    repository: &str,
    metric: &str,
    base_value: f32,
    head_value: f32,
) -> BenchmarkMetricDelta {
    BenchmarkMetricDelta {
        repository: repository.to_string(),
        metric: metric.to_string(),
        base_value,
        head_value,
        delta: head_value - base_value,
    }
}

fn token_roi_value(report: &HistoricalEvalReport, budget: &PackBudget) -> f32 {
    report
        .token_roi
        .iter()
        .find(|row| &row.budget == budget)
        .map(|row| row.useful_targets_per_1k_tokens)
        .unwrap_or_default()
}

fn repo_gap_family_deltas(
    base_repo: &BenchmarkRepoReport,
    head_repo: &BenchmarkRepoReport,
) -> Vec<BenchmarkGapFamilyDelta> {
    let repository = head_repo.name.clone();
    let base = gap_family_counts(base_repo);
    let head = gap_family_counts(head_repo);
    let mut keys = base.keys().cloned().collect::<BTreeSet<_>>();
    keys.extend(head.keys().cloned());
    keys.into_iter()
        .map(|key| {
            let gap = base
                .get(&key)
                .map(|entry| &entry.0)
                .or_else(|| head.get(&key).map(|entry| &entry.0))
                .expect("gap family key should come from base or head");
            let base_missed_count = base.get(&key).map(|entry| entry.1).unwrap_or(0);
            let head_missed_count = head.get(&key).map(|entry| entry.1).unwrap_or(0);
            BenchmarkGapFamilyDelta {
                repository: repository.clone(),
                role: gap.role.clone(),
                signal_gap: gap.signal_gap.clone(),
                package: gap.package.clone(),
                path_family: gap.path_family.clone(),
                target_status: gap.target_status.clone(),
                recommendation_area: gap.recommendation_area.clone(),
                base_missed_count,
                head_missed_count,
                delta: head_missed_count as isize - base_missed_count as isize,
            }
        })
        .collect()
}

fn gap_family_counts(repo: &BenchmarkRepoReport) -> BTreeMap<String, (RetrievalGapSummary, usize)> {
    let Some(report) = &repo.report else {
        return BTreeMap::new();
    };
    report
        .retrieval_gap_summaries
        .iter()
        .map(|gap| (gap_family_key(gap), (gap.clone(), gap.missed_count)))
        .collect()
}

fn gap_family_key(gap: &RetrievalGapSummary) -> String {
    format!(
        "{:?}|{}|{}|{}|{:?}|{:?}",
        gap.role,
        gap.signal_gap,
        gap.package,
        gap.path_family,
        gap.target_status,
        gap.recommendation_area
    )
}

fn resolve_benchmark_repo_path(config_dir: &Path, repo_path: &Path) -> PathBuf {
    if repo_path.is_absolute() {
        repo_path.to_path_buf()
    } else {
        config_dir.join(repo_path)
    }
}

fn benchmark_suite_id(
    config: &BenchmarkSuiteConfig,
    repositories: &[BenchmarkRepoReport],
) -> String {
    let mut input = format!("suite={}\n", config.name);
    for repo in repositories {
        input.push_str(&format!(
            "repo={}\nrepoId={}\nbase={}\nhead={}\nlimit={}\nrankingBudget={}\nmode={:?}\ntarget={}\nroles={:?}\ncommits={}\nerror={}\n",
            repo.name,
            repo.repo_id.as_deref().unwrap_or(""),
            repo.effective_config.base.as_deref().unwrap_or(""),
            repo.effective_config.head.as_deref().unwrap_or(""),
            repo.effective_config.limit,
            repo.effective_config.ranking_budget,
            repo.effective_config.mode,
            repo.effective_config.target_agent,
            repo.effective_config.role_filters,
            repo.evaluated_commits,
            repo.error.as_deref().unwrap_or("")
        ));
    }
    task_hash(&input)
}

fn default_benchmark_limit() -> usize {
    20
}

fn default_benchmark_ranking_budget() -> usize {
    10
}

fn default_benchmark_task_type() -> TaskType {
    TaskType::BugFix
}

fn default_benchmark_target_agent() -> String {
    "generic".to_string()
}

pub fn evaluate_historical_commits(
    repo_root: impl AsRef<Path>,
    options: &HistoricalEvalOptions,
) -> Result<HistoricalEvalReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let inventory = load_or_build_inventory(repo_root, &InventoryOptions::default())?;
    let snapshot_paths = inventory
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let roles_by_path = inventory
        .files
        .iter()
        .map(|file| (file.path.clone(), file.role.clone()))
        .collect::<BTreeMap<_, _>>();
    let samples = historical_commit_samples(
        repo_root,
        &HistoricalCommitOptions {
            limit: options.limit,
            base: options.base.clone(),
            head: options.head.clone(),
        },
    )?;
    let target_agent = normalized_target_agent(&options.target_agent);
    let budget = PackBudget::Standard;
    let ranking_budget = options.ranking_budget.max(1);
    let repo_id = pack_repo_id(repo_root);
    let mut commits = Vec::new();
    let mut ablation_rankings = initial_ablation_rankings();
    let mut gap_reasons_by_commit = Vec::new();

    for sample in samples {
        let changed_path_labels = sample.changed_paths.clone();
        let task = if sample.title.trim().is_empty() {
            format!("change {}", sample.sha)
        } else {
            sample.title.clone()
        };
        let eval_repo = HistoricalEvalWorktree::for_parent(
            repo_root,
            sample.parent_sha.as_deref(),
            &snapshot_paths,
        )?;
        let eval_root = eval_repo.path();
        let plan = prepare_context_plan_with_paths_history_and_semantic(
            eval_root,
            &task,
            options.task_type.clone(),
            &[],
            false,
            options.semantic_enabled,
        )?;
        let signals_by_path = signals_by_path(&plan);
        let recommended_files = plan
            .target_files
            .iter()
            .map(|target| target.path.clone())
            .collect::<Vec<_>>();
        let recommended_tests = plan
            .related_tests
            .iter()
            .map(|test| test.path.clone())
            .collect::<Vec<_>>();
        let recommended_commands = plan
            .recommended_commands
            .iter()
            .map(|command| command.command.clone())
            .collect::<Vec<_>>();
        let recommended_context_files =
            context_file_ranking(&recommended_files, &recommended_tests, ranking_budget);
        let lexical_baseline_files =
            lexical_baseline_context_files(eval_root, &task, ranking_budget)?;
        let file_hits_at_5 =
            changed_file_hits(&sample.safe_changed_files, &recommended_context_files, 5);
        let file_hits_at_10 =
            changed_file_hits(&sample.safe_changed_files, &recommended_context_files, 10);
        let lexical_baseline_hits_at_5 =
            changed_file_hits(&sample.safe_changed_files, &lexical_baseline_files, 5);
        let lexical_baseline_hits_at_10 =
            changed_file_hits(&sample.safe_changed_files, &lexical_baseline_files, 10);
        let missing_files_at_10 =
            missing_changed_files(&sample.safe_changed_files, &recommended_context_files, 10);
        for signal in ablation_signals() {
            let ranking =
                ablated_context_ranking(&recommended_context_files, &signals_by_path, &signal);
            if let Some(rankings) = ablation_rankings
                .iter_mut()
                .find(|entry| entry.disabled_signal == signal)
            {
                rankings.rankings.push(ranking);
            }
        }
        gap_reasons_by_commit.push(retrieval_gap_reasons(
            &missing_files_at_10,
            &lexical_baseline_files,
            &signals_by_path,
        ));
        let source_changed_files = filter_changed_labels_by_role(
            &changed_path_labels,
            &sample.safe_changed_files,
            |role| matches!(role, FileRole::Source),
        );
        let test_changed_files = filter_changed_labels_by_role(
            &changed_path_labels,
            &sample.safe_changed_files,
            |role| matches!(role, FileRole::Test),
        );
        let source_hits_at_5 =
            changed_file_hits(&source_changed_files, &recommended_context_files, 5).len();
        let source_hits_at_10 =
            changed_file_hits(&source_changed_files, &recommended_context_files, 10).len();
        let test_hits_at_5 =
            changed_file_hits(&test_changed_files, &recommended_context_files, 5).len();
        let test_hits_at_10 =
            changed_file_hits(&test_changed_files, &recommended_context_files, 10).len();

        commits.push(HistoricalCommitEval {
            sha: sample.sha,
            task_hash: task_hash(&task),
            task_type: options.task_type.clone(),
            target_agent: target_agent.clone(),
            changed_path_labels,
            safe_changed_files: sample.safe_changed_files,
            excluded_changed_file_count: sample.excluded_changed_file_count,
            recommended_files,
            recommended_tests,
            recommended_context_files,
            recommended_commands,
            lexical_baseline_files,
            file_hits_at_5,
            file_hits_at_10,
            lexical_baseline_hits_at_5,
            lexical_baseline_hits_at_10,
            missing_files_at_10,
            source_files_changed: source_changed_files.len(),
            source_hits_at_5,
            source_hits_at_10,
            test_files_changed: test_changed_files.len(),
            test_hits_at_5,
            test_hits_at_10,
            low_information_task: is_low_information_task(&task),
            confidence: plan.confidence,
            source_text_logged: false,
        });
    }

    let file_recall_at_5 = average_recall(&commits, 5);
    let file_recall_at_10 = average_recall(&commits, 10);
    let lexical_baseline_recall_at_5 = average_lexical_baseline_recall(&commits, 5);
    let lexical_baseline_recall_at_10 = average_lexical_baseline_recall(&commits, 10);
    let refs = HistoricalEvalRefs {
        base: options.base.clone(),
        head: options.head.clone(),
    };
    let effective_filters = HistoricalEvalEffectiveFilters {
        limit: options.limit,
        ranking_budget,
        mode: options.task_type.clone(),
        target_agent: target_agent.clone(),
        budget: budget.clone(),
        semantic_enabled: options.semantic_enabled,
    };
    let eval_range_id = historical_eval_range_id(&repo_id, &effective_filters, &refs);
    let roles_by_label_path = roles_by_path_from_labels(&commits, &roles_by_path);
    let labels_by_path = labels_by_path_from_commits(&commits);

    let ranking_comparison = eval_comparison(&commits, ranking_budget);
    let signal_ablations = signal_ablation_results(
        &commits,
        &ablation_rankings,
        &ranking_comparison.lexical_baseline,
        &eval_range_id,
        ranking_budget,
    );
    let token_roi = token_roi_metrics(&commits, ranking_budget);
    let retrieval_gap_summaries = retrieval_gap_summaries(
        &commits,
        &gap_reasons_by_commit,
        &roles_by_label_path,
        &labels_by_path,
        10,
    );

    Ok(HistoricalEvalReport {
        eval_range_id,
        repo_id,
        evaluated_commits: commits.len(),
        budget,
        effective_filters,
        refs,
        base: options.base.clone(),
        head: options.head.clone(),
        ranking_comparison,
        signal_ablations,
        token_roi,
        retrieval_gap_summaries,
        low_information_commit_count: commits
            .iter()
            .filter(|commit| commit.low_information_task)
            .count(),
        file_recall_at_5,
        file_recall_at_10,
        lexical_baseline_recall_at_5,
        lexical_baseline_recall_at_10,
        ctxpack_lift_at_5: file_recall_at_5 - lexical_baseline_recall_at_5,
        ctxpack_lift_at_10: file_recall_at_10 - lexical_baseline_recall_at_10,
        source_recall_at_5: average_role_recall(&commits, FileRole::Source, 5),
        source_recall_at_10: average_role_recall(&commits, FileRole::Source, 10),
        test_recall_at_5: average_role_recall(&commits, FileRole::Test, 5),
        test_recall_at_10: average_role_recall(&commits, FileRole::Test, 10),
        test_recommendation_rate: test_recommendation_rate(&commits),
        average_recommended_context_files: average_recommended_context_files(&commits),
        top_missing_files: top_missing_files(&commits, &roles_by_label_path, 10),
        commits,
        privacy_status: PrivacyStatus::local_only(),
    })
}

pub(crate) struct HistoricalEvalWorktree<'a> {
    path: PathBuf,
    _source_repo: &'a Path,
    _temp_dir: Option<tempfile::TempDir>,
}

impl<'a> HistoricalEvalWorktree<'a> {
    pub(crate) fn for_parent(
        source_repo: &'a Path,
        parent_sha: Option<&str>,
        snapshot_paths: &[String],
    ) -> Result<Self, InventoryError> {
        let Some(parent_sha) = parent_sha.filter(|sha| !sha.trim().is_empty()) else {
            return Ok(Self {
                path: source_repo.to_path_buf(),
                _source_repo: source_repo,
                _temp_dir: None,
            });
        };

        let temp_dir = tempfile::Builder::new()
            .prefix("ctxpack-historical-eval-")
            .tempdir()
            .map_err(|source| InventoryError::CreateDir {
                path: std::env::temp_dir(),
                source,
            })?;
        let path = temp_dir.path().join("repo");
        fs::create_dir_all(&path).map_err(|source| InventoryError::CreateDir {
            path: path.clone(),
            source,
        })?;
        let revision_paths =
            git_existing_paths_at_revision(source_repo, parent_sha, snapshot_paths)?;
        git_extract_revision_paths(source_repo, parent_sha, &revision_paths, &path)?;

        Ok(Self {
            path,
            _source_repo: source_repo,
            _temp_dir: Some(temp_dir),
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

fn git_existing_paths_at_revision(
    source_repo: &Path,
    revision: &str,
    candidate_paths: &[String],
) -> Result<Vec<String>, InventoryError> {
    if candidate_paths.is_empty() {
        return Ok(Vec::new());
    }

    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(source_repo)
        .args(["ls-tree", "-r", "--name-only", revision])
        .output()
        .map_err(|source| InventoryError::Git {
            repo_root: source_repo.to_path_buf(),
            message: source.to_string(),
        })?;
    if !output.status.success() {
        return Err(InventoryError::Git {
            repo_root: source_repo.to_path_buf(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    let existing_paths = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    Ok(candidate_paths
        .iter()
        .filter(|path| existing_paths.contains(path.as_str()))
        .cloned()
        .collect())
}

fn git_extract_revision_paths(
    source_repo: &Path,
    revision: &str,
    paths: &[String],
    destination: &Path,
) -> Result<(), InventoryError> {
    for chunk in paths.chunks(200) {
        if chunk.is_empty() {
            continue;
        }
        let mut archive = ProcessCommand::new("git")
            .arg("-C")
            .arg(source_repo)
            .args(["archive", "--format=tar", revision, "--"])
            .args(chunk)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|source| InventoryError::Git {
                repo_root: source_repo.to_path_buf(),
                message: source.to_string(),
            })?;
        let archive_stdout = archive.stdout.take().ok_or_else(|| InventoryError::Git {
            repo_root: source_repo.to_path_buf(),
            message: "failed to capture git archive output".to_string(),
        })?;
        let tar_output = ProcessCommand::new("tar")
            .args(["-xf", "-", "-C"])
            .arg(destination)
            .stdin(Stdio::from(archive_stdout))
            .output()
            .map_err(|source| InventoryError::Git {
                repo_root: source_repo.to_path_buf(),
                message: source.to_string(),
            })?;
        let archive_status = archive.wait().map_err(|source| InventoryError::Git {
            repo_root: source_repo.to_path_buf(),
            message: source.to_string(),
        })?;
        if !archive_status.success() {
            return Err(InventoryError::Git {
                repo_root: source_repo.to_path_buf(),
                message: format!("git archive failed for revision {revision}"),
            });
        }
        if !tar_output.status.success() {
            return Err(InventoryError::Git {
                repo_root: source_repo.to_path_buf(),
                message: String::from_utf8_lossy(&tar_output.stderr)
                    .trim()
                    .to_string(),
            });
        }
    }
    Ok(())
}

pub fn eval_trace_for_plan(
    repo_root: impl AsRef<Path>,
    task: &str,
    target_agent: &str,
    plan: &ContextPlan,
) -> EvalTrace {
    eval_trace(repo_root.as_ref(), task, target_agent, plan, None, None)
}

pub fn eval_trace_for_pack(
    repo_root: impl AsRef<Path>,
    task: &str,
    target_agent: &str,
    plan: &ContextPlan,
    pack: &ContextPack,
) -> EvalTrace {
    eval_trace(
        repo_root.as_ref(),
        task,
        target_agent,
        plan,
        Some(pack.id),
        Some(pack.budget.clone()),
    )
}

fn eval_trace(
    repo_root: &Path,
    task: &str,
    target_agent: &str,
    plan: &ContextPlan,
    pack_id: Option<Uuid>,
    budget: Option<PackBudget>,
) -> EvalTrace {
    EvalTrace {
        id: Uuid::new_v4(),
        repo_id: repo_id_for_path(repo_root),
        task_hash: task_hash(task),
        task_type: plan.task_type.clone(),
        pack_id,
        target_agent: normalized_target_agent(target_agent),
        budget,
        recommended_files: plan
            .target_files
            .iter()
            .map(|target| target.path.clone())
            .collect(),
        recommended_tests: plan
            .related_tests
            .iter()
            .map(|test| test.path.clone())
            .collect(),
        recommended_commands: plan
            .recommended_commands
            .iter()
            .map(|command| command.command.clone())
            .collect(),
        created_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        source_text_logged: false,
    }
}

fn context_file_ranking(
    recommended_files: &[String],
    recommended_tests: &[String],
    ranking_budget: usize,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    recommended_files
        .iter()
        .chain(recommended_tests.iter())
        .filter_map(|path| seen.insert(path.clone()).then_some(path.clone()))
        .take(ranking_budget.max(1))
        .collect()
}

fn lexical_baseline_context_files(
    repo_root: &Path,
    task: &str,
    ranking_budget: usize,
) -> Result<Vec<String>, InventoryError> {
    let results = lexical_search(
        repo_root,
        task,
        &SearchOptions {
            limit: ranking_budget.max(1),
        },
    )?;
    Ok(results
        .into_iter()
        .map(|result| result.path)
        .collect::<Vec<_>>())
}

fn changed_file_hits(
    safe_changed_files: &[String],
    recommended_context_files: &[String],
    limit: usize,
) -> Vec<String> {
    let recommended = recommended_context_files
        .iter()
        .take(limit)
        .cloned()
        .collect::<BTreeSet<_>>();
    safe_changed_files
        .iter()
        .filter(|path| recommended.contains(*path))
        .cloned()
        .collect()
}

fn missing_changed_files(
    safe_changed_files: &[String],
    recommended_context_files: &[String],
    limit: usize,
) -> Vec<String> {
    let recommended = recommended_context_files
        .iter()
        .take(limit)
        .cloned()
        .collect::<BTreeSet<_>>();
    safe_changed_files
        .iter()
        .filter(|path| !recommended.contains(*path))
        .cloned()
        .collect()
}

fn filter_changed_labels_by_role(
    labels: &[HistoricalChangedPathLabel],
    safe_changed_files: &[String],
    predicate: impl Fn(&FileRole) -> bool,
) -> Vec<String> {
    let safe_paths = safe_changed_files.iter().collect::<BTreeSet<_>>();
    labels
        .iter()
        .filter(|label| {
            label.label_scope == LabelScope::Safe
                && safe_paths.contains(&label.path)
                && predicate(&label.role)
        })
        .map(|label| label.path.clone())
        .collect()
}

fn roles_by_path_from_labels(
    commits: &[HistoricalCommitEval],
    fallback: &BTreeMap<String, FileRole>,
) -> BTreeMap<String, FileRole> {
    let mut roles = fallback.clone();
    for commit in commits {
        for label in &commit.changed_path_labels {
            roles
                .entry(label.path.clone())
                .or_insert_with(|| label.role.clone());
        }
    }
    roles
}

fn historical_eval_range_id(
    repo_id: &str,
    filters: &HistoricalEvalEffectiveFilters,
    refs: &HistoricalEvalRefs,
) -> String {
    task_hash(&format!(
        "repo={repo_id}\nlimit={}\nrankingBudget={}\nmode={:?}\ntarget={}\nbudget={:?}\nsemantic={}\nbase={}\nhead={}",
        filters.limit,
        filters.ranking_budget,
        filters.mode,
        filters.target_agent,
        filters.budget,
        filters.semantic_enabled,
        refs.base.as_deref().unwrap_or(""),
        refs.head.as_deref().unwrap_or("")
    ))
}

#[derive(Debug, Clone)]
struct SignalAblationRankings {
    disabled_signal: RetrievalSignalKind,
    rankings: Vec<Vec<String>>,
}

fn initial_ablation_rankings() -> Vec<SignalAblationRankings> {
    ablation_signals()
        .into_iter()
        .map(|disabled_signal| SignalAblationRankings {
            disabled_signal,
            rankings: Vec::new(),
        })
        .collect()
}

fn ablation_signals() -> Vec<RetrievalSignalKind> {
    vec![
        RetrievalSignalKind::Lexical,
        RetrievalSignalKind::Semantic,
        RetrievalSignalKind::Symbol,
        RetrievalSignalKind::Dependency,
        RetrievalSignalKind::RelatedTest,
        RetrievalSignalKind::CoChange,
        RetrievalSignalKind::CurrentDiff,
        RetrievalSignalKind::Anchor,
    ]
}

fn signals_by_path(plan: &ContextPlan) -> BTreeMap<String, Vec<RetrievalSignalKind>> {
    let mut signals = BTreeMap::<String, Vec<RetrievalSignalKind>>::new();
    for candidate in &plan.retrieval_candidates {
        let Some(path) = &candidate.path else {
            continue;
        };
        for signal in candidate
            .signal_scores
            .iter()
            .map(|score| score.signal.clone())
        {
            push_signal(signals.entry(path.clone()).or_default(), signal);
        }
        for signal in candidate
            .evidence
            .iter()
            .map(|evidence| evidence.signal.clone())
        {
            push_signal(signals.entry(path.clone()).or_default(), signal);
        }
    }
    for target in &plan.target_files {
        for signal in target
            .attribution
            .iter()
            .map(|evidence| evidence.signal.clone())
        {
            push_signal(signals.entry(target.path.clone()).or_default(), signal);
        }
    }
    for test in &plan.related_tests {
        let entry = signals.entry(test.path.clone()).or_default();
        push_signal(entry, RetrievalSignalKind::RelatedTest);
        for signal in test
            .attribution
            .iter()
            .map(|evidence| evidence.signal.clone())
        {
            push_signal(signals.entry(test.path.clone()).or_default(), signal);
        }
    }
    signals
}

fn push_signal(signals: &mut Vec<RetrievalSignalKind>, signal: RetrievalSignalKind) {
    if !signals.contains(&signal) {
        signals.push(signal);
    }
}

fn ablated_context_ranking(
    ranking: &[String],
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
    disabled_signal: &RetrievalSignalKind,
) -> Vec<String> {
    ranking
        .iter()
        .filter(|path| {
            let Some(signals) = signals_by_path.get(path.as_str()) else {
                return true;
            };
            !(signals.len() == 1 && signals.contains(disabled_signal))
        })
        .cloned()
        .collect()
}

fn signal_ablation_results(
    commits: &[HistoricalCommitEval],
    ablation_rankings: &[SignalAblationRankings],
    lexical_baseline: &RankingMetrics,
    eval_range_id: &str,
    k: usize,
) -> Vec<SignalAblationResult> {
    ablation_rankings
        .iter()
        .map(|ablation| {
            let mut ablated_commits = commits.to_vec();
            for (commit, ranking) in ablated_commits
                .iter_mut()
                .zip(ablation.rankings.iter().cloned())
            {
                commit.recommended_context_files = ranking;
            }
            let metrics = ranking_metrics(&ablated_commits, k, RankingFamily::Combined);
            SignalAblationResult {
                eval_range_id: eval_range_id.to_string(),
                disabled_signal: ablation.disabled_signal.clone(),
                evaluated_commits: commits.len(),
                recall_lift_vs_lexical_at_k: metrics.recall_at_k - lexical_baseline.recall_at_k,
                metrics,
            }
        })
        .collect()
}

fn average_recall(commits: &[HistoricalCommitEval], limit: usize) -> f32 {
    if commits.is_empty() {
        return 0.0;
    }

    let total = commits
        .iter()
        .map(|commit| {
            if commit.safe_changed_files.is_empty() {
                0.0
            } else {
                let hit_count = if limit <= 5 {
                    commit.file_hits_at_5.len()
                } else {
                    commit.file_hits_at_10.len()
                };
                hit_count as f32 / commit.safe_changed_files.len() as f32
            }
        })
        .sum::<f32>();

    total / commits.len() as f32
}

fn average_lexical_baseline_recall(commits: &[HistoricalCommitEval], limit: usize) -> f32 {
    if commits.is_empty() {
        return 0.0;
    }

    let total = commits
        .iter()
        .map(|commit| {
            if commit.safe_changed_files.is_empty() {
                0.0
            } else {
                let hit_count = if limit <= 5 {
                    commit.lexical_baseline_hits_at_5.len()
                } else {
                    commit.lexical_baseline_hits_at_10.len()
                };
                hit_count as f32 / commit.safe_changed_files.len() as f32
            }
        })
        .sum::<f32>();

    total / commits.len() as f32
}

fn average_role_recall(commits: &[HistoricalCommitEval], role: FileRole, limit: usize) -> f32 {
    let mut total = 0.0;
    let mut count = 0usize;
    for commit in commits {
        let (changed, hits) = match role {
            FileRole::Source => (
                commit.source_files_changed,
                if limit <= 5 {
                    commit.source_hits_at_5
                } else {
                    commit.source_hits_at_10
                },
            ),
            FileRole::Test => (
                commit.test_files_changed,
                if limit <= 5 {
                    commit.test_hits_at_5
                } else {
                    commit.test_hits_at_10
                },
            ),
            _ => (0, 0),
        };
        if changed == 0 {
            continue;
        }
        total += hits as f32 / changed as f32;
        count += 1;
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn test_recommendation_rate(commits: &[HistoricalCommitEval]) -> f32 {
    if commits.is_empty() {
        return 0.0;
    }
    let with_tests = commits
        .iter()
        .filter(|commit| !commit.recommended_tests.is_empty())
        .count();
    with_tests as f32 / commits.len() as f32
}

fn average_recommended_context_files(commits: &[HistoricalCommitEval]) -> f32 {
    if commits.is_empty() {
        return 0.0;
    }
    let total = commits
        .iter()
        .map(|commit| commit.recommended_context_files.len())
        .sum::<usize>();
    total as f32 / commits.len() as f32
}

fn eval_comparison(commits: &[HistoricalCommitEval], k: usize) -> EvalComparison {
    let k = k.max(1);
    let combined = ranking_metrics(commits, k, RankingFamily::Combined);
    let lexical_baseline = ranking_metrics(commits, k, RankingFamily::LexicalBaseline);
    let no_context_baseline = ranking_metrics(commits, k, RankingFamily::NoContextBaseline);
    EvalComparison {
        k,
        recall_lift_at_k: combined.recall_at_k - lexical_baseline.recall_at_k,
        precision_lift_at_k: combined.precision_at_k - lexical_baseline.precision_at_k,
        mrr_lift_at_k: combined.mrr_at_k - lexical_baseline.mrr_at_k,
        recall_lift_vs_no_context_at_k: combined.recall_at_k - no_context_baseline.recall_at_k,
        precision_lift_vs_no_context_at_k: combined.precision_at_k
            - no_context_baseline.precision_at_k,
        mrr_lift_vs_no_context_at_k: combined.mrr_at_k - no_context_baseline.mrr_at_k,
        combined,
        lexical_baseline,
        no_context_baseline,
    }
}

#[derive(Debug, Clone, Copy)]
enum RankingFamily {
    Combined,
    LexicalBaseline,
    NoContextBaseline,
}

fn ranking_metrics(
    commits: &[HistoricalCommitEval],
    k: usize,
    family: RankingFamily,
) -> RankingMetrics {
    if commits.is_empty() {
        return RankingMetrics {
            k,
            recall_at_k: 0.0,
            precision_at_k: 0.0,
            mrr_at_k: 0.0,
            role_recall: Vec::new(),
            test_recommendation_rate: 0.0,
            average_recommended_context_files: 0.0,
        };
    }

    let recall_at_k = commits
        .iter()
        .map(|commit| {
            if commit.safe_changed_files.is_empty() {
                0.0
            } else {
                ranking_hits(commit, k, family).len() as f32
                    / commit.safe_changed_files.len() as f32
            }
        })
        .sum::<f32>()
        / commits.len() as f32;
    let precision_at_k = commits
        .iter()
        .map(|commit| ranking_hits(commit, k, family).len() as f32 / k as f32)
        .sum::<f32>()
        / commits.len() as f32;
    let mrr_at_k = commits
        .iter()
        .map(|commit| reciprocal_rank(commit, k, family))
        .sum::<f32>()
        / commits.len() as f32;
    let average_recommended_context_files = commits
        .iter()
        .map(|commit| ranking_for_family(commit, family).len().min(k))
        .sum::<usize>() as f32
        / commits.len() as f32;

    RankingMetrics {
        k,
        recall_at_k,
        precision_at_k,
        mrr_at_k,
        role_recall: role_recall_metrics(commits, k, family),
        test_recommendation_rate: test_recommendation_rate(commits),
        average_recommended_context_files,
    }
}

fn ranking_for_family(commit: &HistoricalCommitEval, family: RankingFamily) -> &[String] {
    match family {
        RankingFamily::Combined => &commit.recommended_context_files,
        RankingFamily::LexicalBaseline => &commit.lexical_baseline_files,
        RankingFamily::NoContextBaseline => &[],
    }
}

fn token_roi_metrics(
    commits: &[HistoricalCommitEval],
    ranking_budget: usize,
) -> Vec<TokenRoiMetric> {
    let k = ranking_budget.max(1);
    let specs = [
        (PackBudget::Brief, k.min(5), 4_000usize),
        (PackBudget::Standard, k, 24_000usize),
        (PackBudget::Deep, k, 100_000usize),
    ];
    let safe_targets = commits
        .iter()
        .map(|commit| commit.safe_changed_files.len())
        .sum::<usize>();
    let mut previous_useful_targets = 0usize;
    specs
        .into_iter()
        .map(|(budget, ranking_cutoff, estimated_tokens)| {
            let useful_targets = commits
                .iter()
                .map(|commit| ranking_hits(commit, ranking_cutoff, RankingFamily::Combined).len())
                .sum::<usize>();
            let marginal = useful_targets as isize - previous_useful_targets as isize;
            let larger_pack_adds_little_value =
                !matches!(budget, PackBudget::Brief) && marginal <= 0;
            previous_useful_targets = useful_targets;
            TokenRoiMetric {
                budget,
                ranking_cutoff,
                estimated_tokens,
                useful_targets,
                safe_targets,
                useful_targets_per_1k_tokens: if estimated_tokens == 0 {
                    0.0
                } else {
                    useful_targets as f32 / (estimated_tokens as f32 / 1000.0)
                },
                recall_at_cutoff: if safe_targets == 0 {
                    0.0
                } else {
                    useful_targets as f32 / safe_targets as f32
                },
                marginal_useful_targets_vs_previous_budget: marginal,
                larger_pack_adds_little_value,
            }
        })
        .collect()
}

fn ranking_hits(commit: &HistoricalCommitEval, k: usize, family: RankingFamily) -> Vec<String> {
    changed_file_hits(
        &commit.safe_changed_files,
        ranking_for_family(commit, family),
        k,
    )
}

fn reciprocal_rank(commit: &HistoricalCommitEval, k: usize, family: RankingFamily) -> f32 {
    let safe_changed_files = commit.safe_changed_files.iter().collect::<BTreeSet<_>>();
    ranking_for_family(commit, family)
        .iter()
        .take(k)
        .position(|path| safe_changed_files.contains(path))
        .map(|index| 1.0 / (index + 1) as f32)
        .unwrap_or(0.0)
}

fn role_recall_metrics(
    commits: &[HistoricalCommitEval],
    k: usize,
    family: RankingFamily,
) -> Vec<RoleRecallMetric> {
    [FileRole::Source, FileRole::Test]
        .into_iter()
        .map(|role| {
            let mut changed_file_count = 0usize;
            let mut hit_count = 0usize;
            for commit in commits {
                let paths = commit
                    .changed_path_labels
                    .iter()
                    .filter(|label| {
                        label.label_scope == LabelScope::Safe
                            && label.role == role
                            && commit.safe_changed_files.contains(&label.path)
                    })
                    .map(|label| label.path.clone())
                    .collect::<Vec<_>>();
                changed_file_count += paths.len();
                hit_count += changed_file_hits(&paths, ranking_for_family(commit, family), k).len();
            }
            RoleRecallMetric {
                role,
                recall_at_k: if changed_file_count == 0 {
                    0.0
                } else {
                    hit_count as f32 / changed_file_count as f32
                },
                changed_file_count,
                hit_count,
            }
        })
        .collect()
}

fn retrieval_gap_reasons(
    missing_files: &[String],
    lexical_baseline_files: &[String],
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
) -> BTreeMap<String, String> {
    let lexical_paths = lexical_baseline_files.iter().collect::<BTreeSet<_>>();
    missing_files
        .iter()
        .map(|path| {
            let reason = if lexical_paths.contains(path) {
                "lexical_only_miss".to_string()
            } else if let Some(signals) = signals_by_path.get(path) {
                format!("ranked_below_budget_{}", signal_family_code(signals))
            } else {
                "no_candidate_signal".to_string()
            };
            (path.clone(), reason)
        })
        .collect()
}

fn retrieval_gap_summaries(
    commits: &[HistoricalCommitEval],
    gap_reasons_by_commit: &[BTreeMap<String, String>],
    roles_by_path: &BTreeMap<String, FileRole>,
    labels_by_path: &BTreeMap<String, HistoricalChangedPathLabel>,
    limit: usize,
) -> Vec<RetrievalGapSummary> {
    let mut summaries = Vec::<RetrievalGapSummary>::new();
    for (commit, gap_reasons) in commits.iter().zip(gap_reasons_by_commit.iter()) {
        for path in &commit.missing_files_at_10 {
            let role = roles_by_path
                .get(path)
                .cloned()
                .unwrap_or(FileRole::Unknown);
            let signal_gap = gap_reasons
                .get(path)
                .cloned()
                .unwrap_or_else(|| "no_candidate_signal".to_string());
            let package = package_family(path);
            let path_family = path_family(path);
            let target_status = labels_by_path
                .get(path)
                .map(gap_target_status)
                .unwrap_or(RetrievalGapTargetStatus::Unknown);
            let recommendation_area = recommendation_area_for_gap(&signal_gap, role.clone());
            if let Some(summary) = summaries.iter_mut().find(|summary| {
                summary.role == role
                    && summary.signal_gap == signal_gap
                    && summary.package == package
                    && summary.path_family == path_family
                    && summary.target_status == target_status
                    && summary.recommendation_area == recommendation_area
            }) {
                summary.missed_count += 1;
                if summary.example_paths.len() < 3 && !summary.example_paths.contains(path) {
                    summary.example_paths.push(path.clone());
                }
            } else {
                summaries.push(RetrievalGapSummary {
                    role,
                    signal_gap,
                    package,
                    path_family,
                    target_status,
                    recommendation_area,
                    missed_count: 1,
                    example_paths: vec![path.clone()],
                });
            }
        }
    }

    summaries.sort_by(|left, right| {
        right
            .missed_count
            .cmp(&left.missed_count)
            .then_with(|| format!("{:?}", left.role).cmp(&format!("{:?}", right.role)))
            .then_with(|| left.signal_gap.cmp(&right.signal_gap))
            .then_with(|| left.package.cmp(&right.package))
            .then_with(|| left.path_family.cmp(&right.path_family))
    });
    summaries.truncate(limit.max(1));
    summaries
}

fn labels_by_path_from_commits(
    commits: &[HistoricalCommitEval],
) -> BTreeMap<String, HistoricalChangedPathLabel> {
    let mut labels = BTreeMap::new();
    for commit in commits {
        for label in &commit.changed_path_labels {
            labels
                .entry(label.path.clone())
                .or_insert_with(|| label.clone());
        }
    }
    labels
}

fn package_family(path: &str) -> String {
    Path::new(path)
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .filter(|component| !component.is_empty())
        .unwrap_or(".")
        .to_string()
}

fn gap_target_status(label: &HistoricalChangedPathLabel) -> RetrievalGapTargetStatus {
    if label.label_scope != LabelScope::Safe {
        return RetrievalGapTargetStatus::PolicyExcluded;
    }
    match label.change_kind {
        ctxpack_index::ChangeKind::Renamed => RetrievalGapTargetStatus::HistoricalRenamed,
        ctxpack_index::ChangeKind::Deleted => RetrievalGapTargetStatus::HistoricalDeleted,
        _ => RetrievalGapTargetStatus::CurrentReachable,
    }
}

fn recommendation_area_for_gap(signal_gap: &str, role: FileRole) -> RetrievalGapRecommendationArea {
    if matches!(role, FileRole::Test) {
        return RetrievalGapRecommendationArea::TestMapping;
    }
    if signal_gap == "lexical_only_miss" {
        return RetrievalGapRecommendationArea::LexicalRanking;
    }
    if signal_gap.contains("history") || signal_gap.contains("co_change") {
        return RetrievalGapRecommendationArea::HistoryRanking;
    }
    if signal_gap.contains("symbol") || signal_gap.contains("dependency") {
        return RetrievalGapRecommendationArea::ParserPrecision;
    }
    if signal_gap.contains("generated") || signal_gap.contains("sensitive") {
        return RetrievalGapRecommendationArea::PolicyExclusion;
    }
    if signal_gap == "no_candidate_signal" {
        return RetrievalGapRecommendationArea::Storage;
    }
    RetrievalGapRecommendationArea::SemanticRetrieval
}

fn signal_family_code(signals: &[RetrievalSignalKind]) -> String {
    if signals.is_empty() {
        return "none".to_string();
    }
    signals
        .iter()
        .map(signal_code)
        .collect::<Vec<_>>()
        .join("_")
}

fn signal_code(signal: &RetrievalSignalKind) -> &'static str {
    match signal {
        RetrievalSignalKind::Lexical => "lexical",
        RetrievalSignalKind::Semantic => "semantic",
        RetrievalSignalKind::Symbol => "symbol",
        RetrievalSignalKind::Dependency => "dependency",
        RetrievalSignalKind::RelatedTest => "related_test",
        RetrievalSignalKind::CoChange => "co_change",
        RetrievalSignalKind::CurrentDiff => "current_diff",
        RetrievalSignalKind::History => "history",
        RetrievalSignalKind::Docs => "docs",
        RetrievalSignalKind::Config => "config",
        RetrievalSignalKind::Anchor => "anchor",
    }
}

fn path_family(path: &str) -> String {
    let path = Path::new(path);
    let parent = path
        .parent()
        .and_then(|parent| parent.to_str())
        .filter(|parent| !parent.is_empty());
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    match parent {
        Some(parent) => format!("{parent}/*{extension}"),
        None => format!("*{extension}"),
    }
}

fn top_missing_files(
    commits: &[HistoricalCommitEval],
    roles_by_path: &BTreeMap<String, FileRole>,
    limit: usize,
) -> Vec<HistoricalMissingFileSummary> {
    let mut counts = BTreeMap::<String, usize>::new();
    for commit in commits {
        for path in &commit.missing_files_at_10 {
            *counts.entry(path.clone()).or_insert(0) += 1;
        }
    }

    let mut missing = counts
        .into_iter()
        .map(|(path, missed_count)| HistoricalMissingFileSummary {
            role: roles_by_path
                .get(&path)
                .cloned()
                .unwrap_or(FileRole::Unknown),
            path,
            missed_count,
        })
        .collect::<Vec<_>>();
    missing.sort_by(|left, right| {
        right
            .missed_count
            .cmp(&left.missed_count)
            .then_with(|| left.path.cmp(&right.path))
    });
    missing.truncate(limit.max(1));
    missing
}
