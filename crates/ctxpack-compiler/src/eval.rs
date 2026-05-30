use crate::packs::pack_repo_id;
use crate::planning::{
    is_low_information_task, normalized_target_agent,
    prepare_context_plan_with_paths_history_and_semantic,
};
use crate::policy::{provider_policy_report, reranker_decision};
use ctxpack_core::{
    CandidateFeatureExport, CandidateFeatureLabel, CandidateFeatureRow, CandidateFeatureSource,
    ContextPack, ContextPlan, Diagnostic, DiagnosticSeverity, EvalTrace, FileRole, PackBudget,
    PolicyQualityReport, PrecisionStatusReport, PrivacyStatus, ProviderDecisionStatus,
    ProviderPolicyReport, QueryConstructionTrace, RetrievalCandidate, RetrievalCandidateKind,
    RetrievalHealthGapFamily, RetrievalHealthMetric, RetrievalHealthReport,
    RetrievalHealthSignalContribution, RetrievalHealthTokenRoi, RetrievalSignalKind, TaskType,
};
use ctxpack_index::{
    historical_commit_samples, lexical_search, load_or_build_inventory, repo_id_for_path,
    semantic_document_report, task_hash, HistoricalChangedPath, HistoricalCommitOptions,
    HistoricalCommitSample, InventoryError, InventoryOptions, LabelScope, SearchOptions,
    SemanticDocumentOptions, SemanticProviderConfig, LEARNED_POLICY_PROFILE_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const HISTORICAL_EVAL_CACHE_SCHEMA_VERSION: &str = "historical-eval-cache-v2.3.1";

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
    #[serde(default)]
    pub semantic_provider: SemanticProviderConfig,
    #[serde(default)]
    pub cache_enabled: bool,
    #[serde(default)]
    pub force_refresh: bool,
    #[serde(default)]
    pub parallelism: usize,
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
    pub semantic_provider: Option<String>,
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
pub struct PairedBaselineAnalysisReport {
    pub repo_id: String,
    pub eval_range_id: String,
    pub evaluated_commits: usize,
    pub k: usize,
    #[serde(default)]
    pub rows: Vec<PairedBaselineRow>,
    #[serde(default)]
    pub token_roi: Vec<TokenRoiMetric>,
    #[serde(default)]
    pub signal_saturation: Vec<SignalSaturationMetric>,
    pub lexical_delta_at_k: f32,
    pub lexical_status: PairedBaselineVerdict,
    pub validation_coverage: f32,
    pub runtime: HistoricalEvalRuntimeSummary,
    #[serde(default)]
    pub gap_summaries: Vec<RetrievalGapSummary>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PairedBaselineRow {
    pub variant: String,
    pub family: PairedBaselineFamily,
    pub metrics: RankingMetrics,
    pub recall_delta_vs_default_at_k: f32,
    pub recall_delta_vs_lexical_at_k: f32,
    pub verdict: PairedBaselineVerdict,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SignalSaturationMetric {
    pub signal: RetrievalSignalKind,
    pub commits_with_signal: usize,
    pub average_candidate_files: f32,
    pub recall_at_k: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairedBaselineFamily {
    Default,
    Baseline,
    SignalOnly,
    Ablation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairedBaselineVerdict {
    Lift,
    Neutral,
    Regression,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPrecisionGateDecision {
    Promote,
    Hold,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPrecisionVariantStatus {
    Evaluated,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPrecisionGateReport {
    pub repo_id: String,
    pub eval_range_id: String,
    pub evaluated_commits: usize,
    pub k: usize,
    pub decision: SemanticPrecisionGateDecision,
    pub decision_reason: String,
    pub variants: Vec<SemanticPrecisionVariant>,
    pub named_wins: Vec<SemanticPrecisionNamedCase>,
    pub named_regressions: Vec<SemanticPrecisionNamedCase>,
    pub named_misses: Vec<SemanticPrecisionNamedCase>,
    pub provider_policy: ProviderPolicyReport,
    pub precision_status: PrecisionStatusReport,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPrecisionVariant {
    pub name: String,
    pub status: SemanticPrecisionVariantStatus,
    pub semantic_enabled: bool,
    pub precision_enabled: bool,
    pub reranker_enabled: bool,
    pub metrics: Option<RankingMetrics>,
    pub file_recall_at_10: Option<f32>,
    pub test_recall_at_10: Option<f32>,
    pub runtime_millis: Option<u64>,
    pub cache_hits: Option<usize>,
    pub cache_misses: Option<usize>,
    pub token_efficiency: Option<f32>,
    pub provider_status: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPrecisionNamedCase {
    pub sha: String,
    pub variant: String,
    pub reason: String,
    pub paths: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalEvalRuntimeSummary {
    pub total_millis: u64,
    pub commit_millis: u64,
    pub overhead_millis: u64,
    pub average_commit_millis: f32,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub parallelism: usize,
    pub git_sample_millis: u64,
    pub ranking_millis: u64,
    pub pack_compiler_millis: u64,
    pub slow_commits: Vec<HistoricalSlowCommitSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalSlowCommitSummary {
    pub sha: String,
    pub elapsed_millis: u64,
    pub safe_changed_file_count: usize,
    pub recommended_context_file_count: usize,
    pub missing_file_count_at_10: usize,
    pub low_information_task: bool,
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
    pub v23_eval_summary: ProductProofV23Summary,
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
pub struct ProductProofV23Summary {
    pub manifest_version: String,
    pub fixed_corpus_id: String,
    pub privacy_label: Option<String>,
    pub paired_baseline_verdicts: Vec<ProductProofBaselineVerdict>,
    pub runtime_total_millis: u64,
    pub feature_export_privacy: ProductProofFeatureExportPrivacy,
    pub learned_policy_status: ProductProofLearnedPolicyStatus,
    pub proof_boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofBaselineVerdict {
    pub repository: String,
    pub lexical_delta_at_k: f32,
    pub lexical_status: PairedBaselineVerdict,
    pub evaluated_commits: usize,
    pub k: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofFeatureExportPrivacy {
    pub schema_version: u32,
    pub local_only: bool,
    pub source_text_logged: bool,
    pub source_free_labels_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofLearnedPolicyStatus {
    pub profile_schema_version: u32,
    pub available: bool,
    pub default_requires_thresholds: bool,
    pub silent_default_allowed: bool,
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
    pub runtime: HistoricalEvalRuntimeSummary,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteConfig {
    #[serde(default = "default_benchmark_manifest_version")]
    pub manifest_version: String,
    pub name: String,
    #[serde(default)]
    pub corpus_id: Option<String>,
    #[serde(default)]
    pub privacy_label: Option<String>,
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
    #[serde(default = "default_benchmark_semantic_provider")]
    pub semantic_provider: String,
    #[serde(default)]
    pub semantic_model: Option<String>,
    #[serde(default)]
    pub semantic_dimensions: Option<usize>,
    #[serde(default)]
    pub cache_enabled: bool,
    #[serde(default)]
    pub force_refresh: bool,
    #[serde(default = "default_benchmark_parallelism")]
    pub parallelism: usize,
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
            semantic_provider: default_benchmark_semantic_provider(),
            semantic_model: None,
            semantic_dimensions: None,
            cache_enabled: false,
            force_refresh: false,
            parallelism: default_benchmark_parallelism(),
            role_filters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoConfig {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub revision_range_id: Option<String>,
    #[serde(default)]
    pub privacy_label: Option<String>,
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
    pub semantic_provider: Option<String>,
    #[serde(default)]
    pub semantic_model: Option<String>,
    #[serde(default)]
    pub semantic_dimensions: Option<usize>,
    #[serde(default)]
    pub cache_enabled: Option<bool>,
    #[serde(default)]
    pub force_refresh: Option<bool>,
    #[serde(default)]
    pub parallelism: Option<usize>,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
    #[serde(default)]
    pub baseline: Option<BenchmarkRepoBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteReport {
    pub manifest_version: String,
    pub suite_name: String,
    pub suite_id: String,
    pub corpus_id: Option<String>,
    pub privacy_label: Option<String>,
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
    pub baseline: Option<BenchmarkRepoBaseline>,
    pub baseline_status: Option<BenchmarkRepoBaselineStatus>,
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
    pub revision_range_id: Option<String>,
    pub privacy_label: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub limit: usize,
    pub ranking_budget: usize,
    pub mode: TaskType,
    pub target_agent: String,
    pub semantic_enabled: bool,
    pub semantic_provider: String,
    pub semantic_model: Option<String>,
    pub semantic_dimensions: Option<usize>,
    pub semantic_provider_role: String,
    pub semantic_quality_backend: bool,
    pub cache_enabled: bool,
    pub force_refresh: bool,
    pub parallelism: usize,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoBaseline {
    #[serde(default)]
    pub file_recall_at_10: Option<f32>,
    #[serde(default)]
    pub lexical_baseline_recall_at_10: Option<f32>,
    #[serde(default)]
    pub total_millis: Option<u64>,
    #[serde(default)]
    pub gap_families: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoBaselineStatus {
    pub compared: bool,
    pub file_recall_at_10_delta: Option<f32>,
    pub lexical_baseline_recall_at_10_delta: Option<f32>,
    pub total_millis_delta: Option<i64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalMissingFileSummary {
    pub path: String,
    pub role: FileRole,
    pub missed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalSignalRanking {
    pub signal: RetrievalSignalKind,
    pub files: Vec<String>,
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
    #[serde(default)]
    pub signal_baseline_files: Vec<HistoricalSignalRanking>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_trace: Option<QueryConstructionTrace>,
    #[serde(default)]
    pub elapsed_millis: u64,
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
    let v23_eval_summary = product_proof_v23_summary(&benchmark_report, &evaluated_reports);

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
        v23_eval_summary,
        limitations: vec![
            "Historical commit subjects are only proxies for real developer prompts.".to_string(),
            "No-context baseline is zero-file until editor anchor traces are available."
                .to_string(),
            "Token ROI is estimated from budget presets, not measured model billing.".to_string(),
            "Useful context at lexical parity is not world-class lift; repeated fixed-corpus lift and process-level context metrics are required.".to_string(),
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
            "v2.3: keep fixed corpus manifests and paired baseline verdicts current.".to_string(),
            "v2.4: add production semantic and precision backends only where eval gates show lift."
                .to_string(),
            "v2.5: expand machine-checkable real-client agent outcome proof.".to_string(),
        ],
        benchmark_report,
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn product_proof_v23_summary(
    benchmark_report: &BenchmarkSuiteReport,
    evaluated_reports: &[&HistoricalEvalReport],
) -> ProductProofV23Summary {
    let paired_baseline_verdicts = benchmark_report
        .repositories
        .iter()
        .filter_map(|repo| {
            let historical = repo.report.as_ref()?;
            let paired = paired_baseline_analysis_report(historical, 0.03, 0.03);
            Some(ProductProofBaselineVerdict {
                repository: repo.name.clone(),
                lexical_delta_at_k: paired.lexical_delta_at_k,
                lexical_status: paired.lexical_status,
                evaluated_commits: paired.evaluated_commits,
                k: paired.k,
            })
        })
        .collect::<Vec<_>>();
    let runtime_total_millis = evaluated_reports
        .iter()
        .map(|report| report.runtime.total_millis)
        .sum();

    ProductProofV23Summary {
        manifest_version: benchmark_report.manifest_version.clone(),
        fixed_corpus_id: benchmark_report
            .corpus_id
            .clone()
            .unwrap_or_else(|| benchmark_report.suite_id.clone()),
        privacy_label: benchmark_report.privacy_label.clone(),
        paired_baseline_verdicts,
        runtime_total_millis,
        feature_export_privacy: ProductProofFeatureExportPrivacy {
            schema_version: 1,
            local_only: true,
            source_text_logged: false,
            source_free_labels_only: true,
        },
        learned_policy_status: ProductProofLearnedPolicyStatus {
            profile_schema_version: LEARNED_POLICY_PROFILE_SCHEMA_VERSION,
            available: true,
            default_requires_thresholds: true,
            silent_default_allowed: false,
        },
        proof_boundary: "ctxpack may be useful at lexical parity, but world-class claims require repeated lift on fixed corpora plus process-level context metrics.".to_string(),
    }
}

pub fn build_retrieval_health_report(
    historical: &HistoricalEvalReport,
    policy: &PolicyQualityReport,
) -> RetrievalHealthReport {
    let metrics = vec![
        health_metric(
            "fileRecallAt5",
            historical.file_recall_at_5,
            "ratio",
            "historical_eval",
        ),
        health_metric(
            "fileRecallAt10",
            historical.file_recall_at_10,
            "ratio",
            "historical_eval",
        ),
        health_metric(
            "testRecallAt10",
            historical.test_recall_at_10,
            "ratio",
            "historical_eval",
        ),
        health_metric(
            "ctxpackLiftAt10",
            historical.ctxpack_lift_at_10,
            "delta",
            "historical_eval",
        ),
        health_metric(
            "contextPrecision",
            policy.context_precision,
            "ratio",
            "feedback",
        ),
        health_metric(
            "validationCoverage",
            policy.validation_coverage,
            "ratio",
            "feedback",
        ),
        health_metric(
            "correctionRate",
            policy.correction_rate,
            "ratio",
            "feedback",
        ),
    ];

    let mut token_roi = historical
        .token_roi
        .iter()
        .map(|row| RetrievalHealthTokenRoi {
            budget: Some(row.budget.clone()),
            source: "historical_eval".to_string(),
            event_count: historical.evaluated_commits,
            useful_files_per_event: 0.0,
            useful_targets_per_1k_tokens: row.useful_targets_per_1k_tokens,
            recall_at_cutoff: row.recall_at_cutoff,
            larger_pack_adds_little_value: row.larger_pack_adds_little_value,
        })
        .collect::<Vec<_>>();
    token_roi.extend(policy.token_roi.iter().map(|row| RetrievalHealthTokenRoi {
        budget: row.budget.clone(),
        source: "feedback".to_string(),
        event_count: row.event_count,
        useful_files_per_event: row.useful_files_per_event,
        useful_targets_per_1k_tokens: 0.0,
        recall_at_cutoff: 0.0,
        larger_pack_adds_little_value: false,
    }));

    let mut signal_contributions = policy
        .signal_contributions
        .iter()
        .map(|signal| RetrievalHealthSignalContribution {
            signal: signal.signal.clone(),
            source: "feedback".to_string(),
            event_count: signal.event_count,
            useful_file_hits: signal.useful_file_hits,
            score: signal.score,
            recall_without_signal: None,
            recall_lift_vs_lexical_at_k: None,
        })
        .collect::<Vec<_>>();
    signal_contributions.extend(historical.signal_ablations.iter().map(|ablation| {
        RetrievalHealthSignalContribution {
            signal: ablation.disabled_signal.clone(),
            source: "historical_ablation".to_string(),
            event_count: ablation.evaluated_commits,
            useful_file_hits: 0,
            score: ablation.metrics.recall_at_k,
            recall_without_signal: Some(ablation.metrics.recall_at_k),
            recall_lift_vs_lexical_at_k: Some(ablation.recall_lift_vs_lexical_at_k),
        }
    }));

    let mut gap_families = historical
        .retrieval_gap_summaries
        .iter()
        .map(|gap| RetrievalHealthGapFamily {
            family: format!("{:?}:{:?}:{}", gap.role, gap.signal_gap, gap.path_family),
            count: gap.missed_count,
            recommendation_area: Some(format!("{:?}", gap.recommendation_area)),
            target_status: Some(format!("{:?}", gap.target_status)),
            safe_path: None,
            source: "historical_eval".to_string(),
        })
        .collect::<Vec<_>>();
    gap_families.extend(policy.repeated_missing_file_families.iter().map(|family| {
        RetrievalHealthGapFamily {
            family: family.path.clone(),
            count: family.count,
            recommendation_area: Some("feedback_missing_file".to_string()),
            target_status: None,
            safe_path: Some(family.path.clone()),
            source: "feedback".to_string(),
        }
    }));

    let health_inputs = [
        historical.file_recall_at_10,
        historical.test_recall_at_10,
        policy.context_precision,
        policy.validation_coverage,
        1.0 - policy.correction_rate.min(1.0),
    ];
    let health_score = health_inputs.iter().sum::<f32>() / health_inputs.len() as f32;
    let mut low_confidence_flags = Vec::new();
    if historical.file_recall_at_10 < 0.5 {
        low_confidence_flags.push("file_recall_at_10_below_0_50".to_string());
    }
    if historical.test_recall_at_10 < 0.5 {
        low_confidence_flags.push("test_recall_at_10_below_0_50".to_string());
    }
    if policy.validation_coverage < 0.5 {
        low_confidence_flags.push("feedback_validation_coverage_below_0_50".to_string());
    }
    if policy.correction_rate > 0.25 {
        low_confidence_flags.push("feedback_correction_rate_above_0_25".to_string());
    }

    RetrievalHealthReport {
        repo_id: historical.repo_id.clone(),
        evaluated_commits: historical.evaluated_commits,
        feedback_events: policy.event_count,
        health_score,
        metrics,
        token_roi,
        signal_contributions,
        gap_families,
        low_confidence_flags,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    }
}

pub fn paired_baseline_analysis_report(
    historical: &HistoricalEvalReport,
    min_lift: f32,
    max_regression: f32,
) -> PairedBaselineAnalysisReport {
    let default = historical.ranking_comparison.combined.clone();
    let lexical = historical.ranking_comparison.lexical_baseline.clone();
    let no_context = historical.ranking_comparison.no_context_baseline.clone();
    let mut rows = Vec::new();

    rows.push(paired_baseline_row(
        "ctxpack_default",
        PairedBaselineFamily::Default,
        default.clone(),
        &default,
        &lexical,
        historical.evaluated_commits,
        min_lift,
        max_regression,
        "Current hybrid ranking under the fixed context-file budget.",
    ));
    rows.push(paired_baseline_row(
        "lexical_baseline",
        PairedBaselineFamily::Baseline,
        lexical.clone(),
        &default,
        &lexical,
        historical.evaluated_commits,
        min_lift,
        max_regression,
        "Exact/BM25-style path and identifier baseline under the same K.",
    ));
    rows.push(paired_baseline_row(
        "no_context",
        PairedBaselineFamily::Baseline,
        no_context,
        &default,
        &lexical,
        historical.evaluated_commits,
        min_lift,
        max_regression,
        "Zero-file baseline for an agent starting without ctxpack context.",
    ));

    for (variant, signal) in [
        ("semantic_only", RetrievalSignalKind::Semantic),
        ("graph_only", RetrievalSignalKind::Dependency),
        ("history_only", RetrievalSignalKind::CoChange),
        ("test_only", RetrievalSignalKind::RelatedTest),
        ("memory_only", RetrievalSignalKind::Memory),
    ] {
        rows.push(paired_baseline_row(
            variant,
            PairedBaselineFamily::SignalOnly,
            signal_only_ranking_metrics(
                &historical.commits,
                historical.ranking_comparison.k,
                signal,
            ),
            &default,
            &lexical,
            historical.evaluated_commits,
            min_lift,
            max_regression,
            "Ranks only candidates carrying this signal in the default source-free candidate set.",
        ));
    }
    rows.push(paired_baseline_row(
        "feedback_weighted",
        PairedBaselineFamily::SignalOnly,
        empty_ranking_metrics(historical.ranking_comparison.k),
        &default,
        &lexical,
        0,
        min_lift,
        max_regression,
        "Insufficient evidence until feedback labels are joined with fixed-corpus candidate rows.",
    ));

    for ablation in &historical.signal_ablations {
        rows.push(paired_baseline_row(
            &format!("without_{:?}", ablation.disabled_signal).to_lowercase(),
            PairedBaselineFamily::Ablation,
            ablation.metrics.clone(),
            &default,
            &lexical,
            historical.evaluated_commits,
            min_lift,
            max_regression,
            "Default ranking with one signal family removed.",
        ));
    }

    let lexical_delta_at_k = default.recall_at_k - lexical.recall_at_k;
    let lexical_status = if lexical_delta_at_k < -max_regression {
        PairedBaselineVerdict::Regression
    } else if lexical_delta_at_k > min_lift {
        PairedBaselineVerdict::Lift
    } else {
        PairedBaselineVerdict::Neutral
    };
    PairedBaselineAnalysisReport {
        repo_id: historical.repo_id.clone(),
        eval_range_id: historical.eval_range_id.clone(),
        evaluated_commits: historical.evaluated_commits,
        k: historical.ranking_comparison.k,
        rows,
        token_roi: historical.token_roi.clone(),
        signal_saturation: signal_saturation_metrics(
            &historical.commits,
            historical.ranking_comparison.k,
        ),
        lexical_delta_at_k,
        lexical_status,
        validation_coverage: historical.test_recommendation_rate,
        runtime: historical.runtime.clone(),
        gap_summaries: historical.retrieval_gap_summaries.clone(),
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    }
}

pub fn semantic_precision_gate_report(
    repo_root: impl AsRef<Path>,
    limit: usize,
    ranking_budget: usize,
    task_type: TaskType,
) -> Result<SemanticPrecisionGateReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let provider_policy = provider_policy_report(repo_root)?;
    let precision =
        semantic_document_report(repo_root, &SemanticDocumentOptions { limit: usize::MAX })?
            .precision_status;
    let default = evaluate_historical_commits(
        repo_root,
        &HistoricalEvalOptions {
            limit,
            ranking_budget,
            task_type: task_type.clone(),
            target_agent: "generic".to_string(),
            base: None,
            head: None,
            semantic_enabled: false,
            semantic_provider: SemanticProviderConfig::default(),
            cache_enabled: false,
            force_refresh: false,
            parallelism: 1,
        },
    )?;
    let semantic = evaluate_historical_commits(
        repo_root,
        &HistoricalEvalOptions {
            semantic_enabled: true,
            ..HistoricalEvalOptions {
                limit,
                ranking_budget,
                task_type,
                target_agent: "generic".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            }
        },
    )?;
    let reranker = reranker_decision(&provider_policy);
    let precision_available = precision.edge_count > 0 && !precision.degraded && !precision.stale;
    let mut variants = vec![
        gate_variant(
            "lexical_baseline",
            SemanticPrecisionVariantStatus::Evaluated,
            false,
            false,
            false,
            Some(default.ranking_comparison.lexical_baseline.clone()),
            &default,
            "evaluated",
            "Exact/path lexical baseline under the same fixed K.",
        ),
        gate_variant(
            "ctxpack_default",
            SemanticPrecisionVariantStatus::Evaluated,
            false,
            false,
            false,
            Some(default.ranking_comparison.combined.clone()),
            &default,
            "evaluated",
            "Current default ranking without semantic default promotion.",
        ),
        gate_variant(
            "local_semantic",
            SemanticPrecisionVariantStatus::Evaluated,
            true,
            false,
            false,
            Some(semantic.ranking_comparison.combined.clone()),
            &semantic,
            "evaluated",
            "Explicit local semantic retrieval using source-free semantic documents.",
        ),
    ];
    variants.push(gate_variant(
        "precision_enriched_semantic",
        if precision_available {
            SemanticPrecisionVariantStatus::Evaluated
        } else {
            SemanticPrecisionVariantStatus::Skipped
        },
        true,
        true,
        false,
        precision_available.then(|| semantic.ranking_comparison.combined.clone()),
        &semantic,
        if precision_available {
            "evaluated"
        } else {
            "skipped"
        },
        if precision_available {
            "Precision overlay was available and included in semantic documents."
        } else {
            "Skipped because no fresh precision overlay was available."
        },
    ));
    variants.push(gate_variant(
        "semantic_precision_full_hybrid",
        if precision_available {
            SemanticPrecisionVariantStatus::Evaluated
        } else {
            SemanticPrecisionVariantStatus::Skipped
        },
        true,
        true,
        false,
        precision_available.then(|| semantic.ranking_comparison.combined.clone()),
        &semantic,
        if precision_available {
            "evaluated"
        } else {
            "skipped"
        },
        "Full hybrid is held unless semantic plus precision beats default gates.",
    ));
    variants.push(gate_variant(
        "policy_allowed_reranked",
        if matches!(reranker.status, ProviderDecisionStatus::Allowed) {
            SemanticPrecisionVariantStatus::Evaluated
        } else {
            SemanticPrecisionVariantStatus::Skipped
        },
        true,
        precision_available,
        matches!(reranker.status, ProviderDecisionStatus::Allowed),
        matches!(reranker.status, ProviderDecisionStatus::Allowed)
            .then(|| semantic.ranking_comparison.combined.clone()),
        &semantic,
        &format!("{:?}", reranker.status).to_ascii_lowercase(),
        &reranker.reason,
    ));

    let named_wins = named_cases(&default, &semantic, "local_semantic", NamedCaseKind::Win);
    let named_regressions = named_cases(
        &default,
        &semantic,
        "local_semantic",
        NamedCaseKind::Regression,
    );
    let named_misses = named_cases(&default, &semantic, "local_semantic", NamedCaseKind::Miss);
    let (decision, decision_reason) =
        gate_decision_from_variants(&variants, &named_regressions, &provider_policy);
    let mut diagnostics = provider_policy.diagnostics.clone();
    if !named_regressions.is_empty() {
        diagnostics.push(Diagnostic {
            code: "semantic_precision_named_regressions".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Semantic/precision gate found named regressions; keep feature opt-in."
                .to_string(),
            paths: named_regressions
                .iter()
                .flat_map(|case| case.paths.clone())
                .collect(),
            count: named_regressions.len(),
        });
    }

    Ok(SemanticPrecisionGateReport {
        repo_id: default.repo_id.clone(),
        eval_range_id: format!("semantic-precision-gate:{}", default.eval_range_id),
        evaluated_commits: default.evaluated_commits,
        k: default.ranking_comparison.k,
        decision,
        decision_reason,
        variants,
        named_wins,
        named_regressions,
        named_misses,
        provider_policy,
        precision_status: precision,
        diagnostics,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}

fn gate_variant(
    name: &str,
    status: SemanticPrecisionVariantStatus,
    semantic_enabled: bool,
    precision_enabled: bool,
    reranker_enabled: bool,
    metrics: Option<RankingMetrics>,
    eval: &HistoricalEvalReport,
    provider_status: &str,
    note: &str,
) -> SemanticPrecisionVariant {
    let token_efficiency = eval
        .token_roi
        .iter()
        .find(|roi| roi.budget == PackBudget::Standard)
        .map(|roi| roi.useful_targets_per_1k_tokens);
    SemanticPrecisionVariant {
        name: name.to_string(),
        status,
        semantic_enabled,
        precision_enabled,
        reranker_enabled,
        metrics,
        file_recall_at_10: Some(eval.file_recall_at_10),
        test_recall_at_10: Some(eval.test_recall_at_10),
        runtime_millis: Some(eval.runtime.total_millis),
        cache_hits: Some(eval.runtime.cache_hits),
        cache_misses: Some(eval.runtime.cache_misses),
        token_efficiency,
        provider_status: provider_status.to_string(),
        note: note.to_string(),
    }
}

#[derive(Debug, Clone, Copy)]
enum NamedCaseKind {
    Win,
    Regression,
    Miss,
}

fn named_cases(
    default: &HistoricalEvalReport,
    variant: &HistoricalEvalReport,
    variant_name: &str,
    kind: NamedCaseKind,
) -> Vec<SemanticPrecisionNamedCase> {
    let default_by_sha = default
        .commits
        .iter()
        .map(|commit| (commit.sha.as_str(), commit))
        .collect::<BTreeMap<_, _>>();
    let mut cases = Vec::new();
    for commit in &variant.commits {
        let Some(default_commit) = default_by_sha.get(commit.sha.as_str()) else {
            continue;
        };
        let default_hits = default_commit.file_hits_at_10.len();
        let variant_hits = commit.file_hits_at_10.len();
        let matched = match kind {
            NamedCaseKind::Win => variant_hits > default_hits,
            NamedCaseKind::Regression => variant_hits < default_hits,
            NamedCaseKind::Miss => !commit.missing_files_at_10.is_empty(),
        };
        if matched {
            cases.push(SemanticPrecisionNamedCase {
                sha: short_sha(&commit.sha),
                variant: variant_name.to_string(),
                reason: match kind {
                    NamedCaseKind::Win => {
                        "Variant retrieved more gold changed files than default.".to_string()
                    }
                    NamedCaseKind::Regression => {
                        "Variant retrieved fewer gold changed files than default.".to_string()
                    }
                    NamedCaseKind::Miss => {
                        "Variant still missed gold changed files at K.".to_string()
                    }
                },
                paths: match kind {
                    NamedCaseKind::Win => commit.file_hits_at_10.clone(),
                    NamedCaseKind::Regression => default_commit.file_hits_at_10.clone(),
                    NamedCaseKind::Miss => commit.missing_files_at_10.clone(),
                },
            });
        }
    }
    cases.truncate(10);
    cases
}

fn gate_decision_from_variants(
    variants: &[SemanticPrecisionVariant],
    named_regressions: &[SemanticPrecisionNamedCase],
    provider_policy: &ProviderPolicyReport,
) -> (SemanticPrecisionGateDecision, String) {
    if !provider_policy.privacy_status.local_only
        || provider_policy.policy.allow_cloud_embeddings
        || provider_policy.policy.allow_cloud_reranking
        || provider_policy.policy.allow_source_transfer
    {
        return (
            SemanticPrecisionGateDecision::Block,
            "Blocked because provider policy is not local/source-free.".to_string(),
        );
    }
    if !named_regressions.is_empty() {
        return (
            SemanticPrecisionGateDecision::Block,
            "Blocked because named regressions were detected.".to_string(),
        );
    }
    let default = variants
        .iter()
        .find(|variant| variant.name == "ctxpack_default")
        .and_then(|variant| variant.metrics.as_ref());
    let semantic = variants
        .iter()
        .find(|variant| variant.name == "local_semantic")
        .and_then(|variant| variant.metrics.as_ref());
    let (Some(default), Some(semantic)) = (default, semantic) else {
        return (
            SemanticPrecisionGateDecision::Hold,
            "Held because required default or semantic metrics were missing.".to_string(),
        );
    };
    let recall_delta = semantic.recall_at_k - default.recall_at_k;
    let precision_delta = semantic.precision_at_k - default.precision_at_k;
    if recall_delta >= 0.05 && precision_delta >= -0.01 {
        (
            SemanticPrecisionGateDecision::Promote,
            format!(
                "Promote: local semantic recall delta {recall_delta:+.3}, precision delta {precision_delta:+.3}."
            ),
        )
    } else if recall_delta < -0.01 || precision_delta < -0.03 {
        (
            SemanticPrecisionGateDecision::Block,
            format!(
                "Blocked: local semantic recall delta {recall_delta:+.3}, precision delta {precision_delta:+.3}."
            ),
        )
    } else {
        (
            SemanticPrecisionGateDecision::Hold,
            format!(
                "Held: local semantic recall delta {recall_delta:+.3}, precision delta {precision_delta:+.3}; keep opt-in."
            ),
        )
    }
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}

fn paired_baseline_row(
    variant: &str,
    family: PairedBaselineFamily,
    metrics: RankingMetrics,
    default: &RankingMetrics,
    lexical: &RankingMetrics,
    evaluated_commits: usize,
    min_lift: f32,
    max_regression: f32,
    note: &str,
) -> PairedBaselineRow {
    let recall_delta_vs_default_at_k = metrics.recall_at_k - default.recall_at_k;
    let recall_delta_vs_lexical_at_k = metrics.recall_at_k - lexical.recall_at_k;
    let has_signal_evidence =
        metrics.average_recommended_context_files > 0.0 || variant == "no_context";
    let verdict = if evaluated_commits == 0 || !has_signal_evidence {
        PairedBaselineVerdict::InsufficientEvidence
    } else if recall_delta_vs_default_at_k < -max_regression {
        PairedBaselineVerdict::Regression
    } else if recall_delta_vs_lexical_at_k > min_lift {
        PairedBaselineVerdict::Lift
    } else {
        PairedBaselineVerdict::Neutral
    };
    PairedBaselineRow {
        variant: variant.to_string(),
        family,
        metrics,
        recall_delta_vs_default_at_k,
        recall_delta_vs_lexical_at_k,
        verdict,
        note: note.to_string(),
    }
}

fn health_metric(name: &str, value: f32, unit: &str, source: &str) -> RetrievalHealthMetric {
    RetrievalHealthMetric {
        name: name.to_string(),
        value,
        unit: unit.to_string(),
        source: source.to_string(),
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
        manifest_version: config.manifest_version.clone(),
        suite_name: config.name.clone(),
        suite_id,
        corpus_id: config.corpus_id.clone(),
        privacy_label: config.privacy_label.clone(),
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
    let mut effective_config = BenchmarkRepoEffectiveConfig {
        revision_range_id: repo_config.revision_range_id.clone(),
        privacy_label: repo_config
            .privacy_label
            .clone()
            .or_else(|| Some("source_free_local".to_string())),
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
        semantic_provider: repo_config
            .semantic_provider
            .clone()
            .unwrap_or_else(|| defaults.semantic_provider.clone()),
        semantic_model: repo_config
            .semantic_model
            .clone()
            .or_else(|| defaults.semantic_model.clone()),
        semantic_dimensions: repo_config
            .semantic_dimensions
            .or(defaults.semantic_dimensions),
        semantic_provider_role: String::new(),
        semantic_quality_backend: false,
        cache_enabled: repo_config.cache_enabled.unwrap_or(defaults.cache_enabled),
        force_refresh: repo_config.force_refresh.unwrap_or(defaults.force_refresh),
        parallelism: repo_config
            .parallelism
            .unwrap_or(defaults.parallelism)
            .max(1),
        role_filters: if repo_config.role_filters.is_empty() {
            defaults.role_filters.clone()
        } else {
            repo_config.role_filters.clone()
        },
    };
    let semantic_provider = benchmark_semantic_provider(&effective_config);
    effective_config.semantic_provider = semantic_provider.provider.clone();
    effective_config.semantic_model = Some(semantic_provider.model.clone());
    effective_config.semantic_dimensions = Some(semantic_provider.dimensions);
    effective_config.semantic_provider_role = semantic_provider.provider_role.clone();
    effective_config.semantic_quality_backend = semantic_provider.quality_backend;
    let repo_path = resolve_benchmark_repo_path(config_dir, &repo_config.path);
    let options = HistoricalEvalOptions {
        limit: effective_config.limit,
        ranking_budget: effective_config.ranking_budget,
        task_type: effective_config.mode.clone(),
        target_agent: effective_config.target_agent.clone(),
        base: effective_config.base.clone(),
        head: effective_config.head.clone(),
        semantic_enabled: effective_config.semantic_enabled,
        semantic_provider,
        cache_enabled: effective_config.cache_enabled,
        force_refresh: effective_config.force_refresh,
        parallelism: effective_config.parallelism,
    };

    match evaluate_historical_commits(&repo_path, &options) {
        Ok(report) => {
            let baseline_status = repo_config
                .baseline
                .as_ref()
                .map(|baseline| baseline_status_for_report(baseline, &report));
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
                baseline: repo_config.baseline.clone(),
                baseline_status,
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
            baseline: repo_config.baseline.clone(),
            baseline_status: repo_config
                .baseline
                .as_ref()
                .map(|baseline| baseline_status_for_error(baseline, &error.to_string())),
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

fn benchmark_semantic_provider(config: &BenchmarkRepoEffectiveConfig) -> SemanticProviderConfig {
    let default = SemanticProviderConfig::default();
    ctxpack_index::normalized_provider(&SemanticProviderConfig {
        provider: if config.semantic_provider.trim().is_empty() {
            default.provider
        } else {
            config.semantic_provider.clone()
        },
        model: config.semantic_model.clone().unwrap_or(default.model),
        dimensions: config.semantic_dimensions.unwrap_or(default.dimensions),
        distance_metric: default.distance_metric,
        provider_role: default.provider_role,
        quality_backend: default.quality_backend,
        local_only: true,
        available: true,
    })
}

fn benchmark_suite_id(
    config: &BenchmarkSuiteConfig,
    repositories: &[BenchmarkRepoReport],
) -> String {
    let mut input = format!(
        "manifest={}\nsuite={}\ncorpus={}\nprivacy={}\n",
        config.manifest_version,
        config.name,
        config.corpus_id.as_deref().unwrap_or(""),
        config.privacy_label.as_deref().unwrap_or("")
    );
    for repo in repositories {
        input.push_str(&format!(
            "repo={}\nrepoId={}\nrevisionRangeId={}\nprivacy={}\nbase={}\nhead={}\nlimit={}\nrankingBudget={}\nparallelism={}\ncache={}\nforceRefresh={}\nmode={:?}\ntarget={}\nroles={:?}\ncommits={}\nerror={}\n",
            repo.name,
            repo.repo_id.as_deref().unwrap_or(""),
            repo.effective_config.revision_range_id.as_deref().unwrap_or(""),
            repo.effective_config.privacy_label.as_deref().unwrap_or(""),
            repo.effective_config.base.as_deref().unwrap_or(""),
            repo.effective_config.head.as_deref().unwrap_or(""),
            repo.effective_config.limit,
            repo.effective_config.ranking_budget,
            repo.effective_config.parallelism,
            repo.effective_config.cache_enabled,
            repo.effective_config.force_refresh,
            repo.effective_config.mode,
            repo.effective_config.target_agent,
            repo.effective_config.role_filters,
            repo.evaluated_commits,
            repo.error.as_deref().unwrap_or("")
        ));
    }
    task_hash(&input)
}

fn baseline_status_for_report(
    baseline: &BenchmarkRepoBaseline,
    report: &HistoricalEvalReport,
) -> BenchmarkRepoBaselineStatus {
    let file_recall_at_10_delta = baseline
        .file_recall_at_10
        .map(|value| report.file_recall_at_10 - value);
    let lexical_baseline_recall_at_10_delta = baseline
        .lexical_baseline_recall_at_10
        .map(|value| report.lexical_baseline_recall_at_10 - value);
    let total_millis_delta = baseline
        .total_millis
        .map(|value| report.runtime.total_millis as i64 - value as i64);
    BenchmarkRepoBaselineStatus {
        compared: file_recall_at_10_delta.is_some()
            || lexical_baseline_recall_at_10_delta.is_some()
            || total_millis_delta.is_some(),
        file_recall_at_10_delta,
        lexical_baseline_recall_at_10_delta,
        total_millis_delta,
        notes: baseline.notes.clone(),
    }
}

fn baseline_status_for_error(
    baseline: &BenchmarkRepoBaseline,
    error: &str,
) -> BenchmarkRepoBaselineStatus {
    let mut notes = baseline.notes.clone();
    notes.push(format!("baseline comparison unavailable: {error}"));
    BenchmarkRepoBaselineStatus {
        compared: false,
        file_recall_at_10_delta: None,
        lexical_baseline_recall_at_10_delta: None,
        total_millis_delta: None,
        notes,
    }
}

fn default_benchmark_manifest_version() -> String {
    "ctxpack-benchmark-corpus-v2.5".to_string()
}

fn default_benchmark_limit() -> usize {
    20
}

fn default_benchmark_ranking_budget() -> usize {
    10
}

fn default_benchmark_parallelism() -> usize {
    1
}

fn default_benchmark_task_type() -> TaskType {
    TaskType::BugFix
}

fn default_benchmark_target_agent() -> String {
    "generic".to_string()
}

fn default_benchmark_semantic_provider() -> String {
    SemanticProviderConfig::default().provider
}

pub fn evaluate_historical_commits(
    repo_root: impl AsRef<Path>,
    options: &HistoricalEvalOptions,
) -> Result<HistoricalEvalReport, InventoryError> {
    let eval_started = Instant::now();
    let repo_root = repo_root.as_ref();
    let target_agent = normalized_target_agent(&options.target_agent);
    let budget = PackBudget::Standard;
    let ranking_budget = options.ranking_budget.max(1);
    let repo_id = pack_repo_id(repo_root);
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
        semantic_provider: options
            .semantic_enabled
            .then(|| options.semantic_provider.provider.clone()),
    };
    let eval_range_id = historical_eval_range_id(&repo_id, &effective_filters, &refs);

    if options.cache_enabled && !options.force_refresh {
        if let Some(mut cached) = load_historical_eval_cache(&repo_id, &eval_range_id)? {
            cached.runtime.cache_hits += 1;
            cached.runtime.cache_misses = 0;
            return Ok(cached);
        }
    }

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
    let git_sample_started = Instant::now();
    let samples = historical_commit_samples(
        repo_root,
        &HistoricalCommitOptions {
            limit: options.limit,
            base: options.base.clone(),
            head: options.head.clone(),
        },
    )?;
    let git_sample_millis = elapsed_millis(git_sample_started);
    let parallelism = options.parallelism.max(1).min(samples.len().max(1));
    let commit_results = evaluate_historical_commit_samples(
        repo_root,
        &snapshot_paths,
        samples,
        &target_agent,
        options,
        ranking_budget,
        parallelism,
    )?;
    let mut commits = Vec::new();
    let mut ablation_rankings = initial_ablation_rankings();
    let mut gap_reasons_by_commit = Vec::new();
    let mut ranking_millis = 0u64;
    let mut pack_compiler_millis = 0u64;
    for result in commit_results {
        for (signal, ranking) in result.ablation_rankings {
            if let Some(rankings) = ablation_rankings
                .iter_mut()
                .find(|entry| entry.disabled_signal == signal)
            {
                rankings.rankings.push(ranking);
            }
        }
        gap_reasons_by_commit.push(result.gap_reasons);
        ranking_millis += result.ranking_millis;
        pack_compiler_millis += result.pack_compiler_millis;
        commits.push(result.commit);
    }

    let file_recall_at_5 = average_recall(&commits, 5);
    let file_recall_at_10 = average_recall(&commits, 10);
    let lexical_baseline_recall_at_5 = average_lexical_baseline_recall(&commits, 5);
    let lexical_baseline_recall_at_10 = average_lexical_baseline_recall(&commits, 10);
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
    let runtime = historical_eval_runtime_summary(
        &commits,
        elapsed_millis(eval_started),
        usize::from(options.cache_enabled),
        parallelism,
        git_sample_millis,
        ranking_millis,
        pack_compiler_millis,
    );

    let report = HistoricalEvalReport {
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
        runtime,
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
    };
    if options.cache_enabled {
        write_historical_eval_cache(&report.repo_id, &report.eval_range_id, &report)?;
    }
    Ok(report)
}

pub(crate) struct HistoricalEvalWorktree<'a> {
    path: PathBuf,
    _source_repo: &'a Path,
    _temp_dir: Option<tempfile::TempDir>,
}

struct HistoricalCommitEvalResult {
    commit: HistoricalCommitEval,
    ablation_rankings: Vec<(RetrievalSignalKind, Vec<String>)>,
    gap_reasons: BTreeMap<String, String>,
    ranking_millis: u64,
    pack_compiler_millis: u64,
}

fn evaluate_historical_commit_samples(
    repo_root: &Path,
    snapshot_paths: &[String],
    samples: Vec<HistoricalCommitSample>,
    target_agent: &str,
    options: &HistoricalEvalOptions,
    ranking_budget: usize,
    parallelism: usize,
) -> Result<Vec<HistoricalCommitEvalResult>, InventoryError> {
    if parallelism <= 1 || samples.len() <= 1 {
        return samples
            .into_iter()
            .map(|sample| {
                evaluate_historical_commit_sample(
                    repo_root,
                    snapshot_paths,
                    sample,
                    target_agent,
                    options,
                    ranking_budget,
                )
            })
            .collect();
    }

    let mut indexed_results = Vec::new();
    for (chunk_index, chunk) in samples.chunks(parallelism.max(1)).enumerate() {
        let mut handles = Vec::new();
        for (offset, sample) in chunk.iter().cloned().enumerate() {
            let repo_root = repo_root.to_path_buf();
            let snapshot_paths = snapshot_paths.to_vec();
            let target_agent = target_agent.to_string();
            let options = options.clone();
            handles.push(thread::spawn(move || {
                let result = evaluate_historical_commit_sample(
                    &repo_root,
                    &snapshot_paths,
                    sample,
                    &target_agent,
                    &options,
                    ranking_budget,
                );
                (offset, result)
            }));
        }
        for handle in handles {
            let (offset, result) = handle.join().map_err(|_| {
                InventoryError::InvalidInput("historical eval worker thread panicked".to_string())
            })?;
            indexed_results.push((chunk_index * parallelism.max(1) + offset, result?));
        }
    }
    indexed_results.sort_by_key(|(index, _)| *index);
    Ok(indexed_results
        .into_iter()
        .map(|(_, result)| result)
        .collect())
}

fn evaluate_historical_commit_sample(
    repo_root: &Path,
    snapshot_paths: &[String],
    sample: HistoricalCommitSample,
    target_agent: &str,
    options: &HistoricalEvalOptions,
    ranking_budget: usize,
) -> Result<HistoricalCommitEvalResult, InventoryError> {
    let commit_started = Instant::now();
    let changed_path_labels = sample.changed_paths.clone();
    let task = if sample.title.trim().is_empty() {
        format!("change {}", sample.sha)
    } else {
        sample.title.clone()
    };
    let eval_repo = HistoricalEvalWorktree::for_parent(
        repo_root,
        sample.parent_sha.as_deref(),
        snapshot_paths,
    )?;
    let eval_root = eval_repo.path();
    let plan_started = Instant::now();
    let plan = prepare_context_plan_with_paths_history_and_semantic(
        eval_root,
        &task,
        options.task_type.clone(),
        &[],
        false,
        options.semantic_enabled,
        options.semantic_provider.clone(),
    )?;
    let pack_compiler_millis = elapsed_millis(plan_started);
    let ranking_started = Instant::now();
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
    let lexical_baseline_files = lexical_baseline_context_files(eval_root, &task, ranking_budget)?;
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
    let ablation_rankings = ablation_signals()
        .into_iter()
        .map(|signal| {
            let ranking =
                ablated_context_ranking(&recommended_context_files, &signals_by_path, &signal);
            (signal, ranking)
        })
        .collect::<Vec<_>>();
    let gap_reasons = retrieval_gap_reasons(
        &missing_files_at_10,
        &lexical_baseline_files,
        &signals_by_path,
    );
    let signal_baseline_files =
        signal_baseline_rankings(&recommended_context_files, &signals_by_path);
    let source_changed_files =
        filter_changed_labels_by_role(&changed_path_labels, &sample.safe_changed_files, |role| {
            matches!(role, FileRole::Source)
        });
    let test_changed_files =
        filter_changed_labels_by_role(&changed_path_labels, &sample.safe_changed_files, |role| {
            matches!(role, FileRole::Test)
        });
    let source_hits_at_5 =
        changed_file_hits(&source_changed_files, &recommended_context_files, 5).len();
    let source_hits_at_10 =
        changed_file_hits(&source_changed_files, &recommended_context_files, 10).len();
    let test_hits_at_5 =
        changed_file_hits(&test_changed_files, &recommended_context_files, 5).len();
    let test_hits_at_10 =
        changed_file_hits(&test_changed_files, &recommended_context_files, 10).len();
    let ranking_millis = elapsed_millis(ranking_started);

    Ok(HistoricalCommitEvalResult {
        commit: HistoricalCommitEval {
            sha: sample.sha,
            task_hash: task_hash(&task),
            task_type: options.task_type.clone(),
            target_agent: target_agent.to_string(),
            changed_path_labels,
            safe_changed_files: sample.safe_changed_files,
            excluded_changed_file_count: sample.excluded_changed_file_count,
            recommended_files,
            recommended_tests,
            recommended_context_files,
            recommended_commands,
            lexical_baseline_files,
            signal_baseline_files,
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
            query_trace: plan.query_trace.clone(),
            elapsed_millis: elapsed_millis(commit_started),
            source_text_logged: false,
        },
        ablation_rankings,
        gap_reasons,
        ranking_millis,
        pack_compiler_millis,
    })
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
        repo_id: feature_repo_id(repo_root),
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

pub fn export_candidate_features_for_task(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    target_agent: &str,
    limit: usize,
    semantic_enabled: bool,
) -> Result<CandidateFeatureExport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let plan = prepare_context_plan_with_paths_history_and_semantic(
        repo_root,
        task,
        task_type.clone(),
        &[],
        true,
        semantic_enabled,
        SemanticProviderConfig::default(),
    )?;
    Ok(candidate_feature_export_from_plan(
        repo_root,
        task,
        &plan,
        Some(task_type),
        Some(normalized_target_agent(target_agent)),
        CandidateFeatureSource::PlanCandidate,
        None,
        limit,
    ))
}

pub fn write_candidate_feature_export(
    repo_root: impl AsRef<Path>,
    export: &CandidateFeatureExport,
) -> Result<PathBuf, InventoryError> {
    let path = candidate_feature_export_path(repo_root.as_ref(), &export.export_id.to_string());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(export).map_err(InventoryError::Serialize)?;
    fs::write(&path, json).map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

pub fn list_candidate_feature_exports(
    repo_root: impl AsRef<Path>,
) -> Result<Vec<CandidateFeatureExport>, InventoryError> {
    let dir = candidate_feature_export_dir(repo_root.as_ref());
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(InventoryError::Read { path: dir, source }),
    };
    let mut exports: Vec<CandidateFeatureExport> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| InventoryError::Read {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
            path: path.clone(),
            source,
        })?;
        exports.push(serde_json::from_str(&content).map_err(|source| {
            InventoryError::Deserialize {
                path: path.clone(),
                source,
            }
        })?);
    }
    exports.sort_by(|left, right| {
        right
            .created_at_unix_seconds
            .cmp(&left.created_at_unix_seconds)
            .then_with(|| left.export_id.cmp(&right.export_id))
    });
    Ok(exports)
}

pub fn load_candidate_feature_export(
    repo_root: impl AsRef<Path>,
    export_id: &str,
) -> Result<CandidateFeatureExport, InventoryError> {
    let path = candidate_feature_export_path(repo_root.as_ref(), export_id);
    let content = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| InventoryError::Deserialize { path, source })
}

pub fn delete_candidate_feature_export(
    repo_root: impl AsRef<Path>,
    export_id: &str,
) -> Result<PathBuf, InventoryError> {
    let path = candidate_feature_export_path(repo_root.as_ref(), export_id);
    fs::remove_file(&path).map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

pub fn compare_candidate_feature_exports(
    base: &CandidateFeatureExport,
    head: &CandidateFeatureExport,
) -> CandidateFeatureComparisonReport {
    let mut base_by_kind = BTreeMap::<RetrievalCandidateKind, usize>::new();
    let mut head_by_kind = BTreeMap::<RetrievalCandidateKind, usize>::new();
    for row in &base.rows {
        *base_by_kind.entry(row.candidate_kind.clone()).or_default() += 1;
    }
    for row in &head.rows {
        *head_by_kind.entry(row.candidate_kind.clone()).or_default() += 1;
    }
    let mut keys = base_by_kind.keys().cloned().collect::<BTreeSet<_>>();
    keys.extend(head_by_kind.keys().cloned());
    let kind_deltas = keys
        .into_iter()
        .map(|kind| CandidateFeatureKindDelta {
            kind: kind.clone(),
            base_count: *base_by_kind.get(&kind).unwrap_or(&0),
            head_count: *head_by_kind.get(&kind).unwrap_or(&0),
        })
        .collect::<Vec<_>>();

    CandidateFeatureComparisonReport {
        base_export_id: base.export_id,
        head_export_id: head.export_id,
        base_row_count: base.row_count,
        head_row_count: head.row_count,
        row_count_delta: head.row_count as isize - base.row_count as isize,
        kind_deltas,
        source_text_logged: base.source_text_logged || head.source_text_logged,
        privacy_status: PrivacyStatus::local_only(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateFeatureComparisonReport {
    pub base_export_id: Uuid,
    pub head_export_id: Uuid,
    pub base_row_count: usize,
    pub head_row_count: usize,
    pub row_count_delta: isize,
    #[serde(default)]
    pub kind_deltas: Vec<CandidateFeatureKindDelta>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateFeatureKindDelta {
    pub kind: RetrievalCandidateKind,
    pub base_count: usize,
    pub head_count: usize,
}

fn candidate_feature_export_from_plan(
    repo_root: &Path,
    task: &str,
    plan: &ContextPlan,
    task_type: Option<TaskType>,
    target_agent: Option<String>,
    export_source: CandidateFeatureSource,
    eval_range_id: Option<String>,
    limit: usize,
) -> CandidateFeatureExport {
    let task_hash = Some(task_hash(task));
    let selected_ranks = selected_candidate_ranks(plan);
    let rows = plan
        .retrieval_candidates
        .iter()
        .take(limit.max(1))
        .enumerate()
        .map(|(index, candidate)| {
            candidate_feature_row(
                candidate,
                index + 1,
                selected_ranks.get(&candidate.path).copied(),
            )
        })
        .collect::<Vec<_>>();
    CandidateFeatureExport {
        export_id: Uuid::new_v4(),
        schema_version: 1,
        repo_id: repo_id_for_path(repo_root),
        task_hash,
        eval_range_id,
        export_source,
        task_type,
        target_agent,
        row_count: rows.len(),
        created_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        rows,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn candidate_feature_row(
    candidate: &RetrievalCandidate,
    rank: usize,
    selected_rank: Option<usize>,
) -> CandidateFeatureRow {
    let lexical_score = feature_signal_score(candidate, RetrievalSignalKind::Lexical);
    let semantic_score = feature_signal_score(candidate, RetrievalSignalKind::Semantic);
    let dependency_score = feature_signal_score(candidate, RetrievalSignalKind::Dependency);
    let history_score = feature_signal_score(candidate, RetrievalSignalKind::History)
        + feature_signal_score(candidate, RetrievalSignalKind::CoChange);
    let test_score = feature_signal_score(candidate, RetrievalSignalKind::RelatedTest);
    let memory_score = feature_signal_score(candidate, RetrievalSignalKind::Memory);
    let feedback_score = 0.0;
    let history_commit_count = candidate
        .evidence
        .iter()
        .map(|evidence| evidence.commit_count)
        .sum::<u32>();
    let test_relation_confidence = candidate
        .evidence
        .iter()
        .find(|evidence| evidence.signal == RetrievalSignalKind::RelatedTest)
        .map(|evidence| evidence.score);
    let memory_count = candidate
        .evidence
        .iter()
        .filter(|evidence| evidence.signal == RetrievalSignalKind::Memory)
        .count() as u32;
    let graph_distance = if dependency_score > 0.0 {
        Some(1)
    } else {
        None
    };
    let mut labels = Vec::new();
    if selected_rank.is_some() {
        labels.push(CandidateFeatureLabel::Selected);
    }
    if labels.is_empty() {
        labels.push(CandidateFeatureLabel::Unknown);
    }
    CandidateFeatureRow {
        candidate_id: candidate_feature_id(candidate, rank),
        candidate_kind: candidate.kind.clone(),
        path: candidate.path.clone(),
        role: candidate.role.clone(),
        rank,
        selected_rank,
        confidence: candidate.confidence,
        reason_code: candidate.reason_code.clone(),
        signal_scores: candidate.signal_scores.clone(),
        lexical_score,
        semantic_score,
        graph_score: dependency_score,
        history_score,
        test_score,
        memory_score,
        feedback_score,
        graph_distance,
        history_commit_count,
        test_relation_confidence,
        memory_count,
        feedback_event_count: 0,
        labels,
        label_scope: "source_free".to_string(),
        source_text_logged: false,
    }
}

fn selected_candidate_ranks(plan: &ContextPlan) -> BTreeMap<Option<String>, usize> {
    let mut selected = BTreeMap::new();
    for (index, target) in plan.target_files.iter().enumerate() {
        selected.insert(Some(target.path.clone()), index + 1);
    }
    for (index, test) in plan.related_tests.iter().enumerate() {
        selected
            .entry(Some(test.path.clone()))
            .or_insert(plan.target_files.len() + index + 1);
    }
    selected
}

fn feature_signal_score(candidate: &RetrievalCandidate, signal: RetrievalSignalKind) -> f32 {
    candidate
        .signal_scores
        .iter()
        .filter(|score| score.signal == signal)
        .map(|score| score.score * score.weight)
        .sum()
}

fn candidate_feature_id(candidate: &RetrievalCandidate, rank: usize) -> String {
    let path = candidate.path.as_deref().unwrap_or("repo-history");
    task_hash(&format!("{:?}:{path}:{rank}", candidate.kind))
}

fn candidate_feature_export_dir(repo_root: &Path) -> PathBuf {
    ctxpack_cache_root()
        .join("repos")
        .join(feature_repo_id(repo_root))
        .join("feature-exports")
}

fn candidate_feature_export_path(repo_root: &Path, export_id: &str) -> PathBuf {
    candidate_feature_export_dir(repo_root).join(format!("{export_id}.json"))
}

fn feature_repo_id(repo_root: &Path) -> String {
    let stable_root = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    repo_id_for_path(&stable_root)
}

fn context_file_ranking(
    recommended_files: &[String],
    recommended_tests: &[String],
    ranking_budget: usize,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let ranking_budget = ranking_budget.max(1);
    recommended_files
        .iter()
        .chain(recommended_tests.iter())
        .filter_map(|path| seen.insert(path.clone()).then_some(path.clone()))
        .take(ranking_budget)
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
        "version={HISTORICAL_EVAL_CACHE_SCHEMA_VERSION}\nrepo={repo_id}\nlimit={}\nrankingBudget={}\nmode={:?}\ntarget={}\nbudget={:?}\nsemantic={}\nsemanticProvider={}\nbase={}\nhead={}",
        filters.limit,
        filters.ranking_budget,
        filters.mode,
        filters.target_agent,
        filters.budget,
        filters.semantic_enabled,
        filters.semantic_provider.as_deref().unwrap_or(""),
        refs.base.as_deref().unwrap_or(""),
        refs.head.as_deref().unwrap_or("")
    ))
}

fn historical_eval_cache_path(repo_id: &str, eval_range_id: &str) -> PathBuf {
    ctxpack_cache_root()
        .join("repos")
        .join(repo_id)
        .join("eval-cache")
        .join(format!("{eval_range_id}.json"))
}

fn load_historical_eval_cache(
    repo_id: &str,
    eval_range_id: &str,
) -> Result<Option<HistoricalEvalReport>, InventoryError> {
    let path = historical_eval_cache_path(repo_id, eval_range_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(InventoryError::Read { path, source }),
    };
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|source| InventoryError::Deserialize { path, source })
}

fn write_historical_eval_cache(
    repo_id: &str,
    eval_range_id: &str,
    report: &HistoricalEvalReport,
) -> Result<(), InventoryError> {
    let path = historical_eval_cache_path(repo_id, eval_range_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(report).map_err(InventoryError::Serialize)?;
    fs::write(&path, json).map_err(|source| InventoryError::Write { path, source })
}

fn ctxpack_cache_root() -> PathBuf {
    if let Ok(value) = std::env::var("CTXPACK_HOME") {
        return PathBuf::from(value);
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".ctxpack")
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

fn signal_baseline_rankings(
    recommended_context_files: &[String],
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
) -> Vec<HistoricalSignalRanking> {
    signal_baseline_signals()
        .into_iter()
        .map(|signal| HistoricalSignalRanking {
            files: recommended_context_files
                .iter()
                .filter(|path| {
                    signals_by_path
                        .get(path.as_str())
                        .is_some_and(|signals| signals.contains(&signal))
                })
                .cloned()
                .collect(),
            signal,
        })
        .collect()
}

fn signal_baseline_signals() -> Vec<RetrievalSignalKind> {
    vec![
        RetrievalSignalKind::Semantic,
        RetrievalSignalKind::Dependency,
        RetrievalSignalKind::RelatedTest,
        RetrievalSignalKind::CoChange,
        RetrievalSignalKind::Memory,
    ]
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

fn signal_only_ranking_metrics(
    commits: &[HistoricalCommitEval],
    k: usize,
    signal: RetrievalSignalKind,
) -> RankingMetrics {
    if commits.is_empty() {
        return empty_ranking_metrics(k);
    }
    let recall_at_k = commits
        .iter()
        .map(|commit| {
            let ranking = signal_ranking_for_commit(commit, &signal);
            if commit.safe_changed_files.is_empty() {
                0.0
            } else {
                changed_file_hits(&commit.safe_changed_files, &ranking, k).len() as f32
                    / commit.safe_changed_files.len() as f32
            }
        })
        .sum::<f32>()
        / commits.len() as f32;
    let precision_at_k = commits
        .iter()
        .map(|commit| {
            let ranking = signal_ranking_for_commit(commit, &signal);
            changed_file_hits(&commit.safe_changed_files, &ranking, k).len() as f32 / k as f32
        })
        .sum::<f32>()
        / commits.len() as f32;
    let mrr_at_k = commits
        .iter()
        .map(|commit| {
            let ranking = signal_ranking_for_commit(commit, &signal);
            reciprocal_rank_for_files(&commit.safe_changed_files, &ranking, k)
        })
        .sum::<f32>()
        / commits.len() as f32;
    let average_recommended_context_files = commits
        .iter()
        .map(|commit| signal_ranking_for_commit(commit, &signal).len().min(k))
        .sum::<usize>() as f32
        / commits.len() as f32;
    RankingMetrics {
        k,
        recall_at_k,
        precision_at_k,
        mrr_at_k,
        role_recall: Vec::new(),
        test_recommendation_rate: test_recommendation_rate(commits),
        average_recommended_context_files,
    }
}

fn signal_saturation_metrics(
    commits: &[HistoricalCommitEval],
    k: usize,
) -> Vec<SignalSaturationMetric> {
    signal_baseline_signals()
        .into_iter()
        .map(|signal| {
            let metrics = signal_only_ranking_metrics(commits, k, signal.clone());
            let commits_with_signal = commits
                .iter()
                .filter(|commit| !signal_ranking_for_commit(commit, &signal).is_empty())
                .count();
            SignalSaturationMetric {
                signal,
                commits_with_signal,
                average_candidate_files: metrics.average_recommended_context_files,
                recall_at_k: metrics.recall_at_k,
            }
        })
        .collect()
}

fn empty_ranking_metrics(k: usize) -> RankingMetrics {
    RankingMetrics {
        k,
        recall_at_k: 0.0,
        precision_at_k: 0.0,
        mrr_at_k: 0.0,
        role_recall: Vec::new(),
        test_recommendation_rate: 0.0,
        average_recommended_context_files: 0.0,
    }
}

fn signal_ranking_for_commit(
    commit: &HistoricalCommitEval,
    signal: &RetrievalSignalKind,
) -> Vec<String> {
    commit
        .signal_baseline_files
        .iter()
        .find(|ranking| &ranking.signal == signal)
        .map(|ranking| ranking.files.clone())
        .unwrap_or_default()
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

fn reciprocal_rank_for_files(
    safe_changed_files: &[String],
    recommended_context_files: &[String],
    k: usize,
) -> f32 {
    let safe_changed_files = safe_changed_files.iter().collect::<BTreeSet<_>>();
    recommended_context_files
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

fn historical_eval_runtime_summary(
    commits: &[HistoricalCommitEval],
    total_millis: u64,
    cache_misses: usize,
    parallelism: usize,
    git_sample_millis: u64,
    ranking_millis: u64,
    pack_compiler_millis: u64,
) -> HistoricalEvalRuntimeSummary {
    let commit_millis = commits
        .iter()
        .map(|commit| commit.elapsed_millis)
        .sum::<u64>();
    let mut slow_commits = commits
        .iter()
        .map(|commit| HistoricalSlowCommitSummary {
            sha: commit.sha.clone(),
            elapsed_millis: commit.elapsed_millis,
            safe_changed_file_count: commit.safe_changed_files.len(),
            recommended_context_file_count: commit.recommended_context_files.len(),
            missing_file_count_at_10: commit.missing_files_at_10.len(),
            low_information_task: commit.low_information_task,
        })
        .collect::<Vec<_>>();
    slow_commits.sort_by(|left, right| {
        right
            .elapsed_millis
            .cmp(&left.elapsed_millis)
            .then_with(|| left.sha.cmp(&right.sha))
    });
    slow_commits.truncate(5);

    HistoricalEvalRuntimeSummary {
        total_millis,
        commit_millis,
        overhead_millis: total_millis.saturating_sub(commit_millis),
        average_commit_millis: if commits.is_empty() {
            0.0
        } else {
            commit_millis as f32 / commits.len() as f32
        },
        cache_hits: 0,
        cache_misses,
        parallelism,
        git_sample_millis,
        ranking_millis,
        pack_compiler_millis,
        slow_commits,
    }
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
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
        RetrievalSignalKind::Memory => "memory",
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

#[cfg(test)]
mod tests {
    use super::*;
    use ctxpack_core::{ProviderPolicy, ProviderPolicyReport};

    #[test]
    fn context_ranking_keeps_validation_tests_inside_budget() {
        let files = (0..8)
            .map(|index| format!("src/file-{index}.ts"))
            .collect::<Vec<_>>();
        let tests = vec![
            "tests/file-0.test.ts".to_string(),
            "tests/file-1.test.ts".to_string(),
            "src/file-1.ts".to_string(),
        ];

        let ranking = context_file_ranking(&files, &tests, 10);

        assert_eq!(ranking.len(), 10);
        assert!(ranking.contains(&"tests/file-0.test.ts".to_string()));
        assert!(ranking.contains(&"tests/file-1.test.ts".to_string()));
        assert_eq!(
            ranking
                .iter()
                .filter(|path| path.as_str() == "src/file-1.ts")
                .count(),
            1
        );
    }

    #[test]
    fn gate_decision_promotes_holds_and_blocks_from_measured_variants() {
        let policy = source_free_provider_policy();
        let promote = vec![
            gate_test_variant("ctxpack_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.47, 0.10),
        ];
        let hold = vec![
            gate_test_variant("ctxpack_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.41, 0.10),
        ];
        let block = vec![
            gate_test_variant("ctxpack_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.35, 0.10),
        ];

        assert_eq!(
            gate_decision_from_variants(&promote, &[], &policy).0,
            SemanticPrecisionGateDecision::Promote
        );
        assert_eq!(
            gate_decision_from_variants(&hold, &[], &policy).0,
            SemanticPrecisionGateDecision::Hold
        );
        assert_eq!(
            gate_decision_from_variants(&block, &[], &policy).0,
            SemanticPrecisionGateDecision::Block
        );
    }

    #[test]
    fn gate_decision_blocks_unsafe_policy_and_named_regression() {
        let mut unsafe_policy = source_free_provider_policy();
        unsafe_policy.policy.allow_cloud_embeddings = true;
        let variants = vec![
            gate_test_variant("ctxpack_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.47, 0.10),
        ];
        assert_eq!(
            gate_decision_from_variants(&variants, &[], &unsafe_policy).0,
            SemanticPrecisionGateDecision::Block
        );
        let named_regression = vec![SemanticPrecisionNamedCase {
            sha: "abc123".to_string(),
            variant: "local_semantic".to_string(),
            reason: "lost a target".to_string(),
            paths: vec!["src/lib.rs".to_string()],
        }];
        assert_eq!(
            gate_decision_from_variants(
                &variants,
                &named_regression,
                &source_free_provider_policy()
            )
            .0,
            SemanticPrecisionGateDecision::Block
        );
    }

    fn gate_test_variant(name: &str, recall: f32, precision: f32) -> SemanticPrecisionVariant {
        SemanticPrecisionVariant {
            name: name.to_string(),
            status: SemanticPrecisionVariantStatus::Evaluated,
            semantic_enabled: name.contains("semantic"),
            precision_enabled: false,
            reranker_enabled: false,
            metrics: Some(RankingMetrics {
                k: 10,
                recall_at_k: recall,
                precision_at_k: precision,
                mrr_at_k: recall,
                role_recall: Vec::new(),
                test_recommendation_rate: 0.0,
                average_recommended_context_files: 1.0,
            }),
            file_recall_at_10: Some(recall),
            test_recall_at_10: Some(0.0),
            runtime_millis: Some(1),
            cache_hits: Some(0),
            cache_misses: Some(0),
            token_efficiency: Some(1.0),
            provider_status: "evaluated".to_string(),
            note: "test variant".to_string(),
        }
    }

    fn source_free_provider_policy() -> ProviderPolicyReport {
        ProviderPolicyReport {
            policy_path: None,
            policy: ProviderPolicy::default(),
            decisions: Vec::new(),
            diagnostics: Vec::new(),
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        }
    }
}
