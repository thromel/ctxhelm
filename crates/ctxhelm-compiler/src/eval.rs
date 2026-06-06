use crate::packs::pack_repo_id;
use crate::planning::{
    is_low_information_task, is_multi_area_task, normalized_target_agent,
    prepare_context_plan_with_paths_history_and_semantic,
    prepare_context_plan_with_paths_history_mode_and_semantic, HistoryMode,
};
use crate::policy::{
    provider_policy_report, query_family_routed_reranker_enabled_for_family, reranker_decision,
    semantic_provider_decision,
};
use ctxhelm_core::{
    context_area_for_path, context_area_resource_uri, CandidateFeatureExport,
    CandidateFeatureLabel, CandidateFeatureRow, CandidateFeatureSource, ContextArea, ContextPack,
    ContextPlan, Diagnostic, DiagnosticSeverity, EvalTrace, FileRole, InspectionPressureBreakdown,
    PackBudget, PolicyQualityReport, PrecisionStatusReport, PrivacyStatus, ProviderDecisionStatus,
    ProviderPolicyReport, QueryConstructionTrace, QueryFacetKind, RetrievalCandidate,
    RetrievalCandidateKind, RetrievalHealthGapFamily, RetrievalHealthMetric, RetrievalHealthReport,
    RetrievalHealthSignalContribution, RetrievalHealthTokenRoi, RetrievalSignalKind, TaskType,
};
use ctxhelm_index::{
    historical_commit_samples_with_safe_paths, inventory_path, legacy_lexical_search_report,
    lexical_search, lexical_search_report, list_memory_cards, load_or_build_inventory,
    persist_memory_card_records, repo_id_for_path, semantic_document_report, task_hash,
    write_eval_history_sidecar, HistoricalChangedPath, HistoricalCommitOptions,
    HistoricalCommitSample, InventoryError, InventoryOptions, LabelScope, SearchOptions,
    SemanticDocumentOptions, SemanticProviderConfig, StorageMemoryCardRecord, StoreConfig,
    LEARNED_POLICY_PROFILE_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const HISTORICAL_EVAL_CACHE_SCHEMA_VERSION: &str = "historical-eval-cache-v2.3.5";
const LEXICAL_BACKEND_CORPUS_SCHEMA_VERSION: &str = "lexical-backend-corpus-v1";
const PARENT_SNAPSHOT_BATCH_READ_TIMEOUT: Duration = Duration::from_millis(500);
const PARENT_SNAPSHOT_SCHEMA_VERSION: &str = "historical-eval-parent-snapshot-v2";
const PARENT_SNAPSHOT_MANIFEST: &str = ".ctxhelm/parent-snapshot.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParentSnapshotManifest {
    manifest_version: String,
    parent_sha: String,
    requested_path_count: usize,
    extracted_path_count: usize,
    failed_batch_count: usize,
    complete: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct ParentSnapshotExtractionStatus {
    requested_path_count: usize,
    extracted_path_count: usize,
    failed_batch_count: usize,
}

impl ParentSnapshotExtractionStatus {
    fn merge(&mut self, other: Self) {
        self.requested_path_count += other.requested_path_count;
        self.extracted_path_count += other.extracted_path_count;
        self.failed_batch_count += other.failed_batch_count;
    }

    fn complete(self) -> bool {
        self.failed_batch_count == 0
    }
}

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
    pub local_metadata_reranker: bool,
    #[serde(default)]
    pub query_family_routed_reranker: bool,
    #[serde(default)]
    pub cache_enabled: bool,
    #[serde(default)]
    pub force_refresh: bool,
    #[serde(default)]
    pub parallelism: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LexicalBackendCorpusOptions {
    pub limit: usize,
    pub ranking_budget: usize,
    pub base: Option<String>,
    pub head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LexicalBackendCorpusReport {
    pub schema_version: String,
    pub eval_range_id: String,
    pub repo_id: String,
    pub refs: HistoricalEvalRefs,
    pub evaluated_commits: usize,
    pub ranking_budget: usize,
    pub bm25: LexicalBackendMetrics,
    pub legacy: LexicalBackendMetrics,
    pub comparison: LexicalBackendComparison,
    pub rows: Vec<LexicalBackendCommitRow>,
    pub runtime: LexicalBackendRuntimeSummary,
    pub privacy_status: PrivacyStatus,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LexicalBackendMetrics {
    pub backend: String,
    pub recall_at_5: f32,
    pub recall_at_10: f32,
    pub mrr_at_10: f32,
    pub average_result_count: f32,
    pub total_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LexicalBackendComparison {
    pub recall_delta_at_5: f32,
    pub recall_delta_at_10: f32,
    pub mrr_delta_at_10: f32,
    pub average_overlap_at_k: f32,
    pub top_path_changed_rate: f32,
    pub bm25_wins_at_10: usize,
    pub legacy_wins_at_10: usize,
    pub ties_at_10: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LexicalBackendRuntimeSummary {
    pub total_millis: u64,
    pub git_sample_millis: u64,
    #[serde(default)]
    pub inventory_warmup_millis: u64,
    pub bm25_total_millis: u64,
    pub legacy_total_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LexicalBackendCommitRow {
    pub sha: String,
    pub task_hash: String,
    pub retrieval_target_files: Vec<String>,
    pub bm25_files: Vec<String>,
    pub legacy_files: Vec<String>,
    pub bm25_hits_at_5: Vec<String>,
    pub bm25_hits_at_10: Vec<String>,
    pub legacy_hits_at_5: Vec<String>,
    pub legacy_hits_at_10: Vec<String>,
    pub bm25_recall_at_10: f32,
    pub legacy_recall_at_10: f32,
    pub overlap_at_k: usize,
    pub top_path_changed: bool,
    pub bm25_millis: u64,
    pub legacy_millis: u64,
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
    pub local_metadata_reranker: bool,
    #[serde(default)]
    pub query_family_routed_reranker: bool,
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
pub struct GraphEdgeAblationResult {
    pub eval_range_id: String,
    pub edge_label: String,
    pub evaluated_commits: usize,
    pub affected_commit_count: usize,
    pub removed_selected_at_10_count: usize,
    pub removed_target_hit_at_10_count: usize,
    pub metrics: RankingMetrics,
    pub recall_delta_at_k: f32,
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
    #[serde(default)]
    pub semantic_contribution: SemanticContributionSummary,
    #[serde(default)]
    pub reranker_contribution: RerankerContributionSummary,
    #[serde(default)]
    pub routed_reranker_contribution: RerankerContributionSummary,
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
    pub protected_evidence_miss_rate_at_10: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_evidence_target_miss_rate_at_10: Option<f32>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticContributionSummary {
    pub evaluated_commits: usize,
    pub commits_with_semantic_selection: usize,
    pub semantic_selected_file_count: usize,
    pub semantic_target_hit_count: usize,
    pub semantic_only_target_hit_count: usize,
    #[serde(default)]
    pub semantic_only_non_target_count: usize,
    pub semantic_lexical_overlap_count: usize,
    pub semantic_missed_target_count: usize,
    pub average_semantic_selected_files: f32,
    pub semantic_target_hit_rate: f32,
    pub semantic_only_target_hit_rate: f32,
    #[serde(default)]
    pub semantic_only_non_target_rate: f32,
    #[serde(default)]
    pub semantic_only_hits: Vec<SemanticPrecisionNamedCase>,
    #[serde(default)]
    pub semantic_only_non_targets: Vec<SemanticPrecisionNamedCase>,
    #[serde(default)]
    pub semantic_missed_target_gap_families: Vec<SemanticMissedTargetGapFamily>,
    #[serde(default)]
    pub query_family_contributions: Vec<SemanticQueryFamilyContribution>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticMissedTargetGapFamily {
    pub signal_gap: String,
    pub missed_count: usize,
    pub example_paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticQueryFamilyContribution {
    pub family: String,
    pub evaluated_commits: usize,
    pub commits_with_semantic_selection: usize,
    pub semantic_selected_file_count: usize,
    pub semantic_target_hit_count: usize,
    pub semantic_only_target_hit_count: usize,
    pub semantic_only_non_target_count: usize,
    pub semantic_missed_target_count: usize,
    pub semantic_only_target_hit_rate: f32,
    pub semantic_only_non_target_rate: f32,
    #[serde(default)]
    pub missed_target_gap_families: Vec<SemanticMissedTargetGapFamily>,
    #[serde(default)]
    pub example_cases: Vec<SemanticPrecisionNamedCase>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RerankerContributionSummary {
    pub evaluated_commits: usize,
    pub improved_commit_count: usize,
    pub regressed_commit_count: usize,
    pub neutral_commit_count: usize,
    pub default_target_hit_count: usize,
    pub reranked_target_hit_count: usize,
    pub target_hit_delta: i32,
    pub reranker_only_target_hit_count: usize,
    pub default_only_target_hit_count: usize,
    pub protected_evidence_miss_rate_delta: f32,
    pub protected_evidence_target_miss_rate_delta: f32,
    #[serde(default)]
    pub improved_cases: Vec<SemanticPrecisionNamedCase>,
    #[serde(default)]
    pub regressed_cases: Vec<SemanticPrecisionNamedCase>,
    #[serde(default)]
    pub default_only_cases: Vec<SemanticPrecisionNamedCase>,
    #[serde(default)]
    pub query_family_contributions: Vec<RerankerQueryFamilyContribution>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RerankerQueryFamilyContribution {
    pub family: String,
    pub evaluated_commits: usize,
    pub improved_commit_count: usize,
    pub regressed_commit_count: usize,
    pub neutral_commit_count: usize,
    pub default_target_hit_count: usize,
    pub reranked_target_hit_count: usize,
    pub target_hit_delta: i32,
    pub reranker_only_target_hit_count: usize,
    pub default_only_target_hit_count: usize,
    pub routing_recommendation: RerankerRoutingRecommendation,
    #[serde(default)]
    pub example_cases: Vec<SemanticPrecisionNamedCase>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RerankerRoutingRecommendation {
    RouteCandidate,
    HoldNeutral,
    HoldChurn,
    BlockRegression,
    #[default]
    InsufficientEvidence,
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub context_area: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub context_area_resource_uri: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub context_area_signal_counts: BTreeMap<String, usize>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub context_area_role_counts: BTreeMap<String, usize>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub context_area_selected_role_counts: BTreeMap<String, usize>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub context_area_unselected_count: usize,
    #[serde(
        default,
        skip_serializing_if = "inspection_pressure_breakdown_is_empty"
    )]
    pub context_area_inspection_pressure_breakdown: InspectionPressureBreakdown,
    pub target_status: RetrievalGapTargetStatus,
    pub recommendation_area: RetrievalGapRecommendationArea,
    pub missed_count: usize,
    pub example_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_read_paths: Vec<String>,
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
    ContextPlanning,
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
    pub release_gate: ProductProofReleaseGate,
    pub limitations: Vec<String>,
    pub helps_when: Vec<String>,
    pub does_not_help_when: Vec<String>,
    pub future_work: Vec<String>,
    #[serde(default)]
    pub recommended_research_actions: Vec<RecommendedResearchAction>,
    pub benchmark_report: BenchmarkSuiteReport,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedResearchAction {
    pub action: String,
    pub priority: u8,
    pub origin: String,
    pub reason: String,
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
pub struct ProductProofReleaseGate {
    pub decision: SemanticPrecisionGateDecision,
    pub decision_reason: String,
    pub default_promotion_allowed: bool,
    #[serde(default)]
    pub lexical_comparison: ProductProofLexicalComparison,
    #[serde(default)]
    pub lexical_backend_comparison: ProductProofLexicalBackendComparison,
    pub corpus_verdicts: Vec<ProductProofCorpusVerdict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofLexicalComparison {
    pub corpus_count: usize,
    pub all_file_beat_count: usize,
    pub all_file_match_count: usize,
    pub all_file_trail_count: usize,
    #[serde(default)]
    pub all_file_explained_trail_count: usize,
    #[serde(default)]
    pub all_file_unexplained_trail_count: usize,
    pub agent_evidence_beat_count: usize,
    pub agent_evidence_match_count: usize,
    pub agent_evidence_trail_count: usize,
    pub context_beat_count: usize,
    pub context_match_count: usize,
    pub context_trail_count: usize,
    pub average_file_recall_at_10: f32,
    pub average_lexical_file_recall_at_10: f32,
    pub average_file_delta_at_10: f32,
    pub average_agent_evidence_recall_at_10: f32,
    pub average_agent_evidence_delta_at_10: f32,
    pub average_context_recall_at_10: f32,
    pub average_lexical_context_recall_at_10: f32,
    pub average_context_delta_at_10: f32,
    pub all_file_claim: ProductProofLexicalClaim,
    pub agent_evidence_claim: ProductProofLexicalClaim,
    pub context_claim: ProductProofLexicalClaim,
}

impl Default for ProductProofLexicalComparison {
    fn default() -> Self {
        Self {
            corpus_count: 0,
            all_file_beat_count: 0,
            all_file_match_count: 0,
            all_file_trail_count: 0,
            all_file_explained_trail_count: 0,
            all_file_unexplained_trail_count: 0,
            agent_evidence_beat_count: 0,
            agent_evidence_match_count: 0,
            agent_evidence_trail_count: 0,
            context_beat_count: 0,
            context_match_count: 0,
            context_trail_count: 0,
            average_file_recall_at_10: 0.0,
            average_lexical_file_recall_at_10: 0.0,
            average_file_delta_at_10: 0.0,
            average_agent_evidence_recall_at_10: 0.0,
            average_agent_evidence_delta_at_10: 0.0,
            average_context_recall_at_10: 0.0,
            average_lexical_context_recall_at_10: 0.0,
            average_context_delta_at_10: 0.0,
            all_file_claim: ProductProofLexicalClaim::NoEvidence,
            agent_evidence_claim: ProductProofLexicalClaim::NoEvidence,
            context_claim: ProductProofLexicalClaim::NoEvidence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductProofLexicalClaim {
    BeatsAllCorpora,
    Mixed,
    MatchesAllCorpora,
    TrailsAnyCorpus,
    NoEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofLexicalBackendComparison {
    pub corpus_count: usize,
    pub evaluated_commit_count: usize,
    pub bm25_beat_count: usize,
    pub bm25_match_count: usize,
    pub bm25_trail_count: usize,
    pub average_bm25_recall_at_10: f32,
    pub average_legacy_recall_at_10: f32,
    pub average_recall_delta_at_10: f32,
    pub average_mrr_delta_at_10: f32,
    pub average_overlap_at_k: f32,
    pub average_top_path_changed_rate: f32,
    pub bm25_wins_at_10: usize,
    pub legacy_wins_at_10: usize,
    pub ties_at_10: usize,
    pub bm25_total_millis: u64,
    pub legacy_total_millis: u64,
    pub bm25_claim: ProductProofLexicalClaim,
}

impl Default for ProductProofLexicalBackendComparison {
    fn default() -> Self {
        Self {
            corpus_count: 0,
            evaluated_commit_count: 0,
            bm25_beat_count: 0,
            bm25_match_count: 0,
            bm25_trail_count: 0,
            average_bm25_recall_at_10: 0.0,
            average_legacy_recall_at_10: 0.0,
            average_recall_delta_at_10: 0.0,
            average_mrr_delta_at_10: 0.0,
            average_overlap_at_k: 0.0,
            average_top_path_changed_rate: 0.0,
            bm25_wins_at_10: 0,
            legacy_wins_at_10: 0,
            ties_at_10: 0,
            bm25_total_millis: 0,
            legacy_total_millis: 0,
            bm25_claim: ProductProofLexicalClaim::NoEvidence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductProofCorpusVerdict {
    pub repository: String,
    pub variant: String,
    pub status: ProductProofCorpusStatus,
    #[serde(default)]
    pub environment_health: BenchmarkRepoEnvironmentHealth,
    pub file_recall_at_10: f32,
    pub lexical_baseline_recall_at_10: f32,
    pub lexical_delta_at_10: f32,
    #[serde(default)]
    pub source_recall_at_10: f32,
    #[serde(default)]
    pub lexical_source_recall_at_10: f32,
    #[serde(default)]
    pub source_delta_at_10: f32,
    #[serde(default)]
    pub agent_evidence_recall_at_10: f32,
    #[serde(default)]
    pub agent_evidence_delta_at_10: f32,
    pub context_recall_at_10: f32,
    pub lexical_context_recall_at_10: f32,
    pub context_delta_at_10: f32,
    #[serde(default)]
    pub context_vs_all_file_delta_at_10: f32,
    #[serde(default)]
    pub lexical_context_vs_all_file_delta_at_10: f32,
    #[serde(default)]
    pub all_file_divergence_explained: bool,
    pub test_recall_at_10: f32,
    #[serde(default)]
    pub validation_command_recall: f32,
    #[serde(default)]
    pub effective_validation_recall_at_10: f32,
    pub protected_evidence_miss_rate_at_10: f32,
    #[serde(default)]
    pub protected_evidence_target_miss_rate_at_10: f32,
    pub runtime_millis: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductProofCorpusStatus {
    Beat,
    Match,
    Trail,
    InsufficientEvidence,
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
    #[serde(default)]
    pub graph_edge_ablations: Vec<GraphEdgeAblationResult>,
    pub token_roi: Vec<TokenRoiMetric>,
    pub retrieval_gap_summaries: Vec<RetrievalGapSummary>,
    #[serde(default)]
    pub graph_edge_profiles: Vec<GraphEdgeProfile>,
    pub runtime: HistoricalEvalRuntimeSummary,
    pub low_information_commit_count: usize,
    #[serde(default)]
    pub broad_scope_commit_count: usize,
    #[serde(default)]
    pub broad_context_area_recall: f32,
    #[serde(default)]
    pub context_area_pressure_summary: ContextAreaPressureSummary,
    #[serde(default)]
    pub context_area_next_read_summary: ContextAreaNextReadSummary,
    #[serde(default)]
    pub candidate_coverage_summary: CandidateCoverageSummary,
    #[serde(default)]
    pub memory_reuse_summary: MemoryReuseSummary,
    #[serde(default)]
    pub recommended_research_actions: Vec<RecommendedResearchAction>,
    pub file_recall_at_5: f32,
    pub file_recall_at_10: f32,
    pub lexical_baseline_recall_at_5: f32,
    pub lexical_baseline_recall_at_10: f32,
    pub ctxhelm_lift_at_5: f32,
    pub ctxhelm_lift_at_10: f32,
    pub source_recall_at_5: f32,
    pub source_recall_at_10: f32,
    pub test_recall_at_5: f32,
    pub test_recall_at_10: f32,
    #[serde(default)]
    pub validation_command_recall: f32,
    #[serde(default)]
    pub effective_validation_recall_at_10: f32,
    pub test_recommendation_rate: f32,
    pub average_recommended_context_files: f32,
    #[serde(default)]
    pub protected_evidence: ProtectedEvidenceSummary,
    pub top_missing_files: Vec<HistoricalMissingFileSummary>,
    pub commits: Vec<HistoricalCommitEval>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedEvidenceSummary {
    pub candidate_count: usize,
    pub missed_at_10_count: usize,
    pub miss_rate_at_10: f32,
    #[serde(default)]
    pub retrieval_target_candidate_count: usize,
    #[serde(default)]
    pub retrieval_target_missed_at_10_count: usize,
    #[serde(default)]
    pub retrieval_target_miss_rate_at_10: f32,
    #[serde(default)]
    pub non_target_candidate_count: usize,
    #[serde(default)]
    pub non_target_missed_at_10_count: usize,
    pub by_signal: Vec<ProtectedEvidenceSignalSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContextAreaPressureSummary {
    pub context_area_count: usize,
    pub zero_selected_area_count: usize,
    pub total_inspection_pressure: usize,
    pub source_like_unselected: usize,
    pub validation_unselected: usize,
    pub docs_unselected: usize,
    pub source_like_pressure: usize,
    pub validation_pressure: usize,
    pub docs_pressure: usize,
    pub highest_pressure_area: Option<ContextAreaPressurePeak>,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextAreaPressurePeak {
    pub area: String,
    pub resource_uri: String,
    pub inspection_pressure: usize,
    pub coverage_percent: u8,
    pub unselected_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContextAreaNextReadSummary {
    pub missed_file_count_at_10: usize,
    pub next_read_recoverable_count: usize,
    #[serde(default)]
    pub agent_evidence_recoverable_count: usize,
    #[serde(default)]
    pub agent_evidence_only_count: usize,
    #[serde(default)]
    pub agent_evidence_only_role_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub top_agent_evidence_only_areas: Vec<CandidateCoverageAreaSummary>,
    pub top_pressure_next_read_recoverable_count: usize,
    pub zero_selected_area_recoverable_count: usize,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CandidateCoverageSummary {
    pub missed_file_count_at_10: usize,
    pub candidate_recoverable_count: usize,
    pub no_candidate_count: usize,
    #[serde(default)]
    pub candidate_recoverable_role_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub candidate_recoverable_signal_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub no_candidate_role_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub top_candidate_recoverable_areas: Vec<CandidateCoverageAreaSummary>,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateCoverageAreaSummary {
    pub context_area: String,
    pub missed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReuseSummary {
    pub commits_with_memory_candidates: usize,
    pub memory_candidate_count: usize,
    pub memory_selected_at_10_count: usize,
    pub memory_target_hit_at_10_count: usize,
    pub memory_target_missed_at_10_count: usize,
    pub memory_unique_target_hit_count: usize,
    pub memory_unique_non_target_count: usize,
    #[serde(default)]
    pub memory_unique_target_hit_with_current_support_count: usize,
    #[serde(default)]
    pub memory_unique_target_hit_without_current_support_count: usize,
    #[serde(default)]
    pub memory_unique_non_target_with_current_support_count: usize,
    #[serde(default)]
    pub memory_unique_non_target_without_current_support_count: usize,
    #[serde(default)]
    pub memory_unique_target_hit_current_support_signal_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub memory_unique_non_target_current_support_signal_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub selected_role_counts: BTreeMap<String, usize>,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateMissedFileProfile {
    pub path: String,
    pub role: FileRole,
    pub context_area: String,
    pub signals: Vec<RetrievalSignalKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedEvidenceSignalSummary {
    pub signal: RetrievalSignalKind,
    pub candidate_count: usize,
    pub missed_at_10_count: usize,
    #[serde(default)]
    pub retrieval_target_candidate_count: usize,
    #[serde(default)]
    pub retrieval_target_missed_at_10_count: usize,
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
    pub local_metadata_reranker: bool,
    #[serde(default)]
    pub cache_enabled: bool,
    #[serde(default)]
    pub force_refresh: bool,
    #[serde(default = "default_benchmark_parallelism")]
    pub parallelism: usize,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
    #[serde(default)]
    pub lexical_backend_comparison: bool,
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
            local_metadata_reranker: false,
            cache_enabled: false,
            force_refresh: false,
            parallelism: default_benchmark_parallelism(),
            role_filters: Vec::new(),
            lexical_backend_comparison: false,
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
    pub local_metadata_reranker: Option<bool>,
    #[serde(default)]
    pub cache_enabled: Option<bool>,
    #[serde(default)]
    pub force_refresh: Option<bool>,
    #[serde(default)]
    pub parallelism: Option<usize>,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
    #[serde(default)]
    pub lexical_backend_comparison: Option<bool>,
    #[serde(default)]
    pub proof_runtime_ceiling_millis: Option<u64>,
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
    #[serde(default)]
    pub environment_health: BenchmarkRepoEnvironmentHealth,
    pub baseline: Option<BenchmarkRepoBaseline>,
    pub baseline_status: Option<BenchmarkRepoBaselineStatus>,
    pub evaluated_commits: usize,
    pub excluded_changed_file_count: usize,
    pub skipped_path_count: usize,
    pub report: Option<HistoricalEvalReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical_backend_corpus: Option<LexicalBackendCorpusReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical_backend_error: Option<String>,
    pub error: Option<String>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRepoEnvironmentHealth {
    pub status: BenchmarkRepoEnvironmentStatus,
    pub git_history_usable: bool,
    pub object_content_usable: Option<bool>,
    pub reason: String,
}

impl Default for BenchmarkRepoEnvironmentHealth {
    fn default() -> Self {
        Self {
            status: BenchmarkRepoEnvironmentStatus::Unknown,
            git_history_usable: false,
            object_content_usable: None,
            reason: "Environment health was not recorded for this benchmark report.".to_string(),
        }
    }
}

impl BenchmarkRepoEnvironmentHealth {
    fn healthy() -> Self {
        Self {
            status: BenchmarkRepoEnvironmentStatus::Healthy,
            git_history_usable: true,
            object_content_usable: Some(true),
            reason: "Git history and object-content reads produced evaluable benchmark evidence."
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkRepoEnvironmentStatus {
    Healthy,
    GitHistoryTimeout,
    GitHistoryUnavailable,
    GitObjectStoreUnavailable,
    Degraded,
    Unknown,
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
    pub local_metadata_reranker: bool,
    pub cache_enabled: bool,
    pub force_refresh: bool,
    pub parallelism: usize,
    #[serde(default)]
    pub role_filters: Vec<FileRole>,
    #[serde(default)]
    pub lexical_backend_comparison: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_runtime_ceiling_millis: Option<u64>,
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
pub struct GraphEdgeProfile {
    pub edge_label: String,
    pub candidate_count: usize,
    pub selected_at_10_count: usize,
    pub retrieval_target_count: usize,
    pub retrieval_target_hit_at_10_count: usize,
    pub retrieval_target_missed_at_10_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalSignalRanking {
    pub signal: RetrievalSignalKind,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalSelectedSignalProfile {
    pub signal: RetrievalSignalKind,
    pub role: FileRole,
    pub selected_at_10_count: usize,
    pub retrieval_target_selected_at_10_count: usize,
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
    #[serde(default)]
    pub retrieval_target_files: Vec<String>,
    pub excluded_changed_file_count: usize,
    pub recommended_files: Vec<String>,
    pub recommended_tests: Vec<String>,
    pub recommended_context_files: Vec<String>,
    pub recommended_commands: Vec<String>,
    pub lexical_baseline_files: Vec<String>,
    #[serde(default)]
    pub signal_baseline_files: Vec<HistoricalSignalRanking>,
    #[serde(default)]
    pub selected_signal_profiles: Vec<HistoricalSelectedSignalProfile>,
    #[serde(default)]
    pub protected_evidence: Vec<HistoricalProtectedEvidenceFile>,
    #[serde(default)]
    pub graph_edge_profiles: Vec<GraphEdgeProfile>,
    pub file_hits_at_5: Vec<String>,
    pub file_hits_at_10: Vec<String>,
    pub lexical_baseline_hits_at_5: Vec<String>,
    pub lexical_baseline_hits_at_10: Vec<String>,
    pub missing_files_at_10: Vec<String>,
    #[serde(default)]
    pub candidate_missed_files_at_10: Vec<String>,
    #[serde(default)]
    pub candidate_missed_file_profiles_at_10: Vec<CandidateMissedFileProfile>,
    pub source_files_changed: usize,
    pub source_hits_at_5: usize,
    pub source_hits_at_10: usize,
    pub test_files_changed: usize,
    pub test_hits_at_5: usize,
    pub test_hits_at_10: usize,
    #[serde(default)]
    pub validation_command_hits: usize,
    #[serde(default)]
    pub effective_validation_hits_at_10: usize,
    pub low_information_task: bool,
    #[serde(default)]
    pub broad_scope_task: bool,
    #[serde(default)]
    pub changed_context_areas: Vec<String>,
    #[serde(default)]
    pub context_area_hits: Vec<String>,
    #[serde(default)]
    pub context_areas: Vec<ContextArea>,
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_trace: Option<QueryConstructionTrace>,
    #[serde(default)]
    pub elapsed_millis: u64,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalProtectedEvidenceFile {
    pub path: String,
    pub signals: Vec<RetrievalSignalKind>,
    pub selected_at_10: bool,
    #[serde(default)]
    pub retrieval_target: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<FileRole>,
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
        .map(|report| report.ctxhelm_lift_at_10)
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
    let release_gate = product_proof_release_gate(&benchmark_report);
    let recommended_research_actions =
        product_proof_recommended_research_actions(&benchmark_report, &release_gate);

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
                label: "averageCtxhelmLiftAt10".to_string(),
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
        release_gate,
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
            "v2.5: ship only variants that beat lexical on every required corpus without unsafe privacy, runtime, or protected-evidence regressions.".to_string(),
        ],
        recommended_research_actions,
        benchmark_report,
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn product_proof_release_gate(benchmark_report: &BenchmarkSuiteReport) -> ProductProofReleaseGate {
    let corpus_verdicts = benchmark_report
        .repositories
        .iter()
        .map(product_proof_corpus_verdict)
        .collect::<Vec<_>>();
    let decision = product_proof_release_decision(benchmark_report, &corpus_verdicts);
    ProductProofReleaseGate {
        default_promotion_allowed: decision.0 == SemanticPrecisionGateDecision::Promote,
        decision: decision.0,
        decision_reason: decision.1,
        lexical_comparison: product_proof_lexical_comparison(&corpus_verdicts),
        lexical_backend_comparison: product_proof_lexical_backend_comparison(benchmark_report),
        corpus_verdicts,
    }
}

fn product_proof_recommended_research_actions(
    benchmark_report: &BenchmarkSuiteReport,
    release_gate: &ProductProofReleaseGate,
) -> Vec<RecommendedResearchAction> {
    let mut actions = Vec::new();
    if benchmark_report.evaluated_repository_count == 0 || release_gate.corpus_verdicts.is_empty() {
        push_research_action(
            &mut actions,
            "collect_benchmark_evidence",
            1,
            "product_proof",
            "No evaluated repository reports were embedded, so product lift is not evidence-backed.",
        );
        return actions;
    }

    if release_gate
        .corpus_verdicts
        .iter()
        .any(|verdict| verdict.status == ProductProofCorpusStatus::InsufficientEvidence)
    {
        push_research_action(
            &mut actions,
            "refresh_fixture_or_history_evidence",
            1,
            "product_proof",
            "At least one corpus has insufficient source-free history or fixture evidence.",
        );
    }

    if release_gate
        .corpus_verdicts
        .iter()
        .any(|verdict| verdict.notes.iter().any(|note| note.contains("runtime")))
    {
        push_research_action(
            &mut actions,
            "reduce_runtime_or_refresh_cache",
            1,
            "product_proof",
            "At least one corpus verdict is constrained by runtime evidence.",
        );
    }

    if release_gate
        .corpus_verdicts
        .iter()
        .any(|verdict| verdict.protected_evidence_target_miss_rate_at_10 > 0.0)
    {
        push_research_action(
            &mut actions,
            "protect_high_confidence_evidence",
            1,
            "product_proof",
            "A corpus still misses protected retrieval-target evidence inside the top-10 budget.",
        );
    }

    if release_gate
        .corpus_verdicts
        .iter()
        .any(|verdict| verdict.status == ProductProofCorpusStatus::Trail)
        || release_gate
            .lexical_comparison
            .all_file_unexplained_trail_count
            > 0
    {
        push_research_action(
            &mut actions,
            "fix_retrieval_or_ranking_regression",
            1,
            "product_proof",
            "A corpus trails lexical without an explained context or validation-channel boundary.",
        );
    }

    if release_gate
        .lexical_comparison
        .all_file_explained_trail_count
        > 0
        && release_gate.lexical_comparison.agent_evidence_trail_count == 0
        && release_gate.lexical_comparison.context_trail_count == 0
    {
        push_research_action(
            &mut actions,
            "analyze_native_baseline_gap",
            2,
            "product_proof",
            "Raw all-file lexical trails are explained while agent-evidence and context-channel claims do not trail.",
        );
    }

    if release_gate.lexical_backend_comparison.bm25_claim
        == ProductProofLexicalClaim::TrailsAnyCorpus
    {
        push_research_action(
            &mut actions,
            "investigate_bm25_backend_regression",
            2,
            "product_proof",
            "BM25 lexical backend comparison trails the legacy lexical backend on at least one corpus.",
        );
    } else if release_gate.lexical_backend_comparison.bm25_claim
        == ProductProofLexicalClaim::NoEvidence
    {
        push_research_action(
            &mut actions,
            "collect_bm25_backend_evidence",
            3,
            "product_proof",
            "No BM25-vs-legacy lexical backend evidence is embedded in this product proof.",
        );
    }

    let historical_reports = benchmark_report
        .repositories
        .iter()
        .filter_map(|repo| repo.report.as_ref())
        .collect::<Vec<_>>();
    if !historical_reports.is_empty() {
        let memory_candidate_count = historical_reports
            .iter()
            .map(|report| report.memory_reuse_summary.memory_candidate_count)
            .sum::<usize>();
        let memory_unique_target_hit_count = historical_reports
            .iter()
            .map(|report| report.memory_reuse_summary.memory_unique_target_hit_count)
            .sum::<usize>();
        let memory_unique_non_target_count = historical_reports
            .iter()
            .map(|report| report.memory_reuse_summary.memory_unique_non_target_count)
            .sum::<usize>();

        if memory_candidate_count == 0 {
            push_research_action(
                &mut actions,
                "prove_memory_reuse",
                3,
                "product_proof",
                "No embedded corpus report has fresh approved memory candidates, so memory reuse lift is unproven.",
            );
        } else if memory_unique_target_hit_count > 0 {
            push_research_action(
                &mut actions,
                "evaluate_memory_reuse_lift",
                2,
                "product_proof",
                "Embedded corpus reports include memory target hits that are absent from lexical baseline evidence.",
            );
        } else if memory_unique_non_target_count > 0 {
            push_research_action(
                &mut actions,
                "reduce_memory_retrieval_noise",
                2,
                "product_proof",
                "Embedded corpus reports include unique memory non-targets without unique target-hit evidence.",
            );
        }
    }

    if release_gate.default_promotion_allowed && actions.is_empty() {
        push_research_action(
            &mut actions,
            "preserve_current_contract",
            3,
            "product_proof",
            "The product proof promotes without source-free evidence of a blocking retrieval, runtime, or privacy gap.",
        );
    }

    actions
}

fn push_research_action(
    actions: &mut Vec<RecommendedResearchAction>,
    action: &str,
    priority: u8,
    source: &str,
    reason: &str,
) {
    if actions
        .iter()
        .any(|existing| existing.action == action && existing.origin == source)
    {
        return;
    }
    actions.push(RecommendedResearchAction {
        action: action.to_string(),
        priority,
        origin: source.to_string(),
        reason: reason.to_string(),
    });
    actions.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.action.cmp(&right.action))
            .then_with(|| left.origin.cmp(&right.origin))
    });
}

fn product_proof_lexical_backend_comparison(
    benchmark_report: &BenchmarkSuiteReport,
) -> ProductProofLexicalBackendComparison {
    let reports = benchmark_report
        .repositories
        .iter()
        .filter_map(|repo| repo.lexical_backend_corpus.as_ref())
        .filter(|report| report.evaluated_commits > 0)
        .collect::<Vec<_>>();
    if reports.is_empty() {
        return ProductProofLexicalBackendComparison::default();
    }

    let corpus_count = reports.len();
    let corpus_count_f32 = corpus_count as f32;
    let bm25_beat_count = reports
        .iter()
        .filter(|report| report.comparison.recall_delta_at_10 > 0.03)
        .count();
    let bm25_trail_count = reports
        .iter()
        .filter(|report| report.comparison.recall_delta_at_10 < -0.03)
        .count();
    let bm25_match_count = corpus_count - bm25_beat_count - bm25_trail_count;
    let evaluated_commit_count = reports
        .iter()
        .map(|report| report.evaluated_commits)
        .sum::<usize>();
    let average_bm25_recall_at_10 = reports
        .iter()
        .map(|report| report.bm25.recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_legacy_recall_at_10 = reports
        .iter()
        .map(|report| report.legacy.recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_recall_delta_at_10 = reports
        .iter()
        .map(|report| report.comparison.recall_delta_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_mrr_delta_at_10 = reports
        .iter()
        .map(|report| report.comparison.mrr_delta_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_overlap_at_k = reports
        .iter()
        .map(|report| report.comparison.average_overlap_at_k)
        .sum::<f32>()
        / corpus_count_f32;
    let average_top_path_changed_rate = reports
        .iter()
        .map(|report| report.comparison.top_path_changed_rate)
        .sum::<f32>()
        / corpus_count_f32;
    let bm25_wins_at_10 = reports
        .iter()
        .map(|report| report.comparison.bm25_wins_at_10)
        .sum::<usize>();
    let legacy_wins_at_10 = reports
        .iter()
        .map(|report| report.comparison.legacy_wins_at_10)
        .sum::<usize>();
    let ties_at_10 = reports
        .iter()
        .map(|report| report.comparison.ties_at_10)
        .sum::<usize>();
    let bm25_total_millis = reports
        .iter()
        .map(|report| report.runtime.bm25_total_millis)
        .sum::<u64>();
    let legacy_total_millis = reports
        .iter()
        .map(|report| report.runtime.legacy_total_millis)
        .sum::<u64>();

    ProductProofLexicalBackendComparison {
        corpus_count,
        evaluated_commit_count,
        bm25_beat_count,
        bm25_match_count,
        bm25_trail_count,
        average_bm25_recall_at_10,
        average_legacy_recall_at_10,
        average_recall_delta_at_10,
        average_mrr_delta_at_10,
        average_overlap_at_k,
        average_top_path_changed_rate,
        bm25_wins_at_10,
        legacy_wins_at_10,
        ties_at_10,
        bm25_total_millis,
        legacy_total_millis,
        bm25_claim: lexical_claim(
            corpus_count,
            bm25_beat_count,
            bm25_match_count,
            bm25_trail_count,
        ),
    }
}

fn product_proof_lexical_comparison(
    corpus_verdicts: &[ProductProofCorpusVerdict],
) -> ProductProofLexicalComparison {
    let evaluated = corpus_verdicts
        .iter()
        .filter(|verdict| verdict.status != ProductProofCorpusStatus::InsufficientEvidence)
        .collect::<Vec<_>>();
    if evaluated.is_empty() {
        return ProductProofLexicalComparison::default();
    }
    let corpus_count = evaluated.len();
    let corpus_count_f32 = corpus_count as f32;
    let all_file_beat_count = evaluated
        .iter()
        .filter(|verdict| verdict.lexical_delta_at_10 > 0.03)
        .count();
    let all_file_trail_count = evaluated
        .iter()
        .filter(|verdict| verdict.lexical_delta_at_10 < -0.03)
        .count();
    let all_file_explained_trail_count = evaluated
        .iter()
        .filter(|verdict| {
            verdict.lexical_delta_at_10 < -0.03 && verdict.all_file_divergence_explained
        })
        .count();
    let all_file_unexplained_trail_count =
        all_file_trail_count.saturating_sub(all_file_explained_trail_count);
    let all_file_match_count = corpus_count - all_file_beat_count - all_file_trail_count;
    let agent_evidence_beat_count = evaluated
        .iter()
        .filter(|verdict| verdict.agent_evidence_delta_at_10 > 0.03)
        .count();
    let agent_evidence_trail_count = evaluated
        .iter()
        .filter(|verdict| verdict.agent_evidence_delta_at_10 < -0.03)
        .count();
    let agent_evidence_match_count =
        corpus_count - agent_evidence_beat_count - agent_evidence_trail_count;
    let context_beat_count = evaluated
        .iter()
        .filter(|verdict| verdict.context_delta_at_10 > 0.03)
        .count();
    let context_trail_count = evaluated
        .iter()
        .filter(|verdict| verdict.context_delta_at_10 < -0.03)
        .count();
    let context_match_count = corpus_count - context_beat_count - context_trail_count;
    let average_file_recall_at_10 = evaluated
        .iter()
        .map(|verdict| verdict.file_recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_lexical_file_recall_at_10 = evaluated
        .iter()
        .map(|verdict| verdict.lexical_baseline_recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_agent_evidence_recall_at_10 = evaluated
        .iter()
        .map(|verdict| verdict.agent_evidence_recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_context_recall_at_10 = evaluated
        .iter()
        .map(|verdict| verdict.context_recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;
    let average_lexical_context_recall_at_10 = evaluated
        .iter()
        .map(|verdict| verdict.lexical_context_recall_at_10)
        .sum::<f32>()
        / corpus_count_f32;

    ProductProofLexicalComparison {
        corpus_count,
        all_file_beat_count,
        all_file_match_count,
        all_file_trail_count,
        all_file_explained_trail_count,
        all_file_unexplained_trail_count,
        agent_evidence_beat_count,
        agent_evidence_match_count,
        agent_evidence_trail_count,
        context_beat_count,
        context_match_count,
        context_trail_count,
        average_file_delta_at_10: average_file_recall_at_10 - average_lexical_file_recall_at_10,
        average_agent_evidence_delta_at_10: average_agent_evidence_recall_at_10
            - average_lexical_file_recall_at_10,
        average_context_delta_at_10: average_context_recall_at_10
            - average_lexical_context_recall_at_10,
        average_file_recall_at_10,
        average_lexical_file_recall_at_10,
        average_agent_evidence_recall_at_10,
        average_context_recall_at_10,
        average_lexical_context_recall_at_10,
        all_file_claim: lexical_claim(
            corpus_count,
            all_file_beat_count,
            all_file_match_count + all_file_explained_trail_count,
            all_file_unexplained_trail_count,
        ),
        agent_evidence_claim: lexical_claim(
            corpus_count,
            agent_evidence_beat_count,
            agent_evidence_match_count,
            agent_evidence_trail_count,
        ),
        context_claim: lexical_claim(
            corpus_count,
            context_beat_count,
            context_match_count,
            context_trail_count,
        ),
    }
}

fn lexical_claim(
    corpus_count: usize,
    beat_count: usize,
    match_count: usize,
    trail_count: usize,
) -> ProductProofLexicalClaim {
    if corpus_count == 0 {
        ProductProofLexicalClaim::NoEvidence
    } else if trail_count > 0 {
        ProductProofLexicalClaim::TrailsAnyCorpus
    } else if beat_count == corpus_count {
        ProductProofLexicalClaim::BeatsAllCorpora
    } else if match_count == corpus_count {
        ProductProofLexicalClaim::MatchesAllCorpora
    } else {
        ProductProofLexicalClaim::Mixed
    }
}

fn product_proof_release_decision(
    benchmark_report: &BenchmarkSuiteReport,
    corpus_verdicts: &[ProductProofCorpusVerdict],
) -> (SemanticPrecisionGateDecision, String) {
    if !benchmark_report.privacy_status.local_only {
        return (
            SemanticPrecisionGateDecision::Block,
            "Blocked because benchmark proof is not local-only.".to_string(),
        );
    }
    if corpus_verdicts.is_empty() {
        return (
            SemanticPrecisionGateDecision::Block,
            "Blocked because no repositories produced product-proof verdicts.".to_string(),
        );
    }
    let failed = corpus_verdicts
        .iter()
        .filter(|verdict| product_proof_verdict_blocks_promotion(verdict))
        .map(|verdict| format!("{}:{:?}", verdict.repository, verdict.status))
        .collect::<Vec<_>>();
    if !failed.is_empty() {
        let environment_failures = corpus_verdicts
            .iter()
            .filter(|verdict| {
                matches!(
                    verdict.status,
                    ProductProofCorpusStatus::InsufficientEvidence
                ) && verdict.environment_health.status != BenchmarkRepoEnvironmentStatus::Healthy
            })
            .map(|verdict| {
                format!(
                    "{}:{:?}",
                    verdict.repository, verdict.environment_health.status
                )
            })
            .collect::<Vec<_>>();
        if !environment_failures.is_empty() {
            return (
                SemanticPrecisionGateDecision::Block,
                format!(
                    "Blocked because benchmark environment health is degraded before retrieval quality can be proven; affected corpora: {}.",
                    environment_failures.join(", ")
                ),
            );
        }
        return (
            SemanticPrecisionGateDecision::Block,
            format!(
                "Blocked because default promotion requires every corpus to beat lexical on the non-test context channel and maintain validation-test recall; failing corpora: {}.",
                failed.join(", ")
            ),
        );
    }
    let source_regressions = corpus_verdicts
        .iter()
        .filter(|verdict| product_proof_source_recall_blocks_promotion(verdict))
        .map(|verdict| format!("{}:{:.3}", verdict.repository, verdict.source_delta_at_10))
        .collect::<Vec<_>>();
    if !source_regressions.is_empty() {
        return (
            SemanticPrecisionGateDecision::Block,
            format!(
                "Blocked because source Recall@10 trailed lexical beyond the promotion tolerance for: {}.",
                source_regressions.join(", ")
            ),
        );
    }
    let stale_or_slow_warm_cache = benchmark_report
        .repositories
        .iter()
        .filter(|repo| product_proof_warm_cache_runtime_blocks_promotion(repo))
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();
    if !stale_or_slow_warm_cache.is_empty() {
        return (
            SemanticPrecisionGateDecision::Block,
            format!(
                "Blocked because cached proof runtime did not satisfy warm-cache thresholds for: {}.",
                stale_or_slow_warm_cache.join(", ")
            ),
        );
    }
    let slow = corpus_verdicts
        .iter()
        .filter_map(|verdict| {
            let Some(repo) = benchmark_report
                .repositories
                .iter()
                .find(|repo| repo.name == verdict.repository)
            else {
                return Some(format!("{}:missing-report", verdict.repository));
            };
            if product_proof_runtime_blocks_promotion(repo, verdict) {
                Some(format!(
                    "{}:{}ms>{}ms",
                    verdict.repository,
                    product_proof_runtime_per_commit_millis(repo, verdict),
                    product_proof_runtime_ceiling_millis(repo)
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if !slow.is_empty() {
        return (
            SemanticPrecisionGateDecision::Block,
            format!(
                "Blocked because proof runtime exceeded the configured per-commit ceiling for: {}.",
                slow.join(", ")
            ),
        );
    }
    (
        SemanticPrecisionGateDecision::Promote,
        "Promote: every evaluated corpus beat lexical or reached a perfect lexical ceiling on non-test context recall, while maintaining validation-test recall under local-only proof thresholds."
            .to_string(),
    )
}

const PRODUCT_PROOF_DEFAULT_RUNTIME_CEILING_MILLIS: u64 = 5_000;
const PRODUCT_PROOF_PERFECT_COLD_START_CEILING_MILLIS: u64 = 10_000;

fn product_proof_runtime_ceiling_millis(repo: &BenchmarkRepoReport) -> u64 {
    repo.effective_config
        .proof_runtime_ceiling_millis
        .unwrap_or(PRODUCT_PROOF_DEFAULT_RUNTIME_CEILING_MILLIS)
}

fn product_proof_runtime_per_commit_millis(
    repo: &BenchmarkRepoReport,
    verdict: &ProductProofCorpusVerdict,
) -> u64 {
    let commits = repo.evaluated_commits.max(1) as u64;
    verdict.runtime_millis / commits
}

fn product_proof_warm_cache_runtime_blocks_promotion(repo: &BenchmarkRepoReport) -> bool {
    let Some(report) = repo.report.as_ref() else {
        return false;
    };
    let runtime = &report.runtime;
    if runtime.cache_hits == 0 {
        return false;
    }
    runtime.cache_misses > 0
        || runtime.total_millis > 1_000
        || runtime.commit_millis > 0
        || runtime.git_sample_millis > 0
        || runtime.ranking_millis > 0
        || runtime.pack_compiler_millis > 0
        || !runtime.slow_commits.is_empty()
}

fn product_proof_runtime_blocks_promotion(
    repo: &BenchmarkRepoReport,
    verdict: &ProductProofCorpusVerdict,
) -> bool {
    let commits = repo.evaluated_commits.max(1);
    let per_commit_millis = product_proof_runtime_per_commit_millis(repo, verdict);
    if per_commit_millis <= product_proof_runtime_ceiling_millis(repo) {
        return false;
    }

    // A one-commit cold historical snapshot can exceed the steady-state
    // throughput threshold even when ctxhelm has perfect context coverage and
    // no protected target misses. Keep that as a diagnostic, not a promotion
    // blocker, as long as it remains below the hard cold-start ceiling.
    !(repo.effective_config.proof_runtime_ceiling_millis.is_none()
        && commits == 1
        && per_commit_millis <= PRODUCT_PROOF_PERFECT_COLD_START_CEILING_MILLIS
        && product_proof_is_perfect_ceiling_match(verdict))
}

fn product_proof_verdict_blocks_promotion(verdict: &ProductProofCorpusVerdict) -> bool {
    if !verdict.all_file_divergence_explained {
        return true;
    }
    match verdict.status {
        ProductProofCorpusStatus::Beat => false,
        ProductProofCorpusStatus::Match => !product_proof_is_perfect_ceiling_match(verdict),
        ProductProofCorpusStatus::Trail | ProductProofCorpusStatus::InsufficientEvidence => true,
    }
}

fn product_proof_source_recall_blocks_promotion(verdict: &ProductProofCorpusVerdict) -> bool {
    verdict.source_delta_at_10 < -0.03
}

fn product_proof_is_perfect_ceiling_match(verdict: &ProductProofCorpusVerdict) -> bool {
    verdict.context_recall_at_10 >= 0.999
        && verdict.lexical_context_recall_at_10 >= 0.999
        && verdict.protected_evidence_target_miss_rate_at_10 == 0.0
}

fn product_proof_corpus_verdict(repo: &BenchmarkRepoReport) -> ProductProofCorpusVerdict {
    let variant = product_proof_variant_label(&repo.effective_config);
    let Some(report) = repo.report.as_ref() else {
        return ProductProofCorpusVerdict {
            repository: repo.name.clone(),
            variant,
            status: ProductProofCorpusStatus::InsufficientEvidence,
            environment_health: repo.environment_health.clone(),
            file_recall_at_10: 0.0,
            lexical_baseline_recall_at_10: 0.0,
            lexical_delta_at_10: 0.0,
            source_recall_at_10: 0.0,
            lexical_source_recall_at_10: 0.0,
            source_delta_at_10: 0.0,
            agent_evidence_recall_at_10: 0.0,
            agent_evidence_delta_at_10: 0.0,
            context_recall_at_10: 0.0,
            lexical_context_recall_at_10: 0.0,
            context_delta_at_10: 0.0,
            context_vs_all_file_delta_at_10: 0.0,
            lexical_context_vs_all_file_delta_at_10: 0.0,
            all_file_divergence_explained: false,
            test_recall_at_10: 0.0,
            validation_command_recall: 0.0,
            effective_validation_recall_at_10: 0.0,
            protected_evidence_miss_rate_at_10: 1.0,
            protected_evidence_target_miss_rate_at_10: 1.0,
            runtime_millis: 0,
            notes: vec![repo
                .error
                .clone()
                .unwrap_or_else(|| "repository did not produce an eval report".to_string())],
        };
    };
    let lexical_delta_at_10 = report.file_recall_at_10 - report.lexical_baseline_recall_at_10;
    let lexical_source_recall_at_10 =
        average_source_recall_for_ranking(&report.commits, |commit| &commit.lexical_baseline_files);
    let source_delta_at_10 = report.source_recall_at_10 - lexical_source_recall_at_10;
    let agent_evidence_recall_at_10 = agent_evidence_recall_at_10(report);
    let agent_evidence_delta_at_10 =
        agent_evidence_recall_at_10 - report.lexical_baseline_recall_at_10;
    let context_comparison = context_recall_comparison_at_10(report);
    let context_vs_all_file_delta_at_10 = context_comparison.ctxhelm - report.file_recall_at_10;
    let lexical_context_vs_all_file_delta_at_10 =
        context_comparison.lexical - report.lexical_baseline_recall_at_10;
    let runtime_per_commit_millis =
        report.runtime.total_millis / (repo.evaluated_commits.max(1) as u64);
    let runtime_ceiling_millis = product_proof_runtime_ceiling_millis(repo);
    let validation_target_count = report
        .commits
        .iter()
        .map(|commit| commit.test_files_changed)
        .sum::<usize>();
    let all_file_recall_trails_lexical = lexical_delta_at_10 < -0.03;
    let all_file_divergence_explained = !all_file_recall_trails_lexical
        || (validation_target_count > 0
            && context_comparison.delta >= -0.03
            && report.effective_validation_recall_at_10 >= 0.80);
    let status = if report.evaluated_commits == 0 {
        ProductProofCorpusStatus::InsufficientEvidence
    } else if validation_target_count > 0 && report.effective_validation_recall_at_10 < 0.80 {
        ProductProofCorpusStatus::Trail
    } else if context_comparison.delta > 0.03 {
        ProductProofCorpusStatus::Beat
    } else if context_comparison.delta < -0.03 {
        ProductProofCorpusStatus::Trail
    } else {
        ProductProofCorpusStatus::Match
    };
    let mut notes = Vec::new();
    if report.evaluated_commits == 0 {
        notes.push(repo.error.clone().unwrap_or_else(|| {
            "repository produced no evaluable commits; git history is unavailable or degraded"
                .to_string()
        }));
    }
    if validation_target_count > 0 && report.test_recall_at_10 == 0.0 {
        notes.push("test recall at 10 is zero".to_string());
    }
    if validation_target_count > 0
        && report.effective_validation_recall_at_10 > report.test_recall_at_10
    {
        notes.push(format!(
            "broad validation commands raise effective validation recall from {:.3} to {:.3}",
            report.test_recall_at_10, report.effective_validation_recall_at_10
        ));
    }
    if all_file_recall_trails_lexical {
        if all_file_divergence_explained {
            notes.push(
                "all-file recall trails lexical only after mixing validation targets into the file channel; context and validation channels are non-regressed"
                    .to_string(),
            );
        } else {
            notes.push(
                "all-file recall trails lexical and is not explained by the separate context and validation channels"
                    .to_string(),
            );
        }
    }
    if report.protected_evidence.miss_rate_at_10 > 0.0 {
        notes.push(format!(
            "protected evidence miss rate at 10 is {:.3} overall, {:.3} for retrieval targets",
            report.protected_evidence.miss_rate_at_10,
            report.protected_evidence.retrieval_target_miss_rate_at_10
        ));
    }
    if source_delta_at_10 < -0.03 {
        notes.push(format!(
            "source recall at 10 trails lexical by {:.3}",
            source_delta_at_10
        ));
    }
    if matches!(status, ProductProofCorpusStatus::Match)
        && context_comparison.ctxhelm >= 0.999
        && context_comparison.lexical >= 0.999
        && report.protected_evidence.retrieval_target_miss_rate_at_10 == 0.0
    {
        notes.push(
            "context channel reached the lexical ceiling with zero protected target misses"
                .to_string(),
        );
    }
    if runtime_per_commit_millis > PRODUCT_PROOF_DEFAULT_RUNTIME_CEILING_MILLIS
        && runtime_per_commit_millis <= runtime_ceiling_millis
        && repo.effective_config.proof_runtime_ceiling_millis.is_some()
    {
        notes.push(format!(
            "proof runtime used repo-scoped {}ms per-commit ceiling; observed {}ms per commit",
            runtime_ceiling_millis, runtime_per_commit_millis
        ));
    }
    if repo.evaluated_commits == 1
        && report.runtime.total_millis > PRODUCT_PROOF_DEFAULT_RUNTIME_CEILING_MILLIS
        && report.runtime.total_millis <= PRODUCT_PROOF_PERFECT_COLD_START_CEILING_MILLIS
        && matches!(status, ProductProofCorpusStatus::Match)
        && context_comparison.ctxhelm >= 0.999
        && context_comparison.lexical >= 0.999
        && report.protected_evidence.retrieval_target_miss_rate_at_10 == 0.0
    {
        notes.push(format!(
            "single-commit cold proof exceeded {}ms but stayed under the {}ms cold-start diagnostic ceiling",
            PRODUCT_PROOF_DEFAULT_RUNTIME_CEILING_MILLIS,
            PRODUCT_PROOF_PERFECT_COLD_START_CEILING_MILLIS
        ));
    }
    if report.runtime.cache_hits > 0 {
        notes.push(format!(
            "warm cache proof hit {} cached report(s) in {}ms",
            report.runtime.cache_hits, report.runtime.total_millis
        ));
    }
    ProductProofCorpusVerdict {
        repository: repo.name.clone(),
        variant,
        status,
        environment_health: repo.environment_health.clone(),
        file_recall_at_10: report.file_recall_at_10,
        lexical_baseline_recall_at_10: report.lexical_baseline_recall_at_10,
        lexical_delta_at_10,
        source_recall_at_10: report.source_recall_at_10,
        lexical_source_recall_at_10,
        source_delta_at_10,
        agent_evidence_recall_at_10,
        agent_evidence_delta_at_10,
        context_recall_at_10: context_comparison.ctxhelm,
        lexical_context_recall_at_10: context_comparison.lexical,
        context_delta_at_10: context_comparison.delta,
        context_vs_all_file_delta_at_10,
        lexical_context_vs_all_file_delta_at_10,
        all_file_divergence_explained,
        test_recall_at_10: report.test_recall_at_10,
        validation_command_recall: report.validation_command_recall,
        effective_validation_recall_at_10: report.effective_validation_recall_at_10,
        protected_evidence_miss_rate_at_10: report.protected_evidence.miss_rate_at_10,
        protected_evidence_target_miss_rate_at_10: report
            .protected_evidence
            .retrieval_target_miss_rate_at_10,
        runtime_millis: report.runtime.total_millis,
        notes,
    }
}

struct ContextRecallComparison {
    ctxhelm: f32,
    lexical: f32,
    delta: f32,
}

fn agent_evidence_recall_at_10(report: &HistoricalEvalReport) -> f32 {
    if report.commits.is_empty() {
        return 0.0;
    }
    report
        .commits
        .iter()
        .map(|commit| {
            if commit.retrieval_target_files.is_empty() {
                return 0.0;
            }
            let context_files = commit
                .recommended_context_files
                .iter()
                .take(10)
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let test_changed_files = filter_changed_labels_by_role(
                &commit.changed_path_labels,
                &commit.retrieval_target_files,
                |role| matches!(role, FileRole::Test),
            );
            let validation_hits = effective_validation_hit_paths(
                &test_changed_files,
                &commit.recommended_tests,
                &commit.recommended_commands,
            );
            let hits = commit
                .retrieval_target_files
                .iter()
                .filter(|path| {
                    context_files.contains(path.as_str()) || validation_hits.contains(*path)
                })
                .count();
            hits as f32 / commit.retrieval_target_files.len() as f32
        })
        .sum::<f32>()
        / report.commits.len() as f32
}

fn context_recall_comparison_at_10(report: &HistoricalEvalReport) -> ContextRecallComparison {
    let mut context_targets = 0usize;
    let mut ctxhelm_hits = 0usize;
    let mut lexical_hits = 0usize;
    for commit in &report.commits {
        let roles = commit
            .changed_path_labels
            .iter()
            .map(|label| (label.path.as_str(), &label.role))
            .collect::<BTreeMap<_, _>>();
        let recommended = commit
            .recommended_context_files
            .iter()
            .take(10)
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let lexical = commit
            .lexical_baseline_files
            .iter()
            .take(10)
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for path in &commit.retrieval_target_files {
            if roles
                .get(path.as_str())
                .is_some_and(|role| matches!(**role, FileRole::Test))
            {
                continue;
            }
            context_targets += 1;
            if recommended.contains(path.as_str()) {
                ctxhelm_hits += 1;
            }
            if lexical.contains(path.as_str()) {
                lexical_hits += 1;
            }
        }
    }
    if context_targets == 0 {
        return ContextRecallComparison {
            ctxhelm: report.file_recall_at_10,
            lexical: report.lexical_baseline_recall_at_10,
            delta: report.file_recall_at_10 - report.lexical_baseline_recall_at_10,
        };
    }
    let ctxhelm = ctxhelm_hits as f32 / context_targets as f32;
    let lexical = lexical_hits as f32 / context_targets as f32;
    ContextRecallComparison {
        ctxhelm,
        lexical,
        delta: ctxhelm - lexical,
    }
}

fn product_proof_variant_label(config: &BenchmarkRepoEffectiveConfig) -> String {
    match (
        config.semantic_enabled,
        config.semantic_quality_backend,
        config.local_metadata_reranker,
    ) {
        (_, _, true) => "local_metadata_reranked".to_string(),
        (true, true, false) => "production_semantic".to_string(),
        (true, false, false) => "semantic_scaffold".to_string(),
        (false, _, false) => "ctxhelm_default".to_string(),
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
        proof_boundary: "ctxhelm may be useful at lexical parity, but world-class claims require repeated lift on fixed corpora plus process-level context metrics.".to_string(),
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
            "ctxhelmLiftAt10",
            historical.ctxhelm_lift_at_10,
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
        "ctxhelm_default",
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
        "Zero-file baseline for an agent starting without ctxhelm context.",
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
    semantic_precision_gate_report_with_provider(
        repo_root,
        limit,
        ranking_budget,
        task_type,
        SemanticProviderConfig::default(),
    )
}

pub fn semantic_precision_gate_report_with_provider(
    repo_root: impl AsRef<Path>,
    limit: usize,
    ranking_budget: usize,
    task_type: TaskType,
    semantic_provider: SemanticProviderConfig,
) -> Result<SemanticPrecisionGateReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let mut provider_policy = provider_policy_report(repo_root)?;
    provider_policy.decisions.push(semantic_provider_decision(
        &provider_policy,
        &semantic_provider,
        true,
    ));
    let precision = semantic_document_report(
        repo_root,
        &SemanticDocumentOptions {
            limit: 500,
            query: None,
            include_symbols: false,
            include_dependencies: false,
            include_related_tests: false,
        },
    )?
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
            semantic_provider: semantic_provider.clone(),
            local_metadata_reranker: false,
            query_family_routed_reranker: false,
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
                task_type: task_type.clone(),
                target_agent: "generic".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: semantic_provider.clone(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            }
        },
    )?;
    let reranked = evaluate_historical_commits(
        repo_root,
        &HistoricalEvalOptions {
            limit,
            ranking_budget,
            task_type: task_type.clone(),
            target_agent: "generic".to_string(),
            base: None,
            head: None,
            semantic_enabled: false,
            semantic_provider: semantic_provider.clone(),
            local_metadata_reranker: true,
            query_family_routed_reranker: false,
            cache_enabled: false,
            force_refresh: false,
            parallelism: 1,
        },
    )?;
    let routed_reranked = evaluate_historical_commits(
        repo_root,
        &HistoricalEvalOptions {
            limit,
            ranking_budget,
            task_type: task_type.clone(),
            target_agent: "generic".to_string(),
            base: None,
            head: None,
            semantic_enabled: false,
            semantic_provider: semantic_provider.clone(),
            local_metadata_reranker: false,
            query_family_routed_reranker: true,
            cache_enabled: false,
            force_refresh: false,
            parallelism: 1,
        },
    )?;
    let reranker = reranker_decision(&provider_policy);
    let precision_available = precision.edge_count > 0 && !precision.degraded && !precision.stale;
    let semantic_provider_status = semantic
        .effective_filters
        .semantic_provider
        .as_deref()
        .unwrap_or("local_hash");
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
            "ctxhelm_default",
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
            "local_metadata_reranked",
            SemanticPrecisionVariantStatus::Evaluated,
            false,
            false,
            true,
            Some(reranked.ranking_comparison.combined.clone()),
            &reranked,
            "evaluated",
            "Eval-only deterministic local metadata reranker over source-free candidate metadata.",
        ),
        gate_variant(
            "query_family_routed_reranked",
            SemanticPrecisionVariantStatus::Evaluated,
            false,
            false,
            true,
            Some(routed_reranked.ranking_comparison.combined.clone()),
            &routed_reranked,
            "evaluated",
            "Eval-only local metadata reranker gated to route-safe source-free query families.",
        ),
        gate_variant(
            "local_semantic",
            SemanticPrecisionVariantStatus::Evaluated,
            true,
            false,
            false,
            Some(semantic.ranking_comparison.combined.clone()),
            &semantic,
            semantic_provider_status,
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
            semantic_provider_status
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
            semantic_provider_status
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
    let mut named_regressions = named_cases(
        &default,
        &semantic,
        "local_semantic",
        NamedCaseKind::Regression,
    );
    named_regressions.extend(named_cases(
        &default,
        &reranked,
        "local_metadata_reranked",
        NamedCaseKind::Regression,
    ));
    named_regressions.extend(named_cases(
        &default,
        &routed_reranked,
        "query_family_routed_reranked",
        NamedCaseKind::Regression,
    ));
    named_regressions.extend(protected_evidence_regressions(
        &default,
        &semantic,
        "local_semantic",
    ));
    named_regressions.extend(protected_evidence_regressions(
        &default,
        &reranked,
        "local_metadata_reranked",
    ));
    named_regressions.extend(protected_evidence_regressions(
        &default,
        &routed_reranked,
        "query_family_routed_reranked",
    ));
    let named_misses = named_cases(&default, &semantic, "local_semantic", NamedCaseKind::Miss);
    let semantic_contribution = semantic_contribution_summary(&semantic);
    let reranker_contribution = reranker_contribution_summary(&default, &reranked);
    let routed_reranker_contribution = reranker_contribution_summary(&default, &routed_reranked);
    let (decision, decision_reason) =
        gate_decision_from_variants(&variants, &named_regressions, &provider_policy);
    let mut diagnostics = provider_policy.diagnostics.clone();
    diagnostics.extend(semantic_contribution_diagnostics(
        &semantic_contribution,
        semantic_provider_status,
    ));
    diagnostics.extend(reranker_contribution_diagnostics(&reranker_contribution));
    diagnostics.extend(routed_reranker_contribution_diagnostics(
        &routed_reranker_contribution,
    ));
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
        semantic_contribution,
        reranker_contribution,
        routed_reranker_contribution,
        provider_policy,
        precision_status: precision,
        diagnostics,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}

#[allow(clippy::too_many_arguments)]
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
        protected_evidence_miss_rate_at_10: Some(eval.protected_evidence.miss_rate_at_10),
        protected_evidence_target_miss_rate_at_10: Some(
            eval.protected_evidence.retrieval_target_miss_rate_at_10,
        ),
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

fn protected_evidence_regressions(
    default: &HistoricalEvalReport,
    variant: &HistoricalEvalReport,
    variant_name: &str,
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
        let default_selected = default_commit
            .protected_evidence
            .iter()
            .filter(|evidence| evidence.selected_at_10)
            .map(|evidence| evidence.path.as_str())
            .collect::<BTreeSet<_>>();
        let demoted_paths = commit
            .protected_evidence
            .iter()
            .filter(|evidence| {
                default_selected.contains(evidence.path.as_str()) && !evidence.selected_at_10
            })
            .map(|evidence| evidence.path.clone())
            .collect::<Vec<_>>();
        if demoted_paths.is_empty() {
            continue;
        }

        let mut paths = demoted_paths;
        paths.sort();
        paths.dedup();
        paths.truncate(5);
        cases.push(SemanticPrecisionNamedCase {
            sha: short_sha(&commit.sha),
            variant: variant_name.to_string(),
            reason: format!(
                "Variant demoted {} protected anchor/current-diff/lexical/symbol path(s) that default kept within K.",
                paths.len()
            ),
            paths,
        });
    }

    cases.truncate(10);
    cases
}

fn semantic_contribution_summary(report: &HistoricalEvalReport) -> SemanticContributionSummary {
    let mut summary = SemanticContributionSummary {
        evaluated_commits: report.commits.len(),
        ..SemanticContributionSummary::default()
    };
    let mut semantic_only_hits = Vec::new();
    let mut semantic_only_non_target_cases = Vec::new();
    let mut missed_gap_families = BTreeMap::<String, (usize, Vec<String>)>::new();
    let mut family_contributions = BTreeMap::<String, SemanticQueryFamilyContribution>::new();

    for commit in &report.commits {
        let semantic_files = commit
            .signal_baseline_files
            .iter()
            .find(|ranking| ranking.signal == RetrievalSignalKind::Semantic)
            .map(|ranking| ranking.files.iter().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        if !semantic_files.is_empty() {
            summary.commits_with_semantic_selection += 1;
        }
        summary.semantic_selected_file_count += semantic_files.len();

        let lexical_files = commit
            .lexical_baseline_files
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let target_files = commit
            .retrieval_target_files
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let semantic_target_hits = semantic_files
            .intersection(&target_files)
            .cloned()
            .collect::<BTreeSet<_>>();
        let semantic_missed_targets = target_files
            .difference(&semantic_target_hits)
            .cloned()
            .collect::<Vec<_>>();
        let semantic_only_target_hits = semantic_target_hits
            .difference(&lexical_files)
            .cloned()
            .collect::<Vec<_>>();
        let semantic_only_files = semantic_files
            .difference(&lexical_files)
            .cloned()
            .collect::<BTreeSet<_>>();
        let semantic_only_non_targets = semantic_only_files
            .difference(&target_files)
            .cloned()
            .collect::<Vec<_>>();
        let family = reranker_query_family(commit);
        let family_entry =
            family_contributions
                .entry(family.clone())
                .or_insert(SemanticQueryFamilyContribution {
                    family,
                    ..SemanticQueryFamilyContribution::default()
                });
        family_entry.evaluated_commits += 1;
        if !semantic_files.is_empty() {
            family_entry.commits_with_semantic_selection += 1;
        }
        family_entry.semantic_selected_file_count += semantic_files.len();
        family_entry.semantic_target_hit_count += semantic_target_hits.len();
        family_entry.semantic_only_target_hit_count += semantic_only_target_hits.len();
        family_entry.semantic_only_non_target_count += semantic_only_non_targets.len();
        family_entry.semantic_missed_target_count += semantic_missed_targets.len();

        summary.semantic_target_hit_count += semantic_target_hits.len();
        summary.semantic_only_target_hit_count += semantic_only_target_hits.len();
        summary.semantic_only_non_target_count += semantic_only_non_targets.len();
        summary.semantic_lexical_overlap_count +=
            semantic_files.intersection(&lexical_files).count();
        summary.semantic_missed_target_count += semantic_missed_targets.len();
        for missed in semantic_missed_targets {
            let gap = semantic_missed_target_gap(commit, &missed);
            let entry = missed_gap_families
                .entry(gap.clone())
                .or_insert((0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 5 && !entry.1.contains(&missed) {
                entry.1.push(missed.clone());
            }
            upsert_semantic_family_gap(family_entry, &gap, &missed);
        }

        if !semantic_only_target_hits.is_empty() {
            let mut paths = semantic_only_target_hits;
            paths.sort();
            paths.truncate(5);
            let case = SemanticPrecisionNamedCase {
                sha: short_sha(&commit.sha),
                variant: "local_semantic".to_string(),
                reason: "Semantic selected retrieval-target path(s) absent from the lexical baseline top K.".to_string(),
                paths,
            };
            push_semantic_family_case(family_entry, case.clone());
            semantic_only_hits.push(case);
        }
        if !semantic_only_non_targets.is_empty() {
            let mut paths = semantic_only_non_targets;
            paths.sort();
            paths.truncate(5);
            let case = SemanticPrecisionNamedCase {
                sha: short_sha(&commit.sha),
                variant: "local_semantic".to_string(),
                reason:
                    "Semantic selected non-target path(s) absent from the lexical baseline top K."
                        .to_string(),
                paths,
            };
            push_semantic_family_case(family_entry, case.clone());
            semantic_only_non_target_cases.push(case);
        }
    }

    if summary.evaluated_commits > 0 {
        summary.average_semantic_selected_files =
            summary.semantic_selected_file_count as f32 / summary.evaluated_commits as f32;
    }
    let target_opportunity =
        summary.semantic_target_hit_count + summary.semantic_missed_target_count;
    if target_opportunity > 0 {
        summary.semantic_target_hit_rate =
            summary.semantic_target_hit_count as f32 / target_opportunity as f32;
        summary.semantic_only_target_hit_rate =
            summary.semantic_only_target_hit_count as f32 / target_opportunity as f32;
    }
    let semantic_only_opportunity =
        summary.semantic_only_target_hit_count + summary.semantic_only_non_target_count;
    if semantic_only_opportunity > 0 {
        summary.semantic_only_non_target_rate =
            summary.semantic_only_non_target_count as f32 / semantic_only_opportunity as f32;
    }
    semantic_only_hits.truncate(10);
    summary.semantic_only_hits = semantic_only_hits;
    semantic_only_non_target_cases.truncate(10);
    summary.semantic_only_non_targets = semantic_only_non_target_cases;
    summary.semantic_missed_target_gap_families = missed_gap_families
        .into_iter()
        .map(
            |(signal_gap, (missed_count, example_paths))| SemanticMissedTargetGapFamily {
                signal_gap,
                missed_count,
                example_paths,
            },
        )
        .collect();
    summary.query_family_contributions = family_contributions
        .into_values()
        .map(|mut family| {
            let target_opportunity =
                family.semantic_target_hit_count + family.semantic_missed_target_count;
            if target_opportunity > 0 {
                family.semantic_only_target_hit_rate =
                    family.semantic_only_target_hit_count as f32 / target_opportunity as f32;
            }
            let semantic_only_opportunity =
                family.semantic_only_target_hit_count + family.semantic_only_non_target_count;
            if semantic_only_opportunity > 0 {
                family.semantic_only_non_target_rate =
                    family.semantic_only_non_target_count as f32 / semantic_only_opportunity as f32;
            }
            family
                .missed_target_gap_families
                .sort_by_key(|entry| Reverse(entry.missed_count));
            family.example_cases.truncate(5);
            family
        })
        .collect();
    summary.query_family_contributions.sort_by(|left, right| {
        right
            .semantic_only_target_hit_count
            .cmp(&left.semantic_only_target_hit_count)
            .then_with(|| {
                left.semantic_only_non_target_count
                    .cmp(&right.semantic_only_non_target_count)
            })
            .then_with(|| {
                right
                    .semantic_target_hit_count
                    .cmp(&left.semantic_target_hit_count)
            })
            .then_with(|| left.family.cmp(&right.family))
    });
    summary
}

fn upsert_semantic_family_gap(
    family: &mut SemanticQueryFamilyContribution,
    signal_gap: &str,
    path: &str,
) {
    if let Some(existing) = family
        .missed_target_gap_families
        .iter_mut()
        .find(|entry| entry.signal_gap == signal_gap)
    {
        existing.missed_count += 1;
        if existing.example_paths.len() < 5 && !existing.example_paths.iter().any(|p| p == path) {
            existing.example_paths.push(path.to_string());
        }
        return;
    }
    family
        .missed_target_gap_families
        .push(SemanticMissedTargetGapFamily {
            signal_gap: signal_gap.to_string(),
            missed_count: 1,
            example_paths: vec![path.to_string()],
        });
}

fn push_semantic_family_case(
    family: &mut SemanticQueryFamilyContribution,
    case: SemanticPrecisionNamedCase,
) {
    if family.example_cases.len() < 5 {
        family.example_cases.push(case);
    }
}

fn semantic_missed_target_gap(commit: &HistoricalCommitEval, path: &str) -> String {
    if commit
        .lexical_baseline_files
        .iter()
        .any(|file| file == path)
    {
        return "semantic_miss_lexical_covered".to_string();
    }
    let non_semantic_signals = commit
        .signal_baseline_files
        .iter()
        .filter(|ranking| ranking.signal != RetrievalSignalKind::Semantic)
        .filter(|ranking| ranking.files.iter().any(|file| file == path))
        .map(|ranking| ranking.signal.clone())
        .collect::<Vec<_>>();
    if non_semantic_signals.iter().any(|signal| {
        matches!(
            signal,
            RetrievalSignalKind::Dependency
                | RetrievalSignalKind::CoChange
                | RetrievalSignalKind::History
                | RetrievalSignalKind::Symbol
        )
    }) {
        return "semantic_miss_nonsemantic_coupled_signal".to_string();
    }
    if commit
        .recommended_context_files
        .iter()
        .take(10)
        .any(|file| file == path)
    {
        return "semantic_miss_final_context_covered".to_string();
    }
    if commit
        .context_area_hits
        .iter()
        .any(|area| area == &context_area_for_path(path))
    {
        return "semantic_miss_area_context_only".to_string();
    }
    if !non_semantic_signals.is_empty() {
        return format!(
            "semantic_miss_nonsemantic_{}_signal",
            signal_family_code(&non_semantic_signals)
        );
    }
    "semantic_miss_no_candidate_signal".to_string()
}

fn semantic_contribution_diagnostics(
    summary: &SemanticContributionSummary,
    provider: &str,
) -> Vec<Diagnostic> {
    if summary.evaluated_commits == 0 {
        return Vec::new();
    }
    if summary.semantic_selected_file_count == 0 {
        return vec![Diagnostic {
            code: "semantic_contribution_no_candidates".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "Semantic provider `{provider}` selected no source-free candidate files during the gate run; inspect provider availability and query construction before expecting semantic lift."
            ),
            paths: Vec::new(),
            count: summary.evaluated_commits,
        }];
    }
    let mut diagnostics = Vec::new();
    if summary.semantic_target_hit_count > 0 && summary.semantic_only_target_hit_count == 0 {
        diagnostics.push(Diagnostic {
            code: "semantic_contribution_no_unique_target_hits".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Semantic provider `{provider}` hit target files, but none were semantic-only target hits beyond the lexical baseline top K."
            ),
            paths: Vec::new(),
            count: summary.semantic_target_hit_count,
        });
    }
    if summary.semantic_only_non_target_count > 0 && summary.semantic_only_target_hit_count == 0 {
        diagnostics.push(Diagnostic {
            code: "semantic_contribution_unique_non_targets".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Semantic provider `{provider}` contributed files outside the lexical baseline, but those unique semantic files were not retrieval targets."
            ),
            paths: summary
                .semantic_only_non_targets
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.semantic_only_non_target_count,
        });
    }
    if let Some(coupled) = summary
        .semantic_missed_target_gap_families
        .iter()
        .find(|family| family.signal_gap == "semantic_miss_nonsemantic_coupled_signal")
    {
        diagnostics.push(Diagnostic {
            code: "semantic_contribution_missed_targets_coupled".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Semantic provider `{provider}` missed target files that already had source-free graph/history/symbol signals; prioritize graph/fusion ordering before adding more semantic text."
            ),
            paths: coupled.example_paths.clone(),
            count: coupled.missed_count,
        });
    }
    if let Some(no_signal) = summary
        .semantic_missed_target_gap_families
        .iter()
        .find(|family| family.signal_gap == "semantic_miss_no_candidate_signal")
    {
        diagnostics.push(Diagnostic {
            code: "semantic_contribution_missed_targets_no_signal".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "Semantic provider `{provider}` missed target files with no lexical, graph, history, or area signal; inspect semantic document/query construction for these path families."
            ),
            paths: no_signal.example_paths.clone(),
            count: no_signal.missed_count,
        });
    }
    if summary.semantic_only_target_hit_count > 0 {
        diagnostics.push(Diagnostic {
            code: "semantic_contribution_unique_target_hits".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Semantic provider `{provider}` contributed target file(s) absent from the lexical baseline top K."
            ),
            paths: summary
                .semantic_only_hits
                .iter()
                .flat_map(|hit| hit.paths.clone())
                .collect(),
            count: summary.semantic_only_target_hit_count,
        });
    }
    for family in &summary.query_family_contributions {
        if family.semantic_only_target_hit_count > 0 && family.semantic_only_non_target_count == 0 {
            diagnostics.push(Diagnostic {
                code: "semantic_query_family_route_candidate".to_string(),
                severity: DiagnosticSeverity::Info,
                message: format!(
                    "Semantic retrieval has unique target hits without unique non-targets for query family `{}`.",
                    family.family
                ),
                paths: semantic_family_example_paths(family),
                count: family.semantic_only_target_hit_count,
            });
        } else if family.semantic_only_target_hit_count > 0
            && family.semantic_only_non_target_count > 0
        {
            diagnostics.push(Diagnostic {
                code: "semantic_query_family_mixed_hold".to_string(),
                severity: DiagnosticSeverity::Info,
                message: format!(
                    "Semantic retrieval has both unique target hits and unique non-targets for query family `{}`; hold for better fusion evidence.",
                    family.family
                ),
                paths: semantic_family_example_paths(family),
                count: family.semantic_only_target_hit_count
                    + family.semantic_only_non_target_count,
            });
        } else if family.semantic_only_non_target_count > 0 {
            diagnostics.push(Diagnostic {
                code: "semantic_query_family_noise_hold".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "Semantic retrieval produced unique non-targets without unique target hits for query family `{}`.",
                    family.family
                ),
                paths: semantic_family_example_paths(family),
                count: family.semantic_only_non_target_count,
            });
        }
    }
    diagnostics
}

fn semantic_family_example_paths(family: &SemanticQueryFamilyContribution) -> Vec<String> {
    let mut paths = Vec::new();
    for case in &family.example_cases {
        for path in &case.paths {
            if paths.len() >= 5 {
                return paths;
            }
            if !paths.contains(path) {
                paths.push(path.clone());
            }
        }
    }
    paths
}

fn reranker_contribution_summary(
    default: &HistoricalEvalReport,
    reranked: &HistoricalEvalReport,
) -> RerankerContributionSummary {
    let default_by_sha = default
        .commits
        .iter()
        .map(|commit| (commit.sha.as_str(), commit))
        .collect::<BTreeMap<_, _>>();
    let mut family_contributions: BTreeMap<String, RerankerQueryFamilyContribution> =
        BTreeMap::new();
    let mut summary = RerankerContributionSummary {
        evaluated_commits: reranked.commits.len(),
        protected_evidence_miss_rate_delta: reranked.protected_evidence.miss_rate_at_10
            - default.protected_evidence.miss_rate_at_10,
        protected_evidence_target_miss_rate_delta: reranked
            .protected_evidence
            .retrieval_target_miss_rate_at_10
            - default.protected_evidence.retrieval_target_miss_rate_at_10,
        ..RerankerContributionSummary::default()
    };

    for commit in &reranked.commits {
        let Some(default_commit) = default_by_sha.get(commit.sha.as_str()) else {
            continue;
        };
        let default_hits = default_commit
            .file_hits_at_10
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let reranked_hits = commit
            .file_hits_at_10
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let reranker_only_hits = reranked_hits
            .difference(&default_hits)
            .cloned()
            .collect::<BTreeSet<_>>();
        let default_only_hits = default_hits
            .difference(&reranked_hits)
            .cloned()
            .collect::<BTreeSet<_>>();

        let family = reranker_query_family(default_commit);
        let family_entry =
            family_contributions
                .entry(family.clone())
                .or_insert(RerankerQueryFamilyContribution {
                    family,
                    ..RerankerQueryFamilyContribution::default()
                });
        family_entry.evaluated_commits += 1;
        family_entry.default_target_hit_count += default_hits.len();
        family_entry.reranked_target_hit_count += reranked_hits.len();
        family_entry.reranker_only_target_hit_count += reranker_only_hits.len();
        family_entry.default_only_target_hit_count += default_only_hits.len();

        summary.default_target_hit_count += default_hits.len();
        summary.reranked_target_hit_count += reranked_hits.len();
        summary.reranker_only_target_hit_count += reranker_only_hits.len();
        summary.default_only_target_hit_count += default_only_hits.len();
        if !default_only_hits.is_empty() {
            let mut paths = default_only_hits.iter().cloned().collect::<Vec<_>>();
            paths.truncate(5);
            summary.default_only_cases.push(SemanticPrecisionNamedCase {
                sha: short_sha(&commit.sha),
                variant: "local_metadata_reranked".to_string(),
                reason: "Reranker replaced target file(s) that default retrieved.".to_string(),
                paths,
            });
        }

        match reranked_hits.len().cmp(&default_hits.len()) {
            std::cmp::Ordering::Greater => {
                summary.improved_commit_count += 1;
                family_entry.improved_commit_count += 1;
                let mut paths = reranker_only_hits.into_iter().collect::<Vec<_>>();
                paths.truncate(5);
                let case = SemanticPrecisionNamedCase {
                    sha: short_sha(&commit.sha),
                    variant: "local_metadata_reranked".to_string(),
                    reason: "Reranker retrieved additional gold changed file(s) beyond default."
                        .to_string(),
                    paths,
                };
                family_entry.example_cases.push(case.clone());
                summary.improved_cases.push(case);
            }
            std::cmp::Ordering::Less => {
                summary.regressed_commit_count += 1;
                family_entry.regressed_commit_count += 1;
                let mut paths = default_only_hits.into_iter().collect::<Vec<_>>();
                paths.truncate(5);
                let case = SemanticPrecisionNamedCase {
                    sha: short_sha(&commit.sha),
                    variant: "local_metadata_reranked".to_string(),
                    reason: "Reranker lost gold changed file(s) that default retrieved."
                        .to_string(),
                    paths,
                };
                family_entry.example_cases.push(case.clone());
                summary.regressed_cases.push(case);
            }
            std::cmp::Ordering::Equal => {
                summary.neutral_commit_count += 1;
                family_entry.neutral_commit_count += 1;
                if !default_only_hits.is_empty() {
                    let mut paths = default_only_hits.into_iter().collect::<Vec<_>>();
                    paths.truncate(5);
                    family_entry.example_cases.push(SemanticPrecisionNamedCase {
                        sha: short_sha(&commit.sha),
                        variant: "local_metadata_reranked".to_string(),
                        reason:
                            "Reranker churned target file(s) without changing target-hit count."
                                .to_string(),
                        paths,
                    });
                }
            }
        }
    }

    summary.target_hit_delta =
        summary.reranked_target_hit_count as i32 - summary.default_target_hit_count as i32;
    summary.query_family_contributions = family_contributions
        .into_values()
        .map(|mut family| {
            family.target_hit_delta =
                family.reranked_target_hit_count as i32 - family.default_target_hit_count as i32;
            family.routing_recommendation = reranker_routing_recommendation(&family);
            family.example_cases.truncate(5);
            family
        })
        .collect();
    summary.query_family_contributions.sort_by(|left, right| {
        right
            .target_hit_delta
            .cmp(&left.target_hit_delta)
            .then_with(|| {
                left.default_only_target_hit_count
                    .cmp(&right.default_only_target_hit_count)
            })
            .then_with(|| left.family.cmp(&right.family))
    });
    summary.improved_cases.truncate(10);
    summary.regressed_cases.truncate(10);
    summary.default_only_cases.truncate(10);
    summary
}

fn reranker_query_family(commit: &HistoricalCommitEval) -> String {
    query_family_from_parts(
        &commit.task_type,
        commit.low_information_task,
        commit.broad_scope_task,
        commit.query_trace.as_ref(),
    )
}

fn query_family_from_parts(
    task_type: &TaskType,
    low_information_task: bool,
    broad_scope_task: bool,
    query_trace: Option<&QueryConstructionTrace>,
) -> String {
    if broad_scope_task {
        return "broad_scope".to_string();
    }
    if low_information_task {
        return "low_information".to_string();
    }
    let Some(trace) = query_trace else {
        return format!("task_type_{task_type:?}").to_ascii_lowercase();
    };
    if trace.facets.iter().any(|facet| {
        matches!(
            facet.kind,
            QueryFacetKind::ExplicitPath | QueryFacetKind::CurrentDiffPath
        )
    }) {
        return "explicit_path".to_string();
    }
    if trace.facets.iter().any(|facet| {
        matches!(
            facet.kind,
            QueryFacetKind::StackFrame | QueryFacetKind::ErrorText
        )
    }) {
        return "stack_or_error".to_string();
    }
    if trace
        .facets
        .iter()
        .any(|facet| matches!(facet.kind, QueryFacetKind::Symbol))
    {
        return "symbol_identifier".to_string();
    }
    if trace
        .facets
        .iter()
        .any(|facet| matches!(facet.kind, QueryFacetKind::CommitClue))
    {
        return "commit_clue".to_string();
    }
    if trace
        .facets
        .iter()
        .any(|facet| matches!(facet.kind, QueryFacetKind::DomainPhrase))
    {
        return "domain_phrase".to_string();
    }
    format!("task_type_{task_type:?}").to_ascii_lowercase()
}

fn reranker_routing_recommendation(
    family: &RerankerQueryFamilyContribution,
) -> RerankerRoutingRecommendation {
    if family.evaluated_commits == 0 {
        return RerankerRoutingRecommendation::InsufficientEvidence;
    }
    if family.regressed_commit_count > 0 || family.target_hit_delta < 0 {
        return RerankerRoutingRecommendation::BlockRegression;
    }
    if family.target_hit_delta > 0 && family.default_only_target_hit_count == 0 {
        return RerankerRoutingRecommendation::RouteCandidate;
    }
    if family.target_hit_delta > 0 && family.default_only_target_hit_count > 0 {
        return RerankerRoutingRecommendation::HoldChurn;
    }
    RerankerRoutingRecommendation::HoldNeutral
}

fn reranker_contribution_diagnostics(summary: &RerankerContributionSummary) -> Vec<Diagnostic> {
    if summary.evaluated_commits == 0 {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    if summary.regressed_commit_count > 0 {
        diagnostics.push(Diagnostic {
            code: "reranker_contribution_regressed_commits".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "Local metadata reranker lost target hits on some commits; keep reranking opt-in or route by query family.".to_string(),
            paths: summary
                .regressed_cases
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.regressed_commit_count,
        });
    }
    if summary.improved_commit_count > 0 {
        diagnostics.push(Diagnostic {
            code: "reranker_contribution_target_lift".to_string(),
            severity: DiagnosticSeverity::Info,
            message:
                "Local metadata reranker retrieved additional target files on measured commits."
                    .to_string(),
            paths: summary
                .improved_cases
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.improved_commit_count,
        });
    } else if summary.regressed_commit_count == 0 {
        diagnostics.push(Diagnostic {
            code: "reranker_contribution_neutral".to_string(),
            severity: DiagnosticSeverity::Info,
            message:
                "Local metadata reranker preserved target hits but did not add target hits in this gate."
                    .to_string(),
            paths: Vec::new(),
            count: summary.neutral_commit_count,
        });
    }
    if summary.protected_evidence_target_miss_rate_delta < 0.0 {
        diagnostics.push(Diagnostic {
            code: "reranker_contribution_protected_target_improved".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Local metadata reranker reduced protected retrieval-target miss-rate by {:.3}.",
                -summary.protected_evidence_target_miss_rate_delta
            ),
            paths: Vec::new(),
            count: summary.reranked_target_hit_count,
        });
    }
    if summary.default_only_target_hit_count > 0 {
        diagnostics.push(Diagnostic {
            code: "reranker_contribution_target_churn".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Local metadata reranker added target hits but also replaced some target files that default retrieved; use this as routing evidence rather than an unconditional default.".to_string(),
            paths: summary
                .default_only_cases
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.default_only_target_hit_count,
        });
    }
    let route_candidate_families = summary
        .query_family_contributions
        .iter()
        .filter(|family| {
            family.routing_recommendation == RerankerRoutingRecommendation::RouteCandidate
        })
        .collect::<Vec<_>>();
    if !route_candidate_families.is_empty() {
        diagnostics.push(Diagnostic {
            code: "reranker_query_family_route_candidate".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Some query families show reranker lift without target churn; keep them as routing candidates pending repeated proof.".to_string(),
            paths: route_candidate_families
                .iter()
                .flat_map(|family| {
                    family
                        .example_cases
                        .iter()
                        .flat_map(|case| case.paths.clone())
                })
                .take(10)
                .collect(),
            count: route_candidate_families.len(),
        });
    }
    let churn_held_families = summary
        .query_family_contributions
        .iter()
        .filter(|family| family.routing_recommendation == RerankerRoutingRecommendation::HoldChurn)
        .collect::<Vec<_>>();
    if !churn_held_families.is_empty() {
        diagnostics.push(Diagnostic {
            code: "reranker_query_family_churn_hold".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Some query families improve net target hits but still churn default targets; hold routing for those families until churn is reduced.".to_string(),
            paths: churn_held_families
                .iter()
                .flat_map(|family| {
                    family
                        .example_cases
                        .iter()
                        .flat_map(|case| case.paths.clone())
                })
                .take(10)
                .collect(),
            count: churn_held_families.len(),
        });
    }
    diagnostics
}

fn routed_reranker_contribution_diagnostics(
    summary: &RerankerContributionSummary,
) -> Vec<Diagnostic> {
    if summary.evaluated_commits == 0 {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    if summary.regressed_commit_count > 0 || summary.target_hit_delta < 0 {
        diagnostics.push(Diagnostic {
            code: "routed_reranker_regression".to_string(),
            severity: DiagnosticSeverity::Warning,
            message:
                "Query-family routed reranker lost target hits; keep routed reranking eval-only."
                    .to_string(),
            paths: summary
                .regressed_cases
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.regressed_commit_count,
        });
    } else if summary.target_hit_delta > 0 && summary.default_only_target_hit_count == 0 {
        diagnostics.push(Diagnostic {
            code: "routed_reranker_clean_lift".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Query-family routed reranker added target hits without default-only target churn in this gate.".to_string(),
            paths: summary
                .improved_cases
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.improved_commit_count,
        });
    } else if summary.target_hit_delta > 0 && summary.default_only_target_hit_count > 0 {
        diagnostics.push(Diagnostic {
            code: "routed_reranker_churn_hold".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Query-family routed reranker added target hits but still churned default targets; hold routing policy.".to_string(),
            paths: summary
                .default_only_cases
                .iter()
                .flat_map(|case| case.paths.clone())
                .take(10)
                .collect(),
            count: summary.default_only_target_hit_count,
        });
    } else {
        diagnostics.push(Diagnostic {
            code: "routed_reranker_neutral".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Query-family routed reranker preserved target hits but did not add target hits in this gate.".to_string(),
            paths: Vec::new(),
            count: summary.neutral_commit_count,
        });
    }
    diagnostics
}

fn protected_evidence_miss_rate_delta(
    default: Option<&SemanticPrecisionVariant>,
    variant: Option<&SemanticPrecisionVariant>,
) -> f32 {
    match (
        default.and_then(|variant| variant.protected_evidence_miss_rate_at_10),
        variant.and_then(|variant| variant.protected_evidence_miss_rate_at_10),
    ) {
        (Some(default_miss_rate), Some(variant_miss_rate)) => variant_miss_rate - default_miss_rate,
        _ => 0.0,
    }
}

fn token_efficiency_delta(
    default: Option<&SemanticPrecisionVariant>,
    variant: Option<&SemanticPrecisionVariant>,
) -> f32 {
    match (
        default.and_then(|variant| variant.token_efficiency),
        variant.and_then(|variant| variant.token_efficiency),
    ) {
        (Some(default_efficiency), Some(variant_efficiency)) => {
            variant_efficiency - default_efficiency
        }
        _ => 0.0,
    }
}

fn runtime_ratio(
    default: Option<&SemanticPrecisionVariant>,
    variant: Option<&SemanticPrecisionVariant>,
) -> f32 {
    match (
        default.and_then(|variant| variant.runtime_millis),
        variant.and_then(|variant| variant.runtime_millis),
    ) {
        (Some(default_runtime), Some(variant_runtime)) if default_runtime > 0 => {
            variant_runtime as f32 / default_runtime as f32
        }
        _ => 1.0,
    }
}

fn evaluated_variant_metrics<'a>(
    variants: &'a [SemanticPrecisionVariant],
    name: &str,
) -> Option<(&'a SemanticPrecisionVariant, &'a RankingMetrics)> {
    variants
        .iter()
        .find(|variant| variant.name == name)
        .and_then(|variant| variant.metrics.as_ref().map(|metrics| (variant, metrics)))
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
    let blocking_regressions = named_regressions
        .iter()
        .filter(|case| is_semantic_promotion_variant(&case.variant))
        .count();
    if blocking_regressions > 0 {
        return (
            SemanticPrecisionGateDecision::Block,
            format!(
                "Blocked because {blocking_regressions} semantic promotion regression(s) were detected."
            ),
        );
    }
    let Some((default_variant, default)) = evaluated_variant_metrics(variants, "ctxhelm_default")
    else {
        return (
            SemanticPrecisionGateDecision::Hold,
            "Held because required default or semantic metrics were missing.".to_string(),
        );
    };
    let Some((semantic_variant, semantic)) = evaluated_variant_metrics(variants, "local_semantic")
    else {
        return (
            SemanticPrecisionGateDecision::Hold,
            "Held because required default or semantic metrics were missing.".to_string(),
        );
    };
    let recall_delta = semantic.recall_at_k - default.recall_at_k;
    let precision_delta = semantic.precision_at_k - default.precision_at_k;
    let runtime_ratio = runtime_ratio(Some(default_variant), Some(semantic_variant));
    let token_efficiency_delta =
        token_efficiency_delta(Some(default_variant), Some(semantic_variant));
    let protected_miss_delta =
        protected_evidence_miss_rate_delta(Some(default_variant), Some(semantic_variant));

    if runtime_ratio > 2.0 && recall_delta < 0.10 {
        return (
            SemanticPrecisionGateDecision::Hold,
            format!(
                "Held: runtime ratio {runtime_ratio:.2}x with recall delta {recall_delta:+.3}; promotion requires stronger quality lift."
            ),
        );
    }
    if token_efficiency_delta < -0.05 && recall_delta < 0.10 {
        return (
            SemanticPrecisionGateDecision::Hold,
            format!(
                "Held: token efficiency delta {token_efficiency_delta:+.3} with recall delta {recall_delta:+.3}; promotion requires token ROI parity or stronger lift."
            ),
        );
    }
    if recall_delta >= 0.05 && precision_delta >= -0.01 {
        (
            SemanticPrecisionGateDecision::Promote,
            format!(
                "Promote: local semantic recall delta {recall_delta:+.3}, precision delta {precision_delta:+.3}, runtime ratio {runtime_ratio:.2}x, token efficiency delta {token_efficiency_delta:+.3}, protected miss-rate delta {protected_miss_delta:+.3}."
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

fn is_semantic_promotion_variant(variant: &str) -> bool {
    matches!(
        variant,
        "local_semantic" | "precision_enriched_semantic" | "semantic_precision_full_hybrid"
    )
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}

#[allow(clippy::too_many_arguments)]
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
        local_metadata_reranker: repo_config
            .local_metadata_reranker
            .unwrap_or(defaults.local_metadata_reranker),
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
        lexical_backend_comparison: repo_config
            .lexical_backend_comparison
            .unwrap_or(defaults.lexical_backend_comparison),
        proof_runtime_ceiling_millis: repo_config
            .proof_runtime_ceiling_millis
            .filter(|ceiling| *ceiling > 0),
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
        local_metadata_reranker: effective_config.local_metadata_reranker,
        query_family_routed_reranker: false,
        cache_enabled: effective_config.cache_enabled,
        force_refresh: effective_config.force_refresh,
        parallelism: effective_config.parallelism,
    };

    match evaluate_historical_commits(&repo_path, &options) {
        Ok(report) => {
            let (lexical_backend_corpus, lexical_backend_error) =
                if effective_config.lexical_backend_comparison {
                    match compare_lexical_backends_on_corpus(
                        &repo_path,
                        &LexicalBackendCorpusOptions {
                            limit: effective_config.limit,
                            ranking_budget: effective_config.ranking_budget,
                            base: effective_config.base.clone(),
                            head: effective_config.head.clone(),
                        },
                    ) {
                        Ok(report) => (Some(report), None),
                        Err(error) => (None, Some(error.to_string())),
                    }
                } else {
                    (None, None)
                };
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
            let report_error = if report.evaluated_commits == 0 {
                Some(
                    "repository produced no evaluable commits; git history is unavailable or degraded"
                        .to_string(),
                )
            } else {
                None
            };
            let environment_health = benchmark_repo_environment_health(
                report.evaluated_commits,
                report_error.as_deref(),
            );
            BenchmarkRepoReport {
                name: repo_config.name.clone(),
                repo_id: Some(report.repo_id.clone()),
                effective_config,
                environment_health,
                baseline: repo_config.baseline.clone(),
                baseline_status,
                evaluated_commits: report.evaluated_commits,
                excluded_changed_file_count,
                skipped_path_count,
                report: Some(report),
                lexical_backend_corpus,
                lexical_backend_error,
                error: report_error,
                privacy_status: PrivacyStatus::local_only(),
            }
        }
        Err(error) => {
            let error = error.to_string();
            BenchmarkRepoReport {
                name: repo_config.name.clone(),
                repo_id: None,
                effective_config,
                environment_health: benchmark_repo_environment_health(0, Some(&error)),
                baseline: repo_config.baseline.clone(),
                baseline_status: repo_config
                    .baseline
                    .as_ref()
                    .map(|baseline| baseline_status_for_error(baseline, &error)),
                evaluated_commits: 0,
                excluded_changed_file_count: 0,
                skipped_path_count: 0,
                report: None,
                lexical_backend_corpus: None,
                lexical_backend_error: None,
                error: Some(error),
                privacy_status: PrivacyStatus::local_only(),
            }
        }
    }
}

fn benchmark_repo_environment_health(
    evaluated_commits: usize,
    error: Option<&str>,
) -> BenchmarkRepoEnvironmentHealth {
    if evaluated_commits > 0 {
        return BenchmarkRepoEnvironmentHealth::healthy();
    }
    let Some(error) = error else {
        return BenchmarkRepoEnvironmentHealth {
            status: BenchmarkRepoEnvironmentStatus::Degraded,
            git_history_usable: false,
            object_content_usable: None,
            reason: "Benchmark produced no evaluable commits without a lower-level error."
                .to_string(),
        };
    };
    let lower = error.to_ascii_lowercase();
    if lower.contains("cat-file")
        || lower.contains("object")
        || lower.contains("loose object")
        || lower.contains("missing blob")
    {
        return BenchmarkRepoEnvironmentHealth {
            status: BenchmarkRepoEnvironmentStatus::GitObjectStoreUnavailable,
            git_history_usable: false,
            object_content_usable: Some(false),
            reason: "Git object-content reads failed or timed out before benchmark evidence could be produced."
                .to_string(),
        };
    }
    if lower.contains("timed out") {
        return BenchmarkRepoEnvironmentHealth {
            status: BenchmarkRepoEnvironmentStatus::GitHistoryTimeout,
            git_history_usable: false,
            object_content_usable: None,
            reason: "Git history sampling timed out before benchmark evidence could be produced."
                .to_string(),
        };
    }
    if lower.contains("no git")
        || lower.contains("not a git repository")
        || lower.contains("does not have any commits")
        || lower.contains("unavailable")
        || lower.contains("ambiguous argument 'head'")
        || lower.contains("unknown revision")
    {
        return BenchmarkRepoEnvironmentHealth {
            status: BenchmarkRepoEnvironmentStatus::GitHistoryUnavailable,
            git_history_usable: false,
            object_content_usable: None,
            reason: "Git history was unavailable for this benchmark repository.".to_string(),
        };
    }
    BenchmarkRepoEnvironmentHealth {
        status: BenchmarkRepoEnvironmentStatus::Degraded,
        git_history_usable: false,
        object_content_usable: None,
        reason: "Benchmark environment did not produce enough source-free evidence for a retrieval-quality verdict."
            .to_string(),
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
            "ctxhelmLiftAt10",
            base.ctxhelm_lift_at_10,
            head.ctxhelm_lift_at_10,
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
    ctxhelm_index::normalized_provider(&SemanticProviderConfig {
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
            "repo={}\nrepoId={}\nrevisionRangeId={}\nprivacy={}\nbase={}\nhead={}\nlimit={}\nrankingBudget={}\nparallelism={}\ncache={}\nforceRefresh={}\nmode={:?}\ntarget={}\nlocalMetadataReranker={}\nroles={:?}\ncommits={}\nerror={}\n",
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
            repo.effective_config.local_metadata_reranker,
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
    "ctxhelm-benchmark-corpus-v2.5".to_string()
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
        local_metadata_reranker: options.local_metadata_reranker,
        query_family_routed_reranker: options.query_family_routed_reranker,
    };
    let eval_range_id = historical_eval_range_id(&repo_id, &effective_filters, &refs);

    if options.cache_enabled && !options.force_refresh {
        if let Some(mut cached) = load_historical_eval_cache(&repo_id, &eval_range_id)? {
            cached.runtime = cached_historical_eval_runtime_summary(
                &cached.runtime,
                elapsed_millis(eval_started),
            );
            return Ok(cached);
        }
    }

    let inventory = load_or_build_inventory(repo_root, &InventoryOptions::default())?;
    let snapshot_paths = inventory
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let safe_paths = snapshot_paths.iter().cloned().collect::<BTreeSet<_>>();
    let roles_by_path = inventory
        .files
        .iter()
        .map(|file| (file.path.clone(), file.role.clone()))
        .collect::<BTreeMap<_, _>>();
    let git_sample_started = Instant::now();
    let mut history_sampling_failed = false;
    let samples = match historical_commit_samples_with_safe_paths(
        repo_root,
        &HistoricalCommitOptions {
            limit: options.limit,
            base: options.base.clone(),
            head: options.head.clone(),
        },
        &safe_paths,
    ) {
        Ok(samples) => samples,
        Err(_) => {
            history_sampling_failed = true;
            Vec::new()
        }
    };
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
    let mut graph_edge_ablation_rankings = BTreeMap::<String, GraphEdgeAblationRankings>::new();
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
        let graph_edge_labels_for_commit = result
            .graph_edge_ablation_rankings
            .iter()
            .map(|ablation| ablation.edge_label.clone())
            .collect::<BTreeSet<_>>();
        for edge_ablation in &result.graph_edge_ablation_rankings {
            let entry = graph_edge_ablation_rankings
                .entry(edge_ablation.edge_label.clone())
                .or_insert_with(|| GraphEdgeAblationRankings {
                    edge_label: edge_ablation.edge_label.clone(),
                    rankings: commits
                        .iter()
                        .map(|commit: &HistoricalCommitEval| {
                            commit.recommended_context_files.clone()
                        })
                        .collect(),
                    affected_commit_count: 0,
                    removed_selected_at_10_count: 0,
                    removed_target_hit_at_10_count: 0,
                });
            entry.rankings.push(edge_ablation.ranking.clone());
            entry.affected_commit_count += usize::from(edge_ablation.affected_commit);
            entry.removed_selected_at_10_count += edge_ablation.removed_selected_at_10_count;
            entry.removed_target_hit_at_10_count += edge_ablation.removed_target_hit_at_10_count;
        }
        for (edge_label, entry) in graph_edge_ablation_rankings.iter_mut() {
            if !graph_edge_labels_for_commit.contains(edge_label) {
                entry
                    .rankings
                    .push(result.commit.recommended_context_files.clone());
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
    let graph_edge_ablations = graph_edge_ablation_results(
        &commits,
        graph_edge_ablation_rankings.into_values().collect(),
        &ranking_comparison.combined,
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
    let graph_edge_profiles = graph_edge_profiles(&commits);
    let protected_evidence = protected_evidence_summary(&commits);
    let context_area_pressure_summary = context_area_pressure_summary(&commits);
    let context_area_next_read_summary = context_area_next_read_summary(&commits);
    let candidate_coverage_summary = candidate_coverage_summary(&commits);
    let memory_reuse_summary = memory_reuse_summary(&commits);
    let runtime = historical_eval_runtime_summary(
        &commits,
        elapsed_millis(eval_started),
        usize::from(options.cache_enabled),
        parallelism,
        git_sample_millis,
        ranking_millis,
        pack_compiler_millis,
    );

    let mut report = HistoricalEvalReport {
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
        graph_edge_ablations,
        token_roi,
        retrieval_gap_summaries,
        graph_edge_profiles,
        runtime,
        low_information_commit_count: commits
            .iter()
            .filter(|commit| commit.low_information_task)
            .count(),
        broad_scope_commit_count: commits
            .iter()
            .filter(|commit| commit.broad_scope_task)
            .count(),
        broad_context_area_recall: average_broad_context_area_recall(&commits),
        context_area_pressure_summary,
        context_area_next_read_summary,
        candidate_coverage_summary,
        memory_reuse_summary,
        recommended_research_actions: Vec::new(),
        file_recall_at_5,
        file_recall_at_10,
        lexical_baseline_recall_at_5,
        lexical_baseline_recall_at_10,
        ctxhelm_lift_at_5: file_recall_at_5 - lexical_baseline_recall_at_5,
        ctxhelm_lift_at_10: file_recall_at_10 - lexical_baseline_recall_at_10,
        source_recall_at_5: average_role_recall(&commits, FileRole::Source, 5),
        source_recall_at_10: average_role_recall(&commits, FileRole::Source, 10),
        test_recall_at_5: average_role_recall(&commits, FileRole::Test, 5),
        test_recall_at_10: average_role_recall(&commits, FileRole::Test, 10),
        validation_command_recall: average_validation_command_recall(&commits),
        effective_validation_recall_at_10: average_effective_validation_recall(&commits),
        test_recommendation_rate: test_recommendation_rate(&commits),
        average_recommended_context_files: average_recommended_context_files(&commits),
        protected_evidence,
        top_missing_files: top_missing_files(&commits, &roles_by_label_path, 10),
        commits,
        privacy_status: PrivacyStatus::local_only(),
    };
    report.recommended_research_actions = historical_recommended_research_actions(&report);
    if options.cache_enabled && !history_sampling_failed {
        write_historical_eval_cache(&report.repo_id, &report.eval_range_id, &report)?;
    }
    Ok(report)
}

fn historical_recommended_research_actions(
    report: &HistoricalEvalReport,
) -> Vec<RecommendedResearchAction> {
    let mut actions = Vec::new();
    if report.evaluated_commits == 0 {
        push_research_action(
            &mut actions,
            "collect_benchmark_evidence",
            1,
            "historical_eval",
            "No safe historical commits were evaluated, so retrieval quality cannot be inferred.",
        );
        return actions;
    }

    if report.ctxhelm_lift_at_10 < -0.03 || report.ranking_comparison.recall_lift_at_k < -0.03 {
        push_research_action(
            &mut actions,
            "fix_retrieval_or_ranking_regression",
            1,
            "historical_eval",
            "ctxhelm trails the lexical baseline under the current ranking budget.",
        );
    }

    if report
        .protected_evidence
        .retrieval_target_missed_at_10_count
        > 0
    {
        push_research_action(
            &mut actions,
            "protect_high_confidence_evidence",
            1,
            "historical_eval",
            "Protected retrieval-target evidence is generated but not preserved inside the top-10 context budget.",
        );
    }

    if report.candidate_coverage_summary.no_candidate_count > 0 {
        push_research_action(
            &mut actions,
            "improve_candidate_generation",
            1,
            "historical_eval",
            "Some missed retrieval targets have no generated source-free candidate.",
        );
    }

    if report
        .candidate_coverage_summary
        .candidate_recoverable_count
        > 0
    {
        push_research_action(
            &mut actions,
            "improve_ranking_or_budget_allocation",
            2,
            "historical_eval",
            "Some missed retrieval targets were generated as candidates but ranked below the selected budget.",
        );
    }

    if report
        .context_area_next_read_summary
        .next_read_recoverable_count
        > 0
    {
        push_research_action(
            &mut actions,
            "improve_progressive_read_guidance",
            2,
            "historical_eval",
            "Context-area next-read paths recover some top-10 misses, so agent-native progressive reads should be refined.",
        );
    }

    if report
        .context_area_next_read_summary
        .agent_evidence_only_count
        > 0
    {
        push_research_action(
            &mut actions,
            "align_agent_evidence_with_next_reads",
            2,
            "historical_eval",
            "Some misses are recoverable through broader agent evidence but absent from progressive next-read paths.",
        );
    }

    if report.memory_reuse_summary.memory_candidate_count == 0 {
        push_research_action(
            &mut actions,
            "collect_or_approve_experience_memory",
            3,
            "historical_eval",
            "No fresh approved memory candidates contributed to this historical evaluation range.",
        );
    } else {
        if report.memory_reuse_summary.memory_unique_target_hit_count > 0 {
            push_research_action(
                &mut actions,
                "evaluate_memory_reuse_lift",
                2,
                "historical_eval",
                "Memory contributed retrieval-target hits that were absent from the lexical baseline.",
            );
        }
        if report.memory_reuse_summary.memory_unique_target_hit_count == 0
            && report.memory_reuse_summary.memory_unique_non_target_count > 0
        {
            push_research_action(
                &mut actions,
                "reduce_memory_retrieval_noise",
                2,
                "historical_eval",
                "Memory contributed unique non-target files without unique target hits in this range.",
            );
        }
        if report.memory_reuse_summary.memory_target_missed_at_10_count
            > report.memory_reuse_summary.memory_target_hit_at_10_count
        {
            push_research_action(
                &mut actions,
                "improve_memory_selection_policy",
                3,
                "historical_eval",
                "Memory candidates were present, but memory missed more retrieval targets than it hit.",
            );
        }
    }

    if report.effective_validation_recall_at_10 < 1.0
        && (report.test_recall_at_10 < 1.0 || report.validation_command_recall < 1.0)
    {
        push_research_action(
            &mut actions,
            "improve_validation_test_mapping",
            2,
            "historical_eval",
            "Validation or test recall is incomplete for the evaluated history range.",
        );
    }

    if report
        .graph_edge_ablations
        .iter()
        .any(|ablation| ablation.removed_target_hit_at_10_count > 0)
    {
        push_research_action(
            &mut actions,
            "tune_graph_edge_budget",
            2,
            "historical_eval",
            "Graph edge ablations show at least one edge family contributes exclusive target hits.",
        );
    }

    if report.graph_edge_profiles.iter().any(|profile| {
        profile.retrieval_target_missed_at_10_count > profile.retrieval_target_hit_at_10_count
    }) {
        push_research_action(
            &mut actions,
            "improve_graph_edge_selection",
            3,
            "historical_eval",
            "Graph edge profiles show more target misses than hits for at least one edge family.",
        );
    }

    if actions.is_empty() {
        push_research_action(
            &mut actions,
            "preserve_current_contract",
            3,
            "historical_eval",
            "No source-free historical summary indicates a concrete retrieval, validation, graph, or budget bottleneck.",
        );
    }

    actions
}

pub fn compare_lexical_backends_on_corpus(
    repo_root: impl AsRef<Path>,
    options: &LexicalBackendCorpusOptions,
) -> Result<LexicalBackendCorpusReport, InventoryError> {
    let eval_started = Instant::now();
    let repo_root = repo_root.as_ref();
    let ranking_budget = options.ranking_budget.max(1);
    let repo_id = pack_repo_id(repo_root);
    let refs = HistoricalEvalRefs {
        base: options.base.clone(),
        head: options.head.clone(),
    };
    let eval_range_id = lexical_backend_corpus_range_id(&repo_id, options, &refs);
    let inventory = load_or_build_inventory(repo_root, &InventoryOptions::default())?;
    let snapshot_paths = inventory
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let safe_paths = snapshot_paths.iter().cloned().collect::<BTreeSet<_>>();
    let git_sample_started = Instant::now();
    let samples = historical_commit_samples_with_safe_paths(
        repo_root,
        &HistoricalCommitOptions {
            limit: options.limit,
            base: options.base.clone(),
            head: options.head.clone(),
        },
        &safe_paths,
    )?;
    let git_sample_millis = elapsed_millis(git_sample_started);
    let mut rows = Vec::new();
    let mut bm25_total_millis = 0u64;
    let mut legacy_total_millis = 0u64;
    let mut inventory_warmup_millis = 0u64;

    for sample in samples {
        let task = if sample.title.trim().is_empty() {
            format!("change {}", sample.sha)
        } else {
            sample.title.clone()
        };
        let parent_snapshot_paths =
            parent_snapshot_candidate_paths(&snapshot_paths, &sample.safe_changed_files);
        let eval_repo = HistoricalEvalWorktree::for_parent(
            repo_root,
            sample.parent_sha.as_deref(),
            &parent_snapshot_paths,
        )?;
        let eval_root = eval_repo.path();
        let retrieval_target_files = retrieval_target_files(eval_root, &sample.safe_changed_files);
        let warmup_started = Instant::now();
        let _ = load_or_build_inventory(eval_root, &InventoryOptions::default())?;
        inventory_warmup_millis += elapsed_millis(warmup_started);

        let bm25_started = Instant::now();
        let bm25_files = search_report_paths(lexical_search_report(
            eval_root,
            &task,
            &SearchOptions {
                limit: ranking_budget,
            },
        )?);
        let bm25_millis = elapsed_millis(bm25_started);
        bm25_total_millis += bm25_millis;

        let legacy_started = Instant::now();
        let legacy_files = search_report_paths(legacy_lexical_search_report(
            eval_root,
            &task,
            &SearchOptions {
                limit: ranking_budget,
            },
        )?);
        let legacy_millis = elapsed_millis(legacy_started);
        legacy_total_millis += legacy_millis;

        let bm25_hits_at_5 = changed_file_hits(&retrieval_target_files, &bm25_files, 5);
        let bm25_hits_at_10 = changed_file_hits(&retrieval_target_files, &bm25_files, 10);
        let legacy_hits_at_5 = changed_file_hits(&retrieval_target_files, &legacy_files, 5);
        let legacy_hits_at_10 = changed_file_hits(&retrieval_target_files, &legacy_files, 10);
        let overlap_at_k = overlap_at_limit(&bm25_files, &legacy_files, ranking_budget);
        let top_path_changed = bm25_files.first() != legacy_files.first();
        let bm25_recall_at_10 = recall_for_hits(&retrieval_target_files, &bm25_hits_at_10);
        let legacy_recall_at_10 = recall_for_hits(&retrieval_target_files, &legacy_hits_at_10);

        rows.push(LexicalBackendCommitRow {
            sha: sample.sha,
            task_hash: task_hash(&task),
            retrieval_target_files,
            bm25_files,
            legacy_files,
            bm25_hits_at_5,
            bm25_hits_at_10,
            legacy_hits_at_5,
            legacy_hits_at_10,
            bm25_recall_at_10,
            legacy_recall_at_10,
            overlap_at_k,
            top_path_changed,
            bm25_millis,
            legacy_millis,
        });
    }

    let bm25 = lexical_backend_metrics(
        "tantivy_bm25_fielded_v6",
        &rows,
        |row| &row.bm25_files,
        |row| &row.bm25_hits_at_5,
        |row| &row.bm25_hits_at_10,
        |row| row.bm25_millis,
    );
    let legacy = lexical_backend_metrics(
        "legacy_heuristic_scanner_v1",
        &rows,
        |row| &row.legacy_files,
        |row| &row.legacy_hits_at_5,
        |row| &row.legacy_hits_at_10,
        |row| row.legacy_millis,
    );
    let comparison = lexical_backend_comparison(&rows, &bm25, &legacy, ranking_budget);

    Ok(LexicalBackendCorpusReport {
        schema_version: LEXICAL_BACKEND_CORPUS_SCHEMA_VERSION.to_string(),
        eval_range_id,
        repo_id,
        refs,
        evaluated_commits: rows.len(),
        ranking_budget,
        bm25,
        legacy,
        comparison,
        rows,
        runtime: LexicalBackendRuntimeSummary {
            total_millis: elapsed_millis(eval_started),
            git_sample_millis,
            inventory_warmup_millis,
            bm25_total_millis,
            legacy_total_millis,
        },
        privacy_status: PrivacyStatus::local_only(),
        source_text_logged: false,
    })
}

pub(crate) struct HistoricalEvalWorktree<'a> {
    path: PathBuf,
    _source_repo: &'a Path,
    _temp_dir: Option<tempfile::TempDir>,
}

struct HistoricalCommitEvalResult {
    commit: HistoricalCommitEval,
    ablation_rankings: Vec<(RetrievalSignalKind, Vec<String>)>,
    graph_edge_ablation_rankings: Vec<GraphEdgeAblationCommitRanking>,
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
    let history_mode = if sample
        .parent_sha
        .as_deref()
        .is_some_and(|sha| !sha.trim().is_empty())
    {
        // Parent snapshots carry a source-free `.ctxhelm/eval-history.json`
        // sidecar built at the parent revision. Using that prior history in
        // ranking exercises the product's co-change signal without leaking the
        // target commit being evaluated.
        HistoryMode::Full
    } else {
        HistoryMode::Disabled
    };
    let parent_snapshot_paths =
        parent_snapshot_candidate_paths(snapshot_paths, &sample.safe_changed_files);
    let eval_repo = HistoricalEvalWorktree::for_parent(
        repo_root,
        sample.parent_sha.as_deref(),
        &parent_snapshot_paths,
    )?;
    let eval_root = eval_repo.path();
    project_source_memory_to_eval_root(repo_root, eval_root);
    let plan_started = Instant::now();
    let plan = prepare_context_plan_with_paths_history_mode_and_semantic(
        eval_root,
        &task,
        options.task_type.clone(),
        &[],
        history_mode,
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
    let multi_area_task = is_multi_area_task(&task);
    let recommended_commands = plan
        .recommended_commands
        .iter()
        .map(|command| command.command.clone())
        .collect::<Vec<_>>();
    let default_context_files = context_file_ranking(
        &recommended_files,
        &recommended_tests,
        ranking_budget,
        !multi_area_task,
    );
    let query_family = query_family_from_parts(
        &options.task_type,
        is_low_information_task(&task),
        multi_area_task,
        plan.query_trace.as_ref(),
    );
    let should_apply_reranker = options.local_metadata_reranker
        || (options.query_family_routed_reranker
            && query_family_routed_reranker_enabled_for_family(&query_family));
    let recommended_context_files = if should_apply_reranker {
        local_metadata_reranked_context_files(
            &plan,
            ranking_budget,
            &default_context_files,
            !multi_area_task,
        )
    } else {
        default_context_files
    };
    let lexical_baseline_files = lexical_baseline_context_files(eval_root, &task, ranking_budget)?;
    let retrieval_target_files = retrieval_target_files(eval_root, &sample.safe_changed_files);
    let retrieval_target_paths = retrieval_target_files
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let candidate_roles_by_path = candidate_roles_by_path(&plan);
    let candidate_paths = candidate_roles_by_path
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let protected_evidence = protected_evidence_files(
        &signals_by_path,
        &budgeted_protected_evidence_paths(&plan, 10),
        &recommended_context_files,
        &retrieval_target_paths,
        &candidate_roles_by_path,
        10,
    );
    let graph_edge_profiles = graph_edge_profiles_for_commit(
        &plan,
        &recommended_context_files,
        &retrieval_target_paths,
        10,
    );
    let file_hits_at_5 = changed_file_hits(&retrieval_target_files, &recommended_context_files, 5);
    let file_hits_at_10 =
        changed_file_hits(&retrieval_target_files, &recommended_context_files, 10);
    let lexical_baseline_hits_at_5 =
        changed_file_hits(&retrieval_target_files, &lexical_baseline_files, 5);
    let lexical_baseline_hits_at_10 =
        changed_file_hits(&retrieval_target_files, &lexical_baseline_files, 10);
    let missing_files_at_10 =
        missing_changed_files(&retrieval_target_files, &recommended_context_files, 10);
    let candidate_missed_files_at_10 = missing_files_at_10
        .iter()
        .filter(|path| candidate_paths.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    let candidate_missed_file_profiles_at_10 =
        candidate_missed_file_profiles(&missing_files_at_10, &plan, &candidate_roles_by_path);
    let ablation_rankings = ablation_signals()
        .into_iter()
        .map(|signal| {
            let ranking =
                ablated_context_ranking(&recommended_context_files, &signals_by_path, &signal);
            (signal, ranking)
        })
        .collect::<Vec<_>>();
    let graph_edge_ablation_rankings = graph_edge_ablation_rankings_for_commit(
        &plan,
        &recommended_context_files,
        &signals_by_path,
        &retrieval_target_paths,
        10,
    );
    let signal_baseline_files =
        signal_baseline_rankings(&recommended_context_files, &signals_by_path);
    let selected_signal_profiles = selected_signal_profiles(
        &recommended_context_files,
        &signals_by_path,
        &candidate_roles_by_path,
        &retrieval_target_paths,
        10,
    );
    let source_changed_files =
        filter_changed_labels_by_role(&changed_path_labels, &retrieval_target_files, |role| {
            matches!(role, FileRole::Source)
        });
    let test_changed_files =
        filter_changed_labels_by_role(&changed_path_labels, &retrieval_target_files, |role| {
            matches!(role, FileRole::Test)
        });
    let source_hits_at_5 =
        changed_file_hits(&source_changed_files, &recommended_context_files, 5).len();
    let source_hits_at_10 =
        changed_file_hits(&source_changed_files, &recommended_context_files, 10).len();
    let test_hits_at_5 = changed_file_hits(&test_changed_files, &recommended_tests, 5).len();
    let test_hits_at_10 = changed_file_hits(&test_changed_files, &recommended_tests, 10).len();
    let validation_command_hits =
        validation_command_hits(&test_changed_files, &recommended_commands).len();
    let effective_validation_hits_at_10 = effective_validation_hits_at_10(
        &test_changed_files,
        &recommended_tests,
        &recommended_commands,
    );
    let broad_scope_task =
        multi_area_task || retrieval_target_files.len() >= 12 || source_changed_files.len() >= 8;
    let changed_context_areas =
        changed_context_areas(&changed_path_labels, &retrieval_target_files);
    let context_area_hits = context_area_hits(
        &changed_context_areas,
        &plan.context_areas,
        &recommended_context_files,
    );
    let gap_reasons = retrieval_gap_reasons(
        &missing_files_at_10,
        &lexical_baseline_files,
        &signals_by_path,
        &context_area_hits,
    );
    let ranking_millis = elapsed_millis(ranking_started);

    Ok(HistoricalCommitEvalResult {
        commit: HistoricalCommitEval {
            sha: sample.sha,
            task_hash: task_hash(&task),
            task_type: options.task_type.clone(),
            target_agent: target_agent.to_string(),
            changed_path_labels,
            safe_changed_files: sample.safe_changed_files,
            retrieval_target_files,
            excluded_changed_file_count: sample.excluded_changed_file_count,
            recommended_files,
            recommended_tests,
            recommended_context_files,
            recommended_commands,
            lexical_baseline_files,
            signal_baseline_files,
            selected_signal_profiles,
            protected_evidence,
            graph_edge_profiles,
            file_hits_at_5,
            file_hits_at_10,
            lexical_baseline_hits_at_5,
            lexical_baseline_hits_at_10,
            missing_files_at_10,
            candidate_missed_files_at_10,
            candidate_missed_file_profiles_at_10,
            source_files_changed: source_changed_files.len(),
            source_hits_at_5,
            source_hits_at_10,
            test_files_changed: test_changed_files.len(),
            test_hits_at_5,
            test_hits_at_10,
            validation_command_hits,
            effective_validation_hits_at_10,
            low_information_task: is_low_information_task(&task),
            broad_scope_task,
            changed_context_areas,
            context_area_hits,
            context_areas: plan.context_areas.clone(),
            confidence: plan.confidence,
            query_trace: plan.query_trace.clone(),
            elapsed_millis: elapsed_millis(commit_started),
            source_text_logged: false,
        },
        ablation_rankings,
        graph_edge_ablation_rankings,
        gap_reasons,
        ranking_millis,
        pack_compiler_millis,
    })
}

fn project_source_memory_to_eval_root(source_repo: &Path, eval_root: &Path) {
    if source_repo == eval_root {
        return;
    }
    let Ok(cards) = list_memory_cards(source_repo, &StoreConfig::default(), false) else {
        return;
    };
    if cards.is_empty() {
        return;
    }
    let records = cards
        .into_iter()
        .map(|card| StorageMemoryCardRecord { card })
        .collect::<Vec<_>>();
    let _ = persist_memory_card_records(eval_root, &StoreConfig::default(), &records);
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

        if let Some(cached_path) =
            cached_historical_eval_parent_worktree(source_repo, parent_sha, snapshot_paths)?
        {
            return Ok(Self {
                path: cached_path,
                _source_repo: source_repo,
                _temp_dir: None,
            });
        }

        let path =
            build_historical_eval_parent_worktree_cache(source_repo, parent_sha, snapshot_paths)?;

        Ok(Self {
            path,
            _source_repo: source_repo,
            _temp_dir: None,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

fn cached_historical_eval_parent_worktree(
    source_repo: &Path,
    parent_sha: &str,
    snapshot_paths: &[String],
) -> Result<Option<PathBuf>, InventoryError> {
    let path = historical_eval_parent_cache_repo_path(source_repo, parent_sha, snapshot_paths);
    if parent_snapshot_cache_is_reusable(&path, parent_sha)? {
        return Ok(Some(path));
    }
    Ok(None)
}

fn parent_snapshot_manifest_is_reusable(
    manifest: &ParentSnapshotManifest,
    parent_sha: &str,
) -> bool {
    manifest.manifest_version == PARENT_SNAPSHOT_SCHEMA_VERSION
        && manifest.parent_sha == parent_sha
        && manifest.complete
}

fn parent_snapshot_cache_is_reusable(
    path: &Path,
    parent_sha: &str,
) -> Result<bool, InventoryError> {
    if !path.join(".ctxhelm/eval-history.json").is_file() {
        return Ok(false);
    }
    let Some(manifest) = read_parent_snapshot_manifest(path)? else {
        return Ok(false);
    };
    Ok(parent_snapshot_manifest_is_reusable(&manifest, parent_sha))
}

fn build_historical_eval_parent_worktree_cache(
    source_repo: &Path,
    parent_sha: &str,
    snapshot_paths: &[String],
) -> Result<PathBuf, InventoryError> {
    let path = historical_eval_parent_cache_repo_path(source_repo, parent_sha, snapshot_paths);
    if parent_snapshot_cache_is_reusable(&path, parent_sha)? {
        return Ok(path);
    }
    let Some(cache_parent) = path.parent() else {
        return Ok(path);
    };
    fs::create_dir_all(cache_parent).map_err(|source| InventoryError::CreateDir {
        path: cache_parent.to_path_buf(),
        source,
    })?;
    let staging_path = cache_parent.join(format!(
        ".tmp-{}-{}",
        parent_sha.chars().take(12).collect::<String>(),
        Uuid::new_v4()
    ));
    fs::create_dir_all(&staging_path).map_err(|source| InventoryError::CreateDir {
        path: staging_path.clone(),
        source,
    })?;

    let build_result = (|| {
        let extraction =
            git_extract_revision_paths(source_repo, parent_sha, snapshot_paths, &staging_path)?;
        write_eval_history_sidecar(source_repo, parent_sha, &staging_path)?;
        write_parent_snapshot_manifest(
            &staging_path,
            &ParentSnapshotManifest {
                manifest_version: PARENT_SNAPSHOT_SCHEMA_VERSION.to_string(),
                parent_sha: parent_sha.to_string(),
                requested_path_count: extraction.requested_path_count,
                extracted_path_count: extraction.extracted_path_count,
                failed_batch_count: extraction.failed_batch_count,
                complete: extraction.complete(),
            },
        )?;
        if !extraction.complete() {
            return Err(InventoryError::Git {
                repo_root: source_repo.to_path_buf(),
                message: format!(
                    "parent snapshot object extraction incomplete for {parent_sha}: requested {}, extracted {}, failed batches {}",
                    extraction.requested_path_count,
                    extraction.extracted_path_count,
                    extraction.failed_batch_count
                ),
            });
        }
        Ok::<(), InventoryError>(())
    })();
    if let Err(error) = build_result {
        let _ = fs::remove_dir_all(&staging_path);
        return Err(error);
    }

    if parent_snapshot_cache_is_reusable(&path, parent_sha)? {
        let _ = fs::remove_dir_all(&staging_path);
        return Ok(path);
    }
    if path.exists() {
        evict_eval_snapshot_repo_cache(&path)?;
        fs::remove_dir_all(&path).map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
    }
    match fs::rename(&staging_path, &path) {
        Ok(()) => Ok(path),
        Err(source) if parent_snapshot_cache_is_reusable(&path, parent_sha)? => {
            let _ = fs::remove_dir_all(&staging_path);
            let _ = source;
            Ok(path)
        }
        Err(source) => {
            let _ = fs::remove_dir_all(&staging_path);
            Err(InventoryError::Write { path, source })
        }
    }
}

fn evict_eval_snapshot_repo_cache(snapshot_path: &Path) -> Result<(), InventoryError> {
    let repo_id = repo_id_for_path(snapshot_path);
    let repo_cache_dir = inventory_path(&repo_id)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            historical_eval_parent_cache_root()
                .join("repos")
                .join(repo_id)
        });
    if !repo_cache_dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&repo_cache_dir).map_err(|source| InventoryError::Write {
        path: repo_cache_dir,
        source,
    })
}

fn read_parent_snapshot_manifest(
    snapshot_path: &Path,
) -> Result<Option<ParentSnapshotManifest>, InventoryError> {
    let manifest_path = snapshot_path.join(PARENT_SNAPSHOT_MANIFEST);
    if !manifest_path.is_file() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&manifest_path).map_err(|source| InventoryError::Read {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest = serde_json::from_str(&contents).map_err(|source| InventoryError::Read {
        path: manifest_path,
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
    })?;
    Ok(Some(manifest))
}

fn write_parent_snapshot_manifest(
    snapshot_path: &Path,
    manifest: &ParentSnapshotManifest,
) -> Result<(), InventoryError> {
    let manifest_path = snapshot_path.join(PARENT_SNAPSHOT_MANIFEST);
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let contents =
        serde_json::to_string_pretty(manifest).map_err(|source| InventoryError::Write {
            path: manifest_path.clone(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
        })?;
    fs::write(&manifest_path, contents).map_err(|source| InventoryError::Write {
        path: manifest_path,
        source,
    })
}

fn historical_eval_parent_cache_repo_path(
    source_repo: &Path,
    parent_sha: &str,
    snapshot_paths: &[String],
) -> PathBuf {
    let parent_prefix = parent_sha.chars().take(12).collect::<String>();
    historical_eval_parent_cache_root()
        .join(repo_id_for_path(source_repo))
        .join("eval-worktrees")
        .join(format!(
            "{}-{}",
            parent_prefix,
            snapshot_paths_fingerprint(snapshot_paths)
        ))
}

fn historical_eval_parent_cache_root() -> PathBuf {
    std::env::var_os("CTXHELM_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".ctxhelm")))
        .unwrap_or_else(|| std::env::temp_dir().join("ctxhelm"))
}

fn snapshot_paths_fingerprint(snapshot_paths: &[String]) -> String {
    let mut fingerprint_source = format!(
        "{PARENT_SNAPSHOT_SCHEMA_VERSION}\n{}\n",
        snapshot_paths.len()
    );
    for path in snapshot_paths {
        fingerprint_source.push_str(path);
        fingerprint_source.push('\n');
    }
    task_hash(&fingerprint_source)[..16].to_string()
}

fn parent_snapshot_candidate_paths(
    snapshot_paths: &[String],
    safe_changed_files: &[String],
) -> Vec<String> {
    let safe_changes = safe_changed_files.iter().collect::<BTreeSet<_>>();
    snapshot_paths
        .iter()
        .filter(|path| safe_changes.contains(path) || !is_generated_eval_artifact_path(path))
        .cloned()
        .collect()
}

fn is_generated_eval_artifact_path(path: &str) -> bool {
    path.starts_with(".ctxhelm/e2e/")
        || path.starts_with(".planning/e2e/")
        || path.starts_with(".planning/milestones/")
}

fn git_extract_revision_paths(
    source_repo: &Path,
    revision: &str,
    paths: &[String],
    destination: &Path,
) -> Result<ParentSnapshotExtractionStatus, InventoryError> {
    let mut status = ParentSnapshotExtractionStatus::default();
    for chunk in paths.chunks(100) {
        status.merge(git_extract_revision_path_chunk(
            source_repo,
            revision,
            chunk,
            destination,
        )?);
    }
    Ok(status)
}

fn git_extract_revision_path_chunk(
    source_repo: &Path,
    revision: &str,
    paths: &[String],
    destination: &Path,
) -> Result<ParentSnapshotExtractionStatus, InventoryError> {
    if paths.is_empty() {
        return Ok(ParentSnapshotExtractionStatus::default());
    }
    let failed_status = || ParentSnapshotExtractionStatus {
        requested_path_count: paths.len(),
        extracted_path_count: 0,
        failed_batch_count: 1,
    };

    let mut input = Vec::new();
    for path in paths {
        input.extend_from_slice(format!("{revision}:{path}\n").as_bytes());
    }
    let mut cat_file = ProcessCommand::new("git");
    cat_file
        .arg("-C")
        .arg(source_repo)
        .args(["-c", "core.fsmonitor=false"])
        .args(["cat-file", "--batch"]);
    match command_output_with_timeout_and_stdin(
        source_repo,
        cat_file,
        &input,
        format!("git cat-file --batch {revision}:<{} paths>", paths.len()),
        PARENT_SNAPSHOT_BATCH_READ_TIMEOUT,
    ) {
        Ok(output) if output.status.success() => {
            write_cat_file_batch_blobs(destination, paths, &output.stdout).map(
                |extracted_path_count| ParentSnapshotExtractionStatus {
                    requested_path_count: paths.len(),
                    extracted_path_count,
                    failed_batch_count: 0,
                },
            )
        }
        Ok(_) | Err(_) => Ok(failed_status()),
    }
}

fn write_cat_file_batch_blobs(
    destination: &Path,
    paths: &[String],
    output: &[u8],
) -> Result<usize, InventoryError> {
    let mut cursor = 0usize;
    let mut extracted = 0usize;
    for path in paths {
        let Some(header_end_offset) = output[cursor..].iter().position(|byte| *byte == b'\n')
        else {
            break;
        };
        let header_end = cursor + header_end_offset;
        let header = String::from_utf8_lossy(&output[cursor..header_end]);
        cursor = header_end + 1;
        if header.ends_with(" missing") {
            continue;
        }
        let mut parts = header.rsplitn(3, ' ');
        let Some(size_text) = parts.next() else {
            break;
        };
        let Some(kind) = parts.next() else {
            break;
        };
        let Ok(size) = size_text.parse::<usize>() else {
            break;
        };
        if kind != "blob" || cursor.saturating_add(size) > output.len() {
            cursor = cursor.saturating_add(size).min(output.len());
            if output.get(cursor) == Some(&b'\n') {
                cursor += 1;
            }
            continue;
        }
        let destination_path = destination.join(path);
        write_parent_snapshot_file(&destination_path, &output[cursor..cursor + size])?;
        extracted += 1;
        cursor += size;
        if output.get(cursor) == Some(&b'\n') {
            cursor += 1;
        }
    }
    Ok(extracted)
}

fn write_parent_snapshot_file(path: &Path, contents: &[u8]) -> Result<(), InventoryError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, contents).map_err(|source| InventoryError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

#[cfg(test)]
fn command_output_with_timeout(
    repo_root: &Path,
    mut command: ProcessCommand,
    display: String,
    timeout: Duration,
) -> Result<std::process::Output, InventoryError> {
    let spool_dir = std::env::temp_dir().join(format!("ctxhelm-command-{}", Uuid::new_v4()));
    fs::create_dir_all(&spool_dir).map_err(|source| InventoryError::CreateDir {
        path: spool_dir.clone(),
        source,
    })?;
    let stdout_path = spool_dir.join("stdout");
    let stderr_path = spool_dir.join("stderr");
    let stdout = fs::File::create(&stdout_path).map_err(|source| InventoryError::Write {
        path: stdout_path.clone(),
        source,
    })?;
    let stderr = fs::File::create(&stderr_path).map_err(|source| InventoryError::Write {
        path: stderr_path.clone(),
        source,
    })?;
    command
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let child = match command.spawn() {
        Ok(child) => child,
        Err(source) => {
            let _ = fs::remove_dir_all(&spool_dir);
            return Err(InventoryError::Git {
                repo_root: repo_root.to_path_buf(),
                message: source.to_string(),
            });
        }
    };
    let output = wait_for_command_output(
        repo_root,
        child,
        display,
        timeout,
        &stdout_path,
        &stderr_path,
    );
    let _ = fs::remove_dir_all(&spool_dir);
    output
}

fn command_output_with_timeout_and_stdin(
    repo_root: &Path,
    mut command: ProcessCommand,
    stdin: &[u8],
    display: String,
    timeout: Duration,
) -> Result<std::process::Output, InventoryError> {
    let spool_dir = std::env::temp_dir().join(format!("ctxhelm-command-{}", Uuid::new_v4()));
    fs::create_dir_all(&spool_dir).map_err(|source| InventoryError::CreateDir {
        path: spool_dir.clone(),
        source,
    })?;
    let stdout_path = spool_dir.join("stdout");
    let stderr_path = spool_dir.join("stderr");
    let stdout = fs::File::create(&stdout_path).map_err(|source| InventoryError::Write {
        path: stdout_path.clone(),
        source,
    })?;
    let stderr = fs::File::create(&stderr_path).map_err(|source| InventoryError::Write {
        path: stderr_path.clone(),
        source,
    })?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(source) => {
            let _ = fs::remove_dir_all(&spool_dir);
            return Err(InventoryError::Git {
                repo_root: repo_root.to_path_buf(),
                message: source.to_string(),
            });
        }
    };
    if let Some(mut child_stdin) = child.stdin.take() {
        if let Err(source) = child_stdin.write_all(stdin) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_dir_all(&spool_dir);
            return Err(InventoryError::Git {
                repo_root: repo_root.to_path_buf(),
                message: source.to_string(),
            });
        }
    }
    let output = wait_for_command_output(
        repo_root,
        child,
        display,
        timeout,
        &stdout_path,
        &stderr_path,
    );
    let _ = fs::remove_dir_all(&spool_dir);
    output
}

fn wait_for_command_output(
    repo_root: &Path,
    mut child: std::process::Child,
    display: String,
    timeout: Duration,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<std::process::Output, InventoryError> {
    let start = Instant::now();
    loop {
        match child.try_wait().map_err(|source| InventoryError::Git {
            repo_root: repo_root.to_path_buf(),
            message: source.to_string(),
        })? {
            Some(status) => {
                let stdout = fs::read(stdout_path).map_err(|source| InventoryError::Read {
                    path: stdout_path.to_path_buf(),
                    source,
                })?;
                let stderr = fs::read(stderr_path).map_err(|source| InventoryError::Read {
                    path: stderr_path.to_path_buf(),
                    source,
                })?;
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(InventoryError::Git {
                    repo_root: repo_root.to_path_buf(),
                    message: format!("{display} timed out after {timeout:?}"),
                });
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }
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

#[allow(clippy::too_many_arguments)]
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
    ctxhelm_cache_root()
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
    reserve_validation_tests: bool,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let ranking_budget = ranking_budget.max(1);
    let test_reserve = if reserve_validation_tests {
        validation_test_reserve(recommended_files, recommended_tests, ranking_budget)
    } else {
        0
    };
    let file_budget = ranking_budget.saturating_sub(test_reserve).max(1);
    let mut ranking = recommended_files
        .iter()
        .filter_map(|path| seen.insert(path.clone()).then_some(path.clone()))
        .take(file_budget)
        .collect::<Vec<_>>();
    ranking.extend(
        recommended_tests
            .iter()
            .filter_map(|path| seen.insert(path.clone()).then_some(path.clone()))
            .take(test_reserve),
    );
    ranking.extend(
        recommended_files
            .iter()
            .filter_map(|path| seen.insert(path.clone()).then_some(path.clone()))
            .take(ranking_budget.saturating_sub(ranking.len())),
    );
    ranking.extend(
        recommended_tests
            .iter()
            .filter_map(|path| seen.insert(path.clone()).then_some(path.clone()))
            .take(ranking_budget.saturating_sub(ranking.len())),
    );
    ranking.truncate(ranking_budget);
    ranking
}

fn validation_test_reserve(
    recommended_files: &[String],
    recommended_tests: &[String],
    ranking_budget: usize,
) -> usize {
    if recommended_tests.is_empty()
        || recommended_files.len() + recommended_tests.len() <= ranking_budget
        || ranking_budget < 8
    {
        return 0;
    }
    let max_reserve = (ranking_budget / 4).max(1);
    recommended_tests.len().min(max_reserve)
}

fn retrieval_target_files(eval_root: &Path, safe_changed_files: &[String]) -> Vec<String> {
    safe_changed_files
        .iter()
        .filter(|path| eval_root.join(path).is_file())
        .cloned()
        .collect()
}

fn local_metadata_reranked_context_files(
    plan: &ContextPlan,
    ranking_budget: usize,
    default_context_files: &[String],
    reserve_validation_tests: bool,
) -> Vec<String> {
    let ranking_budget = ranking_budget.max(1);
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
    let test_reserve = if reserve_validation_tests {
        validation_test_reserve(&recommended_files, &recommended_tests, ranking_budget)
    } else {
        0
    };
    let file_budget = ranking_budget.saturating_sub(test_reserve).max(1);
    let protected_floor =
        local_metadata_protected_floor_paths(plan, default_context_files, file_budget);
    let mut candidates = plan
        .retrieval_candidates
        .iter()
        .filter_map(|candidate| {
            candidate
                .path
                .as_ref()
                .map(|path| (path.clone(), local_metadata_candidate_score(candidate)))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|(left_path, left_score), (right_path, right_score)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left_path.cmp(right_path))
    });

    let mut seen = BTreeSet::new();
    let mut ranking = protected_floor
        .into_iter()
        .filter_map(|path| seen.insert(path.clone()).then_some(path))
        .take(file_budget)
        .collect::<Vec<_>>();
    ranking.extend(
        candidates
            .iter()
            .filter(|(path, _)| {
                plan.retrieval_candidates
                    .iter()
                    .find(|candidate| candidate.path.as_ref() == Some(path))
                    .and_then(|candidate| candidate.role.clone())
                    != Some(FileRole::Test)
            })
            .filter_map(|(path, _)| seen.insert(path.clone()).then_some(path.clone()))
            .take(file_budget.saturating_sub(ranking.len())),
    );
    ranking.extend(
        recommended_tests
            .into_iter()
            .filter_map(|path| seen.insert(path.clone()).then_some(path))
            .take(test_reserve),
    );
    ranking.extend(
        candidates
            .into_iter()
            .filter_map(|(path, _)| seen.insert(path.clone()).then_some(path))
            .take(ranking_budget.saturating_sub(ranking.len())),
    );
    ranking.truncate(ranking_budget);
    ranking
}

fn local_metadata_protected_floor_paths(
    plan: &ContextPlan,
    default_context_files: &[String],
    ranking_budget: usize,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let ranking_budget = ranking_budget.max(1);
    let candidate_by_path = plan
        .retrieval_candidates
        .iter()
        .filter_map(|candidate| candidate.path.as_ref().map(|path| (path, candidate)))
        .collect::<BTreeMap<_, _>>();
    let mut floor = default_context_files
        .iter()
        .filter_map(|path| {
            let candidate = candidate_by_path.get(path)?;
            (metadata_candidate_has_protected_source_signal(candidate) && seen.insert(path.clone()))
                .then_some(path.clone())
        })
        .take(ranking_budget)
        .collect::<Vec<_>>();
    floor.extend(
        plan.retrieval_candidates
            .iter()
            .filter_map(|candidate| {
                let path = candidate.path.as_ref()?;
                (metadata_candidate_has_protected_source_signal(candidate)
                    && seen.insert(path.clone()))
                .then_some(path.clone())
            })
            .take(ranking_budget.saturating_sub(floor.len())),
    );
    floor
}

fn metadata_candidate_has_protected_source_signal(candidate: &RetrievalCandidate) -> bool {
    if candidate.role == Some(FileRole::Test) {
        return false;
    }
    candidate
        .signal_scores
        .iter()
        .map(|score| &score.signal)
        .chain(candidate.evidence.iter().map(|evidence| &evidence.signal))
        .any(is_protected_evidence_signal)
}

fn local_metadata_candidate_score(candidate: &RetrievalCandidate) -> f32 {
    let exact_score = candidate
        .signal_scores
        .iter()
        .filter(|score| is_protected_evidence_signal(&score.signal))
        .map(|score| score.score * score.weight)
        .sum::<f32>();
    candidate.confidence + exact_score + candidate.evidence.len() as f32 * 0.05
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

fn search_report_paths(report: ctxhelm_index::SearchReport) -> Vec<String> {
    report
        .results
        .into_iter()
        .map(|result| result.path)
        .collect()
}

fn lexical_backend_metrics(
    backend: &str,
    rows: &[LexicalBackendCommitRow],
    files: impl Fn(&LexicalBackendCommitRow) -> &Vec<String>,
    hits_at_5: impl Fn(&LexicalBackendCommitRow) -> &Vec<String>,
    hits_at_10: impl Fn(&LexicalBackendCommitRow) -> &Vec<String>,
    millis: impl Fn(&LexicalBackendCommitRow) -> u64,
) -> LexicalBackendMetrics {
    if rows.is_empty() {
        return LexicalBackendMetrics {
            backend: backend.to_string(),
            recall_at_5: 0.0,
            recall_at_10: 0.0,
            mrr_at_10: 0.0,
            average_result_count: 0.0,
            total_millis: 0,
        };
    }

    let recall_at_5 = rows
        .iter()
        .map(|row| recall_for_hits(&row.retrieval_target_files, hits_at_5(row)))
        .sum::<f32>()
        / rows.len() as f32;
    let recall_at_10 = rows
        .iter()
        .map(|row| recall_for_hits(&row.retrieval_target_files, hits_at_10(row)))
        .sum::<f32>()
        / rows.len() as f32;
    let mrr_at_10 = rows
        .iter()
        .map(|row| lexical_backend_reciprocal_rank(&row.retrieval_target_files, files(row), 10))
        .sum::<f32>()
        / rows.len() as f32;
    let average_result_count =
        rows.iter().map(|row| files(row).len()).sum::<usize>() as f32 / rows.len() as f32;
    let total_millis = rows.iter().map(millis).sum::<u64>();

    LexicalBackendMetrics {
        backend: backend.to_string(),
        recall_at_5,
        recall_at_10,
        mrr_at_10,
        average_result_count,
        total_millis,
    }
}

fn lexical_backend_comparison(
    rows: &[LexicalBackendCommitRow],
    bm25: &LexicalBackendMetrics,
    legacy: &LexicalBackendMetrics,
    ranking_budget: usize,
) -> LexicalBackendComparison {
    if rows.is_empty() {
        return LexicalBackendComparison {
            recall_delta_at_5: 0.0,
            recall_delta_at_10: 0.0,
            mrr_delta_at_10: 0.0,
            average_overlap_at_k: 0.0,
            top_path_changed_rate: 0.0,
            bm25_wins_at_10: 0,
            legacy_wins_at_10: 0,
            ties_at_10: 0,
        };
    }

    let average_overlap_at_k =
        rows.iter().map(|row| row.overlap_at_k).sum::<usize>() as f32 / rows.len() as f32;
    let top_path_changed_rate =
        rows.iter().filter(|row| row.top_path_changed).count() as f32 / rows.len() as f32;
    let mut bm25_wins_at_10 = 0usize;
    let mut legacy_wins_at_10 = 0usize;
    let mut ties_at_10 = 0usize;
    for row in rows {
        match row.bm25_recall_at_10.total_cmp(&row.legacy_recall_at_10) {
            std::cmp::Ordering::Greater => bm25_wins_at_10 += 1,
            std::cmp::Ordering::Less => legacy_wins_at_10 += 1,
            std::cmp::Ordering::Equal => ties_at_10 += 1,
        }
    }

    LexicalBackendComparison {
        recall_delta_at_5: bm25.recall_at_5 - legacy.recall_at_5,
        recall_delta_at_10: bm25.recall_at_10 - legacy.recall_at_10,
        mrr_delta_at_10: bm25.mrr_at_10 - legacy.mrr_at_10,
        average_overlap_at_k: average_overlap_at_k.min(ranking_budget as f32),
        top_path_changed_rate,
        bm25_wins_at_10,
        legacy_wins_at_10,
        ties_at_10,
    }
}

fn recall_for_hits(targets: &[String], hits: &[String]) -> f32 {
    if targets.is_empty() {
        0.0
    } else {
        hits.len() as f32 / targets.len() as f32
    }
}

fn lexical_backend_reciprocal_rank(targets: &[String], files: &[String], limit: usize) -> f32 {
    if targets.is_empty() {
        return 0.0;
    }
    let targets = targets.iter().collect::<BTreeSet<_>>();
    for (index, file) in files.iter().take(limit).enumerate() {
        if targets.contains(file) {
            return 1.0 / (index + 1) as f32;
        }
    }
    0.0
}

fn overlap_at_limit(left: &[String], right: &[String], limit: usize) -> usize {
    let left = left.iter().take(limit).collect::<BTreeSet<_>>();
    right
        .iter()
        .take(limit)
        .filter(|path| left.contains(path))
        .count()
}

fn lexical_backend_corpus_range_id(
    repo_id: &str,
    options: &LexicalBackendCorpusOptions,
    refs: &HistoricalEvalRefs,
) -> String {
    task_hash(&format!(
        "version={LEXICAL_BACKEND_CORPUS_SCHEMA_VERSION}\nrepo={repo_id}\nlimit={}\nrankingBudget={}\nbase={}\nhead={}",
        options.limit,
        options.ranking_budget.max(1),
        refs.base.as_deref().unwrap_or(""),
        refs.head.as_deref().unwrap_or("")
    ))
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

fn changed_context_areas(
    labels: &[HistoricalChangedPathLabel],
    retrieval_target_files: &[String],
) -> Vec<String> {
    let retrieval_targets = retrieval_target_files.iter().collect::<BTreeSet<_>>();
    labels
        .iter()
        .filter(|label| {
            label.label_scope == LabelScope::Safe
                && retrieval_targets.contains(&label.path)
                && matches!(
                    label.role,
                    FileRole::Source | FileRole::Test | FileRole::Config | FileRole::Schema
                )
        })
        .map(|label| context_area_for_path(&label.path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn context_area_hits(
    changed_context_areas: &[String],
    context_areas: &[ContextArea],
    recommended_context_files: &[String],
) -> Vec<String> {
    let changed = changed_context_areas.iter().collect::<BTreeSet<_>>();
    let mut surfaced = context_areas
        .iter()
        .map(|area| area.area.clone())
        .collect::<BTreeSet<_>>();
    for path in recommended_context_files {
        surfaced.insert(context_area_for_path(path));
    }
    changed
        .into_iter()
        .filter(|area| surfaced.contains(*area))
        .cloned()
        .collect()
}

fn validation_command_hits(test_changed_files: &[String], commands: &[String]) -> Vec<String> {
    test_changed_files
        .iter()
        .filter(|path| {
            commands
                .iter()
                .any(|command| validation_command_covers_path(command, path))
        })
        .cloned()
        .collect()
}

fn effective_validation_hits_at_10(
    test_changed_files: &[String],
    recommended_tests: &[String],
    commands: &[String],
) -> usize {
    effective_validation_hit_paths(test_changed_files, recommended_tests, commands).len()
}

fn effective_validation_hit_paths(
    test_changed_files: &[String],
    recommended_tests: &[String],
    commands: &[String],
) -> BTreeSet<String> {
    let mut hits = changed_file_hits(test_changed_files, recommended_tests, 10)
        .into_iter()
        .collect::<BTreeSet<_>>();
    hits.extend(validation_command_hits(test_changed_files, commands));
    hits
}

fn validation_command_covers_path(command: &str, path: &str) -> bool {
    let command = command.trim();
    if command.is_empty() {
        return false;
    }
    let lower_path = path.to_ascii_lowercase();
    if matches!(command, "pytest" | "python -m pytest") {
        return lower_path.ends_with(".py");
    }
    if let Some(args) = command.strip_prefix("pytest ") {
        return pytest_args_cover_path(args, path);
    }
    if matches!(command, "cargo test") {
        return lower_path.ends_with(".rs");
    }
    if matches!(command, "./gradlew test" | "gradle test" | "mvn test") {
        return lower_path.ends_with(".java") || lower_path.ends_with(".kt");
    }
    if let Some(selector) = command
        .strip_prefix("./gradlew test --tests ")
        .or_else(|| command.strip_prefix("gradle test --tests "))
    {
        return java_selector_covers_path(selector, path);
    }
    if command.starts_with("mvn ") && command.contains("-Dtest=") {
        return command
            .split_whitespace()
            .find_map(|arg| arg.strip_prefix("-Dtest="))
            .is_some_and(|selector| java_selector_covers_path(selector, path));
    }
    if matches!(
        command,
        "pnpm vitest run"
            | "npm vitest run"
            | "yarn vitest run"
            | "pnpm jest"
            | "npm jest"
            | "yarn jest"
            | "pnpm test"
            | "npm test"
            | "yarn test"
    ) {
        return matches!(
            lower_path.rsplit('.').next(),
            Some("ts" | "tsx" | "js" | "jsx")
        );
    }

    false
}

fn java_selector_covers_path(selector: &str, path: &str) -> bool {
    let selector = selector
        .split('#')
        .next()
        .unwrap_or(selector)
        .trim_matches('"')
        .trim_matches('\'')
        .trim();
    if selector.is_empty() {
        return false;
    }
    let Some(class_path) = path
        .strip_prefix("src/test/java/")
        .or_else(|| path.strip_prefix("src/test/kotlin/"))
    else {
        return false;
    };
    let Some(class_path) = class_path
        .strip_suffix(".java")
        .or_else(|| class_path.strip_suffix(".kt"))
    else {
        return false;
    };
    let fqcn = class_path.replace('/', ".");
    let class_name = fqcn.rsplit('.').next().unwrap_or(fqcn.as_str());
    selector == class_name || selector == fqcn
}

fn pytest_args_cover_path(args: &str, path: &str) -> bool {
    args.split_whitespace()
        .filter(|arg| !arg.starts_with('-'))
        .any(|arg| {
            let target = arg.trim_end_matches('/');
            !target.is_empty()
                && (path == target
                    || path
                        .strip_prefix(target)
                        .is_some_and(|rest| rest.starts_with('/')))
        })
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
        "version={HISTORICAL_EVAL_CACHE_SCHEMA_VERSION}\nrepo={repo_id}\nlimit={}\nrankingBudget={}\nmode={:?}\ntarget={}\nbudget={:?}\nsemantic={}\nsemanticProvider={}\nlocalMetadataReranker={}\nbase={}\nhead={}",
        filters.limit,
        filters.ranking_budget,
        filters.mode,
        filters.target_agent,
        filters.budget,
        filters.semantic_enabled,
        filters.semantic_provider.as_deref().unwrap_or(""),
        filters.local_metadata_reranker,
        refs.base.as_deref().unwrap_or(""),
        refs.head.as_deref().unwrap_or("")
    ))
}

fn historical_eval_cache_path(repo_id: &str, eval_range_id: &str) -> PathBuf {
    ctxhelm_cache_root()
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

fn ctxhelm_cache_root() -> PathBuf {
    if let Ok(value) = std::env::var("CTXHELM_HOME") {
        return PathBuf::from(value);
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".ctxhelm")
}

#[derive(Debug, Clone)]
struct SignalAblationRankings {
    disabled_signal: RetrievalSignalKind,
    rankings: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
struct GraphEdgeAblationCommitRanking {
    edge_label: String,
    ranking: Vec<String>,
    affected_commit: bool,
    removed_selected_at_10_count: usize,
    removed_target_hit_at_10_count: usize,
}

#[derive(Debug, Clone)]
struct GraphEdgeAblationRankings {
    edge_label: String,
    rankings: Vec<Vec<String>>,
    affected_commit_count: usize,
    removed_selected_at_10_count: usize,
    removed_target_hit_at_10_count: usize,
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

fn selected_signal_profiles(
    recommended_context_files: &[String],
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
    candidate_roles_by_path: &BTreeMap<String, FileRole>,
    retrieval_target_paths: &BTreeSet<String>,
    limit: usize,
) -> Vec<HistoricalSelectedSignalProfile> {
    let mut profiles = Vec::<HistoricalSelectedSignalProfile>::new();
    for path in recommended_context_files.iter().take(limit) {
        let role = candidate_roles_by_path
            .get(path)
            .cloned()
            .unwrap_or(FileRole::Unknown);
        let Some(signals) = signals_by_path.get(path) else {
            continue;
        };
        for signal in signals {
            if let Some(profile) = profiles
                .iter_mut()
                .find(|profile| profile.signal == *signal && profile.role == role)
            {
                profile.selected_at_10_count += 1;
                if retrieval_target_paths.contains(path) {
                    profile.retrieval_target_selected_at_10_count += 1;
                }
            } else {
                profiles.push(HistoricalSelectedSignalProfile {
                    signal: signal.clone(),
                    role: role.clone(),
                    selected_at_10_count: 1,
                    retrieval_target_selected_at_10_count: usize::from(
                        retrieval_target_paths.contains(path),
                    ),
                });
            }
        }
    }
    profiles.sort_by(|left, right| {
        signal_order(&left.signal)
            .cmp(&signal_order(&right.signal))
            .then_with(|| file_role_order(&left.role).cmp(&file_role_order(&right.role)))
    });
    profiles
}

fn signal_order(signal: &RetrievalSignalKind) -> u8 {
    match signal {
        RetrievalSignalKind::Anchor => 0,
        RetrievalSignalKind::CurrentDiff => 1,
        RetrievalSignalKind::Lexical => 2,
        RetrievalSignalKind::LexicalExpansion => 3,
        RetrievalSignalKind::Symbol => 4,
        RetrievalSignalKind::Dependency => 5,
        RetrievalSignalKind::RelatedTest => 6,
        RetrievalSignalKind::CoChange => 7,
        RetrievalSignalKind::History => 8,
        RetrievalSignalKind::Semantic => 9,
        RetrievalSignalKind::Config => 10,
        RetrievalSignalKind::Docs => 11,
        RetrievalSignalKind::Memory => 12,
    }
}

fn file_role_order(role: &FileRole) -> u8 {
    match role {
        FileRole::Source => 0,
        FileRole::Test => 1,
        FileRole::Config => 2,
        FileRole::Schema => 3,
        FileRole::Docs => 4,
        FileRole::Generated => 5,
        FileRole::Sensitive => 6,
        FileRole::Unknown => 7,
    }
}

fn protected_evidence_files(
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
    protected_budget_paths: &BTreeSet<String>,
    recommended_context_files: &[String],
    retrieval_target_paths: &BTreeSet<String>,
    candidate_roles_by_path: &BTreeMap<String, FileRole>,
    limit: usize,
) -> Vec<HistoricalProtectedEvidenceFile> {
    let selected = recommended_context_files
        .iter()
        .take(limit)
        .cloned()
        .collect::<BTreeSet<_>>();
    signals_by_path
        .iter()
        .filter(|(path, _)| protected_budget_paths.contains(path.as_str()))
        .filter_map(|(path, signals)| {
            let protected_signals = signals
                .iter()
                .filter(|signal| is_protected_evidence_signal(signal))
                .cloned()
                .collect::<Vec<_>>();
            (!protected_signals.is_empty()).then(|| HistoricalProtectedEvidenceFile {
                path: path.clone(),
                signals: protected_signals,
                selected_at_10: selected.contains(path),
                retrieval_target: retrieval_target_paths.contains(path),
                role: candidate_roles_by_path.get(path).cloned(),
            })
        })
        .collect()
}

fn candidate_roles_by_path(plan: &ContextPlan) -> BTreeMap<String, FileRole> {
    let mut roles = BTreeMap::new();
    for candidate in &plan.retrieval_candidates {
        let (Some(path), Some(role)) = (&candidate.path, candidate.role.clone()) else {
            continue;
        };
        roles.entry(path.clone()).or_insert(role);
    }
    roles
}

fn candidate_missed_file_profiles(
    missing_files_at_10: &[String],
    plan: &ContextPlan,
    candidate_roles_by_path: &BTreeMap<String, FileRole>,
) -> Vec<CandidateMissedFileProfile> {
    let mut signals_by_path = BTreeMap::<String, Vec<RetrievalSignalKind>>::new();
    for candidate in &plan.retrieval_candidates {
        let Some(path) = &candidate.path else {
            continue;
        };
        let signals = signals_by_path.entry(path.clone()).or_default();
        for signal in candidate
            .signal_scores
            .iter()
            .map(|score| score.signal.clone())
            .chain(
                candidate
                    .evidence
                    .iter()
                    .map(|evidence| evidence.signal.clone()),
            )
        {
            if !signals.contains(&signal) {
                signals.push(signal);
            }
        }
        signals.sort_by_key(signal_sort_key);
    }

    missing_files_at_10
        .iter()
        .filter_map(|path| {
            let signals = signals_by_path.get(path)?;
            Some(CandidateMissedFileProfile {
                path: path.clone(),
                role: candidate_roles_by_path
                    .get(path)
                    .cloned()
                    .unwrap_or(FileRole::Unknown),
                context_area: context_area_for_path(path),
                signals: signals.clone(),
            })
        })
        .collect()
}

fn signal_sort_key(signal: &RetrievalSignalKind) -> u8 {
    match signal {
        RetrievalSignalKind::Anchor => 0,
        RetrievalSignalKind::CurrentDiff => 1,
        RetrievalSignalKind::Lexical => 2,
        RetrievalSignalKind::LexicalExpansion => 3,
        RetrievalSignalKind::Symbol => 4,
        RetrievalSignalKind::Dependency => 5,
        RetrievalSignalKind::RelatedTest => 6,
        RetrievalSignalKind::CoChange => 7,
        RetrievalSignalKind::Semantic => 8,
        RetrievalSignalKind::History => 9,
        RetrievalSignalKind::Config => 10,
        RetrievalSignalKind::Docs => 11,
        RetrievalSignalKind::Memory => 12,
    }
}

fn budgeted_protected_evidence_paths(plan: &ContextPlan, limit: usize) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let limit = limit.max(1);
    for candidate in &plan.retrieval_candidates {
        let Some(path) = &candidate.path else {
            continue;
        };
        if candidate.role == Some(FileRole::Test) {
            continue;
        }
        let has_protected_signal = candidate
            .signal_scores
            .iter()
            .map(|score| &score.signal)
            .chain(candidate.evidence.iter().map(|evidence| &evidence.signal))
            .any(is_protected_evidence_signal);
        if has_protected_signal {
            paths.insert(path.clone());
            if paths.len() >= limit {
                break;
            }
        }
    }
    paths
}

fn protected_evidence_summary(commits: &[HistoricalCommitEval]) -> ProtectedEvidenceSummary {
    let mut candidate_count = 0usize;
    let mut missed_at_10_count = 0usize;
    let mut retrieval_target_candidate_count = 0usize;
    let mut retrieval_target_missed_at_10_count = 0usize;
    let mut non_target_candidate_count = 0usize;
    let mut non_target_missed_at_10_count = 0usize;

    for commit in commits {
        for evidence in &commit.protected_evidence {
            candidate_count += 1;
            if !evidence.selected_at_10 {
                missed_at_10_count += 1;
            }
            if evidence.retrieval_target {
                retrieval_target_candidate_count += 1;
                if !evidence.selected_at_10 {
                    retrieval_target_missed_at_10_count += 1;
                }
            } else {
                non_target_candidate_count += 1;
                if !evidence.selected_at_10 {
                    non_target_missed_at_10_count += 1;
                }
            }
        }
    }

    ProtectedEvidenceSummary {
        candidate_count,
        missed_at_10_count,
        miss_rate_at_10: if candidate_count == 0 {
            0.0
        } else {
            missed_at_10_count as f32 / candidate_count as f32
        },
        retrieval_target_candidate_count,
        retrieval_target_missed_at_10_count,
        retrieval_target_miss_rate_at_10: if retrieval_target_candidate_count == 0 {
            0.0
        } else {
            retrieval_target_missed_at_10_count as f32 / retrieval_target_candidate_count as f32
        },
        non_target_candidate_count,
        non_target_missed_at_10_count,
        by_signal: protected_evidence_signal_order()
            .into_iter()
            .map(|signal| {
                let mut candidate_count = 0usize;
                let mut missed_at_10_count = 0usize;
                let mut retrieval_target_candidate_count = 0usize;
                let mut retrieval_target_missed_at_10_count = 0usize;
                for commit in commits {
                    for evidence in &commit.protected_evidence {
                        if evidence.signals.contains(&signal) {
                            candidate_count += 1;
                            if !evidence.selected_at_10 {
                                missed_at_10_count += 1;
                            }
                            if evidence.retrieval_target {
                                retrieval_target_candidate_count += 1;
                                if !evidence.selected_at_10 {
                                    retrieval_target_missed_at_10_count += 1;
                                }
                            }
                        }
                    }
                }
                ProtectedEvidenceSignalSummary {
                    signal,
                    candidate_count,
                    missed_at_10_count,
                    retrieval_target_candidate_count,
                    retrieval_target_missed_at_10_count,
                }
            })
            .collect(),
    }
}

fn graph_edge_profiles(commits: &[HistoricalCommitEval]) -> Vec<GraphEdgeProfile> {
    let mut by_label = BTreeMap::<String, GraphEdgeProfile>::new();
    for commit in commits {
        for profile in &commit.graph_edge_profiles {
            let entry = by_label
                .entry(profile.edge_label.clone())
                .or_insert_with(|| GraphEdgeProfile {
                    edge_label: profile.edge_label.clone(),
                    candidate_count: 0,
                    selected_at_10_count: 0,
                    retrieval_target_count: 0,
                    retrieval_target_hit_at_10_count: 0,
                    retrieval_target_missed_at_10_count: 0,
                });
            entry.candidate_count += profile.candidate_count;
            entry.selected_at_10_count += profile.selected_at_10_count;
            entry.retrieval_target_count += profile.retrieval_target_count;
            entry.retrieval_target_hit_at_10_count += profile.retrieval_target_hit_at_10_count;
            entry.retrieval_target_missed_at_10_count +=
                profile.retrieval_target_missed_at_10_count;
        }
    }
    let mut profiles = by_label.into_values().collect::<Vec<_>>();
    profiles.sort_by(|left, right| {
        right
            .retrieval_target_hit_at_10_count
            .cmp(&left.retrieval_target_hit_at_10_count)
            .then_with(|| right.candidate_count.cmp(&left.candidate_count))
            .then_with(|| left.edge_label.cmp(&right.edge_label))
    });
    profiles
}

fn graph_edge_profiles_for_commit(
    plan: &ContextPlan,
    recommended_context_files: &[String],
    retrieval_target_paths: &BTreeSet<String>,
    limit: usize,
) -> Vec<GraphEdgeProfile> {
    let mut paths_by_label = BTreeMap::<String, BTreeSet<String>>::new();
    for candidate in &plan.retrieval_candidates {
        let Some(path) = &candidate.path else {
            continue;
        };
        for evidence in &candidate.evidence {
            if evidence.signal != RetrievalSignalKind::Dependency {
                continue;
            }
            let Some(label) = evidence
                .edge_label
                .as_deref()
                .filter(|label| !label.trim().is_empty())
            else {
                continue;
            };
            paths_by_label
                .entry(label.to_string())
                .or_default()
                .insert(path.clone());
        }
    }

    let selected_at_limit = recommended_context_files
        .iter()
        .take(limit.max(1))
        .cloned()
        .collect::<BTreeSet<_>>();
    paths_by_label
        .into_iter()
        .map(|(edge_label, paths)| {
            let candidate_count = paths.len();
            let selected_at_10_count = paths
                .iter()
                .filter(|path| selected_at_limit.contains(path.as_str()))
                .count();
            let retrieval_target_count = paths
                .iter()
                .filter(|path| retrieval_target_paths.contains(path.as_str()))
                .count();
            let retrieval_target_hit_at_10_count = paths
                .iter()
                .filter(|path| {
                    retrieval_target_paths.contains(path.as_str())
                        && selected_at_limit.contains(path.as_str())
                })
                .count();
            GraphEdgeProfile {
                edge_label,
                candidate_count,
                selected_at_10_count,
                retrieval_target_count,
                retrieval_target_hit_at_10_count,
                retrieval_target_missed_at_10_count: retrieval_target_count
                    .saturating_sub(retrieval_target_hit_at_10_count),
            }
        })
        .collect()
}

fn graph_edge_labels_by_path(plan: &ContextPlan) -> BTreeMap<String, BTreeSet<String>> {
    let mut paths_by_label = BTreeMap::<String, BTreeSet<String>>::new();
    for candidate in &plan.retrieval_candidates {
        let Some(path) = &candidate.path else {
            continue;
        };
        for evidence in &candidate.evidence {
            if evidence.signal != RetrievalSignalKind::Dependency {
                continue;
            }
            let Some(label) = evidence
                .edge_label
                .as_deref()
                .filter(|label| !label.trim().is_empty())
            else {
                continue;
            };
            paths_by_label
                .entry(path.clone())
                .or_default()
                .insert(label.to_string());
        }
    }
    paths_by_label
}

fn graph_edge_ablation_rankings_for_commit(
    plan: &ContextPlan,
    recommended_context_files: &[String],
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
    retrieval_target_paths: &BTreeSet<String>,
    limit: usize,
) -> Vec<GraphEdgeAblationCommitRanking> {
    let edge_labels_by_path = graph_edge_labels_by_path(plan);
    let edge_labels = edge_labels_by_path
        .values()
        .flat_map(|labels| labels.iter().cloned())
        .collect::<BTreeSet<_>>();
    let selected_at_limit = recommended_context_files
        .iter()
        .take(limit.max(1))
        .cloned()
        .collect::<BTreeSet<_>>();

    edge_labels
        .into_iter()
        .map(|edge_label| {
            let mut removed_selected_at_10_count = 0usize;
            let mut removed_target_hit_at_10_count = 0usize;
            let ranking = recommended_context_files
                .iter()
                .filter(|path| {
                    let remove = graph_edge_label_is_exclusive_support(
                        path,
                        &edge_label,
                        signals_by_path,
                        &edge_labels_by_path,
                    );
                    if remove && selected_at_limit.contains(path.as_str()) {
                        removed_selected_at_10_count += 1;
                        if retrieval_target_paths.contains(path.as_str()) {
                            removed_target_hit_at_10_count += 1;
                        }
                    }
                    !remove
                })
                .cloned()
                .collect::<Vec<_>>();
            GraphEdgeAblationCommitRanking {
                edge_label,
                ranking,
                affected_commit: removed_selected_at_10_count > 0,
                removed_selected_at_10_count,
                removed_target_hit_at_10_count,
            }
        })
        .collect()
}

fn graph_edge_label_is_exclusive_support(
    path: &str,
    edge_label: &str,
    signals_by_path: &BTreeMap<String, Vec<RetrievalSignalKind>>,
    edge_labels_by_path: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    let Some(labels) = edge_labels_by_path.get(path) else {
        return false;
    };
    if labels.is_empty() || labels.iter().any(|label| label != edge_label) {
        return false;
    }
    let Some(signals) = signals_by_path.get(path) else {
        return false;
    };
    !signals.is_empty()
        && signals
            .iter()
            .all(|signal| signal == &RetrievalSignalKind::Dependency)
}

fn protected_evidence_signal_order() -> Vec<RetrievalSignalKind> {
    vec![
        RetrievalSignalKind::Anchor,
        RetrievalSignalKind::CurrentDiff,
        RetrievalSignalKind::Lexical,
        RetrievalSignalKind::Symbol,
    ]
}

fn is_protected_evidence_signal(signal: &RetrievalSignalKind) -> bool {
    matches!(
        signal,
        RetrievalSignalKind::Anchor
            | RetrievalSignalKind::CurrentDiff
            | RetrievalSignalKind::Lexical
            | RetrievalSignalKind::Symbol
    )
}

fn signal_baseline_signals() -> Vec<RetrievalSignalKind> {
    vec![
        RetrievalSignalKind::Lexical,
        RetrievalSignalKind::LexicalExpansion,
        RetrievalSignalKind::Symbol,
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

fn graph_edge_ablation_results(
    commits: &[HistoricalCommitEval],
    mut edge_ablations: Vec<GraphEdgeAblationRankings>,
    combined_baseline: &RankingMetrics,
    lexical_baseline: &RankingMetrics,
    eval_range_id: &str,
    k: usize,
) -> Vec<GraphEdgeAblationResult> {
    edge_ablations.sort_by(|left, right| left.edge_label.cmp(&right.edge_label));
    edge_ablations
        .into_iter()
        .map(|ablation| {
            let mut ablated_commits = commits.to_vec();
            for (commit, ranking) in ablated_commits
                .iter_mut()
                .zip(ablation.rankings.iter().cloned())
            {
                commit.recommended_context_files = ranking;
            }
            let metrics = ranking_metrics(&ablated_commits, k, RankingFamily::Combined);
            GraphEdgeAblationResult {
                eval_range_id: eval_range_id.to_string(),
                edge_label: ablation.edge_label,
                evaluated_commits: commits.len(),
                affected_commit_count: ablation.affected_commit_count,
                removed_selected_at_10_count: ablation.removed_selected_at_10_count,
                removed_target_hit_at_10_count: ablation.removed_target_hit_at_10_count,
                recall_delta_at_k: metrics.recall_at_k - combined_baseline.recall_at_k,
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
            if commit.retrieval_target_files.is_empty() {
                0.0
            } else {
                let hit_count = if limit <= 5 {
                    commit.file_hits_at_5.len()
                } else {
                    commit.file_hits_at_10.len()
                };
                hit_count as f32 / commit.retrieval_target_files.len() as f32
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
            if commit.retrieval_target_files.is_empty() {
                0.0
            } else {
                let hit_count = if limit <= 5 {
                    commit.lexical_baseline_hits_at_5.len()
                } else {
                    commit.lexical_baseline_hits_at_10.len()
                };
                hit_count as f32 / commit.retrieval_target_files.len() as f32
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

fn average_source_recall_for_ranking<'a>(
    commits: &'a [HistoricalCommitEval],
    ranking: impl Fn(&'a HistoricalCommitEval) -> &'a [String],
) -> f32 {
    let mut total = 0.0;
    let mut count = 0usize;
    for commit in commits {
        let source_changed_files = filter_changed_labels_by_role(
            &commit.changed_path_labels,
            &commit.retrieval_target_files,
            |role| matches!(role, FileRole::Source),
        );
        if source_changed_files.is_empty() {
            continue;
        }
        let hits = changed_file_hits(&source_changed_files, ranking(commit), 10).len();
        total += hits as f32 / source_changed_files.len() as f32;
        count += 1;
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn average_validation_command_recall(commits: &[HistoricalCommitEval]) -> f32 {
    average_test_hit_rate(commits, |commit| commit.validation_command_hits)
}

fn average_effective_validation_recall(commits: &[HistoricalCommitEval]) -> f32 {
    average_test_hit_rate(commits, |commit| commit.effective_validation_hits_at_10)
}

fn average_broad_context_area_recall(commits: &[HistoricalCommitEval]) -> f32 {
    let mut total = 0.0;
    let mut count = 0usize;
    for commit in commits {
        if !commit.broad_scope_task || commit.changed_context_areas.is_empty() {
            continue;
        }
        total += commit.context_area_hits.len() as f32 / commit.changed_context_areas.len() as f32;
        count += 1;
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn context_area_pressure_summary(commits: &[HistoricalCommitEval]) -> ContextAreaPressureSummary {
    let mut summary = ContextAreaPressureSummary {
        source_text_logged: false,
        ..ContextAreaPressureSummary::default()
    };
    for commit in commits {
        for area in &commit.context_areas {
            let breakdown = &area.inspection_pressure_breakdown;
            summary.context_area_count += 1;
            summary.zero_selected_area_count += usize::from(area.selected_count == 0);
            summary.total_inspection_pressure += area.inspection_pressure;
            summary.source_like_unselected += breakdown.source_like_unselected;
            summary.validation_unselected += breakdown.validation_unselected;
            summary.docs_unselected += breakdown.docs_unselected;
            summary.source_like_pressure +=
                breakdown.source_like_unselected * breakdown.source_like_weight;
            summary.validation_pressure +=
                breakdown.validation_unselected * breakdown.validation_weight;
            summary.docs_pressure += breakdown.docs_unselected * breakdown.docs_weight;
            let replace_peak = summary
                .highest_pressure_area
                .as_ref()
                .map(|peak| {
                    area.inspection_pressure > peak.inspection_pressure
                        || (area.inspection_pressure == peak.inspection_pressure
                            && area.area < peak.area)
                })
                .unwrap_or(true);
            if replace_peak {
                summary.highest_pressure_area = Some(ContextAreaPressurePeak {
                    area: area.area.clone(),
                    resource_uri: area.resource_uri.clone(),
                    inspection_pressure: area.inspection_pressure,
                    coverage_percent: area.coverage_percent,
                    unselected_count: area.unselected_count,
                });
            }
        }
    }
    summary
}

fn context_area_next_read_summary(commits: &[HistoricalCommitEval]) -> ContextAreaNextReadSummary {
    let mut summary = ContextAreaNextReadSummary {
        source_text_logged: false,
        ..ContextAreaNextReadSummary::default()
    };
    let mut agent_evidence_only_area_counts = BTreeMap::<String, usize>::new();

    for commit in commits {
        if commit.missing_files_at_10.is_empty() || commit.context_areas.is_empty() {
            summary.missed_file_count_at_10 += commit.missing_files_at_10.len();
            continue;
        }

        let missing = commit
            .missing_files_at_10
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let all_next_reads = commit
            .context_areas
            .iter()
            .flat_map(|area| area.next_read_paths.iter().cloned())
            .collect::<BTreeSet<_>>();
        let agent_evidence = all_next_reads
            .iter()
            .cloned()
            .chain(commit.recommended_context_files.iter().cloned())
            .chain(commit.recommended_tests.iter().cloned())
            .collect::<BTreeSet<_>>();
        let roles_by_path = commit
            .changed_path_labels
            .iter()
            .map(|label| (label.path.clone(), label.role.clone()))
            .collect::<BTreeMap<_, _>>();
        let top_pressure_next_reads = commit
            .context_areas
            .iter()
            .filter(|area| is_top_pressure_context_area(area, &commit.context_areas))
            .flat_map(|area| area.next_read_paths.iter().cloned())
            .collect::<BTreeSet<_>>();
        let zero_selected_next_reads = commit
            .context_areas
            .iter()
            .filter(|area| area.selected_count == 0)
            .flat_map(|area| area.next_read_paths.iter().cloned())
            .collect::<BTreeSet<_>>();

        summary.missed_file_count_at_10 += missing.len();
        summary.next_read_recoverable_count += missing.intersection(&all_next_reads).count();
        summary.agent_evidence_recoverable_count += missing.intersection(&agent_evidence).count();
        for path in missing
            .intersection(&agent_evidence)
            .filter(|path| !all_next_reads.contains(*path))
        {
            summary.agent_evidence_only_count += 1;
            let role = roles_by_path
                .get(path.as_str())
                .cloned()
                .unwrap_or(FileRole::Unknown);
            *summary
                .agent_evidence_only_role_counts
                .entry(file_role_label(&role).to_string())
                .or_insert(0) += 1;
            *agent_evidence_only_area_counts
                .entry(context_area_for_path(path))
                .or_insert(0) += 1;
        }
        summary.top_pressure_next_read_recoverable_count +=
            missing.intersection(&top_pressure_next_reads).count();
        summary.zero_selected_area_recoverable_count +=
            missing.intersection(&zero_selected_next_reads).count();
    }

    let mut top_agent_evidence_only_areas = agent_evidence_only_area_counts
        .into_iter()
        .collect::<Vec<_>>();
    top_agent_evidence_only_areas.sort_by(|(left_area, left_count), (right_area, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_area.cmp(right_area))
    });
    summary.top_agent_evidence_only_areas = top_agent_evidence_only_areas
        .into_iter()
        .take(10)
        .map(
            |(context_area, missed_count)| CandidateCoverageAreaSummary {
                context_area,
                missed_count,
            },
        )
        .collect();
    summary
}

fn candidate_coverage_summary(commits: &[HistoricalCommitEval]) -> CandidateCoverageSummary {
    let mut summary = CandidateCoverageSummary {
        source_text_logged: false,
        ..CandidateCoverageSummary::default()
    };
    let mut area_counts = BTreeMap::<String, usize>::new();
    for commit in commits {
        summary.missed_file_count_at_10 += commit.missing_files_at_10.len();
        summary.candidate_recoverable_count += commit.candidate_missed_files_at_10.len();
        let candidate_paths = commit
            .candidate_missed_files_at_10
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let roles_by_path = commit
            .changed_path_labels
            .iter()
            .map(|label| (label.path.clone(), label.role.clone()))
            .collect::<BTreeMap<_, _>>();
        for profile in &commit.candidate_missed_file_profiles_at_10 {
            *summary
                .candidate_recoverable_role_counts
                .entry(file_role_label(&profile.role).to_string())
                .or_insert(0) += 1;
            *area_counts.entry(profile.context_area.clone()).or_insert(0) += 1;
            for signal in &profile.signals {
                *summary
                    .candidate_recoverable_signal_counts
                    .entry(retrieval_signal_label(signal).to_string())
                    .or_insert(0) += 1;
            }
        }
        for path in &commit.missing_files_at_10 {
            if candidate_paths.contains(path) {
                continue;
            }
            let role = roles_by_path
                .get(path)
                .cloned()
                .unwrap_or(FileRole::Unknown);
            *summary
                .no_candidate_role_counts
                .entry(file_role_label(&role).to_string())
                .or_insert(0) += 1;
        }
    }
    summary.no_candidate_count = summary
        .missed_file_count_at_10
        .saturating_sub(summary.candidate_recoverable_count);
    let mut top_areas = area_counts.into_iter().collect::<Vec<_>>();
    top_areas.sort_by(|(left_area, left_count), (right_area, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_area.cmp(right_area))
    });
    summary.top_candidate_recoverable_areas = top_areas
        .into_iter()
        .take(10)
        .map(
            |(context_area, missed_count)| CandidateCoverageAreaSummary {
                context_area,
                missed_count,
            },
        )
        .collect();
    summary
}

fn memory_reuse_summary(commits: &[HistoricalCommitEval]) -> MemoryReuseSummary {
    let mut summary = MemoryReuseSummary {
        source_text_logged: false,
        ..MemoryReuseSummary::default()
    };
    for commit in commits {
        let memory_files = signal_ranking_for_commit(commit, &RetrievalSignalKind::Memory)
            .into_iter()
            .collect::<BTreeSet<_>>();
        if memory_files.is_empty() {
            continue;
        }
        let target_files = commit
            .retrieval_target_files
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let lexical_files = commit
            .lexical_baseline_files
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let memory_target_hits = memory_files
            .intersection(&target_files)
            .cloned()
            .collect::<BTreeSet<_>>();
        let memory_unique_target_hits = memory_target_hits
            .difference(&lexical_files)
            .cloned()
            .collect::<BTreeSet<_>>();
        let memory_unique_non_targets = memory_files
            .difference(&target_files)
            .filter(|path| !lexical_files.contains(*path))
            .cloned()
            .collect::<BTreeSet<_>>();
        let current_support_files = memory_current_support_files(commit);
        let supported_unique_target_hits = memory_unique_target_hits
            .intersection(&current_support_files)
            .count();
        let unsupported_unique_target_hits = memory_unique_target_hits
            .len()
            .saturating_sub(supported_unique_target_hits);
        let supported_unique_non_targets = memory_unique_non_targets
            .intersection(&current_support_files)
            .count();
        let unsupported_unique_non_targets = memory_unique_non_targets
            .len()
            .saturating_sub(supported_unique_non_targets);

        summary.commits_with_memory_candidates += 1;
        summary.memory_candidate_count += memory_files.len();
        summary.memory_target_hit_at_10_count += memory_target_hits.len();
        summary.memory_target_missed_at_10_count +=
            target_files.difference(&memory_target_hits).count();
        summary.memory_unique_target_hit_count += memory_unique_target_hits.len();
        summary.memory_unique_non_target_count += memory_unique_non_targets.len();
        summary.memory_unique_target_hit_with_current_support_count += supported_unique_target_hits;
        summary.memory_unique_target_hit_without_current_support_count +=
            unsupported_unique_target_hits;
        summary.memory_unique_non_target_with_current_support_count += supported_unique_non_targets;
        summary.memory_unique_non_target_without_current_support_count +=
            unsupported_unique_non_targets;
        add_memory_current_support_signal_counts(
            &mut summary.memory_unique_target_hit_current_support_signal_counts,
            &memory_unique_target_hits,
            commit,
        );
        add_memory_current_support_signal_counts(
            &mut summary.memory_unique_non_target_current_support_signal_counts,
            &memory_unique_non_targets,
            commit,
        );

        for profile in commit
            .selected_signal_profiles
            .iter()
            .filter(|profile| profile.signal == RetrievalSignalKind::Memory)
        {
            summary.memory_selected_at_10_count += profile.selected_at_10_count;
            *summary
                .selected_role_counts
                .entry(file_role_label(&profile.role).to_string())
                .or_insert(0) += profile.selected_at_10_count;
        }
    }
    summary
}

fn add_memory_current_support_signal_counts(
    counts: &mut BTreeMap<String, usize>,
    paths: &BTreeSet<String>,
    commit: &HistoricalCommitEval,
) {
    if paths.is_empty() {
        return;
    }
    let lexical_files = commit
        .lexical_baseline_files
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let lexical_support_count = paths.intersection(&lexical_files).count();
    if lexical_support_count > 0 {
        *counts.entry("lexical".to_string()).or_insert(0) += lexical_support_count;
    }
    for ranking in &commit.signal_baseline_files {
        if ranking.signal == RetrievalSignalKind::Memory {
            continue;
        }
        let signal_label = retrieval_signal_label(&ranking.signal).to_string();
        let signal_support_count = ranking
            .files
            .iter()
            .filter(|path| paths.contains(*path))
            .count();
        if signal_support_count > 0 {
            *counts.entry(signal_label).or_insert(0) += signal_support_count;
        }
    }
}

fn memory_current_support_files(commit: &HistoricalCommitEval) -> BTreeSet<String> {
    commit
        .signal_baseline_files
        .iter()
        .filter(|ranking| ranking.signal != RetrievalSignalKind::Memory)
        .flat_map(|ranking| ranking.files.iter().cloned())
        .chain(commit.lexical_baseline_files.iter().cloned())
        .collect()
}

fn file_role_label(role: &FileRole) -> &'static str {
    match role {
        FileRole::Source => "source",
        FileRole::Test => "test",
        FileRole::Config => "config",
        FileRole::Schema => "schema",
        FileRole::Docs => "docs",
        FileRole::Generated => "generated",
        FileRole::Sensitive => "sensitive",
        FileRole::Unknown => "unknown",
    }
}

fn retrieval_signal_label(signal: &RetrievalSignalKind) -> &'static str {
    match signal {
        RetrievalSignalKind::Lexical => "lexical",
        RetrievalSignalKind::LexicalExpansion => "lexical_expansion",
        RetrievalSignalKind::Symbol => "symbol",
        RetrievalSignalKind::Dependency => "dependency",
        RetrievalSignalKind::RelatedTest => "related_test",
        RetrievalSignalKind::Semantic => "semantic",
        RetrievalSignalKind::CoChange => "co_change",
        RetrievalSignalKind::CurrentDiff => "current_diff",
        RetrievalSignalKind::History => "history",
        RetrievalSignalKind::Docs => "docs",
        RetrievalSignalKind::Config => "config",
        RetrievalSignalKind::Anchor => "anchor",
        RetrievalSignalKind::Memory => "memory",
    }
}

fn is_top_pressure_context_area(area: &ContextArea, context_areas: &[ContextArea]) -> bool {
    let Some(max_pressure) = context_areas
        .iter()
        .map(|candidate| candidate.inspection_pressure)
        .max()
    else {
        return false;
    };
    area.inspection_pressure == max_pressure && max_pressure > 0
}

fn average_test_hit_rate(
    commits: &[HistoricalCommitEval],
    hits: impl Fn(&HistoricalCommitEval) -> usize,
) -> f32 {
    let mut total = 0.0;
    let mut count = 0usize;
    for commit in commits {
        if commit.test_files_changed == 0 {
            continue;
        }
        total += hits(commit) as f32 / commit.test_files_changed as f32;
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
            if commit.retrieval_target_files.is_empty() {
                0.0
            } else {
                ranking_hits(commit, k, family).len() as f32
                    / commit.retrieval_target_files.len() as f32
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
            if commit.retrieval_target_files.is_empty() {
                0.0
            } else {
                changed_file_hits(&commit.retrieval_target_files, &ranking, k).len() as f32
                    / commit.retrieval_target_files.len() as f32
            }
        })
        .sum::<f32>()
        / commits.len() as f32;
    let precision_at_k = commits
        .iter()
        .map(|commit| {
            let ranking = signal_ranking_for_commit(commit, &signal);
            changed_file_hits(&commit.retrieval_target_files, &ranking, k).len() as f32 / k as f32
        })
        .sum::<f32>()
        / commits.len() as f32;
    let mrr_at_k = commits
        .iter()
        .map(|commit| {
            let ranking = signal_ranking_for_commit(commit, &signal);
            reciprocal_rank_for_files(&commit.retrieval_target_files, &ranking, k)
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
        .map(|commit| commit.retrieval_target_files.len())
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
        &commit.retrieval_target_files,
        ranking_for_family(commit, family),
        k,
    )
}

fn reciprocal_rank(commit: &HistoricalCommitEval, k: usize, family: RankingFamily) -> f32 {
    let retrieval_target_files = commit
        .retrieval_target_files
        .iter()
        .collect::<BTreeSet<_>>();
    ranking_for_family(commit, family)
        .iter()
        .take(k)
        .position(|path| retrieval_target_files.contains(path))
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
                            && commit.retrieval_target_files.contains(&label.path)
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
    context_area_hits: &[String],
) -> BTreeMap<String, String> {
    let lexical_paths = lexical_baseline_files.iter().collect::<BTreeSet<_>>();
    let context_area_hits = context_area_hits.iter().collect::<BTreeSet<_>>();
    missing_files
        .iter()
        .map(|path| {
            let reason = if lexical_paths.contains(path) {
                "lexical_only_miss".to_string()
            } else if let Some(signals) = signals_by_path.get(path) {
                format!("ranked_below_budget_{}", signal_family_code(signals))
            } else if context_area_hits.contains(&context_area_for_path(path)) {
                "area_context_only".to_string()
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
    let mut merged_profile_keys = BTreeSet::<String>::new();
    for (commit, gap_reasons) in commits.iter().zip(gap_reasons_by_commit.iter()) {
        let validation_hit_paths = effective_validation_hit_paths(
            &filter_changed_labels_by_role(
                &commit.changed_path_labels,
                &commit.retrieval_target_files,
                |role| matches!(role, FileRole::Test),
            ),
            &commit.recommended_tests,
            &commit.recommended_commands,
        );
        for path in &commit.missing_files_at_10 {
            let role = roles_by_path
                .get(path)
                .cloned()
                .unwrap_or(FileRole::Unknown);
            if role == FileRole::Test && validation_hit_paths.contains(path) {
                continue;
            }
            let signal_gap = gap_reasons
                .get(path)
                .cloned()
                .unwrap_or_else(|| "no_candidate_signal".to_string());
            let package = package_family(path);
            let path_family = path_family(path);
            let context_area = context_area_for_path(path);
            let context_area_resource_uri = context_area_resource_uri(&context_area);
            let context_area_profile = context_area_profile_for_gap(commit, &context_area);
            let target_status = labels_by_path
                .get(path)
                .map(gap_target_status)
                .unwrap_or(RetrievalGapTargetStatus::Unknown);
            let recommendation_area = recommendation_area_for_gap(&signal_gap, role.clone());
            let profile_key = gap_context_area_profile_merge_key(
                commit,
                &role,
                &signal_gap,
                &package,
                &path_family,
                &target_status,
                &recommendation_area,
                &context_area,
            );
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
                if summary.next_read_paths.len() < 8 && !summary.next_read_paths.contains(path) {
                    summary.next_read_paths.push(path.clone());
                }
                if merged_profile_keys.insert(profile_key) {
                    merge_context_area_profile(summary, context_area_profile);
                }
            } else {
                let context_area_profile = context_area_profile_fields(context_area_profile);
                merged_profile_keys.insert(profile_key);
                summaries.push(RetrievalGapSummary {
                    role,
                    signal_gap,
                    package,
                    path_family,
                    context_area,
                    context_area_resource_uri,
                    context_area_signal_counts: context_area_profile.signal_counts,
                    context_area_role_counts: context_area_profile.role_counts,
                    context_area_selected_role_counts: context_area_profile.selected_role_counts,
                    context_area_unselected_count: context_area_profile.unselected_count,
                    context_area_inspection_pressure_breakdown: context_area_profile
                        .inspection_pressure_breakdown,
                    target_status,
                    recommendation_area,
                    missed_count: 1,
                    example_paths: vec![path.clone()],
                    next_read_paths: vec![path.clone()],
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

#[allow(clippy::too_many_arguments)]
fn gap_context_area_profile_merge_key(
    commit: &HistoricalCommitEval,
    role: &FileRole,
    signal_gap: &str,
    package: &str,
    path_family: &str,
    target_status: &RetrievalGapTargetStatus,
    recommendation_area: &RetrievalGapRecommendationArea,
    context_area: &str,
) -> String {
    format!(
        "{}|{:?}|{}|{}|{}|{:?}|{:?}|{}",
        commit.sha,
        role,
        signal_gap,
        package,
        path_family,
        target_status,
        recommendation_area,
        context_area
    )
}

fn context_area_profile_for_gap<'a>(
    commit: &'a HistoricalCommitEval,
    context_area: &str,
) -> Option<&'a ContextArea> {
    commit
        .context_areas
        .iter()
        .find(|area| area.area == context_area)
}

#[derive(Default)]
struct ContextAreaProfileFields {
    signal_counts: BTreeMap<String, usize>,
    role_counts: BTreeMap<String, usize>,
    selected_role_counts: BTreeMap<String, usize>,
    unselected_count: usize,
    inspection_pressure_breakdown: InspectionPressureBreakdown,
}

fn context_area_profile_fields(profile: Option<&ContextArea>) -> ContextAreaProfileFields {
    profile
        .map(|area| ContextAreaProfileFields {
            signal_counts: area.signal_counts.clone(),
            role_counts: area.role_counts.clone(),
            selected_role_counts: area.selected_role_counts.clone(),
            unselected_count: area.unselected_count,
            inspection_pressure_breakdown: area.inspection_pressure_breakdown.clone(),
        })
        .unwrap_or_default()
}

fn merge_context_area_profile(summary: &mut RetrievalGapSummary, profile: Option<&ContextArea>) {
    let Some(profile) = profile else {
        return;
    };
    merge_count_maps(
        &mut summary.context_area_signal_counts,
        &profile.signal_counts,
    );
    merge_count_maps(&mut summary.context_area_role_counts, &profile.role_counts);
    merge_count_maps(
        &mut summary.context_area_selected_role_counts,
        &profile.selected_role_counts,
    );
    summary.context_area_unselected_count += profile.unselected_count;
    merge_inspection_pressure_breakdown(
        &mut summary.context_area_inspection_pressure_breakdown,
        &profile.inspection_pressure_breakdown,
    );
}

fn merge_inspection_pressure_breakdown(
    target: &mut InspectionPressureBreakdown,
    source: &InspectionPressureBreakdown,
) {
    target.source_like_unselected += source.source_like_unselected;
    target.validation_unselected += source.validation_unselected;
    target.docs_unselected += source.docs_unselected;
    target.source_like_weight = source.source_like_weight;
    target.validation_weight = source.validation_weight;
    target.docs_weight = source.docs_weight;
    target.total += source.total;
}

fn inspection_pressure_breakdown_is_empty(breakdown: &InspectionPressureBreakdown) -> bool {
    breakdown.source_like_unselected == 0
        && breakdown.validation_unselected == 0
        && breakdown.docs_unselected == 0
        && breakdown.total == 0
}

fn merge_count_maps(target: &mut BTreeMap<String, usize>, source: &BTreeMap<String, usize>) {
    for (key, value) in source {
        *target.entry(key.clone()).or_default() += value;
    }
}

fn is_zero(value: &usize) -> bool {
    *value == 0
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

fn cached_historical_eval_runtime_summary(
    cached: &HistoricalEvalRuntimeSummary,
    total_millis: u64,
) -> HistoricalEvalRuntimeSummary {
    HistoricalEvalRuntimeSummary {
        total_millis,
        commit_millis: 0,
        overhead_millis: total_millis,
        average_commit_millis: 0.0,
        cache_hits: cached.cache_hits.saturating_add(1),
        cache_misses: 0,
        parallelism: cached.parallelism.max(1),
        git_sample_millis: 0,
        ranking_millis: 0,
        pack_compiler_millis: 0,
        slow_commits: Vec::new(),
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
        ctxhelm_index::ChangeKind::Renamed => RetrievalGapTargetStatus::HistoricalRenamed,
        ctxhelm_index::ChangeKind::Deleted => RetrievalGapTargetStatus::HistoricalDeleted,
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
    if signal_gap == "area_context_only" {
        return RetrievalGapRecommendationArea::ContextPlanning;
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
        RetrievalSignalKind::LexicalExpansion => "lexical_expansion",
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
    use ctxhelm_core::{
        ProviderPolicy, ProviderPolicyReport, RetrievalEvidence, RetrievalSignalScore,
    };

    #[test]
    fn parent_snapshot_command_helper_times_out_instead_of_hanging() {
        let temp = tempfile::tempdir().unwrap();
        let mut command = ProcessCommand::new("sh");
        command.args(["-c", "sleep 2"]);

        let error = command_output_with_timeout(
            temp.path(),
            command,
            "fake parent snapshot command".to_string(),
            Duration::from_millis(20),
        )
        .unwrap_err();
        let message = error.to_string();

        assert!(message.contains("fake parent snapshot command"));
        assert!(message.contains("timed out"));
    }

    #[test]
    fn parent_snapshot_batch_reader_extracts_multiple_paths() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests")).unwrap();
        fs::write(
            repo.join("src/lib.py"),
            "def probe_gate():\n    return 'passed'\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/test_lib.py"),
            "def test_probe_gate():\n    assert True\n",
        )
        .unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "ctxhelm@example.test"],
            vec!["config", "user.name", "ctxhelm"],
            vec!["add", "."],
            vec!["commit", "-m", "seed"],
        ] {
            let status = ProcessCommand::new("git")
                .arg("-C")
                .arg(&repo)
                .args(args)
                .status()
                .unwrap();
            assert!(status.success());
        }
        let revision = String::from_utf8(
            ProcessCommand::new("git")
                .arg("-C")
                .arg(&repo)
                .args(["rev-parse", "HEAD"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        let destination = temp.path().join("snapshot");
        let paths = vec![
            "src/lib.py".to_string(),
            "tests/test_lib.py".to_string(),
            "missing.py".to_string(),
        ];

        let status =
            git_extract_revision_paths(&repo, revision.trim(), &paths, &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("src/lib.py")).unwrap(),
            "def probe_gate():\n    return 'passed'\n"
        );
        assert_eq!(
            fs::read_to_string(destination.join("tests/test_lib.py")).unwrap(),
            "def test_probe_gate():\n    assert True\n"
        );
        assert!(!destination.join("missing.py").exists());
        assert_eq!(status.requested_path_count, 3);
        assert_eq!(status.extracted_path_count, 2);
        assert_eq!(status.failed_batch_count, 0);
        assert!(status.complete());
    }

    #[test]
    fn incomplete_parent_snapshot_manifests_are_not_reusable() {
        let complete = ParentSnapshotManifest {
            manifest_version: PARENT_SNAPSHOT_SCHEMA_VERSION.to_string(),
            parent_sha: "abc123".to_string(),
            requested_path_count: 2,
            extracted_path_count: 2,
            failed_batch_count: 0,
            complete: true,
        };
        assert!(parent_snapshot_manifest_is_reusable(&complete, "abc123"));

        let mut incomplete = complete.clone();
        incomplete.failed_batch_count = 1;
        incomplete.complete = false;
        assert!(!parent_snapshot_manifest_is_reusable(&incomplete, "abc123"));

        let mut wrong_parent = complete.clone();
        wrong_parent.parent_sha = "def456".to_string();
        assert!(!parent_snapshot_manifest_is_reusable(
            &wrong_parent,
            "abc123"
        ));
    }

    #[test]
    fn sidecar_only_parent_snapshot_cache_is_not_reusable() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        fs::create_dir_all(cache.join(".ctxhelm")).unwrap();
        fs::write(cache.join(".ctxhelm/eval-history.json"), "{}").unwrap();

        assert!(!parent_snapshot_cache_is_reusable(&cache, "abc123").unwrap());

        write_parent_snapshot_manifest(
            &cache,
            &ParentSnapshotManifest {
                manifest_version: PARENT_SNAPSHOT_SCHEMA_VERSION.to_string(),
                parent_sha: "abc123".to_string(),
                requested_path_count: 1,
                extracted_path_count: 1,
                failed_batch_count: 0,
                complete: true,
            },
        )
        .unwrap();

        assert!(parent_snapshot_cache_is_reusable(&cache, "abc123").unwrap());
    }

    #[test]
    fn parent_snapshot_candidates_keep_changed_generated_artifacts_but_drop_unrelated_ones() {
        let paths = vec![
            ".planning/e2e/old-proof.md".to_string(),
            ".planning/ROADMAP.md".to_string(),
            ".ctxhelm/e2e/proof.json".to_string(),
            "crates/ctxhelm/src/main.rs".to_string(),
        ];
        let candidates =
            parent_snapshot_candidate_paths(&paths, &[".ctxhelm/e2e/proof.json".to_string()]);

        assert!(!candidates.contains(&".planning/e2e/old-proof.md".to_string()));
        assert!(candidates.contains(&".planning/ROADMAP.md".to_string()));
        assert!(candidates.contains(&".ctxhelm/e2e/proof.json".to_string()));
        assert!(candidates.contains(&"crates/ctxhelm/src/main.rs".to_string()));
    }

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

        let ranking = context_file_ranking(&files, &tests, 10, true);

        assert_eq!(ranking.len(), 10);
        assert!(ranking.contains(&"src/file-7.ts".to_string()));
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
    fn context_ranking_keeps_broad_plans_file_first() {
        let files = (0..10)
            .map(|index| format!("src/file-{index}.ts"))
            .collect::<Vec<_>>();
        let tests = vec![
            "tests/file-0.test.ts".to_string(),
            "tests/file-1.test.ts".to_string(),
            "tests/file-2.test.ts".to_string(),
        ];

        let ranking = context_file_ranking(&files, &tests, 10, false);

        assert_eq!(ranking, files);
    }

    #[test]
    fn gate_decision_promotes_holds_and_blocks_from_measured_variants() {
        let policy = source_free_provider_policy();
        let promote = vec![
            gate_test_variant("ctxhelm_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.47, 0.10),
        ];
        let hold = vec![
            gate_test_variant("ctxhelm_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.41, 0.10),
        ];
        let block = vec![
            gate_test_variant("ctxhelm_default", 0.40, 0.10),
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
            gate_test_variant("ctxhelm_default", 0.40, 0.10),
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

    #[test]
    fn gate_decision_does_not_block_semantic_on_eval_only_reranker_regression() {
        let variants = vec![
            gate_test_variant("ctxhelm_default", 0.40, 0.10),
            gate_test_variant("local_semantic", 0.41, 0.10),
            gate_test_variant("local_metadata_reranked", 0.35, 0.10),
        ];
        let eval_only_regression = vec![SemanticPrecisionNamedCase {
            sha: "abc123".to_string(),
            variant: "local_metadata_reranked".to_string(),
            reason: "eval-only reranker demoted protected evidence".to_string(),
            paths: vec!["src/lib.rs".to_string()],
        }];

        let (decision, reason) = gate_decision_from_variants(
            &variants,
            &eval_only_regression,
            &source_free_provider_policy(),
        );

        assert_eq!(decision, SemanticPrecisionGateDecision::Hold);
        assert!(reason.contains("local semantic recall delta"));
    }

    #[test]
    fn gate_decision_holds_slow_or_token_inefficient_variants() {
        let mut default = gate_test_variant("ctxhelm_default", 0.40, 0.10);
        default.runtime_millis = Some(100);
        default.token_efficiency = Some(1.00);
        let mut slow = gate_test_variant("local_semantic", 0.46, 0.10);
        slow.runtime_millis = Some(250);
        slow.token_efficiency = Some(1.00);
        assert_eq!(
            gate_decision_from_variants(
                &[default.clone(), slow],
                &[],
                &source_free_provider_policy()
            )
            .0,
            SemanticPrecisionGateDecision::Hold
        );

        let mut inefficient = gate_test_variant("local_semantic", 0.46, 0.10);
        inefficient.runtime_millis = Some(110);
        inefficient.token_efficiency = Some(0.80);
        assert_eq!(
            gate_decision_from_variants(
                &[default, inefficient],
                &[],
                &source_free_provider_policy()
            )
            .0,
            SemanticPrecisionGateDecision::Hold
        );
    }

    #[test]
    fn protected_evidence_regressions_name_demoted_default_paths() {
        let mut default = empty_historical_eval_report("same");
        default.commits[0].protected_evidence = vec![HistoricalProtectedEvidenceFile {
            path: "src/exact.ts".to_string(),
            signals: vec![RetrievalSignalKind::Lexical],
            selected_at_10: true,
            retrieval_target: true,
            role: Some(FileRole::Source),
        }];
        let mut variant = empty_historical_eval_report("same");
        variant.commits[0].protected_evidence = vec![HistoricalProtectedEvidenceFile {
            path: "src/exact.ts".to_string(),
            signals: vec![RetrievalSignalKind::Lexical],
            selected_at_10: false,
            retrieval_target: true,
            role: Some(FileRole::Source),
        }];

        let regressions = protected_evidence_regressions(&default, &variant, "local_semantic");

        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].paths, vec!["src/exact.ts".to_string()]);
        assert!(regressions[0].reason.contains("demoted"));
    }

    #[test]
    fn semantic_contribution_summary_counts_semantic_only_target_hits() {
        let mut report = empty_historical_eval_report("semantic");
        report.commits[0].query_trace =
            Some(query_trace_with_facets(vec![QueryFacetKind::CommitClue]));
        report.commits[0].retrieval_target_files = vec![
            "src/semantic_only.ts".to_string(),
            "src/shared.ts".to_string(),
            "src/missed.ts".to_string(),
        ];
        report.commits[0].lexical_baseline_files = vec![
            "src/shared.ts".to_string(),
            "src/lexical_only.ts".to_string(),
        ];
        report.commits[0].signal_baseline_files = vec![HistoricalSignalRanking {
            signal: RetrievalSignalKind::Semantic,
            files: vec![
                "src/semantic_only.ts".to_string(),
                "src/shared.ts".to_string(),
                "src/noise.ts".to_string(),
            ],
        }];

        let summary = semantic_contribution_summary(&report);

        assert_eq!(summary.evaluated_commits, 1);
        assert_eq!(summary.commits_with_semantic_selection, 1);
        assert_eq!(summary.semantic_selected_file_count, 3);
        assert_eq!(summary.semantic_target_hit_count, 2);
        assert_eq!(summary.semantic_only_target_hit_count, 1);
        assert_eq!(summary.semantic_only_non_target_count, 1);
        assert_eq!(summary.semantic_lexical_overlap_count, 1);
        assert_eq!(summary.semantic_missed_target_count, 1);
        assert_eq!(summary.average_semantic_selected_files, 3.0);
        assert!((summary.semantic_target_hit_rate - 2.0 / 3.0).abs() < f32::EPSILON);
        assert!((summary.semantic_only_target_hit_rate - 1.0 / 3.0).abs() < f32::EPSILON);
        assert!((summary.semantic_only_non_target_rate - 0.5).abs() < f32::EPSILON);
        assert_eq!(summary.semantic_only_hits.len(), 1);
        assert_eq!(
            summary.semantic_only_hits[0].paths,
            vec!["src/semantic_only.ts".to_string()]
        );
        assert_eq!(summary.semantic_only_non_targets.len(), 1);
        assert_eq!(
            summary.semantic_only_non_targets[0].paths,
            vec!["src/noise.ts".to_string()]
        );
        assert_eq!(summary.semantic_missed_target_gap_families.len(), 1);
        assert_eq!(
            summary.semantic_missed_target_gap_families[0].signal_gap,
            "semantic_miss_no_candidate_signal"
        );
        assert_eq!(
            summary.semantic_missed_target_gap_families[0].missed_count,
            1
        );
        assert_eq!(
            summary.semantic_missed_target_gap_families[0].example_paths,
            vec!["src/missed.ts".to_string()]
        );
        assert_eq!(summary.query_family_contributions.len(), 1);
        let family = &summary.query_family_contributions[0];
        assert_eq!(family.family, "commit_clue");
        assert_eq!(family.evaluated_commits, 1);
        assert_eq!(family.commits_with_semantic_selection, 1);
        assert_eq!(family.semantic_selected_file_count, 3);
        assert_eq!(family.semantic_target_hit_count, 2);
        assert_eq!(family.semantic_only_target_hit_count, 1);
        assert_eq!(family.semantic_only_non_target_count, 1);
        assert_eq!(family.semantic_missed_target_count, 1);
        assert!((family.semantic_only_target_hit_rate - 1.0 / 3.0).abs() < f32::EPSILON);
        assert!((family.semantic_only_non_target_rate - 0.5).abs() < f32::EPSILON);
        assert_eq!(family.missed_target_gap_families.len(), 1);
        assert_eq!(
            family.missed_target_gap_families[0].signal_gap,
            "semantic_miss_no_candidate_signal"
        );
        assert_eq!(family.example_cases.len(), 2);
    }

    #[test]
    fn reranker_contribution_summary_counts_target_hit_deltas() {
        let mut default = empty_historical_eval_report("rerank");
        default.commits[0].file_hits_at_10 = vec![
            "src/shared.rs".to_string(),
            "src/default_only.rs".to_string(),
        ];
        default.protected_evidence.miss_rate_at_10 = 0.50;
        default.protected_evidence.retrieval_target_miss_rate_at_10 = 0.25;

        let mut reranked = empty_historical_eval_report("rerank");
        reranked.commits[0].file_hits_at_10 = vec![
            "src/shared.rs".to_string(),
            "src/reranker_only.rs".to_string(),
            "src/reranker_extra.rs".to_string(),
        ];
        reranked.protected_evidence.miss_rate_at_10 = 0.10;
        reranked.protected_evidence.retrieval_target_miss_rate_at_10 = 0.0;

        let summary = reranker_contribution_summary(&default, &reranked);

        assert_eq!(summary.evaluated_commits, 1);
        assert_eq!(summary.improved_commit_count, 1);
        assert_eq!(summary.regressed_commit_count, 0);
        assert_eq!(summary.neutral_commit_count, 0);
        assert_eq!(summary.default_target_hit_count, 2);
        assert_eq!(summary.reranked_target_hit_count, 3);
        assert_eq!(summary.target_hit_delta, 1);
        assert_eq!(summary.reranker_only_target_hit_count, 2);
        assert_eq!(summary.default_only_target_hit_count, 1);
        assert_eq!(
            summary.improved_cases[0].paths,
            vec!["src/reranker_extra.rs", "src/reranker_only.rs"]
        );
        assert_eq!(
            summary.default_only_cases[0].paths,
            vec!["src/default_only.rs"]
        );
        assert!((summary.protected_evidence_miss_rate_delta + 0.40).abs() < 0.001);
        assert!((summary.protected_evidence_target_miss_rate_delta + 0.25).abs() < 0.001);
    }

    #[test]
    fn reranker_contribution_groups_route_candidates_by_query_family() {
        let mut default = empty_historical_eval_report("route-family");
        default.commits[0].query_trace =
            Some(query_trace_with_facets(vec![QueryFacetKind::Symbol]));
        default.commits[0].file_hits_at_10 = vec!["src/shared.rs".to_string()];

        let mut reranked = empty_historical_eval_report("route-family");
        reranked.commits[0].query_trace = default.commits[0].query_trace.clone();
        reranked.commits[0].file_hits_at_10 = vec![
            "src/shared.rs".to_string(),
            "src/reranker_only.rs".to_string(),
        ];

        let summary = reranker_contribution_summary(&default, &reranked);

        assert_eq!(summary.query_family_contributions.len(), 1);
        let family = &summary.query_family_contributions[0];
        assert_eq!(family.family, "symbol_identifier");
        assert_eq!(family.target_hit_delta, 1);
        assert_eq!(
            family.routing_recommendation,
            RerankerRoutingRecommendation::RouteCandidate
        );
        assert_eq!(family.default_only_target_hit_count, 0);
    }

    #[test]
    fn reranker_contribution_holds_churned_winning_query_family() {
        let mut default = empty_historical_eval_report("churn-family");
        default.commits[0].query_trace =
            Some(query_trace_with_facets(vec![QueryFacetKind::ExplicitPath]));
        default.commits[0].file_hits_at_10 = vec![
            "src/shared.rs".to_string(),
            "src/default_only.rs".to_string(),
        ];

        let mut reranked = empty_historical_eval_report("churn-family");
        reranked.commits[0].query_trace = default.commits[0].query_trace.clone();
        reranked.commits[0].file_hits_at_10 = vec![
            "src/shared.rs".to_string(),
            "src/reranker_only.rs".to_string(),
            "src/reranker_extra.rs".to_string(),
        ];

        let summary = reranker_contribution_summary(&default, &reranked);

        let family = &summary.query_family_contributions[0];
        assert_eq!(family.family, "explicit_path");
        assert_eq!(family.target_hit_delta, 1);
        assert_eq!(family.default_only_target_hit_count, 1);
        assert_eq!(
            family.routing_recommendation,
            RerankerRoutingRecommendation::HoldChurn
        );
        assert_eq!(
            family.example_cases[0].paths,
            vec!["src/reranker_extra.rs", "src/reranker_only.rs"]
        );
    }

    #[test]
    fn query_family_routed_reranker_only_enables_clean_candidate_families() {
        assert!(!query_family_routed_reranker_enabled_for_family(
            "symbol_identifier"
        ));
        assert!(query_family_routed_reranker_enabled_for_family(
            "commit_clue"
        ));
        assert!(!query_family_routed_reranker_enabled_for_family(
            "domain_phrase"
        ));
        assert!(!query_family_routed_reranker_enabled_for_family(
            "explicit_path"
        ));
        assert!(!query_family_routed_reranker_enabled_for_family(
            "broad_scope"
        ));
    }

    #[test]
    fn routed_reranker_diagnostics_report_clean_lift_without_churn() {
        let summary = RerankerContributionSummary {
            evaluated_commits: 2,
            improved_commit_count: 2,
            regressed_commit_count: 0,
            neutral_commit_count: 0,
            default_target_hit_count: 2,
            reranked_target_hit_count: 4,
            target_hit_delta: 2,
            reranker_only_target_hit_count: 2,
            default_only_target_hit_count: 0,
            improved_cases: vec![SemanticPrecisionNamedCase {
                sha: "abc123".to_string(),
                variant: "query_family_routed_reranked".to_string(),
                reason: "routed lift".to_string(),
                paths: vec!["src/lift.rs".to_string()],
            }],
            ..RerankerContributionSummary::default()
        };

        let diagnostics = routed_reranker_contribution_diagnostics(&summary);

        assert_eq!(diagnostics[0].code, "routed_reranker_clean_lift");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Info);
        assert_eq!(diagnostics[0].paths, vec!["src/lift.rs".to_string()]);
    }

    #[test]
    fn reranker_contribution_diagnostics_warn_on_target_loss() {
        let summary = RerankerContributionSummary {
            evaluated_commits: 2,
            improved_commit_count: 0,
            regressed_commit_count: 1,
            neutral_commit_count: 1,
            default_target_hit_count: 2,
            reranked_target_hit_count: 1,
            target_hit_delta: -1,
            default_only_target_hit_count: 1,
            regressed_cases: vec![SemanticPrecisionNamedCase {
                sha: "abc123".to_string(),
                variant: "local_metadata_reranked".to_string(),
                reason: "lost target".to_string(),
                paths: vec!["src/lost.rs".to_string()],
            }],
            ..RerankerContributionSummary::default()
        };

        let diagnostics = reranker_contribution_diagnostics(&summary);

        assert_eq!(
            diagnostics[0].code,
            "reranker_contribution_regressed_commits"
        );
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[0].paths, vec!["src/lost.rs".to_string()]);
    }

    #[test]
    fn graph_edge_profiles_count_edge_family_hits_and_misses() {
        let mut plan = crate::planning::empty_plan_for_task(TaskType::BugFix);
        plan.retrieval_candidates = vec![
            RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/hit.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "dependency_neighbor".to_string(),
                confidence: 0.8,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Dependency,
                    score: 0.8,
                    weight: 0.75,
                }],
                evidence: vec![RetrievalEvidence {
                    signal: RetrievalSignalKind::Dependency,
                    score: 0.8,
                    reason_code: "dependency_neighbor".to_string(),
                    path: Some("src/hit.rs".to_string()),
                    role: Some(FileRole::Source),
                    edge_label: Some("imports".to_string()),
                    commit_ids: Vec::new(),
                    commit_count: 0,
                }],
            },
            RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/miss.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "dependency_neighbor".to_string(),
                confidence: 0.7,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Dependency,
                    score: 0.7,
                    weight: 0.75,
                }],
                evidence: vec![RetrievalEvidence {
                    signal: RetrievalSignalKind::Dependency,
                    score: 0.7,
                    reason_code: "dependency_neighbor".to_string(),
                    path: Some("src/miss.rs".to_string()),
                    role: Some(FileRole::Source),
                    edge_label: Some("precision:calls".to_string()),
                    commit_ids: Vec::new(),
                    commit_count: 0,
                }],
            },
        ];
        let targets = ["src/hit.rs".to_string(), "src/miss.rs".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();

        let profiles =
            graph_edge_profiles_for_commit(&plan, &["src/hit.rs".to_string()], &targets, 10);

        let imports = profiles
            .iter()
            .find(|profile| profile.edge_label == "imports")
            .expect("imports edge profile");
        assert_eq!(imports.candidate_count, 1);
        assert_eq!(imports.selected_at_10_count, 1);
        assert_eq!(imports.retrieval_target_count, 1);
        assert_eq!(imports.retrieval_target_hit_at_10_count, 1);
        assert_eq!(imports.retrieval_target_missed_at_10_count, 0);

        let precision = profiles
            .iter()
            .find(|profile| profile.edge_label == "precision:calls")
            .expect("precision edge profile");
        assert_eq!(precision.candidate_count, 1);
        assert_eq!(precision.selected_at_10_count, 0);
        assert_eq!(precision.retrieval_target_count, 1);
        assert_eq!(precision.retrieval_target_hit_at_10_count, 0);
        assert_eq!(precision.retrieval_target_missed_at_10_count, 1);
    }

    #[test]
    fn graph_edge_ablation_only_removes_exclusive_edge_supported_files() {
        let mut plan = crate::planning::empty_plan_for_task(TaskType::BugFix);
        plan.retrieval_candidates = vec![
            RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/import_only.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "dependency_neighbor".to_string(),
                confidence: 0.8,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Dependency,
                    score: 0.8,
                    weight: 0.75,
                }],
                evidence: vec![RetrievalEvidence {
                    signal: RetrievalSignalKind::Dependency,
                    score: 0.8,
                    reason_code: "dependency_neighbor".to_string(),
                    path: Some("src/import_only.rs".to_string()),
                    role: Some(FileRole::Source),
                    edge_label: Some("imports".to_string()),
                    commit_ids: Vec::new(),
                    commit_count: 0,
                }],
            },
            RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/lexical_and_import.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "lexical_match".to_string(),
                confidence: 0.9,
                signal_scores: vec![
                    RetrievalSignalScore {
                        signal: RetrievalSignalKind::Lexical,
                        score: 0.9,
                        weight: 1.0,
                    },
                    RetrievalSignalScore {
                        signal: RetrievalSignalKind::Dependency,
                        score: 0.4,
                        weight: 0.75,
                    },
                ],
                evidence: vec![
                    RetrievalEvidence {
                        signal: RetrievalSignalKind::Lexical,
                        score: 0.9,
                        reason_code: "lexical_match".to_string(),
                        path: Some("src/lexical_and_import.rs".to_string()),
                        role: Some(FileRole::Source),
                        edge_label: None,
                        commit_ids: Vec::new(),
                        commit_count: 0,
                    },
                    RetrievalEvidence {
                        signal: RetrievalSignalKind::Dependency,
                        score: 0.4,
                        reason_code: "dependency_neighbor".to_string(),
                        path: Some("src/lexical_and_import.rs".to_string()),
                        role: Some(FileRole::Source),
                        edge_label: Some("imports".to_string()),
                        commit_ids: Vec::new(),
                        commit_count: 0,
                    },
                ],
            },
        ];
        let recommended = vec![
            "src/import_only.rs".to_string(),
            "src/lexical_and_import.rs".to_string(),
        ];
        let targets = recommended.iter().cloned().collect::<BTreeSet<_>>();
        let signals = signals_by_path(&plan);

        let ablations =
            graph_edge_ablation_rankings_for_commit(&plan, &recommended, &signals, &targets, 10);
        let imports = ablations
            .iter()
            .find(|ablation| ablation.edge_label == "imports")
            .expect("imports ablation");

        assert_eq!(
            imports.ranking,
            vec!["src/lexical_and_import.rs".to_string()]
        );
        assert!(imports.affected_commit);
        assert_eq!(imports.removed_selected_at_10_count, 1);
        assert_eq!(imports.removed_target_hit_at_10_count, 1);
    }

    #[test]
    fn semantic_contribution_diagnostics_explain_no_unique_hits_or_candidates() {
        let no_unique = SemanticContributionSummary {
            evaluated_commits: 1,
            commits_with_semantic_selection: 1,
            semantic_selected_file_count: 2,
            semantic_target_hit_count: 1,
            semantic_only_target_hit_count: 0,
            semantic_only_non_target_count: 1,
            semantic_lexical_overlap_count: 2,
            semantic_missed_target_count: 0,
            average_semantic_selected_files: 2.0,
            semantic_target_hit_rate: 1.0,
            semantic_only_target_hit_rate: 0.0,
            semantic_only_non_target_rate: 1.0,
            semantic_only_hits: Vec::new(),
            semantic_only_non_targets: vec![SemanticPrecisionNamedCase {
                sha: "abc123".to_string(),
                variant: "local_semantic".to_string(),
                reason: "noise".to_string(),
                paths: vec!["src/noise.ts".to_string()],
            }],
            semantic_missed_target_gap_families: Vec::new(),
            query_family_contributions: Vec::new(),
        };
        let diagnostics = semantic_contribution_diagnostics(&no_unique, "local_fastembed");
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0].code,
            "semantic_contribution_no_unique_target_hits"
        );
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Info);
        assert_eq!(
            diagnostics[1].code,
            "semantic_contribution_unique_non_targets"
        );
        assert_eq!(diagnostics[1].paths, vec!["src/noise.ts".to_string()]);

        let no_candidates = SemanticContributionSummary {
            evaluated_commits: 1,
            ..SemanticContributionSummary::default()
        };
        let diagnostics = semantic_contribution_diagnostics(&no_candidates, "local_fastembed");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "semantic_contribution_no_candidates");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    }

    #[test]
    fn semantic_contribution_diagnostics_classify_missed_target_gap_families() {
        let summary = SemanticContributionSummary {
            evaluated_commits: 1,
            commits_with_semantic_selection: 1,
            semantic_selected_file_count: 2,
            semantic_target_hit_count: 0,
            semantic_only_target_hit_count: 0,
            semantic_only_non_target_count: 0,
            semantic_lexical_overlap_count: 0,
            semantic_missed_target_count: 2,
            average_semantic_selected_files: 2.0,
            semantic_target_hit_rate: 0.0,
            semantic_only_target_hit_rate: 0.0,
            semantic_only_non_target_rate: 0.0,
            semantic_only_hits: Vec::new(),
            semantic_only_non_targets: Vec::new(),
            semantic_missed_target_gap_families: vec![
                SemanticMissedTargetGapFamily {
                    signal_gap: "semantic_miss_nonsemantic_coupled_signal".to_string(),
                    missed_count: 1,
                    example_paths: vec!["src/coupled.ts".to_string()],
                },
                SemanticMissedTargetGapFamily {
                    signal_gap: "semantic_miss_no_candidate_signal".to_string(),
                    missed_count: 1,
                    example_paths: vec!["src/no-signal.ts".to_string()],
                },
            ],
            query_family_contributions: Vec::new(),
        };

        let diagnostics = semantic_contribution_diagnostics(&summary, "local_fastembed");

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0].code,
            "semantic_contribution_missed_targets_coupled"
        );
        assert_eq!(diagnostics[0].paths, vec!["src/coupled.ts".to_string()]);
        assert_eq!(
            diagnostics[1].code,
            "semantic_contribution_missed_targets_no_signal"
        );
        assert_eq!(diagnostics[1].severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[1].paths, vec!["src/no-signal.ts".to_string()]);
    }

    #[test]
    fn semantic_contribution_diagnostics_classify_query_family_routes() {
        let summary = SemanticContributionSummary {
            evaluated_commits: 3,
            commits_with_semantic_selection: 3,
            semantic_selected_file_count: 6,
            semantic_only_target_hit_count: 3,
            semantic_only_non_target_count: 3,
            query_family_contributions: vec![
                SemanticQueryFamilyContribution {
                    family: "domain_phrase".to_string(),
                    semantic_only_target_hit_count: 2,
                    semantic_only_non_target_count: 0,
                    example_cases: vec![SemanticPrecisionNamedCase {
                        sha: "abc123".to_string(),
                        variant: "local_semantic".to_string(),
                        reason: "target".to_string(),
                        paths: vec!["src/domain_target.ts".to_string()],
                    }],
                    ..SemanticQueryFamilyContribution::default()
                },
                SemanticQueryFamilyContribution {
                    family: "symbol_identifier".to_string(),
                    semantic_only_target_hit_count: 1,
                    semantic_only_non_target_count: 2,
                    example_cases: vec![SemanticPrecisionNamedCase {
                        sha: "def456".to_string(),
                        variant: "local_semantic".to_string(),
                        reason: "mixed".to_string(),
                        paths: vec![
                            "src/symbol_target.ts".to_string(),
                            "src/symbol_noise.ts".to_string(),
                        ],
                    }],
                    ..SemanticQueryFamilyContribution::default()
                },
                SemanticQueryFamilyContribution {
                    family: "commit_clue".to_string(),
                    semantic_only_target_hit_count: 0,
                    semantic_only_non_target_count: 1,
                    example_cases: vec![SemanticPrecisionNamedCase {
                        sha: "fedcba".to_string(),
                        variant: "local_semantic".to_string(),
                        reason: "noise".to_string(),
                        paths: vec!["src/commit_noise.ts".to_string()],
                    }],
                    ..SemanticQueryFamilyContribution::default()
                },
            ],
            ..SemanticContributionSummary::default()
        };

        let diagnostics = semantic_contribution_diagnostics(&summary, "local_fastembed");

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code
            == "semantic_query_family_route_candidate"
            && diagnostic.message.contains("domain_phrase")
            && diagnostic.paths == vec!["src/domain_target.ts".to_string()]));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code
            == "semantic_query_family_mixed_hold"
            && diagnostic.message.contains("symbol_identifier")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code
            == "semantic_query_family_noise_hold"
            && diagnostic.severity == DiagnosticSeverity::Warning
            && diagnostic.message.contains("commit_clue")));
    }

    #[test]
    fn memory_reuse_summary_counts_unique_memory_target_hits() {
        let mut report = empty_historical_eval_report("memory");
        let commit = &mut report.commits[0];
        commit.retrieval_target_files = vec![
            "src/payments/handler.ts".to_string(),
            "src/payments/signature.ts".to_string(),
        ];
        commit.lexical_baseline_files = vec!["src/payments/signature.ts".to_string()];
        commit.signal_baseline_files = vec![
            HistoricalSignalRanking {
                signal: RetrievalSignalKind::Memory,
                files: vec![
                    "src/payments/handler.ts".to_string(),
                    "docs/checkout.md".to_string(),
                ],
            },
            HistoricalSignalRanking {
                signal: RetrievalSignalKind::Dependency,
                files: vec!["src/payments/handler.ts".to_string()],
            },
            HistoricalSignalRanking {
                signal: RetrievalSignalKind::LexicalExpansion,
                files: vec!["docs/checkout.md".to_string()],
            },
        ];
        commit.selected_signal_profiles = vec![
            HistoricalSelectedSignalProfile {
                signal: RetrievalSignalKind::Memory,
                role: FileRole::Source,
                selected_at_10_count: 1,
                retrieval_target_selected_at_10_count: 1,
            },
            HistoricalSelectedSignalProfile {
                signal: RetrievalSignalKind::Memory,
                role: FileRole::Docs,
                selected_at_10_count: 1,
                retrieval_target_selected_at_10_count: 0,
            },
        ];

        let summary = memory_reuse_summary(&report.commits);

        assert_eq!(summary.commits_with_memory_candidates, 1);
        assert_eq!(summary.memory_candidate_count, 2);
        assert_eq!(summary.memory_selected_at_10_count, 2);
        assert_eq!(summary.memory_target_hit_at_10_count, 1);
        assert_eq!(summary.memory_target_missed_at_10_count, 1);
        assert_eq!(summary.memory_unique_target_hit_count, 1);
        assert_eq!(summary.memory_unique_non_target_count, 1);
        assert_eq!(
            summary.memory_unique_target_hit_with_current_support_count,
            1
        );
        assert_eq!(
            summary.memory_unique_target_hit_without_current_support_count,
            0
        );
        assert_eq!(
            summary.memory_unique_non_target_with_current_support_count,
            1
        );
        assert_eq!(
            summary.memory_unique_non_target_without_current_support_count,
            0
        );
        assert_eq!(
            summary.memory_unique_target_hit_current_support_signal_counts["dependency"],
            1
        );
        assert_eq!(
            summary.memory_unique_non_target_current_support_signal_counts["lexical_expansion"],
            1
        );
        assert_eq!(summary.selected_role_counts["source"], 1);
        assert_eq!(summary.selected_role_counts["docs"], 1);
        assert!(!summary.source_text_logged);

        report.memory_reuse_summary = summary;
        let actions = historical_recommended_research_actions(&report);
        assert!(actions
            .iter()
            .any(|action| action.action == "evaluate_memory_reuse_lift"
                && action.origin == "historical_eval"));
    }

    #[test]
    fn protected_evidence_summary_counts_misses_by_signal() {
        let commits = vec![HistoricalCommitEval {
            sha: "abc123".to_string(),
            task_hash: "task".to_string(),
            task_type: TaskType::BugFix,
            target_agent: "generic".to_string(),
            changed_path_labels: Vec::new(),
            safe_changed_files: Vec::new(),
            retrieval_target_files: Vec::new(),
            excluded_changed_file_count: 0,
            recommended_files: Vec::new(),
            recommended_tests: Vec::new(),
            recommended_context_files: Vec::new(),
            recommended_commands: Vec::new(),
            lexical_baseline_files: Vec::new(),
            signal_baseline_files: Vec::new(),
            selected_signal_profiles: Vec::new(),
            protected_evidence: vec![
                HistoricalProtectedEvidenceFile {
                    path: "src/exact.ts".to_string(),
                    signals: vec![RetrievalSignalKind::Lexical],
                    selected_at_10: true,
                    retrieval_target: true,
                    role: Some(FileRole::Source),
                },
                HistoricalProtectedEvidenceFile {
                    path: "src/symbol.ts".to_string(),
                    signals: vec![RetrievalSignalKind::Symbol],
                    selected_at_10: false,
                    retrieval_target: false,
                    role: Some(FileRole::Source),
                },
            ],
            graph_edge_profiles: Vec::new(),
            file_hits_at_5: Vec::new(),
            file_hits_at_10: Vec::new(),
            lexical_baseline_hits_at_5: Vec::new(),
            lexical_baseline_hits_at_10: Vec::new(),
            missing_files_at_10: Vec::new(),
            candidate_missed_files_at_10: Vec::new(),
            candidate_missed_file_profiles_at_10: Vec::new(),
            source_files_changed: 0,
            source_hits_at_5: 0,
            source_hits_at_10: 0,
            test_files_changed: 0,
            test_hits_at_5: 0,
            test_hits_at_10: 0,
            validation_command_hits: 0,
            effective_validation_hits_at_10: 0,
            low_information_task: false,
            broad_scope_task: false,
            changed_context_areas: Vec::new(),
            context_area_hits: Vec::new(),
            context_areas: Vec::new(),
            confidence: 0.5,
            query_trace: None,
            elapsed_millis: 0,
            source_text_logged: false,
        }];

        let summary = protected_evidence_summary(&commits);

        assert_eq!(summary.candidate_count, 2);
        assert_eq!(summary.missed_at_10_count, 1);
        assert_eq!(summary.miss_rate_at_10, 0.5);
        assert_eq!(summary.retrieval_target_candidate_count, 1);
        assert_eq!(summary.retrieval_target_missed_at_10_count, 0);
        assert_eq!(summary.retrieval_target_miss_rate_at_10, 0.0);
        assert_eq!(summary.non_target_candidate_count, 1);
        assert_eq!(summary.non_target_missed_at_10_count, 1);
        assert_eq!(
            summary
                .by_signal
                .iter()
                .find(|row| row.signal == RetrievalSignalKind::Symbol)
                .unwrap()
                .missed_at_10_count,
            1
        );
        assert_eq!(
            summary
                .by_signal
                .iter()
                .find(|row| row.signal == RetrievalSignalKind::Lexical)
                .unwrap()
                .retrieval_target_candidate_count,
            1
        );
    }

    #[test]
    fn protected_evidence_files_labels_target_status_and_role() {
        let mut signals_by_path = BTreeMap::new();
        signals_by_path.insert(
            "src/protected.ts".to_string(),
            vec![RetrievalSignalKind::Lexical],
        );
        signals_by_path.insert(
            "src/expanded.ts".to_string(),
            vec![RetrievalSignalKind::Lexical],
        );
        let protected_budget_paths = BTreeSet::from(["src/protected.ts".to_string()]);
        let retrieval_target_paths = BTreeSet::from(["src/protected.ts".to_string()]);
        let candidate_roles_by_path =
            BTreeMap::from([("src/protected.ts".to_string(), FileRole::Source)]);

        let files = protected_evidence_files(
            &signals_by_path,
            &protected_budget_paths,
            &["src/protected.ts".to_string()],
            &retrieval_target_paths,
            &candidate_roles_by_path,
            10,
        );

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/protected.ts");
        assert!(files[0].selected_at_10);
        assert!(files[0].retrieval_target);
        assert_eq!(files[0].role, Some(FileRole::Source));
    }

    #[test]
    fn local_metadata_reranker_preserves_protected_source_floor() {
        let mut plan = crate::planning::empty_plan_for_task(TaskType::BugFix);
        plan.retrieval_candidates = vec![RetrievalCandidate {
            kind: RetrievalCandidateKind::File,
            path: Some("src/exact.rs".to_string()),
            role: Some(FileRole::Source),
            reason_code: "lexical_match".to_string(),
            confidence: 0.1,
            signal_scores: vec![RetrievalSignalScore {
                signal: RetrievalSignalKind::Lexical,
                score: 0.1,
                weight: 1.0,
            }],
            evidence: Vec::new(),
        }];
        for index in 0..12 {
            plan.retrieval_candidates.push(RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some(format!("src/semantic_{index}.rs")),
                role: Some(FileRole::Source),
                reason_code: "semantic_neighbor".to_string(),
                confidence: 0.95,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Semantic,
                    score: 0.95,
                    weight: 1.0,
                }],
                evidence: vec![RetrievalEvidence {
                    signal: RetrievalSignalKind::Semantic,
                    score: 0.95,
                    reason_code: "semantic_neighbor".to_string(),
                    path: Some(format!("src/semantic_{index}.rs")),
                    role: Some(FileRole::Source),
                    edge_label: None,
                    commit_ids: Vec::new(),
                    commit_count: 0,
                }],
            });
        }

        let default_context_files = vec!["src/semantic_0.rs".to_string()];
        let ranking =
            local_metadata_reranked_context_files(&plan, 10, &default_context_files, true);

        assert_eq!(ranking.first(), Some(&"src/exact.rs".to_string()));
        assert!(ranking.contains(&"src/exact.rs".to_string()));
        assert_eq!(ranking.len(), 10);
    }

    #[test]
    fn local_metadata_protected_floor_excludes_tests() {
        let mut plan = crate::planning::empty_plan_for_task(TaskType::BugFix);
        plan.target_files = vec![ctxhelm_core::TargetFile {
            path: "src/exact.rs".to_string(),
            reason: "symbol match".to_string(),
            line_range: None,
            confidence: 0.2,
            attribution: Vec::new(),
        }];
        plan.related_tests = vec![ctxhelm_core::RelatedTest {
            path: "tests/exact_test.rs".to_string(),
            confidence: 0.99,
            command: Some("cargo test exact_test".to_string()),
            reason: "lexical test match".to_string(),
            attribution: Vec::new(),
        }];
        plan.retrieval_candidates = vec![
            RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("tests/exact_test.rs".to_string()),
                role: Some(FileRole::Test),
                reason_code: "lexical_match".to_string(),
                confidence: 0.99,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Lexical,
                    score: 0.99,
                    weight: 1.0,
                }],
                evidence: Vec::new(),
            },
            RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/exact.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "symbol_match".to_string(),
                confidence: 0.2,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Symbol,
                    score: 0.2,
                    weight: 1.0,
                }],
                evidence: Vec::new(),
            },
        ];

        let default_context_files = vec![
            "tests/exact_test.rs".to_string(),
            "src/exact.rs".to_string(),
        ];
        let floor = local_metadata_protected_floor_paths(&plan, &default_context_files, 10);
        let ranking =
            local_metadata_reranked_context_files(&plan, 10, &default_context_files, true);

        assert_eq!(floor, vec!["src/exact.rs".to_string()]);
        assert!(ranking.contains(&"tests/exact_test.rs".to_string()));
    }

    #[test]
    fn selected_signal_profiles_count_roles_signals_and_target_hits() {
        let recommended = vec![
            ".planning/STATE.md".to_string(),
            "src/lib.rs".to_string(),
            "src/history.rs".to_string(),
            "scripts/release.sh".to_string(),
        ];
        let mut signals = BTreeMap::new();
        signals.insert(
            ".planning/STATE.md".to_string(),
            vec![RetrievalSignalKind::Lexical, RetrievalSignalKind::Docs],
        );
        signals.insert(
            "src/lib.rs".to_string(),
            vec![
                RetrievalSignalKind::Lexical,
                RetrievalSignalKind::Dependency,
            ],
        );
        signals.insert(
            "src/history.rs".to_string(),
            vec![
                RetrievalSignalKind::CoChange,
                RetrievalSignalKind::Dependency,
            ],
        );
        signals.insert(
            "scripts/release.sh".to_string(),
            vec![RetrievalSignalKind::Lexical],
        );
        let roles = BTreeMap::from([
            (".planning/STATE.md".to_string(), FileRole::Docs),
            ("src/lib.rs".to_string(), FileRole::Source),
            ("src/history.rs".to_string(), FileRole::Source),
            ("scripts/release.sh".to_string(), FileRole::Unknown),
        ]);
        let targets = BTreeSet::from([
            ".planning/STATE.md".to_string(),
            "src/history.rs".to_string(),
        ]);

        let profiles = selected_signal_profiles(&recommended, &signals, &roles, &targets, 3);

        assert!(profiles.contains(&HistoricalSelectedSignalProfile {
            signal: RetrievalSignalKind::Lexical,
            role: FileRole::Docs,
            selected_at_10_count: 1,
            retrieval_target_selected_at_10_count: 1,
        }));
        assert!(profiles.contains(&HistoricalSelectedSignalProfile {
            signal: RetrievalSignalKind::Dependency,
            role: FileRole::Source,
            selected_at_10_count: 2,
            retrieval_target_selected_at_10_count: 1,
        }));
        assert!(profiles.contains(&HistoricalSelectedSignalProfile {
            signal: RetrievalSignalKind::CoChange,
            role: FileRole::Source,
            selected_at_10_count: 1,
            retrieval_target_selected_at_10_count: 1,
        }));
        assert!(!profiles.iter().any(|profile| {
            profile.signal == RetrievalSignalKind::Lexical && profile.role == FileRole::Unknown
        }));
    }

    #[test]
    fn product_proof_release_gate_blocks_mixed_or_trailing_corpora() {
        let mut beat = empty_historical_eval_report("beat");
        set_recall_metrics(&mut beat, 0.50, 0.40);
        let mut match_non_ceiling = empty_historical_eval_report("match");
        set_recall_metrics(&mut match_non_ceiling, 0.50, 0.50);
        let mut trail = empty_historical_eval_report("trail");
        set_recall_metrics(&mut trail, 0.39, 0.43);

        let report = build_product_proof_report(benchmark_report_with_repos(vec![
            ("beat-repo", beat),
            ("match-repo", match_non_ceiling),
            ("trail-repo", trail),
        ]));

        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
        assert!(!report.release_gate.default_promotion_allowed);
        assert!(report
            .release_gate
            .corpus_verdicts
            .iter()
            .any(|verdict| verdict.repository == "beat-repo"
                && verdict.status == ProductProofCorpusStatus::Beat));
        assert!(report
            .release_gate
            .corpus_verdicts
            .iter()
            .any(|verdict| verdict.repository == "match-repo"
                && verdict.status == ProductProofCorpusStatus::Match));
        assert!(report
            .release_gate
            .corpus_verdicts
            .iter()
            .any(|verdict| verdict.repository == "trail-repo"
                && verdict.status == ProductProofCorpusStatus::Trail));
        assert_eq!(
            report.release_gate.lexical_comparison.all_file_claim,
            ProductProofLexicalClaim::TrailsAnyCorpus
        );
        assert_eq!(
            report.release_gate.lexical_comparison.all_file_beat_count,
            1
        );
        assert_eq!(
            report.release_gate.lexical_comparison.all_file_match_count,
            1
        );
        assert_eq!(
            report.release_gate.lexical_comparison.all_file_trail_count,
            1
        );
        assert_eq!(
            report
                .release_gate
                .lexical_comparison
                .all_file_explained_trail_count,
            0
        );
        assert_eq!(
            report
                .release_gate
                .lexical_comparison
                .all_file_unexplained_trail_count,
            1
        );
        assert!(report
            .recommended_research_actions
            .iter()
            .any(
                |action| action.action == "fix_retrieval_or_ranking_regression"
                    && action.priority == 1
                    && action.origin == "product_proof"
            ));
    }

    #[test]
    fn benchmark_suite_embeds_report_when_history_sampling_is_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo-without-git-history");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn example() {}\n").unwrap();
        let config = BenchmarkSuiteConfig {
            manifest_version: default_benchmark_manifest_version(),
            name: "history-unavailable".to_string(),
            corpus_id: Some("history-unavailable-corpus".to_string()),
            privacy_label: Some("source_free_local".to_string()),
            description: None,
            defaults: BenchmarkDefaults {
                limit: 3,
                ranking_budget: 10,
                parallelism: 1,
                ..BenchmarkDefaults::default()
            },
            repositories: vec![BenchmarkRepoConfig {
                name: "historyless".to_string(),
                path: repo,
                revision_range_id: Some("missing-history".to_string()),
                privacy_label: None,
                base: None,
                head: None,
                limit: None,
                ranking_budget: None,
                mode: None,
                target_agent: None,
                semantic_enabled: None,
                semantic_provider: None,
                semantic_model: None,
                semantic_dimensions: None,
                local_metadata_reranker: None,
                cache_enabled: Some(false),
                force_refresh: Some(true),
                parallelism: Some(1),
                role_filters: Vec::new(),
                lexical_backend_comparison: None,
                proof_runtime_ceiling_millis: None,
                baseline: None,
            }],
        };

        let report = run_benchmark_suite_config(&config, temp.path()).unwrap();
        let repo_report = &report.repositories[0];

        assert_eq!(repo_report.name, "historyless");
        assert_eq!(repo_report.evaluated_commits, 0);
        assert!(repo_report.report.is_some());
        assert!(repo_report
            .error
            .as_deref()
            .unwrap()
            .contains("no evaluable commits"));
        assert_eq!(
            repo_report.environment_health.status,
            BenchmarkRepoEnvironmentStatus::GitHistoryUnavailable
        );
        assert!(!repo_report.environment_health.git_history_usable);
        assert_eq!(report.evaluated_repository_count, 0);
        let proof = build_product_proof_report(report);
        let verdict = &proof.release_gate.corpus_verdicts[0];
        assert_eq!(
            verdict.status,
            ProductProofCorpusStatus::InsufficientEvidence
        );
        assert_eq!(
            verdict.environment_health.status,
            BenchmarkRepoEnvironmentStatus::GitHistoryUnavailable
        );
        assert!(verdict
            .notes
            .iter()
            .any(|note| note.contains("git history")));
        assert_eq!(
            proof.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
        assert!(proof
            .release_gate
            .decision_reason
            .contains("environment health is degraded"));
    }

    #[test]
    fn benchmark_repo_environment_health_classifies_source_free_failures() {
        let timeout = benchmark_repo_environment_health(
            0,
            Some("git command failed: rev-list timed out after 10s"),
        );
        assert_eq!(
            timeout.status,
            BenchmarkRepoEnvironmentStatus::GitHistoryTimeout
        );
        assert_eq!(timeout.object_content_usable, None);

        let object = benchmark_repo_environment_health(
            0,
            Some("git cat-file --batch object read timed out"),
        );
        assert_eq!(
            object.status,
            BenchmarkRepoEnvironmentStatus::GitObjectStoreUnavailable
        );
        assert_eq!(object.object_content_usable, Some(false));

        let incomplete_parent_snapshot = benchmark_repo_environment_health(
            0,
            Some("parent snapshot object extraction incomplete for abc123"),
        );
        assert_eq!(
            incomplete_parent_snapshot.status,
            BenchmarkRepoEnvironmentStatus::GitObjectStoreUnavailable
        );
        assert_eq!(
            incomplete_parent_snapshot.object_content_usable,
            Some(false)
        );

        let healthy = benchmark_repo_environment_health(2, None);
        assert_eq!(healthy.status, BenchmarkRepoEnvironmentStatus::Healthy);
        assert!(healthy.git_history_usable);
    }

    #[test]
    fn product_proof_release_gate_promotes_beat_local_corpora() {
        let mut left = empty_historical_eval_report("left");
        set_recall_metrics(&mut left, 0.50, 0.40);
        let mut right = empty_historical_eval_report("right");
        set_recall_metrics(&mut right, 0.61, 0.55);

        let report = build_product_proof_report(benchmark_report_with_repos(vec![
            ("left", left),
            ("right", right),
        ]));

        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Promote
        );
        assert!(report.release_gate.default_promotion_allowed);
        assert!(report
            .recommended_research_actions
            .iter()
            .any(|action| action.action == "collect_bm25_backend_evidence"
                || action.action == "preserve_current_contract"));
    }

    #[test]
    fn product_proof_release_gate_blocks_source_recall_regression() {
        let mut eval = empty_historical_eval_report("source-regression");
        set_recall_metrics(&mut eval, 0.67, 0.50);
        eval.source_recall_at_10 = 0.0;
        let commit = &mut eval.commits[0];
        commit.changed_path_labels = vec![
            historical_changed_path_label("src/a.py", FileRole::Source),
            historical_changed_path_label("src/b.py", FileRole::Source),
            historical_changed_path_label("docs/a.md", FileRole::Docs),
            historical_changed_path_label("docs/b.md", FileRole::Docs),
            historical_changed_path_label("docs/c.md", FileRole::Docs),
            historical_changed_path_label("docs/d.md", FileRole::Docs),
        ];
        commit.retrieval_target_files = vec![
            "src/a.py".to_string(),
            "src/b.py".to_string(),
            "docs/a.md".to_string(),
            "docs/b.md".to_string(),
            "docs/c.md".to_string(),
            "docs/d.md".to_string(),
        ];
        commit.recommended_context_files = vec![
            "docs/a.md".to_string(),
            "docs/b.md".to_string(),
            "docs/c.md".to_string(),
            "docs/d.md".to_string(),
        ];
        commit.lexical_baseline_files = vec![
            "src/a.py".to_string(),
            "src/b.py".to_string(),
            "docs/a.md".to_string(),
        ];
        commit.source_files_changed = 2;
        commit.source_hits_at_10 = 0;

        let report = build_product_proof_report(benchmark_report_with_repos(vec![(
            "source-regression",
            eval,
        )]));
        let verdict = &report.release_gate.corpus_verdicts[0];

        assert_eq!(verdict.status, ProductProofCorpusStatus::Beat);
        assert_eq!(verdict.source_recall_at_10, 0.0);
        assert_eq!(verdict.lexical_source_recall_at_10, 1.0);
        assert_eq!(verdict.source_delta_at_10, -1.0);
        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
        assert!(report
            .release_gate
            .decision_reason
            .contains("source Recall@10 trailed lexical"));
    }

    #[test]
    fn product_proof_release_gate_accepts_perfect_ceiling_match() {
        let mut ceiling = empty_historical_eval_report("ceiling");
        set_recall_metrics(&mut ceiling, 1.0, 1.0);
        ceiling.runtime.total_millis = 7_500;
        let mut beat = empty_historical_eval_report("beat");
        set_recall_metrics(&mut beat, 0.61, 0.55);

        let report = build_product_proof_report(benchmark_report_with_repos(vec![
            ("ceiling-repo", ceiling),
            ("beat-repo", beat),
        ]));
        let ceiling_verdict = report
            .release_gate
            .corpus_verdicts
            .iter()
            .find(|verdict| verdict.repository == "ceiling-repo")
            .unwrap();

        assert_eq!(ceiling_verdict.status, ProductProofCorpusStatus::Match);
        assert!(ceiling_verdict
            .notes
            .iter()
            .any(|note| note.contains("lexical ceiling")));
        assert!(ceiling_verdict
            .notes
            .iter()
            .any(|note| note.contains("cold proof exceeded 5000ms")));
        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Promote
        );
        assert!(report.release_gate.default_promotion_allowed);
    }

    #[test]
    fn product_proof_release_gate_blocks_hard_cold_runtime_ceiling() {
        let mut ceiling = empty_historical_eval_report("ceiling");
        set_recall_metrics(&mut ceiling, 1.0, 1.0);
        ceiling.runtime.total_millis = 10_001;

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("ceiling", ceiling)]));

        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
        assert!(report
            .release_gate
            .decision_reason
            .contains("runtime exceeded"));
    }

    #[test]
    fn product_proof_release_gate_accepts_repo_scoped_runtime_ceiling() {
        let mut slow = empty_historical_eval_report("slow-cold-fixture");
        set_recall_metrics(&mut slow, 0.72, 0.55);
        slow.runtime.total_millis = 13_878;

        let mut suite = benchmark_report_with_repos(vec![("large-cold-fixture", slow)]);
        suite.repositories[0]
            .effective_config
            .proof_runtime_ceiling_millis = Some(15_000);

        let report = build_product_proof_report(suite);
        let verdict = report
            .release_gate
            .corpus_verdicts
            .iter()
            .find(|verdict| verdict.repository == "large-cold-fixture")
            .unwrap();

        assert_eq!(verdict.status, ProductProofCorpusStatus::Beat);
        assert!(verdict
            .notes
            .iter()
            .any(|note| note.contains("repo-scoped 15000ms per-commit ceiling")));
        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Promote
        );
    }

    #[test]
    fn cached_eval_runtime_reports_warm_lookup_cost() {
        let cold = HistoricalEvalRuntimeSummary {
            total_millis: 12_000,
            commit_millis: 10_000,
            overhead_millis: 2_000,
            average_commit_millis: 2_000.0,
            cache_hits: 0,
            cache_misses: 1,
            parallelism: 4,
            git_sample_millis: 500,
            ranking_millis: 600,
            pack_compiler_millis: 8_000,
            slow_commits: vec![HistoricalSlowCommitSummary {
                sha: "abc123".to_string(),
                elapsed_millis: 2_500,
                safe_changed_file_count: 3,
                recommended_context_file_count: 10,
                missing_file_count_at_10: 1,
                low_information_task: false,
            }],
        };

        let warm = cached_historical_eval_runtime_summary(&cold, 7);

        assert_eq!(warm.total_millis, 7);
        assert_eq!(warm.commit_millis, 0);
        assert_eq!(warm.overhead_millis, 7);
        assert_eq!(warm.average_commit_millis, 0.0);
        assert_eq!(warm.cache_hits, 1);
        assert_eq!(warm.cache_misses, 0);
        assert_eq!(warm.parallelism, 4);
        assert_eq!(warm.git_sample_millis, 0);
        assert_eq!(warm.ranking_millis, 0);
        assert_eq!(warm.pack_compiler_millis, 0);
        assert!(warm.slow_commits.is_empty());
    }

    #[test]
    fn product_proof_release_gate_blocks_stale_warm_cache_runtime() {
        let mut cached = empty_historical_eval_report("cached-stale-runtime");
        set_recall_metrics(&mut cached, 0.60, 0.40);
        cached.runtime.total_millis = 12;
        cached.runtime.commit_millis = 9_000;
        cached.runtime.average_commit_millis = 9_000.0;
        cached.runtime.cache_hits = 1;
        cached.runtime.cache_misses = 0;
        cached.runtime.slow_commits = vec![HistoricalSlowCommitSummary {
            sha: "abc123".to_string(),
            elapsed_millis: 9_000,
            safe_changed_file_count: 1,
            recommended_context_file_count: 10,
            missing_file_count_at_10: 0,
            low_information_task: false,
        }];

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("cached", cached)]));

        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
        assert!(report
            .release_gate
            .decision_reason
            .contains("warm-cache thresholds"));
    }

    #[test]
    fn product_proof_release_gate_blocks_slow_warm_cache_lookup() {
        let mut cached = empty_historical_eval_report("cached-slow-runtime");
        set_recall_metrics(&mut cached, 0.60, 0.40);
        cached.runtime.total_millis = 1_001;
        cached.runtime.commit_millis = 0;
        cached.runtime.overhead_millis = 1_001;
        cached.runtime.average_commit_millis = 0.0;
        cached.runtime.cache_hits = 1;
        cached.runtime.cache_misses = 0;
        cached.runtime.git_sample_millis = 0;
        cached.runtime.ranking_millis = 0;
        cached.runtime.pack_compiler_millis = 0;
        cached.runtime.slow_commits = Vec::new();

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("cached", cached)]));

        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
        assert!(report
            .release_gate
            .decision_reason
            .contains("warm-cache thresholds"));
    }

    #[test]
    fn product_proof_release_gate_promotes_fast_warm_cache_lookup() {
        let mut cached = empty_historical_eval_report("cached-fast-runtime");
        set_recall_metrics(&mut cached, 0.60, 0.40);
        cached.runtime = cached_historical_eval_runtime_summary(&cached.runtime, 7);

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("cached", cached)]));

        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Promote
        );
        assert!(report.release_gate.corpus_verdicts[0]
            .notes
            .iter()
            .any(|note| note.contains("warm cache proof hit")));
    }

    #[test]
    fn product_proof_release_gate_separates_context_and_validation_channels() {
        let mut eval = empty_historical_eval_report("channels");
        set_recall_metrics(&mut eval, 0.50, 0.50);
        eval.test_recall_at_10 = 1.0;
        eval.effective_validation_recall_at_10 = 1.0;
        let commit = &mut eval.commits[0];
        commit.changed_path_labels = vec![
            HistoricalChangedPathLabel {
                path: "src/auth/session.ts".to_string(),
                change_kind: ctxhelm_index::ChangeKind::Modified,
                role: FileRole::Source,
                label_scope: LabelScope::Safe,
                excluded_reason: None,
                old_path: None,
            },
            HistoricalChangedPathLabel {
                path: "tests/auth/session.test.ts".to_string(),
                change_kind: ctxhelm_index::ChangeKind::Modified,
                role: FileRole::Test,
                label_scope: LabelScope::Safe,
                excluded_reason: None,
                old_path: None,
            },
        ];
        commit.retrieval_target_files = vec![
            "src/auth/session.ts".to_string(),
            "tests/auth/session.test.ts".to_string(),
        ];
        commit.recommended_context_files = vec!["src/auth/session.ts".to_string()];
        commit.lexical_baseline_files = vec!["tests/auth/session.test.ts".to_string()];
        commit.recommended_tests = vec!["tests/auth/session.test.ts".to_string()];
        commit.test_files_changed = 1;
        commit.test_hits_at_10 = 1;
        commit.effective_validation_hits_at_10 = 1;

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("channel-repo", eval)]));
        let verdict = &report.release_gate.corpus_verdicts[0];

        assert_eq!(verdict.status, ProductProofCorpusStatus::Beat);
        assert_eq!(verdict.context_recall_at_10, 1.0);
        assert_eq!(verdict.lexical_context_recall_at_10, 0.0);
        assert_eq!(verdict.test_recall_at_10, 1.0);
        assert!(verdict.all_file_divergence_explained);
    }

    #[test]
    fn product_proof_release_gate_explains_validation_separated_all_file_divergence() {
        let mut eval = empty_historical_eval_report("explained-divergence");
        set_recall_metrics(&mut eval, 0.60, 0.80);
        eval.source_recall_at_10 = 1.0;
        eval.test_recall_at_10 = 1.0;
        eval.effective_validation_recall_at_10 = 1.0;
        let commit = &mut eval.commits[0];
        commit.changed_path_labels = vec![
            historical_changed_path_label("src/auth/session.ts", FileRole::Source),
            historical_changed_path_label("src/auth/middleware.ts", FileRole::Source),
            historical_changed_path_label("src/routes/login.ts", FileRole::Source),
            historical_changed_path_label("tests/auth/session.test.ts", FileRole::Test),
            historical_changed_path_label("tests/auth/redirect.test.ts", FileRole::Test),
        ];
        commit.retrieval_target_files = commit
            .changed_path_labels
            .iter()
            .map(|label| label.path.clone())
            .collect();
        commit.recommended_context_files = vec![
            "src/auth/session.ts".to_string(),
            "src/auth/middleware.ts".to_string(),
            "src/routes/login.ts".to_string(),
        ];
        commit.lexical_baseline_files = vec![
            "src/auth/session.ts".to_string(),
            "src/auth/middleware.ts".to_string(),
            "tests/auth/session.test.ts".to_string(),
            "tests/auth/redirect.test.ts".to_string(),
        ];
        commit.recommended_tests = vec![
            "tests/auth/session.test.ts".to_string(),
            "tests/auth/redirect.test.ts".to_string(),
        ];
        commit.source_files_changed = 3;
        commit.source_hits_at_10 = 3;
        commit.test_files_changed = 2;
        commit.test_hits_at_10 = 2;
        commit.effective_validation_hits_at_10 = 2;

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("explained", eval)]));
        let verdict = &report.release_gate.corpus_verdicts[0];

        assert_eq!(verdict.status, ProductProofCorpusStatus::Beat);
        assert!((verdict.lexical_delta_at_10 + 0.20).abs() < 0.001);
        assert!((verdict.agent_evidence_recall_at_10 - 1.0).abs() < 0.001);
        assert!((verdict.agent_evidence_delta_at_10 - 0.20).abs() < 0.001);
        assert!((verdict.context_delta_at_10 - (1.0 / 3.0)).abs() < 0.001);
        assert!(verdict.context_vs_all_file_delta_at_10 > 0.39);
        assert!(verdict.lexical_context_vs_all_file_delta_at_10 < -0.13);
        assert!(verdict.all_file_divergence_explained);
        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Promote
        );
        assert_eq!(
            report.release_gate.lexical_comparison.all_file_claim,
            ProductProofLexicalClaim::MatchesAllCorpora
        );
        assert_eq!(
            report.release_gate.lexical_comparison.all_file_trail_count,
            1
        );
        assert_eq!(
            report
                .release_gate
                .lexical_comparison
                .all_file_explained_trail_count,
            1
        );
        assert_eq!(
            report
                .release_gate
                .lexical_comparison
                .all_file_unexplained_trail_count,
            0
        );
        assert_eq!(
            report.release_gate.lexical_comparison.agent_evidence_claim,
            ProductProofLexicalClaim::BeatsAllCorpora
        );
        assert_eq!(
            report.release_gate.lexical_comparison.context_claim,
            ProductProofLexicalClaim::BeatsAllCorpora
        );
        assert!(
            report
                .release_gate
                .lexical_comparison
                .average_file_delta_at_10
                < 0.0
        );
        assert!(
            report
                .release_gate
                .lexical_comparison
                .average_agent_evidence_delta_at_10
                > 0.0
        );
        assert!(
            report
                .release_gate
                .lexical_comparison
                .average_context_delta_at_10
                > 0.0
        );
    }

    #[test]
    fn product_proof_release_gate_blocks_unexplained_all_file_divergence() {
        let mut eval = empty_historical_eval_report("unexplained-divergence");
        set_recall_metrics(&mut eval, 0.50, 0.80);
        let commit = &mut eval.commits[0];
        commit.changed_path_labels = vec![
            historical_changed_path_label("src/auth/session.ts", FileRole::Source),
            historical_changed_path_label("src/auth/middleware.ts", FileRole::Source),
        ];
        commit.retrieval_target_files = commit
            .changed_path_labels
            .iter()
            .map(|label| label.path.clone())
            .collect();
        commit.recommended_context_files = vec![
            "src/auth/session.ts".to_string(),
            "src/auth/middleware.ts".to_string(),
        ];
        commit.lexical_baseline_files = vec!["src/auth/session.ts".to_string()];

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("unexplained", eval)]));
        let verdict = &report.release_gate.corpus_verdicts[0];

        assert_eq!(verdict.status, ProductProofCorpusStatus::Beat);
        assert!(!verdict.all_file_divergence_explained);
        assert_eq!(
            report.release_gate.decision,
            SemanticPrecisionGateDecision::Block
        );
    }

    #[test]
    fn product_proof_release_gate_accepts_broad_validation_command_coverage() {
        let mut eval = empty_historical_eval_report("validation-command");
        set_recall_metrics(&mut eval, 0.50, 0.40);
        eval.test_recall_at_10 = 0.5;
        eval.validation_command_recall = 1.0;
        eval.effective_validation_recall_at_10 = 1.0;
        let commit = &mut eval.commits[0];
        commit.changed_path_labels = vec![
            HistoricalChangedPathLabel {
                path: "src/eval/runner.py".to_string(),
                change_kind: ctxhelm_index::ChangeKind::Modified,
                role: FileRole::Source,
                label_scope: LabelScope::Safe,
                excluded_reason: None,
                old_path: None,
            },
            HistoricalChangedPathLabel {
                path: "tests/agents/test_requirement_analyzer.py".to_string(),
                change_kind: ctxhelm_index::ChangeKind::Modified,
                role: FileRole::Test,
                label_scope: LabelScope::Safe,
                excluded_reason: None,
                old_path: None,
            },
            HistoricalChangedPathLabel {
                path: "tests/core/test_retry_routing.py".to_string(),
                change_kind: ctxhelm_index::ChangeKind::Modified,
                role: FileRole::Test,
                label_scope: LabelScope::Safe,
                excluded_reason: None,
                old_path: None,
            },
        ];
        commit.retrieval_target_files = vec![
            "src/eval/runner.py".to_string(),
            "tests/agents/test_requirement_analyzer.py".to_string(),
            "tests/core/test_retry_routing.py".to_string(),
        ];
        commit.recommended_context_files = vec!["src/eval/runner.py".to_string()];
        commit.lexical_baseline_files = Vec::new();
        commit.recommended_tests = vec!["tests/agents/test_requirement_analyzer.py".to_string()];
        commit.recommended_commands = vec!["pytest".to_string()];
        commit.test_files_changed = 2;
        commit.test_hits_at_10 = 1;
        commit.validation_command_hits = 2;
        commit.effective_validation_hits_at_10 = 2;

        let report =
            build_product_proof_report(benchmark_report_with_repos(vec![("broad-repo", eval)]));
        let verdict = &report.release_gate.corpus_verdicts[0];

        assert_eq!(verdict.status, ProductProofCorpusStatus::Beat);
        assert_eq!(verdict.test_recall_at_10, 0.5);
        assert_eq!(verdict.validation_command_recall, 1.0);
        assert_eq!(verdict.effective_validation_recall_at_10, 1.0);
        assert!(verdict
            .notes
            .iter()
            .any(|note| note.contains("broad validation commands raise")));
    }

    #[test]
    fn validation_command_coverage_recognizes_broad_pytest() {
        let tests = vec![
            "tests/agents/test_requirement_analyzer.py".to_string(),
            "tests/core/test_retry_routing.py".to_string(),
        ];
        let commands = vec!["pytest".to_string()];

        assert_eq!(validation_command_hits(&tests, &commands).len(), 2);
        assert_eq!(effective_validation_hits_at_10(&tests, &[], &commands), 2);
    }

    #[test]
    fn validation_command_coverage_recognizes_java_class_selectors() {
        let tests = vec![
            "src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpToolsTest.java".to_string(),
            "src/test/java/org/refactoringminer/mcp/RefactoringMinerMcpServiceRepositoryTest.java"
                .to_string(),
        ];
        let commands = vec![
            "./gradlew test --tests org.refactoringminer.mcp.RefactoringMinerMcpToolsTest"
                .to_string(),
            "mvn -Dtest=RefactoringMinerMcpServiceRepositoryTest test".to_string(),
        ];

        assert_eq!(validation_command_hits(&tests, &commands), tests);
        assert_eq!(effective_validation_hits_at_10(&tests, &[], &commands), 2);
    }

    #[test]
    fn broad_context_area_recall_counts_surfaced_areas() {
        let commits = vec![HistoricalCommitEval {
            sha: "abc123".to_string(),
            task_hash: "task".to_string(),
            task_type: TaskType::BugFix,
            target_agent: "generic".to_string(),
            changed_path_labels: vec![
                historical_changed_path_label("schema_agent/core/fd_router.py", FileRole::Source),
                historical_changed_path_label("docs/architecture.md", FileRole::Docs),
                historical_changed_path_label("tests/agents/test_base.py", FileRole::Test),
            ],
            safe_changed_files: Vec::new(),
            retrieval_target_files: Vec::new(),
            excluded_changed_file_count: 0,
            recommended_files: Vec::new(),
            recommended_tests: vec!["tests/agents/test_base.py".to_string()],
            recommended_context_files: Vec::new(),
            recommended_commands: Vec::new(),
            lexical_baseline_files: Vec::new(),
            signal_baseline_files: Vec::new(),
            selected_signal_profiles: Vec::new(),
            protected_evidence: Vec::new(),
            graph_edge_profiles: Vec::new(),
            file_hits_at_5: Vec::new(),
            file_hits_at_10: Vec::new(),
            lexical_baseline_hits_at_5: Vec::new(),
            lexical_baseline_hits_at_10: Vec::new(),
            missing_files_at_10: vec![
                "schema_agent/core/fd_router.py".to_string(),
                "docs/architecture.md".to_string(),
                "tests/agents/test_base.py".to_string(),
            ],
            candidate_missed_files_at_10: vec![
                "schema_agent/core/fd_router.py".to_string(),
                "docs/architecture.md".to_string(),
            ],
            candidate_missed_file_profiles_at_10: vec![
                CandidateMissedFileProfile {
                    path: "schema_agent/core/fd_router.py".to_string(),
                    role: FileRole::Source,
                    context_area: "schema_agent/core".to_string(),
                    signals: vec![
                        RetrievalSignalKind::Dependency,
                        RetrievalSignalKind::LexicalExpansion,
                    ],
                },
                CandidateMissedFileProfile {
                    path: "docs/architecture.md".to_string(),
                    role: FileRole::Docs,
                    context_area: "docs".to_string(),
                    signals: vec![RetrievalSignalKind::Lexical],
                },
            ],
            source_files_changed: 4,
            source_hits_at_5: 1,
            source_hits_at_10: 1,
            test_files_changed: 0,
            test_hits_at_5: 0,
            test_hits_at_10: 0,
            validation_command_hits: 0,
            effective_validation_hits_at_10: 0,
            low_information_task: false,
            broad_scope_task: true,
            changed_context_areas: vec![
                "schema_agent/agents".to_string(),
                "schema_agent/core".to_string(),
            ],
            context_area_hits: vec!["schema_agent/core".to_string()],
            context_areas: vec![
                ContextArea {
                    area: "schema_agent/core".to_string(),
                    reason: "selected implementation area".to_string(),
                    resource_uri: "ctxhelm://repo/context-area/schema_agent%2Fcore".to_string(),
                    representative_paths: vec!["schema_agent/core/fd_registry.py".to_string()],
                    next_read_paths: vec!["schema_agent/core/fd_router.py".to_string()],
                    signal_counts: BTreeMap::from([("lexical".to_string(), 2)]),
                    role_counts: BTreeMap::from([("source".to_string(), 3)]),
                    selected_role_counts: BTreeMap::from([("source".to_string(), 1)]),
                    candidate_count: 3,
                    selected_count: 1,
                    unselected_count: 2,
                    coverage_percent: 33,
                    inspection_pressure: 6,
                    inspection_pressure_breakdown: InspectionPressureBreakdown {
                        source_like_unselected: 2,
                        validation_unselected: 0,
                        docs_unselected: 0,
                        source_like_weight: 3,
                        validation_weight: 2,
                        docs_weight: 1,
                        total: 6,
                    },
                },
                ContextArea {
                    area: "docs".to_string(),
                    reason: "zero-selected docs area".to_string(),
                    resource_uri: "ctxhelm://repo/context-area/docs".to_string(),
                    representative_paths: vec!["docs/architecture.md".to_string()],
                    next_read_paths: vec!["docs/architecture.md".to_string()],
                    signal_counts: BTreeMap::from([("docs".to_string(), 1)]),
                    role_counts: BTreeMap::from([("docs".to_string(), 1)]),
                    selected_role_counts: BTreeMap::new(),
                    candidate_count: 1,
                    selected_count: 0,
                    unselected_count: 1,
                    coverage_percent: 0,
                    inspection_pressure: 1,
                    inspection_pressure_breakdown: InspectionPressureBreakdown {
                        source_like_unselected: 0,
                        validation_unselected: 0,
                        docs_unselected: 1,
                        source_like_weight: 3,
                        validation_weight: 2,
                        docs_weight: 1,
                        total: 1,
                    },
                },
            ],
            confidence: 0.5,
            query_trace: None,
            elapsed_millis: 0,
            source_text_logged: false,
        }];

        assert_eq!(average_broad_context_area_recall(&commits), 0.5);
        let pressure = context_area_pressure_summary(&commits);
        assert_eq!(pressure.context_area_count, 2);
        assert_eq!(pressure.zero_selected_area_count, 1);
        assert_eq!(pressure.total_inspection_pressure, 7);
        assert_eq!(pressure.source_like_pressure, 6);
        assert_eq!(pressure.docs_pressure, 1);
        assert_eq!(
            pressure.highest_pressure_area.as_ref().unwrap().area,
            "schema_agent/core"
        );
        assert!(!pressure.source_text_logged);
        let next_read = context_area_next_read_summary(&commits);
        assert_eq!(next_read.missed_file_count_at_10, 3);
        assert_eq!(next_read.next_read_recoverable_count, 2);
        assert_eq!(next_read.agent_evidence_recoverable_count, 3);
        assert_eq!(next_read.agent_evidence_only_count, 1);
        assert_eq!(
            next_read.agent_evidence_only_role_counts.get("test"),
            Some(&1)
        );
        assert_eq!(
            next_read.top_agent_evidence_only_areas,
            vec![CandidateCoverageAreaSummary {
                context_area: "tests/agents".to_string(),
                missed_count: 1,
            }]
        );
        assert_eq!(next_read.top_pressure_next_read_recoverable_count, 1);
        assert_eq!(next_read.zero_selected_area_recoverable_count, 1);
        assert!(!next_read.source_text_logged);
    }

    #[test]
    fn retrieval_gap_reasons_distinguish_area_only_context() {
        let reasons = retrieval_gap_reasons(
            &["schema_agent/core/fd_registry.py".to_string()],
            &[],
            &BTreeMap::new(),
            &["schema_agent/core".to_string()],
        );

        assert_eq!(
            reasons.get("schema_agent/core/fd_registry.py"),
            Some(&"area_context_only".to_string())
        );
        assert_eq!(
            recommendation_area_for_gap("area_context_only", FileRole::Source),
            RetrievalGapRecommendationArea::ContextPlanning
        );
    }

    #[test]
    fn retrieval_gap_summaries_skip_validation_covered_tests() {
        let commit = HistoricalCommitEval {
            sha: "abc123".to_string(),
            task_hash: "task".to_string(),
            task_type: TaskType::BugFix,
            target_agent: "generic".to_string(),
            changed_path_labels: vec![
                historical_changed_path_label("src/auth/session.ts", FileRole::Source),
                historical_changed_path_label("src/auth/token.ts", FileRole::Source),
                historical_changed_path_label("tests/auth/session.test.ts", FileRole::Test),
            ],
            safe_changed_files: vec![
                "src/auth/session.ts".to_string(),
                "src/auth/token.ts".to_string(),
                "tests/auth/session.test.ts".to_string(),
            ],
            retrieval_target_files: vec![
                "src/auth/session.ts".to_string(),
                "src/auth/token.ts".to_string(),
                "tests/auth/session.test.ts".to_string(),
            ],
            excluded_changed_file_count: 0,
            recommended_files: Vec::new(),
            recommended_tests: vec!["tests/auth/session.test.ts".to_string()],
            recommended_context_files: Vec::new(),
            recommended_commands: Vec::new(),
            lexical_baseline_files: Vec::new(),
            signal_baseline_files: Vec::new(),
            selected_signal_profiles: Vec::new(),
            protected_evidence: Vec::new(),
            graph_edge_profiles: Vec::new(),
            file_hits_at_5: Vec::new(),
            file_hits_at_10: Vec::new(),
            lexical_baseline_hits_at_5: Vec::new(),
            lexical_baseline_hits_at_10: Vec::new(),
            missing_files_at_10: vec![
                "src/auth/session.ts".to_string(),
                "src/auth/token.ts".to_string(),
                "tests/auth/session.test.ts".to_string(),
            ],
            candidate_missed_files_at_10: vec![
                "src/auth/session.ts".to_string(),
                "tests/auth/session.test.ts".to_string(),
            ],
            candidate_missed_file_profiles_at_10: vec![
                CandidateMissedFileProfile {
                    path: "src/auth/session.ts".to_string(),
                    role: FileRole::Source,
                    context_area: "src/auth".to_string(),
                    signals: vec![RetrievalSignalKind::Dependency],
                },
                CandidateMissedFileProfile {
                    path: "tests/auth/session.test.ts".to_string(),
                    role: FileRole::Test,
                    context_area: "tests/auth".to_string(),
                    signals: vec![RetrievalSignalKind::RelatedTest],
                },
            ],
            source_files_changed: 2,
            source_hits_at_5: 0,
            source_hits_at_10: 0,
            test_files_changed: 1,
            test_hits_at_5: 1,
            test_hits_at_10: 1,
            validation_command_hits: 0,
            effective_validation_hits_at_10: 1,
            low_information_task: false,
            broad_scope_task: false,
            changed_context_areas: Vec::new(),
            context_area_hits: Vec::new(),
            context_areas: vec![ContextArea {
                area: "src/auth".to_string(),
                reason: "source area surfaced by lexical and dependency evidence".to_string(),
                resource_uri: "ctxhelm://repo/context-area/src%2Fauth".to_string(),
                representative_paths: vec!["src/auth/session.ts".to_string()],
                next_read_paths: vec!["src/auth/session.ts".to_string()],
                signal_counts: BTreeMap::from([
                    ("dependency".to_string(), 1),
                    ("lexical".to_string(), 2),
                ]),
                role_counts: BTreeMap::from([("source".to_string(), 3)]),
                selected_role_counts: BTreeMap::from([("source".to_string(), 1)]),
                candidate_count: 3,
                selected_count: 1,
                unselected_count: 2,
                coverage_percent: 33,
                inspection_pressure: 6,
                inspection_pressure_breakdown: InspectionPressureBreakdown {
                    source_like_unselected: 2,
                    validation_unselected: 0,
                    docs_unselected: 0,
                    source_like_weight: 3,
                    validation_weight: 2,
                    docs_weight: 1,
                    total: 6,
                },
            }],
            confidence: 0.5,
            query_trace: None,
            elapsed_millis: 0,
            source_text_logged: false,
        };
        let mut roles_by_path = BTreeMap::new();
        roles_by_path.insert("src/auth/session.ts".to_string(), FileRole::Source);
        roles_by_path.insert("src/auth/token.ts".to_string(), FileRole::Source);
        roles_by_path.insert("tests/auth/session.test.ts".to_string(), FileRole::Test);
        let labels_by_path = labels_by_path_from_commits(std::slice::from_ref(&commit));
        let gap_reasons = BTreeMap::from([
            (
                "src/auth/session.ts".to_string(),
                "no_candidate_signal".to_string(),
            ),
            (
                "src/auth/token.ts".to_string(),
                "no_candidate_signal".to_string(),
            ),
            (
                "tests/auth/session.test.ts".to_string(),
                "lexical_only_miss".to_string(),
            ),
        ]);

        let summaries = retrieval_gap_summaries(
            &[commit],
            &[gap_reasons],
            &roles_by_path,
            &labels_by_path,
            10,
        );

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].role, FileRole::Source);
        assert_eq!(summaries[0].missed_count, 2);
        assert_eq!(
            summaries[0].example_paths,
            vec!["src/auth/session.ts", "src/auth/token.ts"]
        );
        assert_eq!(summaries[0].context_area, "src/auth");
        assert_eq!(
            summaries[0].context_area_resource_uri,
            "ctxhelm://repo/context-area/src%2Fauth"
        );
        assert_eq!(
            summaries[0].context_area_signal_counts.get("lexical"),
            Some(&2)
        );
        assert_eq!(
            summaries[0].context_area_signal_counts.get("dependency"),
            Some(&1)
        );
        assert_eq!(
            summaries[0].context_area_role_counts.get("source"),
            Some(&3)
        );
        assert_eq!(
            summaries[0].context_area_selected_role_counts.get("source"),
            Some(&1)
        );
        assert_eq!(summaries[0].context_area_unselected_count, 2);
        assert_eq!(
            summaries[0]
                .context_area_inspection_pressure_breakdown
                .source_like_unselected,
            2
        );
        assert_eq!(
            summaries[0]
                .context_area_inspection_pressure_breakdown
                .total,
            6
        );
        assert_eq!(
            summaries[0].next_read_paths,
            vec!["src/auth/session.ts", "src/auth/token.ts"]
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
            protected_evidence_miss_rate_at_10: Some(0.0),
            protected_evidence_target_miss_rate_at_10: Some(0.0),
            provider_status: "evaluated".to_string(),
            note: "test variant".to_string(),
        }
    }

    fn historical_changed_path_label(path: &str, role: FileRole) -> HistoricalChangedPathLabel {
        HistoricalChangedPathLabel {
            path: path.to_string(),
            change_kind: ctxhelm_index::ChangeKind::Modified,
            role,
            label_scope: LabelScope::Safe,
            excluded_reason: None,
            old_path: None,
        }
    }

    fn benchmark_report_with_repos(
        reports: Vec<(&str, HistoricalEvalReport)>,
    ) -> BenchmarkSuiteReport {
        let evaluated_commit_count = reports
            .iter()
            .map(|(_, report)| report.evaluated_commits)
            .sum::<usize>();
        let repositories = reports
            .into_iter()
            .map(|(name, report)| BenchmarkRepoReport {
                name: name.to_string(),
                repo_id: Some(report.repo_id.clone()),
                effective_config: benchmark_effective_config(),
                environment_health: BenchmarkRepoEnvironmentHealth::healthy(),
                baseline: None,
                baseline_status: None,
                evaluated_commits: report.evaluated_commits,
                excluded_changed_file_count: 0,
                skipped_path_count: 0,
                report: Some(report),
                lexical_backend_corpus: None,
                lexical_backend_error: None,
                error: None,
                privacy_status: PrivacyStatus::local_only(),
            })
            .collect::<Vec<_>>();
        BenchmarkSuiteReport {
            manifest_version: "ctxhelm-benchmark-corpus-test".to_string(),
            suite_name: "test-suite".to_string(),
            suite_id: "suite-id".to_string(),
            corpus_id: Some("fixed-test-corpus".to_string()),
            privacy_label: Some("source_free_local".to_string()),
            description: None,
            generated_at_unix_seconds: 0,
            repository_count: repositories.len(),
            evaluated_repository_count: repositories.len(),
            evaluated_commit_count,
            repositories,
            privacy_status: PrivacyStatus::local_only(),
        }
    }

    fn benchmark_effective_config() -> BenchmarkRepoEffectiveConfig {
        BenchmarkRepoEffectiveConfig {
            revision_range_id: Some("test-range".to_string()),
            privacy_label: Some("source_free_local".to_string()),
            base: None,
            head: None,
            limit: 1,
            ranking_budget: 10,
            mode: TaskType::BugFix,
            target_agent: "generic".to_string(),
            semantic_enabled: false,
            semantic_provider: "local_hash".to_string(),
            semantic_model: None,
            semantic_dimensions: None,
            semantic_provider_role: "scaffold".to_string(),
            semantic_quality_backend: false,
            local_metadata_reranker: false,
            cache_enabled: false,
            force_refresh: false,
            parallelism: 1,
            role_filters: Vec::new(),
            lexical_backend_comparison: false,
            proof_runtime_ceiling_millis: None,
        }
    }

    fn set_recall_metrics(report: &mut HistoricalEvalReport, ctxhelm: f32, lexical: f32) {
        report.file_recall_at_10 = ctxhelm;
        report.lexical_baseline_recall_at_10 = lexical;
        report.ctxhelm_lift_at_10 = ctxhelm - lexical;
        report.ranking_comparison.combined.recall_at_k = ctxhelm;
        report.ranking_comparison.lexical_baseline.recall_at_k = lexical;
    }

    fn empty_historical_eval_report(sha_suffix: &str) -> HistoricalEvalReport {
        HistoricalEvalReport {
            eval_range_id: "range".to_string(),
            repo_id: "repo".to_string(),
            evaluated_commits: 1,
            budget: PackBudget::Standard,
            effective_filters: HistoricalEvalEffectiveFilters {
                limit: 1,
                ranking_budget: 10,
                mode: TaskType::BugFix,
                target_agent: "generic".to_string(),
                budget: PackBudget::Standard,
                semantic_enabled: false,
                semantic_provider: None,
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
            },
            refs: HistoricalEvalRefs {
                base: None,
                head: None,
            },
            base: None,
            head: None,
            ranking_comparison: EvalComparison {
                k: 10,
                combined: RankingMetrics {
                    k: 10,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 0.0,
                },
                lexical_baseline: RankingMetrics {
                    k: 10,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 0.0,
                },
                no_context_baseline: RankingMetrics {
                    k: 10,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 0.0,
                },
                recall_lift_at_k: 0.0,
                precision_lift_at_k: 0.0,
                mrr_lift_at_k: 0.0,
                recall_lift_vs_no_context_at_k: 0.0,
                precision_lift_vs_no_context_at_k: 0.0,
                mrr_lift_vs_no_context_at_k: 0.0,
            },
            signal_ablations: Vec::new(),
            graph_edge_ablations: Vec::new(),
            token_roi: Vec::new(),
            retrieval_gap_summaries: Vec::new(),
            graph_edge_profiles: Vec::new(),
            runtime: HistoricalEvalRuntimeSummary {
                total_millis: 1,
                commit_millis: 1,
                overhead_millis: 0,
                average_commit_millis: 1.0,
                cache_hits: 0,
                cache_misses: 0,
                parallelism: 1,
                git_sample_millis: 0,
                ranking_millis: 0,
                pack_compiler_millis: 0,
                slow_commits: Vec::new(),
            },
            low_information_commit_count: 0,
            broad_scope_commit_count: 0,
            broad_context_area_recall: 0.0,
            context_area_pressure_summary: ContextAreaPressureSummary::default(),
            context_area_next_read_summary: ContextAreaNextReadSummary::default(),
            candidate_coverage_summary: CandidateCoverageSummary::default(),
            memory_reuse_summary: MemoryReuseSummary::default(),
            recommended_research_actions: Vec::new(),
            file_recall_at_5: 0.0,
            file_recall_at_10: 0.0,
            lexical_baseline_recall_at_5: 0.0,
            lexical_baseline_recall_at_10: 0.0,
            ctxhelm_lift_at_5: 0.0,
            ctxhelm_lift_at_10: 0.0,
            source_recall_at_5: 0.0,
            source_recall_at_10: 0.0,
            test_recall_at_5: 0.0,
            test_recall_at_10: 0.0,
            validation_command_recall: 0.0,
            effective_validation_recall_at_10: 0.0,
            test_recommendation_rate: 0.0,
            average_recommended_context_files: 0.0,
            protected_evidence: ProtectedEvidenceSummary::default(),
            top_missing_files: Vec::new(),
            commits: vec![HistoricalCommitEval {
                sha: format!("abc123{sha_suffix}"),
                task_hash: "task".to_string(),
                task_type: TaskType::BugFix,
                target_agent: "generic".to_string(),
                changed_path_labels: Vec::new(),
                safe_changed_files: Vec::new(),
                retrieval_target_files: Vec::new(),
                excluded_changed_file_count: 0,
                recommended_files: Vec::new(),
                recommended_tests: Vec::new(),
                recommended_context_files: Vec::new(),
                recommended_commands: Vec::new(),
                lexical_baseline_files: Vec::new(),
                signal_baseline_files: Vec::new(),
                selected_signal_profiles: Vec::new(),
                protected_evidence: Vec::new(),
                graph_edge_profiles: Vec::new(),
                file_hits_at_5: Vec::new(),
                file_hits_at_10: Vec::new(),
                lexical_baseline_hits_at_5: Vec::new(),
                lexical_baseline_hits_at_10: Vec::new(),
                missing_files_at_10: Vec::new(),
                candidate_missed_files_at_10: Vec::new(),
                candidate_missed_file_profiles_at_10: Vec::new(),
                source_files_changed: 0,
                source_hits_at_5: 0,
                source_hits_at_10: 0,
                test_files_changed: 0,
                test_hits_at_5: 0,
                test_hits_at_10: 0,
                validation_command_hits: 0,
                effective_validation_hits_at_10: 0,
                low_information_task: false,
                broad_scope_task: false,
                changed_context_areas: Vec::new(),
                context_area_hits: Vec::new(),
                context_areas: Vec::new(),
                confidence: 0.0,
                query_trace: None,
                elapsed_millis: 0,
                source_text_logged: false,
            }],
            privacy_status: PrivacyStatus::local_only(),
        }
    }

    fn query_trace_with_facets(kinds: Vec<QueryFacetKind>) -> QueryConstructionTrace {
        QueryConstructionTrace {
            task_hash: "task".to_string(),
            facets: kinds
                .into_iter()
                .map(|kind| ctxhelm_core::QueryFacet {
                    kind,
                    value: "safe-query-token".to_string(),
                    origin: "test".to_string(),
                    weight: 1.0,
                })
                .collect(),
            retriever_queries: ctxhelm_core::RetrieverQuerySet {
                lexical_terms: Vec::new(),
                semantic_phrases: Vec::new(),
                symbol_terms: Vec::new(),
                graph_seeds: Vec::new(),
                history_terms: Vec::new(),
                test_terms: Vec::new(),
            },
            fusion_controls: ctxhelm_core::FusionControlSummary {
                anchor_dominance: true,
                exact_evidence_protected: true,
                semantic_candidate_cap: 8,
                semantic_weight: 0.7,
            },
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
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
