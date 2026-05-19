use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::privacy::PrivacyStatus;
use crate::repo::FileRole;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    BugFix,
    Feature,
    Refactor,
    Review,
    Test,
    Explain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetFile {
    pub path: String,
    pub reason: String,
    pub line_range: Option<LineRange>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTest {
    pub path: String,
    pub reason: String,
    pub command: Option<String>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RetrievalCandidateKind {
    File,
    Test,
    Symbol,
    Doc,
    Commit,
    Config,
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalSignalKind {
    Lexical,
    Symbol,
    Dependency,
    RelatedTest,
    Semantic,
    CoChange,
    CurrentDiff,
    History,
    Docs,
    Config,
    Anchor,
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalSignalScore {
    pub signal: RetrievalSignalKind,
    pub score: f32,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalEvidence {
    pub signal: RetrievalSignalKind,
    pub score: f32,
    pub reason_code: String,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub edge_label: Option<String>,
    #[serde(default)]
    pub commit_ids: Vec<String>,
    #[serde(default)]
    pub commit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalCandidate {
    pub kind: RetrievalCandidateKind,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub reason_code: String,
    pub confidence: f32,
    #[serde(default)]
    pub signal_scores: Vec<RetrievalSignalScore>,
    #[serde(default)]
    pub evidence: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFeatureSource {
    PlanCandidate,
    HistoricalEval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFeatureLabel {
    Unknown,
    Selected,
    Gold,
    Read,
    Edited,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateFeatureExport {
    pub export_id: Uuid,
    pub schema_version: u32,
    pub repo_id: String,
    pub task_hash: Option<String>,
    pub eval_range_id: Option<String>,
    pub export_source: CandidateFeatureSource,
    pub task_type: Option<TaskType>,
    pub target_agent: Option<String>,
    pub row_count: usize,
    pub created_at_unix_seconds: u64,
    #[serde(default)]
    pub rows: Vec<CandidateFeatureRow>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateFeatureRow {
    pub candidate_id: String,
    pub candidate_kind: RetrievalCandidateKind,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub rank: usize,
    pub selected_rank: Option<usize>,
    pub confidence: f32,
    pub reason_code: String,
    #[serde(default)]
    pub signal_scores: Vec<RetrievalSignalScore>,
    pub lexical_score: f32,
    pub semantic_score: f32,
    pub graph_score: f32,
    pub history_score: f32,
    pub test_score: f32,
    pub memory_score: f32,
    pub feedback_score: f32,
    pub graph_distance: Option<u32>,
    pub history_commit_count: u32,
    pub test_relation_confidence: Option<f32>,
    pub memory_count: u32,
    pub feedback_event_count: u32,
    #[serde(default)]
    pub labels: Vec<CandidateFeatureLabel>,
    pub label_scope: String,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackBudget {
    Brief,
    Standard,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackOption {
    pub budget: PackBudget,
    pub resource_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RiskFlag {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCardKind {
    Domain,
    Experience,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryFreshness {
    Fresh,
    Stale,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryReviewStatus {
    Deterministic,
    Pending,
    Approved,
    Rejected,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCard {
    pub id: String,
    pub kind: MemoryCardKind,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub source_links: Vec<String>,
    #[serde(default)]
    pub input_hashes: Vec<String>,
    pub freshness: MemoryFreshness,
    pub review_status: MemoryReviewStatus,
    pub disabled: bool,
    pub confidence: f32,
    pub reason: String,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectedMemory {
    pub card: MemoryCard,
    pub score: f32,
    pub reason: String,
    #[serde(default)]
    pub evidence: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheStatusKind {
    Hit,
    Miss,
    Rebuilt,
    WriteFailed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CacheStatus {
    pub status: CacheStatusKind,
    pub path: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatusKind {
    Written,
    Skipped,
    WriteFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TraceStatus {
    pub status: TraceStatusKind,
    pub path: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextPlan {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub confidence: f32,
    pub target_files: Vec<TargetFile>,
    pub related_tests: Vec<RelatedTest>,
    pub recommended_commands: Vec<Command>,
    pub pack_options: Vec<PackOption>,
    pub missing_info_questions: Vec<String>,
    pub risk_flags: Vec<RiskFlag>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub retrieval_candidates: Vec<RetrievalCandidate>,
    #[serde(default)]
    pub selected_memory: Vec<SelectedMemory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_trace: Option<QueryConstructionTrace>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryFacetKind {
    OriginalTask,
    ExplicitPath,
    CurrentDiffPath,
    Symbol,
    StackFrame,
    ErrorText,
    DomainPhrase,
    CommitClue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryFacet {
    pub kind: QueryFacetKind,
    pub value: String,
    pub origin: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetrieverQuerySet {
    #[serde(default)]
    pub lexical_terms: Vec<String>,
    #[serde(default)]
    pub semantic_phrases: Vec<String>,
    #[serde(default)]
    pub symbol_terms: Vec<String>,
    #[serde(default)]
    pub graph_seeds: Vec<String>,
    #[serde(default)]
    pub history_terms: Vec<String>,
    #[serde(default)]
    pub test_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FusionControlSummary {
    pub anchor_dominance: bool,
    pub exact_evidence_protected: bool,
    pub semantic_candidate_cap: usize,
    pub semantic_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryConstructionTrace {
    pub task_hash: String,
    #[serde(default)]
    pub facets: Vec<QueryFacet>,
    pub retriever_queries: RetrieverQuerySet,
    pub fusion_controls: FusionControlSummary,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackSection {
    pub title: String,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextPack {
    pub id: Uuid,
    pub task_id: Uuid,
    pub repo_id: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub target_agent: String,
    pub budget: PackBudget,
    pub sections: Vec<PackSection>,
    pub token_estimate: usize,
    pub confidence: f32,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InspectorContentKind {
    SourceFree,
    SourceBearing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InspectorSectionView {
    pub title: String,
    pub kind: String,
    pub content_kind: InspectorContentKind,
    pub source_bearing: bool,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InspectorTargetFileView {
    pub path: String,
    pub reason: String,
    pub line_range: Option<LineRange>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InspectorRelatedTestView {
    pub path: String,
    pub reason: String,
    pub command: Option<String>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InspectorCandidateView {
    pub id: String,
    pub kind: RetrievalCandidateKind,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub reason_code: String,
    pub confidence: f32,
    #[serde(default)]
    pub signal_scores: Vec<RetrievalSignalScore>,
    #[serde(default)]
    pub evidence: Vec<RetrievalEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InspectorMemoryView {
    pub id: String,
    pub kind: MemoryCardKind,
    pub title: String,
    pub freshness: MemoryFreshness,
    pub review_status: MemoryReviewStatus,
    pub disabled: bool,
    pub confidence: f32,
    pub score: f32,
    pub reason: String,
    #[serde(default)]
    pub source_links: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<RetrievalEvidence>,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackInspectorView {
    pub pack_id: Uuid,
    pub task_id: Uuid,
    pub repo_id: String,
    pub workspace_id: Option<String>,
    pub task_hash: String,
    pub task_type: TaskType,
    pub target_agent: String,
    pub budget: PackBudget,
    pub token_estimate: usize,
    pub confidence: f32,
    pub source_text_logged: bool,
    pub source_bearing_section_count: usize,
    pub privacy_status: PrivacyStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub sections: Vec<InspectorSectionView>,
    #[serde(default)]
    pub target_files: Vec<InspectorTargetFileView>,
    #[serde(default)]
    pub related_tests: Vec<InspectorRelatedTestView>,
    #[serde(default)]
    pub validation_commands: Vec<Command>,
    #[serde(default)]
    pub selected_memory: Vec<InspectorMemoryView>,
    #[serde(default)]
    pub retrieval_candidates: Vec<InspectorCandidateView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvalTrace {
    pub id: Uuid,
    pub repo_id: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub pack_id: Option<Uuid>,
    pub target_agent: String,
    pub budget: Option<PackBudget>,
    pub recommended_files: Vec<String>,
    pub recommended_tests: Vec<String>,
    pub recommended_commands: Vec<String>,
    pub created_at_unix_seconds: u64,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackOutcome {
    Passed,
    Failed,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionFeedbackEvent {
    pub id: Uuid,
    pub schema_version: u32,
    pub repo_id: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub pack_id: Option<Uuid>,
    pub target_agent: String,
    pub budget: Option<PackBudget>,
    pub outcome: FeedbackOutcome,
    #[serde(default)]
    pub recommended_files: Vec<String>,
    #[serde(default)]
    pub recommended_tests: Vec<String>,
    #[serde(default)]
    pub recommended_commands: Vec<String>,
    #[serde(default)]
    pub read_files: Vec<String>,
    #[serde(default)]
    pub edited_files: Vec<String>,
    #[serde(default)]
    pub tested_files: Vec<String>,
    #[serde(default)]
    pub tested_commands: Vec<String>,
    #[serde(default)]
    pub user_corrected_files: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at_unix_seconds: u64,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackSummary {
    pub repo_id: String,
    pub event_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub blocked_count: usize,
    pub unknown_count: usize,
    pub read_file_count: usize,
    pub edited_file_count: usize,
    pub tested_file_count: usize,
    pub user_corrected_file_count: usize,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyQualityReport {
    pub repo_id: String,
    pub event_count: usize,
    pub sample_warning: Option<String>,
    pub context_precision: f32,
    pub read_precision: f32,
    pub edit_recall_proxy: f32,
    pub validation_coverage: f32,
    pub correction_rate: f32,
    pub token_roi: Vec<PolicyTokenRoi>,
    pub repeated_missing_file_families: Vec<RepeatedMissingFileFamily>,
    pub signal_contributions: Vec<PolicySignalContribution>,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTokenRoi {
    pub budget: Option<PackBudget>,
    pub event_count: usize,
    pub useful_files_per_event: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepeatedMissingFileFamily {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicySignalContribution {
    pub signal: RetrievalSignalKind,
    pub event_count: usize,
    pub useful_file_hits: usize,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalHealthReport {
    pub repo_id: String,
    pub evaluated_commits: usize,
    pub feedback_events: usize,
    pub health_score: f32,
    #[serde(default)]
    pub metrics: Vec<RetrievalHealthMetric>,
    #[serde(default)]
    pub token_roi: Vec<RetrievalHealthTokenRoi>,
    #[serde(default)]
    pub signal_contributions: Vec<RetrievalHealthSignalContribution>,
    #[serde(default)]
    pub gap_families: Vec<RetrievalHealthGapFamily>,
    #[serde(default)]
    pub low_confidence_flags: Vec<String>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalHealthMetric {
    pub name: String,
    pub value: f32,
    pub unit: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalHealthTokenRoi {
    pub budget: Option<PackBudget>,
    pub source: String,
    pub event_count: usize,
    pub useful_files_per_event: f32,
    pub useful_targets_per_1k_tokens: f32,
    pub recall_at_cutoff: f32,
    pub larger_pack_adds_little_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalHealthSignalContribution {
    pub signal: RetrievalSignalKind,
    pub source: String,
    pub event_count: usize,
    pub useful_file_hits: usize,
    pub score: f32,
    pub recall_without_signal: Option<f32>,
    pub recall_lift_vs_lexical_at_k: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalHealthGapFamily {
    pub family: String,
    pub count: usize,
    pub recommendation_area: Option<String>,
    pub target_status: Option<String>,
    pub safe_path: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    File,
    Test,
    Memory,
    Feedback,
    Community,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeView {
    pub id: String,
    pub kind: GraphNodeKind,
    pub label: String,
    pub path: Option<String>,
    pub role: Option<FileRole>,
    pub weight: f32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdgeView {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub weight: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphCommunityView {
    pub id: String,
    pub label: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphNeighborhoodReport {
    pub repo_id: String,
    pub task_hash: Option<String>,
    #[serde(default)]
    pub anchors: Vec<String>,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub capped: bool,
    #[serde(default)]
    pub nodes: Vec<GraphNodeView>,
    #[serde(default)]
    pub edges: Vec<GraphEdgeView>,
    #[serde(default)]
    pub communities: Vec<GraphCommunityView>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticProviderStatusReport {
    pub repo_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub dimensions: usize,
    pub distance_metric: String,
    pub provider_role: String,
    pub quality_backend: bool,
    pub local_only: bool,
    pub provider_available: bool,
    pub provider_status: String,
    pub cache_location: String,
    pub degraded: bool,
    pub enabled_by_default: bool,
    pub cloud_embeddings_allowed: bool,
    pub cloud_reranking_allowed: bool,
    pub semantic_document_count: usize,
    pub semantic_facet_count: usize,
    pub precision_status: PrecisionStatusReport,
    pub local_vector_count: usize,
    pub stored_vector_count: usize,
    pub indexing_freshness: String,
    #[serde(default)]
    pub usage: Vec<SemanticUsageSummary>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrecisionStatus {
    Unavailable,
    Available,
    Stale,
    Invalid,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrecisionStatusReport {
    pub status: PrecisionStatus,
    pub provider: Option<String>,
    pub overlay_path: Option<String>,
    pub edge_count: usize,
    pub rejected_edge_count: usize,
    pub stale: bool,
    pub degraded: bool,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDocumentFacetKind {
    Metadata,
    Symbol,
    Dependency,
    RelatedTest,
    Doc,
    Precision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDocumentFacet {
    pub kind: SemanticDocumentFacetKind,
    pub label: String,
    pub value: String,
    pub path: Option<String>,
    pub line_range: Option<LineRange>,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDocument {
    pub id: String,
    pub path: String,
    pub role: FileRole,
    pub language: Option<String>,
    pub safe_hash: String,
    pub summary: String,
    #[serde(default)]
    pub facets: Vec<SemanticDocumentFacet>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDocumentReport {
    pub document_count: usize,
    pub facet_count: usize,
    #[serde(default)]
    pub documents: Vec<SemanticDocument>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
    pub precision_status: PrecisionStatusReport,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticUsageSummary {
    pub surface: String,
    pub semantic_enabled: bool,
    pub semantic_candidate_count: usize,
    pub remote_embeddings_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalPolicyExperimentReport {
    pub repo_id: String,
    pub task_hash: String,
    #[serde(default)]
    pub rows: Vec<RetrievalPolicyExperimentRow>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalPolicyExperimentRow {
    pub policy: String,
    pub semantic_enabled: bool,
    pub graph_enabled: bool,
    pub file_recall_at_10: Option<f32>,
    pub test_recall_at_10: Option<f32>,
    pub context_precision: Option<f32>,
    pub validation_coverage: Option<f32>,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentPreviewSurfaceKind {
    AgentsMd,
    McpTool,
    McpResource,
    NativeRule,
    NativeCommand,
    AdapterSnippet,
    CliFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreviewSurface {
    pub kind: AgentPreviewSurfaceKind,
    pub label: String,
    pub path: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreviewStep {
    pub order: usize,
    pub action: String,
    pub owner: String,
    pub source_bearing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreview {
    pub target_agent: String,
    pub display_name: String,
    pub pack_resource_uri: String,
    #[serde(default)]
    pub mcp_tools: Vec<String>,
    #[serde(default)]
    pub mcp_resources: Vec<String>,
    #[serde(default)]
    pub guidance: Vec<AgentPreviewSurface>,
    #[serde(default)]
    pub native_rules: Vec<AgentPreviewSurface>,
    #[serde(default)]
    pub next_steps: Vec<AgentPreviewStep>,
    #[serde(default)]
    pub boundary: Vec<String>,
    pub source_text_included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreviewReport {
    pub repo_id: String,
    pub task_hash: String,
    pub task_type: TaskType,
    pub budget: PackBudget,
    pub source_text_logged: bool,
    pub privacy_status: PrivacyStatus,
    #[serde(default)]
    pub previews: Vec<AgentPreview>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalPolicyProfile {
    pub id: String,
    pub status: PolicyProfileStatus,
    #[serde(default = "default_policy_profile_schema_version")]
    pub profile_schema_version: u32,
    pub created_at_unix_seconds: u64,
    pub source_report_event_count: usize,
    #[serde(default)]
    pub training_corpus_id: Option<String>,
    #[serde(default)]
    pub training_sources: Vec<PolicyTrainingSource>,
    #[serde(default)]
    pub metric_summary: Vec<PolicyMetricSummary>,
    pub rationale: String,
    pub weights: Vec<PolicySignalWeight>,
    pub safety_floors: Vec<PolicySafetyFloor>,
    pub regression_warnings: Vec<String>,
    #[serde(default)]
    pub baseline_thresholds: Vec<PolicyBaselineThreshold>,
    #[serde(default = "default_policy_default_eligible")]
    pub default_eligible: bool,
    pub rollback_profile_id: Option<String>,
    pub source_text_logged: bool,
}

fn default_policy_profile_schema_version() -> u32 {
    2
}

fn default_policy_default_eligible() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyProfileStatus {
    Candidate,
    Active,
    Disabled,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicySignalWeight {
    pub signal: RetrievalSignalKind,
    pub weight: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicySafetyFloor {
    pub signal: RetrievalSignalKind,
    pub minimum_weight: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTrainingSource {
    pub source_kind: String,
    pub source_id: Option<String>,
    pub schema_version: Option<String>,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyMetricSummary {
    pub metric: String,
    pub value: f32,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyBaselineThreshold {
    pub metric: String,
    pub value: f32,
    pub threshold: f32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyProfileActionReport {
    pub repo_id: String,
    pub profile_id: String,
    pub action: String,
    pub active_profile_id: Option<String>,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceManifest {
    pub schema_version: u32,
    pub workspace_id: Option<String>,
    pub repos: Vec<WorkspaceRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepo {
    pub id: Option<String>,
    pub path: String,
    pub label: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRepoState {
    Available,
    Missing,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepoPrivacyStatus {
    pub local_only: bool,
    pub source_text_logged: bool,
    pub source_text_persisted: bool,
    pub remote_sync_used: bool,
}

impl WorkspaceRepoPrivacyStatus {
    pub fn local_source_free() -> Self {
        Self {
            local_only: true,
            source_text_logged: false,
            source_text_persisted: false,
            remote_sync_used: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepoDiagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub repo_id: Option<String>,
    pub label: Option<String>,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepoStatus {
    pub repo_id: String,
    pub label: String,
    pub path_label: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub state: WorkspaceRepoState,
    pub git_root: bool,
    pub file_count: usize,
    pub ignored_count: usize,
    pub generated_count: usize,
    pub sensitive_count: usize,
    pub storage_compatibility: Option<String>,
    pub storage_database_present: bool,
    pub memory_card_count: usize,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInventoryReport {
    pub schema_version: u32,
    pub workspace_root: String,
    pub manifest_path: String,
    pub repo_count: usize,
    pub available_repo_count: usize,
    pub file_count: usize,
    pub ignored_count: usize,
    pub generated_count: usize,
    pub sensitive_count: usize,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    pub repos: Vec<WorkspaceRepoStatus>,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepoPlan {
    pub repo_id: String,
    pub label: String,
    pub path_label: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub confidence: f32,
    pub reason: String,
    pub context_plan: ContextPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextPlan {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub confidence: f32,
    pub workspace_root: String,
    pub manifest_path: String,
    pub selected_repo_count: usize,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    pub repo_plans: Vec<WorkspaceRepoPlan>,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepoPack {
    pub repo_id: String,
    pub label: String,
    pub path_label: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub confidence: f32,
    pub reason: String,
    pub context_pack: ContextPack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextPack {
    pub id: Uuid,
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub target_agent: String,
    pub budget: PackBudget,
    pub confidence: f32,
    pub token_estimate: usize,
    pub workspace_root: String,
    pub manifest_path: String,
    pub selected_repo_count: usize,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub repo_packs: Vec<WorkspaceRepoPack>,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SharedArtifactKind {
    ContextCards,
    BenchmarkReport,
    PolicyProfile,
    FeedbackSummary,
    ProofReport,
    WorkspaceManifest,
    TeamPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SharedArtifactStatus {
    Present,
    Missing,
    Imported,
    Disabled,
    Regenerated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedArtifactEntry {
    pub id: String,
    pub kind: SharedArtifactKind,
    pub status: SharedArtifactStatus,
    pub path_label: String,
    pub content_hash: Option<String>,
    pub size_bytes: u64,
    pub generated_at_unix_seconds: u64,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedArtifactManifest {
    pub schema_version: u32,
    pub repo_id: String,
    pub repo_label: String,
    pub exported_at_unix_seconds: u64,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    pub artifacts: Vec<SharedArtifactEntry>,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedArtifactInspectionReport {
    pub manifest_path: String,
    pub artifact_count: usize,
    pub compatible: bool,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    pub artifacts: Vec<SharedArtifactEntry>,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeamPrivacyPolicy {
    pub schema_version: u32,
    pub name: String,
    pub allow_workspace_indexing: bool,
    pub allow_artifact_export: bool,
    pub allow_cloud_embeddings: bool,
    pub allow_cloud_reranking: bool,
    pub allow_source_snippets_in_shared_artifacts: bool,
    pub redact_secrets: bool,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeamPolicyReport {
    pub policy_path: String,
    pub policy: TeamPrivacyPolicy,
    pub allowed_artifacts: Vec<SharedArtifactKind>,
    pub blocked_artifacts: Vec<SharedArtifactKind>,
    pub degraded_artifacts: Vec<SharedArtifactKind>,
    pub redacted_artifacts: Vec<SharedArtifactKind>,
    pub source_text_logged: bool,
    pub privacy_status: WorkspaceRepoPrivacyStatus,
    #[serde(default)]
    pub diagnostics: Vec<WorkspaceRepoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentOutcomeComparisonReport {
    pub repo_id: String,
    pub event_count: usize,
    pub sample_warning: Option<String>,
    pub budgets: Vec<BudgetOutcome>,
    pub changed_sample_warning: bool,
    pub low_information_warning: bool,
    pub source_text_logged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BudgetOutcome {
    pub budget: Option<PackBudget>,
    pub event_count: usize,
    pub pass_rate: f32,
    pub blocked_rate: f32,
    pub correction_rate: f32,
    pub validation_coverage: f32,
    pub average_recommended_context_size: f32,
    pub useful_target_files_per_1k_tokens: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileRole;

    #[test]
    fn task_type_serializes_as_snake_case() {
        let json = serde_json::to_string(&TaskType::BugFix).unwrap();
        assert_eq!(json, "\"bug_fix\"");
    }

    #[test]
    fn context_plan_public_json_shape_is_stable() {
        let plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 1.0,
            target_files: vec![TargetFile {
                path: "src/lib.rs".to_string(),
                reason: "public API surface".to_string(),
                line_range: Some(LineRange { start: 1, end: 7 }),
                confidence: 0.5,
                attribution: Vec::new(),
            }],
            related_tests: vec![],
            recommended_commands: vec![],
            pack_options: vec![PackOption {
                budget: PackBudget::Brief,
                resource_uri: "ctxpack://packs/brief".to_string(),
            }],
            missing_info_questions: vec![],
            risk_flags: vec![],
            diagnostics: vec![Diagnostic {
                code: "source_policy_excluded".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: "Skipped policy-excluded source file".to_string(),
                paths: vec![".env".to_string()],
                count: 1,
            }],
            retrieval_candidates: Vec::new(),
            selected_memory: Vec::new(),
            query_trace: None,
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&plan).unwrap();
        let expected = serde_json::json!({
            "taskId": "00000000-0000-0000-0000-000000000000",
            "taskType": "bug_fix",
            "confidence": 1.0,
            "targetFiles": [{
                "path": "src/lib.rs",
                "reason": "public API surface",
                "lineRange": {
                    "start": 1,
                    "end": 7
                },
                "confidence": 0.5,
                "attribution": []
            }],
            "relatedTests": [],
            "recommendedCommands": [],
            "packOptions": [{
                "budget": "brief",
                "resourceUri": "ctxpack://packs/brief"
            }],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "diagnostics": [{
                "code": "source_policy_excluded",
                "severity": "warning",
                "message": "Skipped policy-excluded source file",
                "paths": [".env"],
                "count": 1
            }],
            "retrievalCandidates": [],
            "selectedMemory": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });

        assert_eq!(value, expected);

        let object = value.as_object().unwrap();
        for key in [
            "taskId",
            "taskType",
            "confidence",
            "targetFiles",
            "relatedTests",
            "recommendedCommands",
            "packOptions",
            "missingInfoQuestions",
            "riskFlags",
            "diagnostics",
            "retrievalCandidates",
            "selectedMemory",
            "privacyStatus",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskType"], "bug_fix");
        assert!(value["targetFiles"].is_array());
        assert_eq!(value["targetFiles"][0]["lineRange"]["start"], 1);
        assert_eq!(value["packOptions"][0]["budget"], "brief");
        assert_eq!(
            value["packOptions"][0]["resourceUri"],
            "ctxpack://packs/brief"
        );
        assert_eq!(value["diagnostics"][0]["severity"], "warning");
        assert_eq!(value["diagnostics"][0]["paths"][0], ".env");
        assert_eq!(value["privacyStatus"]["localOnly"], true);

        assert!(value.get("task_id").is_none());
        assert!(value.get("task_type").is_none());
        assert!(value.get("target_files").is_none());
        assert!(value.get("related_tests").is_none());
        assert!(value.get("risk_flags").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("prompt").is_none());
        assert!(value.get("privacy_status").is_none());
    }

    #[test]
    fn retrieval_contracts_serialize_additive_camel_case_fields() {
        let attribution = vec![RetrievalEvidence {
            signal: RetrievalSignalKind::Lexical,
            score: 0.8,
            reason_code: "lexical_match".to_string(),
            path: Some("src/lib.rs".to_string()),
            role: Some(FileRole::Source),
            edge_label: Some("imports".to_string()),
            commit_ids: vec!["abc1234".to_string()],
            commit_count: 1,
        }];
        let plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 1.0,
            target_files: vec![TargetFile {
                path: "src/lib.rs".to_string(),
                reason: "public API surface".to_string(),
                line_range: None,
                confidence: 0.8,
                attribution: attribution.clone(),
            }],
            related_tests: vec![RelatedTest {
                path: "tests/lib_test.rs".to_string(),
                reason: "related test".to_string(),
                command: Some("cargo test".to_string()),
                confidence: 0.7,
                attribution: attribution.clone(),
            }],
            recommended_commands: vec![],
            pack_options: vec![],
            missing_info_questions: vec![],
            risk_flags: vec![],
            diagnostics: vec![],
            retrieval_candidates: vec![RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some("src/lib.rs".to_string()),
                role: Some(FileRole::Source),
                reason_code: "lexical_match".to_string(),
                confidence: 0.8,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Lexical,
                    score: 0.8,
                    weight: 1.0,
                }],
                evidence: attribution,
            }],
            selected_memory: Vec::new(),
            query_trace: None,
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&plan).unwrap();

        assert!(value.get("retrievalCandidates").is_some());
        assert!(value["targetFiles"][0].get("attribution").is_some());
        assert!(value["relatedTests"][0].get("attribution").is_some());
        assert_eq!(value["retrievalCandidates"][0]["kind"], "file");
        assert_eq!(
            value["retrievalCandidates"][0]["signalScores"][0]["signal"],
            "lexical"
        );
        assert_eq!(
            value["targetFiles"][0]["attribution"][0]["reasonCode"],
            "lexical_match"
        );
    }

    #[test]
    fn semantic_document_contracts_are_camel_case_and_source_free() {
        let report = SemanticDocumentReport {
            document_count: 1,
            facet_count: 2,
            documents: vec![SemanticDocument {
                id: "sem_doc_1".to_string(),
                path: "src/auth/session.ts".to_string(),
                role: FileRole::Source,
                language: Some("typescript".to_string()),
                safe_hash: "abc123".to_string(),
                summary: "source-free auth session document".to_string(),
                facets: vec![
                    SemanticDocumentFacet {
                        kind: SemanticDocumentFacetKind::Symbol,
                        label: "function".to_string(),
                        value: "getSession(req): Promise<Session>".to_string(),
                        path: Some("src/auth/session.ts".to_string()),
                        line_range: Some(LineRange { start: 4, end: 4 }),
                        weight: 1.0,
                    },
                    SemanticDocumentFacet {
                        kind: SemanticDocumentFacetKind::Precision,
                        label: "precision:references".to_string(),
                        value: "source-free precision edge for getSession".to_string(),
                        path: Some("tests/auth/session.test.ts".to_string()),
                        line_range: None,
                        weight: 0.95,
                    },
                ],
                source_text_logged: false,
                privacy_status: PrivacyStatus::local_only(),
            }],
            diagnostics: Vec::new(),
            cache_status: CacheStatus {
                status: CacheStatusKind::Hit,
                path: Some(".ctxpack/index.json".to_string()),
                diagnostics: Vec::new(),
            },
            precision_status: PrecisionStatusReport {
                status: PrecisionStatus::Available,
                provider: Some("fixture_scip".to_string()),
                overlay_path: Some(".ctxpack/precision-edges.json".to_string()),
                edge_count: 1,
                rejected_edge_count: 0,
                stale: false,
                degraded: false,
                diagnostics: Vec::new(),
            },
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&report).unwrap();

        assert_eq!(value["documentCount"], 1);
        assert_eq!(value["facetCount"], 2);
        assert_eq!(value["documents"][0]["sourceTextLogged"], false);
        assert_eq!(value["documents"][0]["facets"][0]["lineRange"]["start"], 4);
        assert_eq!(value["precisionStatus"]["status"], "available");
        assert_eq!(value["privacyStatus"]["localOnly"], true);
        assert!(value.get("document_count").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.to_string().contains("getSession"));
        assert!(!value.to_string().contains("return "));
    }

    #[test]
    fn query_trace_contract_is_source_free_and_camel_case() {
        let trace = QueryConstructionTrace {
            task_hash: "hash123".to_string(),
            facets: vec![
                QueryFacet {
                    kind: QueryFacetKind::ExplicitPath,
                    value: "src/auth/session.ts".to_string(),
                    origin: "task_path".to_string(),
                    weight: 1.0,
                },
                QueryFacet {
                    kind: QueryFacetKind::Symbol,
                    value: "getSession".to_string(),
                    origin: "task_symbol".to_string(),
                    weight: 0.9,
                },
            ],
            retriever_queries: RetrieverQuerySet {
                lexical_terms: vec!["src/auth/session.ts".to_string(), "getSession".to_string()],
                semantic_phrases: vec!["getSession".to_string()],
                symbol_terms: vec!["getSession".to_string()],
                graph_seeds: vec!["src/auth/session.ts".to_string()],
                history_terms: Vec::new(),
                test_terms: vec!["src/auth/session.ts".to_string()],
            },
            fusion_controls: FusionControlSummary {
                anchor_dominance: true,
                exact_evidence_protected: true,
                semantic_candidate_cap: 8,
                semantic_weight: 0.7,
            },
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&trace).unwrap();

        assert_eq!(value["taskHash"], "hash123");
        assert_eq!(value["facets"][0]["kind"], "explicit_path");
        assert_eq!(value["retrieverQueries"]["symbolTerms"][0], "getSession");
        assert_eq!(value["fusionControls"]["anchorDominance"], true);
        assert_eq!(value["sourceTextLogged"], false);
        assert_eq!(value["privacyStatus"]["localOnly"], true);
        assert!(value.get("task_hash").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(!value.to_string().contains("return "));
    }

    #[test]
    fn retrieval_additive_fields_default_when_missing_from_old_json() {
        let old_json = serde_json::json!({
            "taskId": "00000000-0000-0000-0000-000000000000",
            "taskType": "bug_fix",
            "confidence": 1.0,
            "targetFiles": [{
                "path": "src/lib.rs",
                "reason": "public API surface",
                "lineRange": null,
                "confidence": 0.5
            }],
            "relatedTests": [{
                "path": "tests/lib_test.rs",
                "reason": "related test",
                "command": "cargo test",
                "confidence": 0.5
            }],
            "recommendedCommands": [],
            "packOptions": [],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "diagnostics": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });

        let plan: ContextPlan = serde_json::from_value(old_json).unwrap();

        assert!(plan.retrieval_candidates.is_empty());
        assert!(plan.selected_memory.is_empty());
        assert!(plan.target_files[0].attribution.is_empty());
        assert!(plan.related_tests[0].attribution.is_empty());
    }

    #[test]
    fn retrieval_attribution_serializes_without_source_or_prompt_text_fields() {
        let evidence = RetrievalEvidence {
            signal: RetrievalSignalKind::CoChange,
            score: 0.6,
            reason_code: "changed_together".to_string(),
            path: Some("src/lib.rs".to_string()),
            role: Some(FileRole::Source),
            edge_label: None,
            commit_ids: vec!["abc1234".to_string()],
            commit_count: 3,
        };

        let serialized = serde_json::to_string(&evidence).unwrap();

        for forbidden in [
            "taskText",
            "task_text",
            "sourceSnippet",
            "source_snippet",
            "symbolSignature",
            "symbol_signature",
            "commitSubject",
            "commit_subject",
            "prompt",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "attribution leaked forbidden field {forbidden}: {serialized}"
            );
        }
    }

    #[test]
    fn retrieval_candidate_kind_serializes_all_phase_three_required_kinds() {
        let cases = [
            (RetrievalCandidateKind::File, "file"),
            (RetrievalCandidateKind::Test, "test"),
            (RetrievalCandidateKind::Symbol, "symbol"),
            (RetrievalCandidateKind::Doc, "doc"),
            (RetrievalCandidateKind::Commit, "commit"),
            (RetrievalCandidateKind::Config, "config"),
            (RetrievalCandidateKind::Memory, "memory"),
        ];

        for (kind, expected) in cases {
            assert_eq!(serde_json::to_value(kind).unwrap(), expected);
        }
    }

    #[test]
    fn memory_contracts_are_source_free_and_camel_case() {
        let card = MemoryCard {
            id: "domain:auth".to_string(),
            kind: MemoryCardKind::Domain,
            title: "Auth".to_string(),
            summary: "Auth requests depend on session and middleware files.".to_string(),
            source_links: vec!["src/auth/session.ts".to_string()],
            input_hashes: vec!["hash-1".to_string()],
            freshness: MemoryFreshness::Fresh,
            review_status: MemoryReviewStatus::Deterministic,
            disabled: false,
            confidence: 0.82,
            reason: "Generated from safe inventory metadata.".to_string(),
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(SelectedMemory {
            card,
            score: 0.8,
            reason: "task overlaps source links".to_string(),
            evidence: vec![RetrievalEvidence {
                signal: RetrievalSignalKind::Memory,
                score: 0.8,
                reason_code: "memory_task_overlap".to_string(),
                path: Some("src/auth/session.ts".to_string()),
                role: Some(FileRole::Source),
                edge_label: None,
                commit_ids: Vec::new(),
                commit_count: 0,
            }],
        })
        .unwrap();

        assert_eq!(value["card"]["kind"], "domain");
        assert_eq!(value["card"]["freshness"], "fresh");
        assert_eq!(value["card"]["reviewStatus"], "deterministic");
        assert_eq!(value["card"]["sourceLinks"][0], "src/auth/session.ts");
        assert_eq!(value["evidence"][0]["signal"], "memory");
        for forbidden in ["sourceText", "prompt", "rawTranscript", "logOutput"] {
            assert!(value.get(forbidden).is_none());
            assert!(value["card"].get(forbidden).is_none());
        }
    }

    #[test]
    fn context_pack_public_json_shape_is_stable() {
        let pack = ContextPack {
            id: Uuid::nil(),
            task_id: Uuid::nil(),
            repo_id: "repo-1".to_string(),
            task_hash: "hash-1".to_string(),
            task_type: TaskType::BugFix,
            target_agent: "codex".to_string(),
            budget: PackBudget::Brief,
            sections: vec![PackSection {
                title: "Task".to_string(),
                kind: "task".to_string(),
                content: "Fix auth redirect".to_string(),
            }],
            token_estimate: 12,
            confidence: 0.7,
            warnings: vec!["one warning".to_string()],
            diagnostics: vec![Diagnostic {
                code: "source_unreadable".to_string(),
                severity: DiagnosticSeverity::Error,
                message: "Could not read requested source file".to_string(),
                paths: vec!["src/lib.rs".to_string()],
                count: 1,
            }],
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&pack).unwrap();

        let object = value.as_object().unwrap();
        for key in [
            "id",
            "taskId",
            "repoId",
            "taskHash",
            "taskType",
            "targetAgent",
            "budget",
            "sections",
            "tokenEstimate",
            "confidence",
            "warnings",
            "diagnostics",
            "privacyStatus",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["id"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["repoId"], "repo-1");
        assert_eq!(value["taskHash"], "hash-1");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["targetAgent"], "codex");
        assert_eq!(value["budget"], "brief");
        assert_eq!(value["sections"][0]["title"], "Task");
        assert_eq!(value["tokenEstimate"], 12);
        assert_eq!(value["diagnostics"][0]["severity"], "error");
        assert_eq!(value["diagnostics"][0]["paths"][0], "src/lib.rs");
        assert_eq!(value["privacyStatus"]["localOnly"], true);

        assert!(value.get("task_id").is_none());
        assert!(value.get("repo_id").is_none());
        assert!(value.get("task_hash").is_none());
        assert!(value.get("target_agent").is_none());
        assert!(value.get("token_estimate").is_none());
        assert!(value.get("riskFlags").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("task").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("prompt").is_none());
    }

    #[test]
    fn diagnostics_public_json_shape_is_source_free_and_backward_compatible() {
        let diagnostic = Diagnostic {
            code: "cache_write_failed".to_string(),
            severity: DiagnosticSeverity::Info,
            message: "Inventory cache was not persisted".to_string(),
            paths: vec![".ctxpack/repos/repo-1/inventory.json".to_string()],
            count: 1,
        };

        let value = serde_json::to_value(&diagnostic).unwrap();
        let object = value.as_object().unwrap();
        for key in ["code", "severity", "message", "paths", "count"] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["severity"], "info");
        assert!(value.get("source").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("snippet").is_none());
        assert!(value.get("prompt").is_none());

        let old_plan_json = serde_json::json!({
            "taskId": "00000000-0000-0000-0000-000000000000",
            "taskType": "bug_fix",
            "confidence": 1.0,
            "targetFiles": [],
            "relatedTests": [],
            "recommendedCommands": [],
            "packOptions": [],
            "missingInfoQuestions": [],
            "riskFlags": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });
        let plan: ContextPlan = serde_json::from_value(old_plan_json).unwrap();
        assert!(plan.diagnostics.is_empty());

        let old_pack_json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "taskId": "00000000-0000-0000-0000-000000000000",
            "repoId": "repo-1",
            "taskHash": "hash-1",
            "taskType": "bug_fix",
            "targetAgent": "codex",
            "budget": "brief",
            "sections": [],
            "tokenEstimate": 0,
            "confidence": 1.0,
            "warnings": [],
            "privacyStatus": {
                "localOnly": true,
                "remoteEmbeddingsUsed": false,
                "remoteRerankingUsed": false,
                "redactionsApplied": 0
            }
        });
        let pack: ContextPack = serde_json::from_value(old_pack_json).unwrap();
        assert!(pack.diagnostics.is_empty());
    }

    #[test]
    fn eval_trace_public_json_shape_is_source_free() {
        let trace = EvalTrace {
            id: Uuid::nil(),
            repo_id: "repo-1".to_string(),
            task_hash: "hash-1".to_string(),
            task_type: TaskType::BugFix,
            pack_id: Some(Uuid::nil()),
            target_agent: "codex".to_string(),
            budget: Some(PackBudget::Brief),
            recommended_files: vec!["src/auth.ts".to_string()],
            recommended_tests: vec!["tests/auth.test.ts".to_string()],
            recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };

        let value = serde_json::to_value(&trace).unwrap();

        let object = value.as_object().unwrap();
        for key in [
            "id",
            "repoId",
            "taskHash",
            "taskType",
            "packId",
            "targetAgent",
            "budget",
            "recommendedFiles",
            "recommendedTests",
            "recommendedCommands",
            "createdAtUnixSeconds",
            "sourceTextLogged",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["repoId"], "repo-1");
        assert_eq!(value["taskHash"], "hash-1");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["packId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["sourceTextLogged"], false);
        assert!(value.get("repo_id").is_none());
        assert!(value.get("task_hash").is_none());
        assert!(value.get("target_agent").is_none());
        assert!(value.get("task").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("source_text").is_none());
        assert!(value.get("source_text_logged").is_none());
    }

    #[test]
    fn feedback_event_public_json_shape_is_source_free() {
        let event = SessionFeedbackEvent {
            id: Uuid::nil(),
            schema_version: 1,
            repo_id: "repo-1".to_string(),
            task_hash: "hash-1".to_string(),
            task_type: TaskType::BugFix,
            pack_id: Some(Uuid::nil()),
            target_agent: "codex".to_string(),
            budget: Some(PackBudget::Brief),
            outcome: FeedbackOutcome::Passed,
            recommended_files: vec!["src/auth.ts".to_string()],
            recommended_tests: vec!["tests/auth.test.ts".to_string()],
            recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            read_files: vec!["src/auth.ts".to_string()],
            edited_files: vec!["src/auth.ts".to_string()],
            tested_files: vec!["tests/auth.test.ts".to_string()],
            tested_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
            user_corrected_files: Vec::new(),
            tags: vec!["accepted_fix".to_string()],
            created_at_unix_seconds: 1,
            source_text_logged: false,
        };

        let value = serde_json::to_value(&event).unwrap();
        let object = value.as_object().unwrap();
        for key in [
            "id",
            "schemaVersion",
            "repoId",
            "taskHash",
            "taskType",
            "packId",
            "targetAgent",
            "budget",
            "outcome",
            "recommendedFiles",
            "recommendedTests",
            "recommendedCommands",
            "readFiles",
            "editedFiles",
            "testedFiles",
            "testedCommands",
            "userCorrectedFiles",
            "tags",
            "createdAtUnixSeconds",
            "sourceTextLogged",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["outcome"], "passed");
        assert_eq!(value["sourceTextLogged"], false);
        assert!(value.get("task").is_none());
        assert!(value.get("prompt").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("terminalLog").is_none());
        assert!(value.get("repo_id").is_none());
        assert!(value.get("task_hash").is_none());
    }

    #[test]
    fn workspace_contracts_are_camel_case_and_source_free() {
        let repo_status = WorkspaceRepoStatus {
            repo_id: "repo-a".to_string(),
            label: "web".to_string(),
            path_label: "web".to_string(),
            tags: vec!["frontend".to_string()],
            state: WorkspaceRepoState::Available,
            git_root: true,
            file_count: 3,
            ignored_count: 0,
            generated_count: 1,
            sensitive_count: 1,
            storage_compatibility: Some("compatible".to_string()),
            storage_database_present: true,
            memory_card_count: 2,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            diagnostics: vec![WorkspaceRepoDiagnostic {
                code: "workspace_repo_label_sensitive".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: "Label looks like a sensitive path name.".to_string(),
                repo_id: Some("repo-a".to_string()),
                label: Some("web".to_string()),
                paths: vec!["web".to_string()],
            }],
        };
        let report = WorkspaceInventoryReport {
            schema_version: 1,
            workspace_root: "/tmp/workspace".to_string(),
            manifest_path: "/tmp/workspace/.ctxpack/workspace.json".to_string(),
            repo_count: 1,
            available_repo_count: 1,
            file_count: 3,
            ignored_count: 0,
            generated_count: 1,
            sensitive_count: 1,
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            repos: vec![repo_status],
            diagnostics: Vec::new(),
        };

        let value = serde_json::to_value(&report).unwrap();
        let object = value.as_object().unwrap();
        for key in [
            "schemaVersion",
            "workspaceRoot",
            "manifestPath",
            "repoCount",
            "availableRepoCount",
            "fileCount",
            "ignoredCount",
            "generatedCount",
            "sensitiveCount",
            "sourceTextLogged",
            "privacyStatus",
            "repos",
            "diagnostics",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }

        assert_eq!(value["sourceTextLogged"], false);
        assert_eq!(value["repos"][0]["state"], "available");
        assert_eq!(value["repos"][0]["storageCompatibility"], "compatible");
        assert_eq!(value["repos"][0]["privacyStatus"]["localOnly"], true);
        assert!(value.get("schema_version").is_none());
        assert!(value.get("workspace_root").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("prompt").is_none());
        assert!(value.get("terminalLog").is_none());

        let serialized = serde_json::to_string(&report).unwrap();
        for forbidden in [
            "sourceText\":\"",
            "sourceTextLogged\":true",
            "prompt",
            "terminalLog",
            "transcript",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "workspace contract leaked forbidden field/value {forbidden}: {serialized}"
            );
        }
    }

    #[test]
    fn workspace_context_plan_contract_is_source_free() {
        let context_plan = ContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 0.7,
            target_files: Vec::new(),
            related_tests: Vec::new(),
            recommended_commands: Vec::new(),
            pack_options: Vec::new(),
            missing_info_questions: Vec::new(),
            risk_flags: Vec::new(),
            diagnostics: Vec::new(),
            retrieval_candidates: Vec::new(),
            selected_memory: Vec::new(),
            query_trace: None,
            privacy_status: PrivacyStatus::local_only(),
        };
        let workspace_plan = WorkspaceContextPlan {
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            confidence: 0.8,
            workspace_root: "/tmp/workspace".to_string(),
            manifest_path: "/tmp/workspace/.ctxpack/workspace.json".to_string(),
            selected_repo_count: 1,
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            repo_plans: vec![WorkspaceRepoPlan {
                repo_id: "repo-a".to_string(),
                label: "api".to_string(),
                path_label: "api".to_string(),
                tags: vec!["backend".to_string()],
                confidence: 0.8,
                reason: "Task matched 2 target file(s) and 1 related test(s).".to_string(),
                context_plan,
            }],
            diagnostics: Vec::new(),
        };

        let value = serde_json::to_value(&workspace_plan).unwrap();
        assert_eq!(value["taskId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(value["taskType"], "bug_fix");
        assert_eq!(value["selectedRepoCount"], 1);
        assert_eq!(value["sourceTextLogged"], false);
        assert_eq!(value["repoPlans"][0]["repoId"], "repo-a");
        assert!(value.get("repo_plans").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("sourceText").is_none());
        assert!(value.get("prompt").is_none());
        assert!(value.get("terminalLog").is_none());
    }

    #[test]
    fn workspace_context_pack_contract_keeps_repo_boundaries() {
        let context_pack = ContextPack {
            id: Uuid::nil(),
            task_id: Uuid::nil(),
            repo_id: "repo-a".to_string(),
            task_hash: "task-hash".to_string(),
            task_type: TaskType::BugFix,
            target_agent: "codex".to_string(),
            budget: PackBudget::Brief,
            sections: vec![PackSection {
                title: "Validation".to_string(),
                kind: "validation".to_string(),
                content: "Run targeted tests.".to_string(),
            }],
            token_estimate: 12,
            confidence: 0.8,
            warnings: Vec::new(),
            diagnostics: Vec::new(),
            privacy_status: PrivacyStatus::local_only(),
        };
        let workspace_pack = WorkspaceContextPack {
            id: Uuid::nil(),
            task_id: Uuid::nil(),
            task_type: TaskType::BugFix,
            target_agent: "codex".to_string(),
            budget: PackBudget::Brief,
            confidence: 0.8,
            token_estimate: 12,
            workspace_root: "/tmp/workspace".to_string(),
            manifest_path: "/tmp/workspace/.ctxpack/workspace.json".to_string(),
            selected_repo_count: 1,
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            warnings: vec!["Workspace pack limited to 1 repository.".to_string()],
            repo_packs: vec![WorkspaceRepoPack {
                repo_id: "repo-a".to_string(),
                label: "api".to_string(),
                path_label: "api".to_string(),
                tags: Vec::new(),
                confidence: 0.8,
                reason: "Task matched target files.".to_string(),
                context_pack,
            }],
            diagnostics: Vec::new(),
        };

        let value = serde_json::to_value(&workspace_pack).unwrap();
        assert_eq!(value["repoPacks"][0]["repoId"], "repo-a");
        assert_eq!(value["repoPacks"][0]["contextPack"]["repoId"], "repo-a");
        assert_eq!(value["sourceTextLogged"], false);
        assert_eq!(value["privacyStatus"]["localOnly"], true);
        assert!(value.get("repo_packs").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("prompt").is_none());
        assert!(value.get("terminalLog").is_none());
    }

    #[test]
    fn shared_artifact_and_team_policy_contracts_are_source_free() {
        let artifact = SharedArtifactEntry {
            id: "feedback-summary".to_string(),
            kind: SharedArtifactKind::FeedbackSummary,
            status: SharedArtifactStatus::Present,
            path_label: ".ctxpack/feedback-summary.json".to_string(),
            content_hash: Some("hash".to_string()),
            size_bytes: 12,
            generated_at_unix_seconds: 1,
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            diagnostics: Vec::new(),
        };
        let manifest = SharedArtifactManifest {
            schema_version: 1,
            repo_id: "repo-a".to_string(),
            repo_label: "api".to_string(),
            exported_at_unix_seconds: 1,
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            artifacts: vec![artifact],
            diagnostics: Vec::new(),
        };
        let policy = TeamPrivacyPolicy {
            schema_version: 1,
            name: "local-source-free".to_string(),
            allow_workspace_indexing: true,
            allow_artifact_export: true,
            allow_cloud_embeddings: false,
            allow_cloud_reranking: false,
            allow_source_snippets_in_shared_artifacts: false,
            redact_secrets: true,
            source_text_logged: false,
        };
        let report = TeamPolicyReport {
            policy_path: ".ctxpack/team-policy.json".to_string(),
            policy,
            allowed_artifacts: vec![SharedArtifactKind::FeedbackSummary],
            blocked_artifacts: vec![SharedArtifactKind::ProofReport],
            degraded_artifacts: Vec::new(),
            redacted_artifacts: vec![SharedArtifactKind::ContextCards],
            source_text_logged: false,
            privacy_status: WorkspaceRepoPrivacyStatus::local_source_free(),
            diagnostics: Vec::new(),
        };

        let manifest_value = serde_json::to_value(&manifest).unwrap();
        let report_value = serde_json::to_value(&report).unwrap();
        assert_eq!(manifest_value["sourceTextLogged"], false);
        assert_eq!(manifest_value["artifacts"][0]["kind"], "feedback_summary");
        assert_eq!(report_value["policy"]["allowCloudEmbeddings"], false);
        assert_eq!(report_value["sourceTextLogged"], false);
        for value in [manifest_value, report_value] {
            assert!(value.get("source").is_none());
            assert!(value.get("sourceText").is_none());
            assert!(value.get("prompt").is_none());
            assert!(value.get("terminalLog").is_none());
            assert!(value.get("transcript").is_none());
        }
    }
}
