use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctxpack_compiler::{
    build_agent_preview_report, build_graph_neighborhood_report, build_product_proof_report,
    build_retrieval_health_report, compare_benchmark_suite_reports,
    compile_context_pack_with_plan_and_paths_for_agent_and_semantic, compile_pack_inspector_view,
    compile_workspace_context_pack, eval_trace_for_pack, eval_trace_for_plan,
    evaluate_historical_commits, generate_context_cards, generate_experience_cards,
    load_benchmark_suite_config, load_benchmark_suite_report,
    prepare_context_plan_with_paths_and_semantic, prepare_workspace_context_plan,
    render_pack_inspector_html, render_pack_inspector_markdown, render_pack_markdown,
    retrieval_policy_experiment_report, run_benchmark_suite, semantic_provider_status_report,
    BenchmarkComparisonReport, BenchmarkRegressionThreshold, BenchmarkSuiteReport,
    ContextCardsOptions, ContextCardsReport, ExperienceCardsOptions, ExperienceCardsReport,
    HistoricalEvalOptions, HistoricalEvalReport, ProductProofReport,
};
use ctxpack_core::{
    run_init, run_setup_check, AgentAdapter, AgentOutcomeComparisonReport, AgentPreviewReport,
    Diagnostic, DiagnosticSeverity, EvalTrace, FeedbackOutcome, FeedbackSummary,
    GraphNeighborhoodReport, InitAction, InitOptions, InitReport, MemoryReviewStatus, PackBudget,
    PolicyProfileActionReport, PolicyQualityReport, PrivacyStatus, RepoRoot, RetrievalHealthReport,
    RetrievalPolicyExperimentReport, RetrievalPolicyProfile, SemanticProviderStatusReport,
    SessionFeedbackEvent, SetupCheckReport, SetupCheckStatus, SharedArtifactInspectionReport,
    SharedArtifactManifest, TaskType, TeamPolicyReport, WorkspaceContextPack, WorkspaceContextPlan,
    WorkspaceInventoryReport, WorkspaceManifest, WorkspaceRepo,
};
use ctxpack_index::{
    apply_policy_profile, co_change_hints, current_diff_summary, dependency_edges,
    disable_policy_profile, extract_symbols, import_precision_edges, lexical_search,
    list_eval_traces, list_feedback_events, list_memory_cards, list_policy_profiles,
    outcome_comparison_report, policy_quality_report, propose_policy_profile,
    related_dependency_edges, related_tests, rollback_policy_profile, semantic_search,
    storage_status_for_repo, summarize_feedback_events, symbol_search, sync_inventory_to_store,
    sync_semantic_index_to_store, try_append_eval_trace, try_append_feedback_event,
    update_memory_card_review_status, vacuum_store, write_inventory, CoChangeOptions,
    CurrentDiffOptions, DependencyOptions, InventoryOptions, InventoryReport,
    PrecisionImportReport, SearchOptions, SemanticOptions, StorageBenchmarkRunRecord,
    StorageContextPackRecord, StorageGapRecord, StorageIndexReport, StorageMetricRecord,
    StorageProofReportRecord, StorageReport, StorageSemanticIndexReport, StorageStatusReport,
    StoreConfig, SymbolOptions, FEEDBACK_EVENT_SCHEMA_VERSION, WORKSPACE_MANIFEST_SCHEMA_VERSION,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(name = "ctxpack")]
#[command(about = "Agent-native context packs for coding agents")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init(InitArgs),
    #[command(about = "Run a read-only validation of generated setup artifacts")]
    SetupCheck(SetupCheckArgs),
    #[command(about = "Verify install, upgrade, release manifest, and local state compatibility")]
    Doctor(DoctorArgs),
    Index(IndexArgs),
    PrepareTask(PrepareTaskArgs),
    GetPack(GetPackArgs),
    Search {
        query: String,
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(
            long,
            help = "Use explicit local semantic retrieval instead of lexical search."
        )]
        semantic: bool,
    },
    Symbols(SymbolsArgs),
    RelatedTests(RelatedTestsArgs),
    CoChanges(CoChangesArgs),
    Dependencies(DependenciesArgs),
    Precision(PrecisionArgs),
    Storage(StorageArgs),
    Cards(CardsArgs),
    Memory(MemoryArgs),
    Eval(EvalArgs),
    Workspace(WorkspaceArgs),
    Inspector(InspectorArgs),
    Agent(AgentArgs),
    Graph(GraphArgs),
    Semantic(SemanticArgs),
    ServeMcp,
}

#[derive(Debug, Args)]
struct InitArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    cursor: bool,
    #[arg(long)]
    claude: bool,
    #[arg(long)]
    opencode: bool,
}

#[derive(Debug, Args)]
struct SetupCheckArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Validate the generated Cursor rule file.")]
    cursor: bool,
    #[arg(
        long,
        help = "Validate generated Claude command and MCP snippet files."
    )]
    claude: bool,
    #[arg(long, help = "Validate the generated OpenCode MCP snippet file.")]
    opencode: bool,
}

#[derive(Debug, Args)]
struct DoctorArgs {
    #[arg(long, help = "Repository used for local state compatibility checks.")]
    repo: Option<PathBuf>,
    #[arg(
        long,
        help = "ctxpack binary to verify. Defaults to the running executable."
    )]
    binary: Option<PathBuf>,
    #[arg(long, help = "Release manifest JSON from a release archive.")]
    release_manifest: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct IndexArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    include_generated: bool,
    #[arg(long)]
    include_sensitive: bool,
    #[arg(long, help = "Also sync safe inventory records into SQLite storage.")]
    store: bool,
    #[arg(
        long,
        help = "Also build local source-free semantic vector metadata in SQLite storage."
    )]
    semantic: bool,
    #[arg(long, help = "Override the SQLite storage database path.")]
    store_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct StorageArgs {
    #[command(subcommand)]
    command: StorageCommand,
}

#[derive(Debug, Subcommand)]
enum StorageCommand {
    Init(StoragePathArgs),
    Status(StoragePathArgs),
    Repair(StoragePathArgs),
    Vacuum(StoragePathArgs),
    Reset(StorageResetArgs),
}

#[derive(Debug, Args)]
struct StoragePathArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Override the SQLite storage database path.")]
    path: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct StorageResetArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Override the SQLite storage database path.")]
    path: Option<PathBuf>,
    #[arg(
        long,
        help = "Actually delete the storage database. Without this, reset is a dry run."
    )]
    yes: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspaceArgs {
    #[command(subcommand)]
    command: WorkspaceCommand,
}

#[derive(Debug, Args)]
struct InspectorArgs {
    #[command(subcommand)]
    command: InspectorCommand,
}

#[derive(Debug, Args)]
struct AgentArgs {
    #[command(subcommand)]
    command: AgentCommand,
}

#[derive(Debug, Args)]
struct GraphArgs {
    #[command(subcommand)]
    command: GraphCommand,
}

#[derive(Debug, Args)]
struct SemanticArgs {
    #[command(subcommand)]
    command: SemanticCommand,
}

#[derive(Debug, Subcommand)]
enum SemanticCommand {
    #[command(about = "Inspect local semantic provider status and cloud policy gates.")]
    Status(SemanticStatusArgs),
}

#[derive(Debug, Args)]
struct SemanticStatusArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional task/query used to inspect per-plan semantic usage."
    )]
    query: Option<String>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    #[command(about = "Preview agent-specific ctxpack context usage.")]
    Preview(AgentPreviewArgs),
}

#[derive(Debug, Args)]
struct AgentPreviewArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(long, value_enum, default_value_t = Budget::Brief)]
    budget: Budget,
    #[arg(
        long = "path",
        help = "Active/open file path to pin as a context anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(
        long,
        default_value = "all",
        help = "codex, claude-code, cursor, opencode, generic, or all."
    )]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval in the preview planner."
    )]
    semantic: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Subcommand)]
enum GraphCommand {
    #[command(about = "Generate a source-free graph neighborhood report.")]
    Neighborhood(GraphNeighborhoodArgs),
}

#[derive(Debug, Args)]
struct GraphNeighborhoodArgs {
    #[arg(help = "Optional task used to derive anchors when no --path is provided.")]
    task: Option<String>,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(
        long = "path",
        help = "Repo-relative or absolute path anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(long, default_value_t = 40)]
    max_nodes: usize,
    #[arg(long, default_value_t = 80)]
    max_edges: usize,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Subcommand)]
enum InspectorCommand {
    #[command(about = "Export a source-free pack inspector artifact.")]
    Export(InspectorExportArgs),
}

#[derive(Debug, Args)]
struct InspectorExportArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(long, value_enum, default_value_t = Budget::Brief)]
    budget: Budget,
    #[arg(
        long = "path",
        help = "Active/open file path to pin as a context anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(
        long = "current-diff",
        help = "Add safe changed paths from the current local diff as context anchors."
    )]
    include_current_diff: bool,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval in the context pack planner."
    )]
    semantic: bool,
    #[arg(long, value_enum, default_value_t = InspectorFormat::Json)]
    format: InspectorFormat,
    #[arg(long, help = "Write the artifact to a file instead of stdout.")]
    output: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommand {
    #[command(about = "Export, inspect, or import source-free shared artifact manifests.")]
    Artifacts(WorkspaceArtifactsArgs),
    #[command(about = "Create a local source-free workspace manifest.")]
    Init(WorkspaceInitArgs),
    #[command(about = "Route a task to likely workspace repositories and return per-repo plans.")]
    PrepareTask(WorkspacePrepareTaskArgs),
    #[command(about = "Compile a repo-boundary-aware workspace context pack.")]
    GetPack(WorkspaceGetPackArgs),
    #[command(about = "Create or inspect a local team privacy policy template.")]
    Policy(WorkspacePolicyArgs),
    #[command(about = "Inspect source-free local workspace inventory status.")]
    Status(WorkspaceStatusArgs),
}

#[derive(Debug, Args)]
struct WorkspaceArtifactsArgs {
    #[command(subcommand)]
    command: WorkspaceArtifactsCommand,
}

#[derive(Debug, Subcommand)]
enum WorkspaceArtifactsCommand {
    #[command(about = "Export a source-free shared artifact manifest.")]
    Export(WorkspaceArtifactExportArgs),
    #[command(about = "Inspect a source-free shared artifact manifest.")]
    Inspect(WorkspaceArtifactInspectArgs),
    #[command(about = "Import a compatible source-free shared artifact manifest.")]
    Import(WorkspaceArtifactImportArgs),
}

#[derive(Debug, Args)]
struct WorkspaceArtifactExportArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspaceArtifactInspectArgs {
    input: PathBuf,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspaceArtifactImportArgs {
    input: PathBuf,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspacePolicyArgs {
    #[command(subcommand)]
    command: WorkspacePolicyCommand,
}

#[derive(Debug, Subcommand)]
enum WorkspacePolicyCommand {
    #[command(about = "Write a local source-free team privacy policy template.")]
    Init(WorkspacePolicyPathArgs),
    #[command(about = "Inspect local source-free team privacy policy effects.")]
    Status(WorkspacePolicyPathArgs),
}

#[derive(Debug, Args)]
struct WorkspacePolicyPathArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspaceInitArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long = "member", help = "Additional local repository path to include.")]
    members: Vec<PathBuf>,
    #[arg(long, help = "Display label for the workspace root repository.")]
    label: Option<String>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspaceStatusArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Override the workspace manifest path.")]
    manifest: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspacePrepareTaskArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Override the workspace manifest path.")]
    manifest: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(
        long = "path",
        help = "Workspace-relative, repo-relative, or absolute active path anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval inside each selected workspace repository."
    )]
    semantic: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Json)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct WorkspaceGetPackArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Override the workspace manifest path.")]
    manifest: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(long, value_enum, default_value_t = Budget::Brief)]
    budget: Budget,
    #[arg(
        long = "path",
        help = "Workspace-relative, repo-relative, or absolute active path anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval inside each selected workspace repository."
    )]
    semantic: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct RelatedTestsArgs {
    paths: Vec<String>,
    #[arg(long)]
    repo: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct SymbolsArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    query: Option<String>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(Debug, Args)]
struct CoChangesArgs {
    paths: Vec<String>,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 10)]
    limit: usize,
}

#[derive(Debug, Args)]
struct DependenciesArgs {
    paths: Vec<String>,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(
        long,
        help = "Return all safe local dependency edges instead of anchor-related edges."
    )]
    all: bool,
}

#[derive(Debug, Args)]
struct PrecisionArgs {
    #[command(subcommand)]
    command: PrecisionCommand,
}

#[derive(Debug, Subcommand)]
enum PrecisionCommand {
    #[command(about = "Import source-free precision edges from a local SCIP/LSP bridge JSON file")]
    Import(PrecisionImportArgs),
}

#[derive(Debug, Args)]
struct PrecisionImportArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Path to a source-free precision edge JSON file.")]
    input: PathBuf,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct CardsArgs {
    #[command(subcommand)]
    command: CardsCommand,
}

#[derive(Debug, Subcommand)]
enum CardsCommand {
    Generate(CardsGenerateArgs),
}

#[derive(Debug, Args)]
struct CardsGenerateArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 40)]
    limit: usize,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct MemoryArgs {
    #[command(subcommand)]
    command: MemoryCommand,
}

#[derive(Debug, Subcommand)]
enum MemoryCommand {
    #[command(about = "List source-free local memory cards.")]
    List(MemoryListArgs),
    #[command(about = "Show one source-free local memory card.")]
    Show(MemoryCardActionArgs),
    #[command(about = "Generate pending source-free experience cards from local eval traces.")]
    GenerateExperience(MemoryExperienceArgs),
    #[command(about = "Regenerate deterministic domain cards and pending experience cards.")]
    Regenerate(MemoryExperienceArgs),
    #[command(about = "Approve a pending memory card for pack inclusion.")]
    Approve(MemoryCardActionArgs),
    #[command(about = "Reject a memory card before pack inclusion.")]
    Reject(MemoryCardActionArgs),
    #[command(about = "Disable a memory card without deleting it.")]
    Disable(MemoryCardActionArgs),
}

#[derive(Debug, Args)]
struct MemoryListArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    include_disabled: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct MemoryExperienceArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct MemoryCardActionArgs {
    card_id: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct PrepareTaskArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(
        long = "path",
        help = "Active/open file path to pin as a context anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(
        long = "current-diff",
        help = "Add safe changed paths from the current local diff as context anchors."
    )]
    include_current_diff: bool,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval in the context planner."
    )]
    semantic: bool,
    #[arg(
        long,
        help = "Disable local eval trace recording for this read command."
    )]
    no_trace: bool,
}

#[derive(Debug, Args)]
struct GetPackArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
    #[arg(long, value_enum, default_value_t = Budget::Brief)]
    budget: Budget,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
    #[arg(
        long = "path",
        help = "Active/open file path to pin as a context anchor. Repeatable."
    )]
    paths: Vec<String>,
    #[arg(
        long = "current-diff",
        help = "Add safe changed paths from the current local diff as context anchors."
    )]
    include_current_diff: bool,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval in the context pack planner."
    )]
    semantic: bool,
    #[arg(
        long,
        help = "Disable local eval trace recording for this read command."
    )]
    no_trace: bool,
    #[arg(long, help = "Persist source-free pack metadata into SQLite storage.")]
    store: bool,
    #[arg(long, help = "Override the SQLite storage database path.")]
    store_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct EvalArgs {
    #[command(subcommand)]
    command: EvalCommand,
}

#[derive(Debug, Subcommand)]
enum EvalCommand {
    Traces(EvalTracesArgs),
    Checklist(EvalTracesArgs),
    Feedback(EvalFeedbackArgs),
    Policy(EvalPolicyArgs),
    Outcome(EvalOutcomeArgs),
    History(EvalHistoryArgs),
    Health(EvalHealthArgs),
    Benchmark(EvalBenchmarkArgs),
    Compare(EvalCompareArgs),
    Proof(EvalProofArgs),
}

#[derive(Debug, Args)]
struct EvalTracesArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(Debug, Args)]
struct EvalFeedbackArgs {
    #[command(subcommand)]
    command: EvalFeedbackCommand,
}

#[derive(Debug, Subcommand)]
enum EvalFeedbackCommand {
    #[command(about = "Record a source-free agent-session feedback event.")]
    Record(EvalFeedbackRecordArgs),
    #[command(about = "List recent source-free feedback events.")]
    List(EvalFeedbackListArgs),
    #[command(about = "Summarize recent source-free feedback events.")]
    Summary(EvalFeedbackListArgs),
}

#[derive(Debug, Args)]
struct EvalFeedbackRecordArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, help = "Source-free task hash from a ctxpack trace or pack.")]
    task_hash: String,
    #[arg(long, value_enum, default_value_t = Mode::BugFix)]
    mode: Mode,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Optional ctxpack pack UUID associated with this feedback."
    )]
    pack_id: Option<Uuid>,
    #[arg(long, value_enum)]
    budget: Option<Budget>,
    #[arg(long, value_enum, default_value_t = FeedbackOutcomeArg::Unknown)]
    outcome: FeedbackOutcomeArg,
    #[arg(long = "recommended-file")]
    recommended_files: Vec<String>,
    #[arg(long = "recommended-test")]
    recommended_tests: Vec<String>,
    #[arg(long = "recommended-command")]
    recommended_commands: Vec<String>,
    #[arg(long = "read-file")]
    read_files: Vec<String>,
    #[arg(long = "edited-file")]
    edited_files: Vec<String>,
    #[arg(long = "tested-file")]
    tested_files: Vec<String>,
    #[arg(long = "tested-command")]
    tested_commands: Vec<String>,
    #[arg(long = "corrected-file")]
    user_corrected_files: Vec<String>,
    #[arg(long = "tag")]
    tags: Vec<String>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalFeedbackListArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalPolicyArgs {
    #[command(subcommand)]
    command: EvalPolicyCommand,
}

#[derive(Debug, Subcommand)]
enum EvalPolicyCommand {
    #[command(about = "Generate a source-free policy quality report from feedback events.")]
    Report(EvalFeedbackListArgs),
    #[command(about = "Propose a local retrieval policy profile from feedback evidence.")]
    Tune(EvalFeedbackListArgs),
    #[command(about = "List local retrieval policy profiles.")]
    List(EvalPolicyListArgs),
    #[command(about = "Apply a local retrieval policy profile.")]
    Apply(EvalPolicyActionArgs),
    #[command(about = "Disable a local retrieval policy profile.")]
    Disable(EvalPolicyActionArgs),
    #[command(about = "Roll back the active local retrieval policy profile.")]
    Rollback(EvalPolicyListArgs),
    #[command(about = "Compare source-free retrieval policy experiment rows.")]
    Experiments(EvalPolicyExperimentArgs),
}

#[derive(Debug, Args)]
struct EvalPolicyExperimentArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 10)]
    limit: usize,
    #[arg(long, default_value_t = 10)]
    budget: usize,
    #[arg(long, value_enum, default_value_t = Mode::BugFix)]
    mode: Mode,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalPolicyListArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalPolicyActionArgs {
    profile_id: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalOutcomeArgs {
    #[command(subcommand)]
    command: EvalOutcomeCommand,
}

#[derive(Debug, Subcommand)]
enum EvalOutcomeCommand {
    #[command(about = "Compare plan-only, brief, standard, and deep feedback outcomes.")]
    Compare(EvalFeedbackListArgs),
}

#[derive(Debug, Clone, ValueEnum)]
enum FeedbackOutcomeArg {
    Passed,
    Failed,
    Blocked,
    Unknown,
}

#[derive(Debug, Args)]
struct EvalHistoryArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(
        long,
        default_value_t = 10,
        help = "Fixed context-file ranking budget used for combined, lexical, and ablation metrics."
    )]
    budget: usize,
    #[arg(long, help = "Start revision for a stable historical eval range.")]
    base: Option<String>,
    #[arg(long, help = "End revision for a stable historical eval range.")]
    head: Option<String>,
    #[arg(long, value_enum, default_value_t = Mode::BugFix)]
    mode: Mode,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval during historical eval."
    )]
    semantic: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
    #[arg(long, help = "Persist source-free eval metrics into SQLite storage.")]
    store: bool,
    #[arg(long, help = "Override the SQLite storage database path.")]
    store_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct EvalHealthArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
    #[arg(
        long,
        default_value_t = 10,
        help = "Fixed context-file ranking budget used for combined, lexical, and ablation metrics."
    )]
    budget: usize,
    #[arg(long, help = "Start revision for a stable historical eval range.")]
    base: Option<String>,
    #[arg(long, help = "End revision for a stable historical eval range.")]
    head: Option<String>,
    #[arg(long, value_enum, default_value_t = Mode::BugFix)]
    mode: Mode,
    #[arg(long, default_value = "generic")]
    target_agent: String,
    #[arg(
        long,
        help = "Enable explicit local semantic retrieval during historical eval."
    )]
    semantic: bool,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalBenchmarkArgs {
    #[arg(
        long,
        help = "Path to a JSON benchmark suite file containing named repositories and eval budgets."
    )]
    config: PathBuf,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
    #[arg(
        long,
        help = "Persist source-free benchmark metadata into each repo's SQLite storage."
    )]
    store: bool,
}

#[derive(Debug, Args)]
struct EvalCompareArgs {
    #[arg(long, help = "Path to the baseline benchmark JSON report.")]
    base_report: PathBuf,
    #[arg(long, help = "Path to the head benchmark JSON report.")]
    head_report: PathBuf,
    #[arg(
        long,
        value_parser = parse_regression_threshold,
        help = "Regression threshold as metric=max_drop, e.g. fileRecallAt10=0.05"
    )]
    threshold: Vec<BenchmarkRegressionThreshold>,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
}

#[derive(Debug, Args)]
struct EvalProofArgs {
    #[arg(
        long,
        help = "Path to a JSON benchmark suite file used to generate the product proof report."
    )]
    config: PathBuf,
    #[arg(long, value_enum, default_value_t = PackFormat::Markdown)]
    format: PackFormat,
    #[arg(
        long,
        help = "Persist source-free product-proof metadata into each repo's SQLite storage."
    )]
    store: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    BugFix,
    Feature,
    Refactor,
    Review,
    Test,
    Explain,
}

#[derive(Debug, Clone, ValueEnum)]
enum Budget {
    Brief,
    Standard,
    Deep,
}

#[derive(Debug, Clone, ValueEnum)]
enum PackFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, ValueEnum)]
enum InspectorFormat {
    Json,
    Markdown,
    Html,
}

impl From<Mode> for TaskType {
    fn from(value: Mode) -> Self {
        match value {
            Mode::BugFix => TaskType::BugFix,
            Mode::Feature => TaskType::Feature,
            Mode::Refactor => TaskType::Refactor,
            Mode::Review => TaskType::Review,
            Mode::Test => TaskType::Test,
            Mode::Explain => TaskType::Explain,
        }
    }
}

impl From<Budget> for PackBudget {
    fn from(value: Budget) -> Self {
        match value {
            Budget::Brief => PackBudget::Brief,
            Budget::Standard => PackBudget::Standard,
            Budget::Deep => PackBudget::Deep,
        }
    }
}

impl From<FeedbackOutcomeArg> for FeedbackOutcome {
    fn from(value: FeedbackOutcomeArg) -> Self {
        match value {
            FeedbackOutcomeArg::Passed => FeedbackOutcome::Passed,
            FeedbackOutcomeArg::Failed => FeedbackOutcome::Failed,
            FeedbackOutcomeArg::Blocked => FeedbackOutcome::Blocked,
            FeedbackOutcomeArg::Unknown => FeedbackOutcome::Unknown,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let report = run_init(&repo.path, &init_options(&args))?;
            print_init_report(&report);
        }
        Command::SetupCheck(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let report = run_setup_check(&repo.path, &setup_check_options(&args))?;
            let passed = report.passed;
            print_setup_check_report(&report);
            if !passed {
                std::process::exit(1);
            }
        }
        Command::Doctor(args) => {
            let report = build_doctor_report(&args)?;
            match args.format {
                PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                PackFormat::Markdown => print_doctor_report(&report),
            }
            if !report["passed"].as_bool().unwrap_or(false) {
                std::process::exit(1);
            }
        }
        Command::Index(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let report = write_inventory(
                &repo.path,
                &InventoryOptions {
                    include_generated: args.include_generated,
                    include_sensitive: args.include_sensitive,
                },
            )?;
            print_inventory_report(&report);
            if args.store || args.semantic {
                let storage = sync_inventory_to_store(
                    &repo.path,
                    &InventoryOptions {
                        include_generated: args.include_generated,
                        include_sensitive: args.include_sensitive,
                    },
                    &StoreConfig {
                        path_override: args.store_path.clone(),
                    },
                )?;
                print_storage_index_report(&storage);
            }
            if args.semantic {
                let storage = sync_semantic_index_to_store(
                    &repo.path,
                    &SemanticOptions {
                        enabled: true,
                        limit: usize::MAX,
                        ..SemanticOptions::default()
                    },
                    &StoreConfig {
                        path_override: args.store_path.clone(),
                    },
                )?;
                print_semantic_storage_report(&storage);
            }
        }
        Command::PrepareTask(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let paths = context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
            let mut plan = prepare_context_plan_with_paths_and_semantic(
                &repo.path,
                &args.task,
                args.mode.into(),
                &paths,
                args.semantic,
            )?;
            if args.no_trace {
                plan.diagnostics.push(trace_disabled_diagnostic());
            } else {
                let trace = eval_trace_for_plan(&repo.path, &args.task, &args.target_agent, &plan);
                plan.diagnostics
                    .extend(try_append_eval_trace(&repo.path, &trace).diagnostics);
            }
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Command::GetPack(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let paths = context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
            let (plan, mut pack) = compile_context_pack_with_plan_and_paths_for_agent_and_semantic(
                &repo.path,
                &args.task,
                args.mode.into(),
                args.budget.into(),
                &paths,
                &args.target_agent,
                args.semantic,
            )?;
            if args.no_trace {
                pack.diagnostics.push(trace_disabled_diagnostic());
            } else {
                let trace =
                    eval_trace_for_pack(&repo.path, &args.task, &args.target_agent, &plan, &pack);
                pack.diagnostics
                    .extend(try_append_eval_trace(&repo.path, &trace).diagnostics);
            }
            if args.store {
                let selected_candidate_ids = plan
                    .retrieval_candidates
                    .iter()
                    .filter_map(|candidate| candidate.path.clone())
                    .collect::<Vec<_>>();
                let storage = ctxpack_index::persist_context_pack_record(
                    &repo.path,
                    &StoreConfig {
                        path_override: args.store_path,
                    },
                    &StorageContextPackRecord {
                        pack_id: pack.id.to_string(),
                        task_hash: pack.task_hash.clone(),
                        budget: json_label(&pack.budget),
                        target_agent: pack.target_agent.clone(),
                        confidence: pack.confidence,
                        selected_candidate_ids,
                        warnings: pack.warnings.clone(),
                        privacy_status: privacy_status_label(&pack.privacy_status),
                    },
                )?;
                pack.diagnostics.push(Diagnostic {
                    code: "storage_pack_metadata_persisted".to_string(),
                    severity: DiagnosticSeverity::Info,
                    message: format!(
                        "Stored source-free pack metadata in {}",
                        storage.database_path.display()
                    ),
                    count: 1,
                    paths: vec![storage.database_path.display().to_string()],
                });
            }
            match args.format {
                PackFormat::Markdown => println!("{}", render_pack_markdown(&pack)),
                PackFormat::Json => println!("{}", serde_json::to_string_pretty(&pack)?),
            }
        }
        Command::Inspector(args) => match args.command {
            InspectorCommand::Export(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let paths =
                    context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
                let (plan, pack) = compile_context_pack_with_plan_and_paths_for_agent_and_semantic(
                    &repo.path,
                    &args.task,
                    args.mode.into(),
                    args.budget.into(),
                    &paths,
                    &args.target_agent,
                    args.semantic,
                )?;
                let view = compile_pack_inspector_view(&plan, &pack);
                let artifact = match args.format {
                    InspectorFormat::Json => serde_json::to_string_pretty(&view)?,
                    InspectorFormat::Markdown => render_pack_inspector_markdown(&view),
                    InspectorFormat::Html => render_pack_inspector_html(&view),
                };
                write_or_print(args.output.as_deref(), &artifact)?;
            }
        },
        Command::Agent(args) => match args.command {
            AgentCommand::Preview(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = build_agent_preview_report(
                    &repo.path,
                    &args.task,
                    args.mode.into(),
                    args.budget.into(),
                    &args.target_agent,
                    &args.paths,
                    args.semantic,
                )?;
                match args.format {
                    PackFormat::Markdown => println!("{}", render_agent_preview_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Graph(args) => match args.command {
            GraphCommand::Neighborhood(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = build_graph_neighborhood_report(
                    &repo.path,
                    args.task.as_deref(),
                    args.mode.into(),
                    &args.paths,
                    args.max_nodes,
                    args.max_edges,
                )?;
                match args.format {
                    PackFormat::Markdown => {
                        println!("{}", render_graph_neighborhood_report(&report))
                    }
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Semantic(args) => match args.command {
            SemanticCommand::Status(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = semantic_provider_status_report(
                    &repo.path,
                    args.query.as_deref(),
                    args.mode.into(),
                )?;
                match args.format {
                    PackFormat::Markdown => {
                        println!("{}", render_semantic_provider_status(&report))
                    }
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Search {
            query,
            repo,
            limit,
            semantic,
        } => {
            let start = repo.unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            if semantic {
                let results = semantic_search(
                    &repo.path,
                    &query,
                    &SemanticOptions {
                        enabled: true,
                        limit,
                        ..SemanticOptions::default()
                    },
                )?;
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                let results = lexical_search(&repo.path, &query, &SearchOptions { limit })?;
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
        }
        Command::Symbols(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            if let Some(query) = args.query {
                let results =
                    symbol_search(&repo.path, &query, &SymbolOptions { limit: args.limit })?;
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                let mut symbols = extract_symbols(&repo.path)?;
                symbols.truncate(args.limit.max(1));
                println!("{}", serde_json::to_string_pretty(&symbols)?);
            }
        }
        Command::RelatedTests(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let results = related_tests(&repo.path, &args.paths)?;
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        Command::CoChanges(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let results = co_change_hints(
                &repo.path,
                &args.paths,
                &CoChangeOptions { limit: args.limit },
            )?;
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        Command::Dependencies(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let options = DependencyOptions { limit: args.limit };
            let results = if args.all || args.paths.is_empty() {
                dependency_edges(&repo.path, &options)?
            } else {
                related_dependency_edges(&repo.path, &args.paths, &options)?
            };
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        Command::Precision(args) => match args.command {
            PrecisionCommand::Import(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = import_precision_edges(&repo.path, &args.input)?;
                match args.format {
                    PackFormat::Markdown => print_precision_import_report(&report),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Storage(args) => {
            match args.command {
                StorageCommand::Init(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = ctxpack_index::initialize_store(
                        &repo.path,
                        &StoreConfig {
                            path_override: args.path,
                        },
                    )?;
                    match args.format {
                        PackFormat::Markdown => print_storage_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
                StorageCommand::Status(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = storage_status_for_repo(
                        &repo.path,
                        &StoreConfig {
                            path_override: args.path,
                        },
                    )?;
                    match args.format {
                        PackFormat::Markdown => print_storage_status_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
                StorageCommand::Repair(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = ctxpack_index::initialize_store(
                        &repo.path,
                        &StoreConfig {
                            path_override: args.path,
                        },
                    )?;
                    match args.format {
                        PackFormat::Markdown => print_storage_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
                StorageCommand::Vacuum(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let status = storage_status_for_repo(
                        &repo.path,
                        &StoreConfig {
                            path_override: args.path,
                        },
                    )?;
                    vacuum_store(&status.database_path)?;
                    let report = storage_status_for_repo(
                        &repo.path,
                        &StoreConfig {
                            path_override: Some(status.database_path),
                        },
                    )?;
                    match args.format {
                        PackFormat::Markdown => print_storage_status_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
                StorageCommand::Reset(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let status = storage_status_for_repo(
                        &repo.path,
                        &StoreConfig {
                            path_override: args.path,
                        },
                    )?;
                    if args.yes && status.database_path.exists() {
                        fs::remove_file(&status.database_path)?;
                    }
                    let report = StorageStatusReport {
                        file_records: if args.yes { 0 } else { status.file_records },
                        symbol_records: if args.yes { 0 } else { status.symbol_records },
                        context_pack_records: if args.yes {
                            0
                        } else {
                            status.context_pack_records
                        },
                        benchmark_run_records: if args.yes {
                            0
                        } else {
                            status.benchmark_run_records
                        },
                        proof_report_records: if args.yes {
                            0
                        } else {
                            status.proof_report_records
                        },
                        semantic_vector_records: if args.yes {
                            0
                        } else {
                            status.semantic_vector_records
                        },
                        memory_card_records: if args.yes {
                            0
                        } else {
                            status.memory_card_records
                        },
                        diagnostics: if args.yes {
                            Vec::new()
                        } else {
                            vec![Diagnostic {
                            code: "storage_reset_dry_run".to_string(),
                            severity: DiagnosticSeverity::Warning,
                            message: "Storage reset was a dry run; pass --yes to delete the database.".to_string(),
                            count: 1,
                            paths: vec![status.database_path.display().to_string()],
                        }]
                        },
                        ..status
                    };
                    match args.format {
                        PackFormat::Markdown => print_storage_status_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
            }
        }
        Command::Cards(args) => match args.command {
            CardsCommand::Generate(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report =
                    generate_context_cards(&repo.path, &ContextCardsOptions { limit: args.limit })?;
                match args.format {
                    PackFormat::Markdown => println!("{}", render_cards_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Memory(args) => match args.command {
            MemoryCommand::List(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let cards =
                    list_memory_cards(&repo.path, &StoreConfig::default(), args.include_disabled)?;
                match args.format {
                    PackFormat::Markdown => println!("{}", render_memory_cards(&cards)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&cards)?),
                }
            }
            MemoryCommand::GenerateExperience(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = generate_experience_cards(
                    &repo.path,
                    &ExperienceCardsOptions { limit: args.limit },
                )?;
                match args.format {
                    PackFormat::Markdown => println!("{}", render_experience_cards_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            MemoryCommand::Show(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let cards = list_memory_cards(&repo.path, &StoreConfig::default(), true)?;
                let Some(card) = cards.into_iter().find(|card| card.id == args.card_id) else {
                    anyhow::bail!("memory card not found: {}", args.card_id);
                };
                match args.format {
                    PackFormat::Markdown => println!("{}", render_memory_cards(&[card])),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&card)?),
                }
            }
            MemoryCommand::Regenerate(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let cards =
                    generate_context_cards(&repo.path, &ContextCardsOptions { limit: args.limit })?;
                let experience = generate_experience_cards(
                    &repo.path,
                    &ExperienceCardsOptions { limit: args.limit },
                )?;
                match args.format {
                    PackFormat::Markdown => {
                        println!("{}", render_cards_report(&cards));
                        println!("{}", render_experience_cards_report(&experience));
                    }
                    PackFormat::Json => println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "domainCards": cards,
                            "experienceCards": experience,
                        }))?
                    ),
                }
            }
            MemoryCommand::Approve(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = update_memory_card_review_status(
                    &repo.path,
                    &StoreConfig::default(),
                    &args.card_id,
                    MemoryReviewStatus::Approved,
                    false,
                )?;
                match args.format {
                    PackFormat::Markdown => print_storage_status_report(&report),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            MemoryCommand::Reject(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = update_memory_card_review_status(
                    &repo.path,
                    &StoreConfig::default(),
                    &args.card_id,
                    MemoryReviewStatus::Rejected,
                    true,
                )?;
                match args.format {
                    PackFormat::Markdown => print_storage_status_report(&report),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            MemoryCommand::Disable(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = update_memory_card_review_status(
                    &repo.path,
                    &StoreConfig::default(),
                    &args.card_id,
                    MemoryReviewStatus::Disabled,
                    true,
                )?;
                match args.format {
                    PackFormat::Markdown => print_storage_status_report(&report),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Eval(args) => match args.command {
            EvalCommand::Traces(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let traces = list_eval_traces(&repo.path, args.limit)?;
                println!("{}", serde_json::to_string_pretty(&traces)?);
            }
            EvalCommand::Checklist(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let traces = list_eval_traces(&repo.path, args.limit)?;
                let retrieval_gap_summaries = evaluate_historical_commits(
                    &repo.path,
                    &HistoricalEvalOptions {
                        limit: args.limit,
                        ranking_budget: 10,
                        task_type: TaskType::BugFix,
                        target_agent: "generic".to_string(),
                        base: None,
                        head: None,
                        semantic_enabled: false,
                    },
                )
                .map(|report| report.retrieval_gap_summaries)
                .unwrap_or_default();
                println!(
                    "{}",
                    render_eval_checklist_with_gaps(&traces, &retrieval_gap_summaries)
                );
            }
            EvalCommand::Feedback(args) => match args.command {
                EvalFeedbackCommand::Record(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let repo_id = ctxpack_index::repo_id_for_path(&fs::canonicalize(&repo.path)?);
                    let event = SessionFeedbackEvent {
                        id: Uuid::new_v4(),
                        schema_version: FEEDBACK_EVENT_SCHEMA_VERSION,
                        repo_id,
                        task_hash: args.task_hash,
                        task_type: args.mode.into(),
                        pack_id: args.pack_id,
                        target_agent: args.target_agent,
                        budget: args.budget.map(Into::into),
                        outcome: args.outcome.into(),
                        recommended_files: args.recommended_files,
                        recommended_tests: args.recommended_tests,
                        recommended_commands: args.recommended_commands,
                        read_files: args.read_files,
                        edited_files: args.edited_files,
                        tested_files: args.tested_files,
                        tested_commands: args.tested_commands,
                        user_corrected_files: args.user_corrected_files,
                        tags: args.tags,
                        created_at_unix_seconds: current_unix_seconds(),
                        source_text_logged: false,
                    };
                    let status = try_append_feedback_event(&repo.path, &event);
                    match args.format {
                        PackFormat::Markdown => {
                            print_feedback_record_status(&event, &status);
                        }
                        PackFormat::Json => {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "event": event,
                                    "status": status
                                }))?
                            );
                        }
                    }
                }
                EvalFeedbackCommand::List(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let events = list_feedback_events(&repo.path, args.limit)?;
                    match args.format {
                        PackFormat::Markdown => println!("{}", render_feedback_events(&events)),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&events)?),
                    }
                }
                EvalFeedbackCommand::Summary(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let repo_id = ctxpack_index::repo_id_for_path(&fs::canonicalize(&repo.path)?);
                    let events = list_feedback_events(&repo.path, args.limit)?;
                    let summary = summarize_feedback_events(&repo_id, &events);
                    match args.format {
                        PackFormat::Markdown => println!("{}", render_feedback_summary(&summary)),
                        PackFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&summary)?)
                        }
                    }
                }
            },
            EvalCommand::Policy(args) => match args.command {
                EvalPolicyCommand::Report(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = policy_quality_report(&repo.path, args.limit)?;
                    match args.format {
                        PackFormat::Markdown => {
                            println!("{}", render_policy_quality_report(&report))
                        }
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
                EvalPolicyCommand::Tune(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let profile = propose_policy_profile(&repo.path, args.limit)?;
                    match args.format {
                        PackFormat::Markdown => println!("{}", render_policy_profiles(&[profile])),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&profile)?),
                    }
                }
                EvalPolicyCommand::List(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let profiles = list_policy_profiles(&repo.path)?;
                    match args.format {
                        PackFormat::Markdown => println!("{}", render_policy_profiles(&profiles)),
                        PackFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&profiles)?)
                        }
                    }
                }
                EvalPolicyCommand::Apply(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = apply_policy_profile(&repo.path, &args.profile_id)?;
                    print_policy_action_report(&report, &args.format)?;
                }
                EvalPolicyCommand::Disable(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = disable_policy_profile(&repo.path, &args.profile_id)?;
                    print_policy_action_report(&report, &args.format)?;
                }
                EvalPolicyCommand::Rollback(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = rollback_policy_profile(&repo.path)?;
                    print_policy_action_report(&report, &args.format)?;
                }
                EvalPolicyCommand::Experiments(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = retrieval_policy_experiment_report(
                        &repo.path,
                        &args.task,
                        args.mode.into(),
                        args.limit,
                        args.budget,
                    )?;
                    match args.format {
                        PackFormat::Markdown => {
                            println!("{}", render_policy_experiment_report(&report))
                        }
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
            },
            EvalCommand::Outcome(args) => match args.command {
                EvalOutcomeCommand::Compare(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = outcome_comparison_report(&repo.path, args.limit)?;
                    match args.format {
                        PackFormat::Markdown => {
                            println!("{}", render_outcome_comparison_report(&report))
                        }
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
            },
            EvalCommand::History(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = evaluate_historical_commits(
                    &repo.path,
                    &HistoricalEvalOptions {
                        limit: args.limit,
                        ranking_budget: args.budget,
                        task_type: args.mode.into(),
                        target_agent: args.target_agent,
                        base: args.base,
                        head: args.head,
                        semantic_enabled: args.semantic,
                    },
                )?;
                if args.store {
                    let status = persist_historical_eval_report(
                        &repo.path,
                        &StoreConfig {
                            path_override: args.store_path,
                        },
                        &report,
                    )?;
                    eprintln!(
                        "Stored source-free eval metadata in {}",
                        status.database_path.display()
                    );
                }
                match args.format {
                    PackFormat::Markdown => println!("{}", render_historical_eval_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            EvalCommand::Health(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let historical = evaluate_historical_commits(
                    &repo.path,
                    &HistoricalEvalOptions {
                        limit: args.limit,
                        ranking_budget: args.budget,
                        task_type: args.mode.into(),
                        target_agent: args.target_agent,
                        base: args.base,
                        head: args.head,
                        semantic_enabled: args.semantic,
                    },
                )?;
                let policy = policy_quality_report(&repo.path, args.limit)?;
                let report = build_retrieval_health_report(&historical, &policy);
                match args.format {
                    PackFormat::Markdown => println!("{}", render_retrieval_health_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            EvalCommand::Benchmark(args) => {
                let config = if args.store {
                    Some(load_benchmark_suite_config(&args.config)?)
                } else {
                    None
                };
                let report = run_benchmark_suite(&args.config)?;
                if let Some(config) = config.as_ref() {
                    persist_benchmark_suite_report(config, &report, &args.config)?;
                }
                match args.format {
                    PackFormat::Markdown => println!("{}", render_benchmark_suite_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            EvalCommand::Compare(args) => {
                let base = load_benchmark_suite_report(&args.base_report)?;
                let head = load_benchmark_suite_report(&args.head_report)?;
                let report = compare_benchmark_suite_reports(&base, &head, &args.threshold);
                match args.format {
                    PackFormat::Markdown => {
                        println!("{}", render_benchmark_comparison_report(&report))
                    }
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            EvalCommand::Proof(args) => {
                let config = if args.store {
                    Some(load_benchmark_suite_config(&args.config)?)
                } else {
                    None
                };
                let benchmark = run_benchmark_suite(&args.config)?;
                let report = build_product_proof_report(benchmark);
                if let Some(config) = config.as_ref() {
                    persist_product_proof_report(config, &report, &args.config)?;
                }
                match args.format {
                    PackFormat::Markdown => println!("{}", render_product_proof_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::Workspace(args) => match args.command {
            WorkspaceCommand::Artifacts(args) => match args.command {
                WorkspaceArtifactsCommand::Export(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let manifest = ctxpack_index::export_shared_artifact_manifest(&repo.path)?;
                    match args.format {
                        PackFormat::Markdown => print_shared_artifact_manifest(&manifest),
                        PackFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&manifest)?)
                        }
                    }
                }
                WorkspaceArtifactsCommand::Inspect(args) => {
                    let report = ctxpack_index::inspect_shared_artifact_manifest(&args.input)?;
                    match args.format {
                        PackFormat::Markdown => print_shared_artifact_inspection(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                    if !report.compatible {
                        std::process::exit(1);
                    }
                }
                WorkspaceArtifactsCommand::Import(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report =
                        ctxpack_index::import_shared_artifact_manifest(&repo.path, &args.input)?;
                    match args.format {
                        PackFormat::Markdown => print_shared_artifact_inspection(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                    if !report.compatible {
                        std::process::exit(1);
                    }
                }
            },
            WorkspaceCommand::Init(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let mut repos = vec![workspace_repo_entry(
                    &repo.path,
                    &repo.path,
                    args.label.as_deref(),
                )?];
                for member in args.members {
                    let member_repo = RepoRoot::discover_from(&member)?;
                    repos.push(workspace_repo_entry(&repo.path, &member_repo.path, None)?);
                }
                let manifest = WorkspaceManifest {
                    schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
                    workspace_id: None,
                    repos,
                };
                let manifest_path = ctxpack_index::write_workspace_manifest(&repo.path, &manifest)?;
                let report =
                    ctxpack_index::workspace_inventory_status(&repo.path, Some(&manifest_path))?;
                match args.format {
                    PackFormat::Markdown => {
                        println!("Created workspace manifest at {}", manifest_path.display());
                        print_workspace_status_report(&report);
                    }
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            WorkspaceCommand::PrepareTask(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let plan = prepare_workspace_context_plan(
                    &repo.path,
                    args.manifest.as_deref(),
                    &args.task,
                    args.mode.into(),
                    &args.paths,
                    args.semantic,
                )?;
                match args.format {
                    PackFormat::Markdown => print_workspace_context_plan(&plan),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&plan)?),
                }
            }
            WorkspaceCommand::GetPack(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let pack = compile_workspace_context_pack(
                    &repo.path,
                    args.manifest.as_deref(),
                    &args.task,
                    args.mode.into(),
                    args.budget.into(),
                    &args.paths,
                    &args.target_agent,
                    args.semantic,
                )?;
                match args.format {
                    PackFormat::Markdown => print_workspace_context_pack(&pack),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&pack)?),
                }
            }
            WorkspaceCommand::Policy(args) => match args.command {
                WorkspacePolicyCommand::Init(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = ctxpack_index::write_team_policy_template(&repo.path)?;
                    match args.format {
                        PackFormat::Markdown => print_team_policy_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
                WorkspacePolicyCommand::Status(args) => {
                    let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                    let repo = RepoRoot::discover_from(&start)?;
                    let report = ctxpack_index::team_policy_report(&repo.path)?;
                    match args.format {
                        PackFormat::Markdown => print_team_policy_report(&report),
                        PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    }
                }
            },
            WorkspaceCommand::Status(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = ctxpack_index::workspace_inventory_status(
                    &repo.path,
                    args.manifest.as_deref(),
                )?;
                match args.format {
                    PackFormat::Markdown => print_workspace_status_report(&report),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
        },
        Command::ServeMcp => {
            ctxpack_mcp::run_stdio_server()?;
        }
    }
    Ok(())
}

fn init_options(args: &InitArgs) -> InitOptions {
    adapter_options(args.cursor, args.claude, args.opencode)
}

fn setup_check_options(args: &SetupCheckArgs) -> InitOptions {
    adapter_options(args.cursor, args.claude, args.opencode)
}

fn workspace_repo_entry(
    workspace_root: &Path,
    repo_root: &Path,
    label: Option<&str>,
) -> Result<WorkspaceRepo> {
    let canonical = fs::canonicalize(repo_root)?;
    let path = workspace_manifest_path_label(workspace_root, &canonical);
    let label = label
        .map(str::to_string)
        .or_else(|| {
            canonical
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "repo".to_string());
    Ok(WorkspaceRepo {
        id: Some(ctxpack_index::repo_id_for_path(&canonical)),
        path,
        label: Some(label),
        tags: Vec::new(),
    })
}

fn workspace_manifest_path_label(workspace_root: &Path, repo_root: &Path) -> String {
    repo_root
        .strip_prefix(workspace_root)
        .ok()
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| {
            path.components()
                .filter_map(|component| match component {
                    std::path::Component::Normal(part) => Some(part.to_string_lossy()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|| {
            if repo_root == workspace_root {
                ".".to_string()
            } else {
                repo_root.display().to_string()
            }
        })
}

fn parse_regression_threshold(
    input: &str,
) -> std::result::Result<BenchmarkRegressionThreshold, String> {
    let Some((metric, max_drop)) = input.split_once('=') else {
        return Err("expected metric=max_drop".to_string());
    };
    let metric = metric.trim();
    if metric.is_empty() {
        return Err("metric name cannot be empty".to_string());
    }
    let max_drop = max_drop
        .trim()
        .parse::<f32>()
        .map_err(|_| "max_drop must be a number".to_string())?;
    if max_drop < 0.0 {
        return Err("max_drop must be non-negative".to_string());
    }
    Ok(BenchmarkRegressionThreshold {
        metric: metric.to_string(),
        max_drop,
    })
}

fn adapter_options(cursor: bool, claude: bool, opencode: bool) -> InitOptions {
    let mut adapters = Vec::new();
    if cursor {
        adapters.push(AgentAdapter::Cursor);
    }
    if claude {
        adapters.push(AgentAdapter::Claude);
    }
    if opencode {
        adapters.push(AgentAdapter::OpenCode);
    }
    InitOptions { adapters }
}

fn print_init_report(report: &InitReport) {
    println!("Initialized ctxpack in {}", report.repo_root.display());
    for file in &report.files {
        let action = match file.action {
            InitAction::Created => "created",
            InitAction::Updated => "updated",
            InitAction::Unchanged => "unchanged",
            InitAction::Skipped => "skipped",
        };
        println!("- {action}: {}", file.path.display());
    }
    if report
        .files
        .iter()
        .any(|file| file.action == InitAction::Skipped)
    {
        println!(
            "  skipped adapter files can be generated by rerunning init with --cursor, --claude, or --opencode."
        );
    }
    println!();
    println!("Next steps:");
    for step in &report.next_steps {
        println!("- {}: {}", step.label, step.command);
        println!("  {}", step.detail);
    }
    println!();
    println!(
        "ctxpack writes repo-local setup files only; it does not mutate global agent config. Copy/paste or merge agent configuration explicitly."
    );
    println!("{}", report.codex_mcp_setup);
}

fn print_setup_check_report(report: &SetupCheckReport) {
    println!("Setup check for {}", report.repo_root.display());
    for item in &report.items {
        let status = match item.status {
            SetupCheckStatus::Pass => "pass",
            SetupCheckStatus::Warn => "warn",
            SetupCheckStatus::Fail => "fail",
        };
        println!("- {status}: {}", item.name);
        println!("  {}", item.detail);
    }
    println!();
    println!(
        "setup-check is read-only and validates generated setup artifacts only; it does not mutate global agent config."
    );
    if report.passed {
        println!("Result: passed");
    } else {
        println!("Result: failed");
    }
}

fn build_doctor_report(args: &DoctorArgs) -> Result<serde_json::Value> {
    let binary_path = match &args.binary {
        Some(path) => path.clone(),
        None => std::env::current_exe()?,
    };
    let binary_absolute = binary_path.is_absolute();
    let binary_exists = binary_path.is_file();
    let binary_version = command_first_line(&binary_path, "--version");
    let binary_help = command_success(&binary_path, "--help");

    let mut checks = vec![
        serde_json::json!({
            "name": "binary_path_absolute",
            "passed": binary_absolute,
            "detail": if binary_absolute {
                "binary path is absolute"
            } else {
                "use an absolute ctxpack binary path for MCP clients"
            },
        }),
        serde_json::json!({
            "name": "binary_exists",
            "passed": binary_exists,
            "detail": if binary_exists {
                "binary exists"
            } else {
                "binary path does not point to a file"
            },
        }),
        serde_json::json!({
            "name": "binary_version",
            "passed": binary_version.is_some(),
            "detail": binary_version
                .clone()
                .unwrap_or_else(|| "ctxpack --version failed".to_string()),
        }),
        serde_json::json!({
            "name": "binary_help",
            "passed": binary_help,
            "detail": if binary_help {
                "ctxpack --help succeeded"
            } else {
                "ctxpack --help failed"
            },
        }),
    ];

    let mut release_manifest = serde_json::Value::Null;
    if let Some(path) = &args.release_manifest {
        let manifest_text = fs::read_to_string(path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_text)?;
        let manifest_version = manifest["version"].as_str().unwrap_or_default();
        let version_matches = binary_version
            .as_deref()
            .map(|version| version.contains(manifest_version))
            .unwrap_or(false);
        let manifest_local_only = manifest["privacyStatus"]["localOnly"]
            .as_bool()
            .unwrap_or(false);
        let has_archive_checksum = manifest["archive"]["sha256"].as_str().is_some();
        let has_binary_checksum = manifest["binary"]["sha256"].as_str().is_some();
        checks.push(serde_json::json!({
            "name": "release_manifest_version",
            "passed": version_matches,
            "detail": format!("manifest version {manifest_version} matches active binary"),
        }));
        checks.push(serde_json::json!({
            "name": "release_manifest_privacy",
            "passed": manifest_local_only,
            "detail": "release manifest declares local-only privacy status",
        }));
        checks.push(serde_json::json!({
            "name": "release_manifest_checksums",
            "passed": has_archive_checksum && has_binary_checksum,
            "detail": "release manifest includes archive and binary SHA-256 values",
        }));
        release_manifest = serde_json::json!({
            "path": path,
            "version": manifest_version,
            "archiveName": manifest["archive"]["name"].clone(),
            "auditReport": manifest["auditReport"].clone(),
            "localOnly": manifest_local_only,
        });
    }

    let mut storage = serde_json::Value::Null;
    if let Some(repo_input) = &args.repo {
        let repo = RepoRoot::discover_from(repo_input)?;
        let status = storage_status_for_repo(
            &repo.path,
            &StoreConfig {
                path_override: None,
            },
        )?;
        let compatibility_label = format!("{:?}", status.compatibility);
        let no_existing_state = status.schema_version.is_none();
        let compatible = matches!(
            compatibility_label.as_str(),
            "Compatible" | "MissingMetadata"
        ) || no_existing_state;
        checks.push(serde_json::json!({
            "name": "local_state_compatibility",
            "passed": compatible,
            "detail": format!("storage compatibility: {compatibility_label}"),
        }));
        storage = serde_json::json!({
            "repo": repo.path,
            "databasePath": status.database_path,
            "schemaVersion": status.schema_version,
            "compatibility": status.compatibility,
            "diagnostics": status.diagnostics,
        });
    }

    let passed = checks
        .iter()
        .all(|check| check["passed"].as_bool().unwrap_or(false));

    Ok(serde_json::json!({
        "passed": passed,
        "binary": {
            "path": binary_path,
            "absolute": binary_absolute,
            "exists": binary_exists,
            "version": binary_version,
            "helpOk": binary_help,
        },
        "releaseManifest": release_manifest,
        "storage": storage,
        "checks": checks,
        "privacyStatus": {
            "localOnly": true,
            "remoteEmbeddingsUsed": false,
            "remoteRerankingUsed": false,
            "sourceTextLogged": false,
        },
        "mutatesGlobalAgentConfig": false,
    }))
}

fn command_first_line(binary_path: &Path, arg: &str) -> Option<String> {
    let output = std::process::Command::new(binary_path)
        .arg(arg)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::to_string)
}

fn command_success(binary_path: &Path, arg: &str) -> bool {
    std::process::Command::new(binary_path)
        .arg(arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn print_doctor_report(report: &serde_json::Value) {
    println!("# ctxpack Install Doctor");
    println!();
    println!("- Passed: `{}`", report["passed"]);
    println!("- Binary: `{}`", report["binary"]["path"]);
    if let Some(version) = report["binary"]["version"].as_str() {
        println!("- Version: `{version}`");
    }
    println!("- Global agent config mutation: `false`");
    println!("- Privacy: local-only, sourceTextLogged=false");
    if !report["releaseManifest"].is_null() {
        println!(
            "- Release manifest: version `{}`, archive `{}`",
            report["releaseManifest"]["version"], report["releaseManifest"]["archiveName"]
        );
    }
    if !report["storage"].is_null() {
        println!(
            "- Local state compatibility: `{}`",
            report["storage"]["compatibility"]
        );
    }
    println!();
    println!("## Checks");
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            let marker = if check["passed"].as_bool().unwrap_or(false) {
                "pass"
            } else {
                "fail"
            };
            println!("- {marker}: {} — {}", check["name"], check["detail"]);
        }
    }
}

fn print_inventory_report(report: &InventoryReport) {
    println!("Indexed {}", report.repo_root.display());
    println!("- repo id: {}", report.repo_id);
    println!("- files: {}", report.file_count);
    println!(
        "- generated excluded by default: {}",
        report.generated_count
    );
    println!(
        "- sensitive excluded by default: {}",
        report.sensitive_count
    );
    println!("- inventory: {}", report.inventory_path.display());
}

fn print_storage_index_report(report: &StorageIndexReport) {
    println!("Storage sync {}", report.repo_root.display());
    println!("- repo id: {}", report.repo_id);
    println!("- database: {}", report.database_path.display());
    println!("- schema version: {}", report.schema_version);
    println!("- compatibility: {:?}", report.compatibility);
    println!("- reused records: {}", report.reused_records);
    println!("- created records: {}", report.created_records);
    println!("- updated records: {}", report.updated_records);
    println!("- deleted records: {}", report.deleted_records);
    println!("- skipped files: {}", report.skipped_files);
    println!("- generated paths: {}", report.generated_paths);
    println!("- sensitive paths: {}", report.sensitive_paths);
    print_diagnostics(&report.diagnostics);
}

fn print_semantic_storage_report(report: &StorageSemanticIndexReport) {
    println!("Semantic storage sync");
    println!("- repo id: {}", report.repo_id);
    println!("- database: {}", report.database_path.display());
    println!("- schema version: {}", report.schema_version);
    println!("- reused records: {}", report.reused_records);
    println!("- created records: {}", report.created_records);
    println!("- updated records: {}", report.updated_records);
    println!("- deleted records: {}", report.deleted_records);
    println!(
        "- semantic vector records: {}",
        report.semantic_vector_records
    );
    println!("- compatibility: {:?}", report.compatibility);
    print_diagnostics(&report.diagnostics);
}

fn print_precision_import_report(report: &PrecisionImportReport) {
    println!("Precision edge import");
    println!("- provider: {}", report.provider);
    println!("- overlay: {}", report.path);
    println!("- accepted edges: {}", report.accepted_edges);
    println!("- rejected edges: {}", report.rejected_edges);
    print_diagnostics(&report.diagnostics);
}

fn print_storage_report(report: &StorageReport) {
    println!("# ctxpack Storage");
    println!();
    println!("- Repo ID: `{}`", report.repo_id);
    println!("- Repo root: `{}`", report.repo_root.display());
    println!("- Database: `{}`", report.database_path.display());
    println!("- Schema version: `{}`", report.schema_version);
    println!("- ctxpack version: `{}`", report.ctxpack_version);
    println!("- Ranking version: `{}`", report.ranking_version);
    println!("- Compiler version: `{}`", report.compiler_version);
    println!("- Privacy: `{:?}`", report.privacy_mode);
    println!("- Compatibility: `{:?}`", report.compatibility);
    print_diagnostics(&report.diagnostics);
}

fn print_storage_status_report(report: &StorageStatusReport) {
    println!("# ctxpack Storage Status");
    println!();
    if let Some(repo_id) = &report.repo_id {
        println!("- Repo ID: `{repo_id}`");
    }
    if let Some(repo_root) = &report.repo_root {
        println!("- Repo root: `{}`", repo_root.display());
    }
    println!("- Database: `{}`", report.database_path.display());
    println!("- Schema version: `{:?}`", report.schema_version);
    println!("- Compatibility: `{:?}`", report.compatibility);
    println!("- File records: `{}`", report.file_records);
    println!("- Symbol records: `{}`", report.symbol_records);
    println!("- Context pack records: `{}`", report.context_pack_records);
    println!(
        "- Benchmark run records: `{}`",
        report.benchmark_run_records
    );
    println!("- Proof report records: `{}`", report.proof_report_records);
    println!(
        "- Semantic vector records: `{}`",
        report.semantic_vector_records
    );
    println!("- Memory card records: `{}`", report.memory_card_records);
    print_diagnostics(&report.diagnostics);
}

fn print_workspace_status_report(report: &WorkspaceInventoryReport) {
    println!("# ctxpack Workspace Status");
    println!();
    println!("- Workspace root: `{}`", report.workspace_root);
    println!("- Manifest: `{}`", report.manifest_path);
    println!("- Repositories: `{}`", report.repo_count);
    println!(
        "- Available repositories: `{}`",
        report.available_repo_count
    );
    println!("- Files: `{}`", report.file_count);
    println!("- Generated paths: `{}`", report.generated_count);
    println!("- Sensitive paths: `{}`", report.sensitive_count);
    println!("- Source text logged: `{}`", report.source_text_logged);
    println!(
        "- Privacy: local-only `{}`",
        report.privacy_status.local_only
    );
    println!();
    println!("## Repositories");
    for repo in &report.repos {
        println!(
            "- `{}` ({}) — {:?}, files `{}`, storage `{:?}`, memory cards `{}`",
            repo.label,
            repo.path_label,
            repo.state,
            repo.file_count,
            repo.storage_compatibility,
            repo.memory_card_count
        );
        for diagnostic in &repo.diagnostics {
            println!(
                "  - `{:?}` `{}`: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            );
        }
    }
    if !report.diagnostics.is_empty() {
        println!();
        println!("## Workspace Diagnostics");
        for diagnostic in &report.diagnostics {
            println!(
                "- `{:?}` `{}`: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            );
        }
    }
}

fn print_workspace_context_plan(plan: &WorkspaceContextPlan) {
    println!("# ctxpack Workspace Context Plan");
    println!();
    println!("- Workspace root: `{}`", plan.workspace_root);
    println!("- Manifest: `{}`", plan.manifest_path);
    println!("- Task type: `{:?}`", plan.task_type);
    println!("- Confidence: `{:.2}`", plan.confidence);
    println!("- Selected repositories: `{}`", plan.selected_repo_count);
    println!("- Source text logged: `{}`", plan.source_text_logged);
    println!("- Privacy: local-only `{}`", plan.privacy_status.local_only);
    println!();
    println!("## Repository Plans");
    for repo in &plan.repo_plans {
        println!(
            "- `{}` ({}) confidence `{:.2}`: {}",
            repo.label, repo.path_label, repo.confidence, repo.reason
        );
        for target in &repo.context_plan.target_files {
            println!("  - target `{}`: {}", target.path, target.reason);
        }
        for test in &repo.context_plan.related_tests {
            println!("  - test `{}`: {}", test.path, test.reason);
        }
    }
    if !plan.diagnostics.is_empty() {
        println!();
        println!("## Workspace Diagnostics");
        for diagnostic in &plan.diagnostics {
            println!(
                "- `{:?}` `{}`: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            );
        }
    }
}

fn print_workspace_context_pack(pack: &WorkspaceContextPack) {
    println!("# ctxpack Workspace Context Pack");
    println!();
    println!("- Pack ID: `{}`", pack.id);
    println!("- Task ID: `{}`", pack.task_id);
    println!("- Workspace root: `{}`", pack.workspace_root);
    println!("- Manifest: `{}`", pack.manifest_path);
    println!("- Target agent: `{}`", pack.target_agent);
    println!("- Budget: `{:?}`", pack.budget);
    println!("- Confidence: `{:.2}`", pack.confidence);
    println!("- Estimated tokens: `{}`", pack.token_estimate);
    println!("- Selected repositories: `{}`", pack.selected_repo_count);
    println!("- Source text logged: `{}`", pack.source_text_logged);
    println!("- Privacy: local-only `{}`", pack.privacy_status.local_only);
    if !pack.warnings.is_empty() {
        println!();
        println!("## Warnings");
        for warning in &pack.warnings {
            println!("- {warning}");
        }
    }
    for repo in &pack.repo_packs {
        println!();
        println!(
            "## Repository: {} ({})\n\nConfidence `{:.2}`. {}",
            repo.label, repo.path_label, repo.confidence, repo.reason
        );
        println!("{}", render_pack_markdown(&repo.context_pack));
    }
    if !pack.diagnostics.is_empty() {
        println!();
        println!("## Workspace Diagnostics");
        for diagnostic in &pack.diagnostics {
            println!(
                "- `{:?}` `{}`: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            );
        }
    }
}

fn print_shared_artifact_manifest(manifest: &SharedArtifactManifest) {
    println!("# ctxpack Shared Artifact Manifest");
    println!();
    println!("- Repo ID: `{}`", manifest.repo_id);
    println!("- Repo label: `{}`", manifest.repo_label);
    println!("- Artifacts: `{}`", manifest.artifacts.len());
    println!("- Source text logged: `{}`", manifest.source_text_logged);
    println!(
        "- Privacy: local-only `{}`",
        manifest.privacy_status.local_only
    );
    for artifact in &manifest.artifacts {
        println!(
            "- `{:?}` `{}`: {:?}, hash `{:?}`",
            artifact.kind, artifact.path_label, artifact.status, artifact.content_hash
        );
    }
}

fn print_shared_artifact_inspection(report: &SharedArtifactInspectionReport) {
    println!("# ctxpack Shared Artifact Inspection");
    println!();
    println!("- Manifest: `{}`", report.manifest_path);
    println!("- Compatible: `{}`", report.compatible);
    println!("- Artifacts: `{}`", report.artifact_count);
    println!("- Source text logged: `{}`", report.source_text_logged);
    println!(
        "- Privacy: local-only `{}`",
        report.privacy_status.local_only
    );
    for artifact in &report.artifacts {
        println!(
            "- `{:?}` `{}`: {:?}",
            artifact.kind, artifact.path_label, artifact.status
        );
    }
    for diagnostic in &report.diagnostics {
        println!(
            "- `{:?}` `{}`: {}",
            diagnostic.severity, diagnostic.code, diagnostic.message
        );
    }
}

fn print_team_policy_report(report: &TeamPolicyReport) {
    println!("# ctxpack Team Policy");
    println!();
    println!("- Policy: `{}`", report.policy_path);
    println!("- Name: `{}`", report.policy.name);
    println!(
        "- Workspace indexing: `{}`",
        report.policy.allow_workspace_indexing
    );
    println!(
        "- Artifact export: `{}`",
        report.policy.allow_artifact_export
    );
    println!(
        "- Cloud embeddings: `{}`",
        report.policy.allow_cloud_embeddings
    );
    println!(
        "- Cloud reranking: `{}`",
        report.policy.allow_cloud_reranking
    );
    println!("- Redact secrets: `{}`", report.policy.redact_secrets);
    println!("- Source text logged: `{}`", report.source_text_logged);
    println!("- Allowed artifacts: `{}`", report.allowed_artifacts.len());
    println!("- Blocked artifacts: `{}`", report.blocked_artifacts.len());
    println!(
        "- Degraded artifacts: `{}`",
        report.degraded_artifacts.len()
    );
    println!(
        "- Redacted artifacts: `{}`",
        report.redacted_artifacts.len()
    );
    for diagnostic in &report.diagnostics {
        println!(
            "- `{:?}` `{}`: {}",
            diagnostic.severity, diagnostic.code, diagnostic.message
        );
    }
}

fn print_diagnostics(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    println!();
    println!("## Diagnostics");
    for diagnostic in diagnostics {
        println!(
            "- `{:?}` `{}`: {}",
            diagnostic.severity, diagnostic.code, diagnostic.message
        );
    }
}

fn persist_historical_eval_report(
    repo: &Path,
    config: &StoreConfig,
    report: &HistoricalEvalReport,
) -> Result<StorageStatusReport> {
    Ok(ctxpack_index::persist_benchmark_run_record(
        repo,
        config,
        &StorageBenchmarkRunRecord {
            run_id: report.eval_range_id.clone(),
            suite_id: "historical-eval".to_string(),
            revision_id: report.head.clone().or_else(|| report.base.clone()),
            budget: Some(report.effective_filters.ranking_budget.to_string()),
            privacy_status: privacy_status_label(&report.privacy_status),
            metrics: historical_eval_metrics(report),
            gaps: retrieval_gap_records(&report.retrieval_gap_summaries),
        },
    )?)
}

fn persist_benchmark_suite_report(
    config: &ctxpack_compiler::BenchmarkSuiteConfig,
    report: &BenchmarkSuiteReport,
    config_path: &Path,
) -> Result<()> {
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    for repo_report in &report.repositories {
        let Some(repo_config) = config
            .repositories
            .iter()
            .find(|candidate| candidate.name == repo_report.name)
        else {
            continue;
        };
        let repo_path = if repo_config.path.is_absolute() {
            repo_config.path.clone()
        } else {
            config_dir.join(&repo_config.path)
        };
        let Some(history) = repo_report.report.as_ref() else {
            continue;
        };
        let _status = ctxpack_index::persist_benchmark_run_record(
            &repo_path,
            &StoreConfig::default(),
            &StorageBenchmarkRunRecord {
                run_id: format!("{}:{}", report.suite_id, repo_report.name),
                suite_id: report.suite_id.clone(),
                revision_id: history.head.clone().or_else(|| history.base.clone()),
                budget: Some(repo_report.effective_config.ranking_budget.to_string()),
                privacy_status: privacy_status_label(&repo_report.privacy_status),
                metrics: historical_eval_metrics(history),
                gaps: retrieval_gap_records(&history.retrieval_gap_summaries),
            },
        )?;
    }
    Ok(())
}

fn persist_product_proof_report(
    config: &ctxpack_compiler::BenchmarkSuiteConfig,
    report: &ProductProofReport,
    config_path: &Path,
) -> Result<()> {
    persist_benchmark_suite_report(config, &report.benchmark_report, config_path)?;
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    for repo_report in &report.benchmark_report.repositories {
        let Some(repo_config) = config
            .repositories
            .iter()
            .find(|candidate| candidate.name == repo_report.name)
        else {
            continue;
        };
        let repo_path = if repo_config.path.is_absolute() {
            repo_config.path.clone()
        } else {
            config_dir.join(&repo_config.path)
        };
        let _status = ctxpack_index::persist_proof_report_record(
            &repo_path,
            &StoreConfig::default(),
            &StorageProofReportRecord {
                proof_id: format!("{}:{}", report.suite_id, repo_report.name),
                run_id: Some(format!("{}:{}", report.suite_id, repo_report.name)),
                headline_metrics_json: serde_json::to_string(&report.headline_metrics)?,
                limitations_json: serde_json::to_string(&report.limitations)?,
                privacy_status: privacy_status_label(&report.privacy_status),
            },
        )?;
    }
    Ok(())
}

fn historical_eval_metrics(report: &HistoricalEvalReport) -> Vec<StorageMetricRecord> {
    vec![
        metric("fileRecallAt5", report.file_recall_at_5),
        metric("fileRecallAt10", report.file_recall_at_10),
        metric(
            "lexicalBaselineRecallAt10",
            report.lexical_baseline_recall_at_10,
        ),
        metric("ctxpackLiftAt10", report.ctxpack_lift_at_10),
        metric("sourceRecallAt10", report.source_recall_at_10),
        metric("testRecallAt10", report.test_recall_at_10),
        metric("testRecommendationRate", report.test_recommendation_rate),
        metric(
            "averageRecommendedContextFiles",
            report.average_recommended_context_files,
        ),
    ]
}

fn metric(name: &str, value: f32) -> StorageMetricRecord {
    StorageMetricRecord {
        name: name.to_string(),
        value,
        budget: None,
        target_kind: None,
    }
}

fn retrieval_gap_records(gaps: &[ctxpack_compiler::RetrievalGapSummary]) -> Vec<StorageGapRecord> {
    gaps.iter()
        .map(|gap| StorageGapRecord {
            family: format!(
                "{}:{}:{}",
                json_label(&gap.role),
                gap.signal_gap,
                gap.path_family
            ),
            recommendation_area: Some(json_label(&gap.recommendation_area)),
            target_status: Some(json_label(&gap.target_status)),
            safe_path: gap.example_paths.first().cloned(),
            count: gap.missed_count,
        })
        .collect()
}

fn privacy_status_label(status: &PrivacyStatus) -> String {
    serde_json::to_string(status).unwrap_or_else(|_| "local_only".to_string())
}

fn json_label<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .map(|json| json.trim_matches('"').to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn write_or_print(output_path: Option<&Path>, artifact: &str) -> Result<()> {
    if let Some(path) = output_path {
        fs::write(path, artifact)?;
    } else {
        println!("{artifact}");
    }
    Ok(())
}

fn context_anchor_paths(
    repo: &Path,
    explicit_paths: Vec<String>,
    include_current_diff: bool,
) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    for path in explicit_paths {
        let path = path.trim();
        if !path.is_empty() && seen.insert(path.to_string()) {
            paths.push(path.to_string());
        }
    }
    if include_current_diff {
        let diff = current_diff_summary(
            repo,
            &CurrentDiffOptions {
                include_untracked: true,
            },
        )?;
        for path in diff
            .staged
            .into_iter()
            .chain(diff.unstaged.into_iter())
            .chain(diff.untracked.into_iter())
        {
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

fn trace_disabled_diagnostic() -> Diagnostic {
    Diagnostic {
        code: "trace_recording_disabled".to_string(),
        severity: DiagnosticSeverity::Info,
        message: "Eval trace recording was disabled for this command.".to_string(),
        paths: Vec::new(),
        count: 1,
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn print_feedback_record_status(event: &SessionFeedbackEvent, status: &ctxpack_core::TraceStatus) {
    println!("# ctxpack Feedback Event\n");
    println!("- Event ID: `{}`", event.id);
    println!("- Task hash: `{}`", event.task_hash);
    println!("- Task type: `{:?}`", event.task_type);
    println!("- Target agent: `{}`", event.target_agent);
    println!("- Outcome: `{:?}`", event.outcome);
    println!("- Source text logged: `{}`", event.source_text_logged);
    println!("- Status: `{:?}`", status.status);
    if let Some(path) = &status.path {
        println!("- Path: `{path}`");
    }
    if !status.diagnostics.is_empty() {
        println!("\n## Diagnostics\n");
        for diagnostic in &status.diagnostics {
            println!("- `{}`: {}", diagnostic.code, diagnostic.message);
        }
    }
}

fn render_feedback_events(events: &[SessionFeedbackEvent]) -> String {
    let mut output = String::from("# ctxpack Feedback Events\n\n");
    output.push_str(
        "Source-free feedback events compare ctxpack recommendations with what agents actually read, edited, tested, and validated.\n\n",
    );
    if events.is_empty() {
        output.push_str("No feedback events found for this repository.\n");
        return output;
    }
    for (index, event) in events.iter().enumerate() {
        output.push_str(&format!(
            "## Event {}\n\n- Event ID: `{}`\n- Task hash: `{}`\n- Task type: `{:?}`\n- Target agent: `{}`\n- Outcome: `{:?}`\n",
            index + 1,
            event.id,
            event.task_hash,
            event.task_type,
            event.target_agent,
            event.outcome
        ));
        if let Some(pack_id) = event.pack_id {
            output.push_str(&format!("- Pack ID: `{pack_id}`\n"));
        }
        if let Some(budget) = &event.budget {
            output.push_str(&format!("- Budget: `{:?}`\n", budget));
        }
        output.push_str(&format!(
            "- Created at: `{}`\n- Source text logged: `{}`\n\n",
            event.created_at_unix_seconds, event.source_text_logged
        ));
        push_named_values(&mut output, "Recommended files", &event.recommended_files);
        push_named_values(&mut output, "Read files", &event.read_files);
        push_named_values(&mut output, "Edited files", &event.edited_files);
        push_named_values(&mut output, "Tested files", &event.tested_files);
        push_named_values(
            &mut output,
            "User-corrected files",
            &event.user_corrected_files,
        );
        push_named_values(&mut output, "Tags", &event.tags);
        output.push('\n');
    }
    output
}

fn render_feedback_summary(summary: &FeedbackSummary) -> String {
    format!(
        "# ctxpack Feedback Summary\n\n- Repo ID: `{}`\n- Events: `{}`\n- Passed: `{}`\n- Failed: `{}`\n- Blocked: `{}`\n- Unknown: `{}`\n- Unique read files: `{}`\n- Unique edited files: `{}`\n- Unique tested files: `{}`\n- Unique user-corrected files: `{}`\n- Source text logged: `{}`\n",
        summary.repo_id,
        summary.event_count,
        summary.passed_count,
        summary.failed_count,
        summary.blocked_count,
        summary.unknown_count,
        summary.read_file_count,
        summary.edited_file_count,
        summary.tested_file_count,
        summary.user_corrected_file_count,
        summary.source_text_logged
    )
}

fn render_policy_quality_report(report: &PolicyQualityReport) -> String {
    let mut output = String::from("# ctxpack Policy Quality Report\n\n");
    output.push_str(&format!("- Repo ID: `{}`\n", report.repo_id));
    output.push_str(&format!("- Events: `{}`\n", report.event_count));
    if let Some(warning) = &report.sample_warning {
        output.push_str(&format!("- Sample warning: `{warning}`\n"));
    }
    output.push_str(&format!(
        "- Context precision: `{:.2}`\n- Read precision: `{:.2}`\n- Edit recall proxy: `{:.2}`\n- Validation coverage: `{:.2}`\n- Correction rate: `{:.2}`\n- Source text logged: `{}`\n\n",
        report.context_precision,
        report.read_precision,
        report.edit_recall_proxy,
        report.validation_coverage,
        report.correction_rate,
        report.source_text_logged
    ));

    output.push_str("## Token ROI\n\n");
    if report.token_roi.is_empty() {
        output.push_str("_No token ROI samples._\n\n");
    } else {
        for metric in &report.token_roi {
            output.push_str(&format!(
                "- Budget: `{:?}` events `{}` useful files/event `{:.2}`\n",
                metric.budget, metric.event_count, metric.useful_files_per_event
            ));
        }
        output.push('\n');
    }

    output.push_str("## Repeated Missing File Families\n\n");
    if report.repeated_missing_file_families.is_empty() {
        output.push_str("_No repeated missing file families._\n\n");
    } else {
        for family in &report.repeated_missing_file_families {
            output.push_str(&format!("- `{}`: `{}`\n", family.path, family.count));
        }
        output.push('\n');
    }

    output.push_str("## Signal Contributions\n\n");
    for signal in &report.signal_contributions {
        output.push_str(&format!(
            "- `{:?}`: events `{}`, useful hits `{}`, score `{:.2}`\n",
            signal.signal, signal.event_count, signal.useful_file_hits, signal.score
        ));
    }
    output
}

fn render_policy_profiles(profiles: &[RetrievalPolicyProfile]) -> String {
    let mut output = String::from("# ctxpack Policy Profiles\n\n");
    if profiles.is_empty() {
        output.push_str("No policy profiles found for this repository.\n");
        return output;
    }
    for profile in profiles {
        output.push_str(&format!(
            "## `{}`\n\n- Status: `{:?}`\n- Events: `{}`\n- Created at: `{}`\n- Source text logged: `{}`\n- Rationale: {}\n\n",
            profile.id,
            profile.status,
            profile.source_report_event_count,
            profile.created_at_unix_seconds,
            profile.source_text_logged,
            profile.rationale
        ));
        output.push_str("### Weights\n\n");
        for weight in &profile.weights {
            output.push_str(&format!(
                "- `{:?}`: `{:.2}` — {}\n",
                weight.signal, weight.weight, weight.rationale
            ));
        }
        output.push_str("\n### Safety Floors\n\n");
        for floor in &profile.safety_floors {
            output.push_str(&format!(
                "- `{:?}`: min `{:.2}` — {}\n",
                floor.signal, floor.minimum_weight, floor.reason
            ));
        }
        if !profile.regression_warnings.is_empty() {
            output.push_str("\n### Regression Warnings\n\n");
            for warning in &profile.regression_warnings {
                output.push_str(&format!("- {warning}\n"));
            }
        }
        output.push('\n');
    }
    output
}

fn print_policy_action_report(
    report: &PolicyProfileActionReport,
    format: &PackFormat,
) -> Result<()> {
    match format {
        PackFormat::Markdown => {
            println!(
                "# ctxpack Policy Profile Action\n\n- Repo ID: `{}`\n- Profile ID: `{}`\n- Action: `{}`\n- Active profile ID: `{:?}`\n- Source text logged: `{}`",
                report.repo_id,
                report.profile_id,
                report.action,
                report.active_profile_id,
                report.source_text_logged
            );
        }
        PackFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
    }
    Ok(())
}

fn render_outcome_comparison_report(report: &AgentOutcomeComparisonReport) -> String {
    let mut output = String::from("# ctxpack Agent Outcome Comparison\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Events: `{}`\n- Changed sample warning: `{}`\n- Low information warning: `{}`\n- Source text logged: `{}`\n",
        report.repo_id,
        report.event_count,
        report.changed_sample_warning,
        report.low_information_warning,
        report.source_text_logged
    ));
    if let Some(warning) = &report.sample_warning {
        output.push_str(&format!("- Sample warning: `{warning}`\n"));
    }
    output.push_str("\n## Budgets\n\n");
    if report.budgets.is_empty() {
        output.push_str("_No budget outcomes found._\n");
        return output;
    }
    for budget in &report.budgets {
        output.push_str(&format!(
            "- Budget: `{:?}` events `{}` pass `{:.2}` blocked `{:.2}` corrections `{:.2}` validation `{:.2}` avg context `{:.2}` useful/1k `{:.2}`\n",
            budget.budget,
            budget.event_count,
            budget.pass_rate,
            budget.blocked_rate,
            budget.correction_rate,
            budget.validation_coverage,
            budget.average_recommended_context_size,
            budget.useful_target_files_per_1k_tokens
        ));
    }
    output
}

fn render_retrieval_health_report(report: &RetrievalHealthReport) -> String {
    let mut output = String::from("# ctxpack Retrieval Health Report\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Evaluated commits: `{}`\n- Feedback events: `{}`\n- Health score: `{:.2}`\n- Source text logged: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.evaluated_commits,
        report.feedback_events,
        report.health_score,
        report.source_text_logged,
        report.privacy_status.local_only
    ));

    output.push_str("## Metrics\n\n");
    for metric in &report.metrics {
        output.push_str(&format!(
            "- `{}`: `{:.2}` {} ({})\n",
            metric.name, metric.value, metric.unit, metric.source
        ));
    }
    if report.metrics.is_empty() {
        output.push_str("- No metrics available.\n");
    }
    output.push('\n');

    output.push_str("## Token ROI\n\n");
    for row in &report.token_roi {
        output.push_str(&format!(
            "- Source `{}` budget `{:?}` events `{}` useful/event `{:.2}` useful/1k `{:.2}` recall `{:.2}` larger pack adds little value `{}`\n",
            row.source,
            row.budget,
            row.event_count,
            row.useful_files_per_event,
            row.useful_targets_per_1k_tokens,
            row.recall_at_cutoff,
            row.larger_pack_adds_little_value
        ));
    }
    if report.token_roi.is_empty() {
        output.push_str("- No token ROI available.\n");
    }
    output.push('\n');

    output.push_str("## Signal Contributions\n\n");
    for signal in &report.signal_contributions {
        output.push_str(&format!(
            "- `{:?}` from `{}`: events `{}`, useful hits `{}`, score `{:.2}`, recall without signal `{:?}`, lift vs lexical `{:?}`\n",
            signal.signal,
            signal.source,
            signal.event_count,
            signal.useful_file_hits,
            signal.score,
            signal.recall_without_signal,
            signal.recall_lift_vs_lexical_at_k
        ));
    }
    if report.signal_contributions.is_empty() {
        output.push_str("- No signal contribution evidence available.\n");
    }
    output.push('\n');

    output.push_str("## Gap Families\n\n");
    for gap in &report.gap_families {
        output.push_str(&format!(
            "- `{}` from `{}` count `{}` area `{:?}` status `{:?}` path `{:?}`\n",
            gap.family,
            gap.source,
            gap.count,
            gap.recommendation_area,
            gap.target_status,
            gap.safe_path
        ));
    }
    if report.gap_families.is_empty() {
        output.push_str("- No repeated gap families found.\n");
    }
    output.push('\n');

    output.push_str("## Low Confidence Flags\n\n");
    if report.low_confidence_flags.is_empty() {
        output.push_str("- No low-confidence flags.\n");
    } else {
        for flag in &report.low_confidence_flags {
            output.push_str(&format!("- `{flag}`\n"));
        }
    }
    output
}

fn render_agent_preview_report(report: &AgentPreviewReport) -> String {
    let mut output = String::from("# ctxpack Agent Preview\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Task hash: `{}`\n- Task type: `{:?}`\n- Budget: `{:?}`\n- Agent previews: `{}`\n- Source text logged: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.task_hash,
        report.task_type,
        report.budget,
        report.previews.len(),
        report.source_text_logged,
        report.privacy_status.local_only
    ));

    for preview in &report.previews {
        output.push_str(&format!(
            "## {}\n\n- Target agent: `{}`\n- Pack resource: `{}`\n- Source text included: `{}`\n\n",
            preview.display_name,
            preview.target_agent,
            preview.pack_resource_uri,
            preview.source_text_included
        ));

        output.push_str("### MCP Tools\n\n");
        push_plain_path_list(&mut output, &preview.mcp_tools, "No MCP tools.");

        output.push_str("\n### MCP Resources\n\n");
        push_plain_path_list(&mut output, &preview.mcp_resources, "No MCP resources.");

        output.push_str("\n### Guidance\n\n");
        for surface in &preview.guidance {
            output.push_str(&format!(
                "- `{:?}` `{}` path `{:?}` — {}\n",
                surface.kind, surface.label, surface.path, surface.summary
            ));
        }
        for surface in &preview.native_rules {
            output.push_str(&format!(
                "- `{:?}` `{}` path `{:?}` — {}\n",
                surface.kind, surface.label, surface.path, surface.summary
            ));
        }

        output.push_str("\n### Next Steps\n\n");
        for step in &preview.next_steps {
            output.push_str(&format!(
                "{}. `{}` owns this step; source-bearing `{}`: {}\n",
                step.order, step.owner, step.source_bearing, step.action
            ));
        }

        output.push_str("\n### Boundary\n\n");
        for boundary in &preview.boundary {
            output.push_str(&format!("- {boundary}\n"));
        }
        output.push('\n');
    }

    if !report.diagnostics.is_empty() {
        output.push_str("## Diagnostics\n\n");
        print_diagnostics_to_string(&mut output, &report.diagnostics);
    }

    output
}

fn render_graph_neighborhood_report(report: &GraphNeighborhoodReport) -> String {
    let mut output = String::from("# ctxpack Graph Neighborhood\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Task hash: `{:?}`\n- Anchors: `{}`\n- Nodes: `{}` / max `{}`\n- Edges: `{}` / max `{}`\n- Capped: `{}`\n- Source text logged: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.task_hash,
        report.anchors.len(),
        report.nodes.len(),
        report.max_nodes,
        report.edges.len(),
        report.max_edges,
        report.capped,
        report.source_text_logged,
        report.privacy_status.local_only
    ));

    output.push_str("## Anchors\n\n");
    push_plain_path_list(&mut output, &report.anchors, "No anchors selected.");

    output.push_str("\n## Communities\n\n");
    if report.communities.is_empty() {
        output.push_str("- No communities.\n");
    } else {
        for community in &report.communities {
            output.push_str(&format!(
                "- `{}`: nodes `{}`, edges `{}` — {}\n",
                community.label, community.node_count, community.edge_count, community.summary
            ));
        }
    }

    output.push_str("\n## Nodes\n\n");
    if report.nodes.is_empty() {
        output.push_str("- No nodes.\n");
    } else {
        for node in &report.nodes {
            output.push_str(&format!(
                "- `{}` {:?} label `{}` path `{:?}` role `{:?}` weight `{:.2}` source `{}`\n",
                node.id, node.kind, node.label, node.path, node.role, node.weight, node.source
            ));
        }
    }

    output.push_str("\n## Edges\n\n");
    if report.edges.is_empty() {
        output.push_str("- No edges.\n");
    } else {
        for edge in &report.edges {
            output.push_str(&format!(
                "- `{}` -> `{}` `{}` weight `{:.2}` — {}\n",
                edge.source, edge.target, edge.kind, edge.weight, edge.reason
            ));
        }
    }

    if !report.diagnostics.is_empty() {
        output.push_str("\n## Diagnostics\n\n");
        print_diagnostics_to_string(&mut output, &report.diagnostics);
    }

    output
}

fn render_semantic_provider_status(report: &SemanticProviderStatusReport) -> String {
    let mut output = String::from("# ctxpack Semantic Provider Status\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Provider: `{}`\n- Model: `{}`\n- Dimensions: `{}`\n- Distance metric: `{}`\n- Enabled by default: `{}`\n- Cloud embeddings allowed: `{}`\n- Cloud reranking allowed: `{}`\n- Local vector count: `{}`\n- Stored vector count: `{}`\n- Indexing freshness: `{}`\n- Source text logged: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.provider_kind,
        report.model_id,
        report.dimensions,
        report.distance_metric,
        report.enabled_by_default,
        report.cloud_embeddings_allowed,
        report.cloud_reranking_allowed,
        report.local_vector_count,
        report.stored_vector_count,
        report.indexing_freshness,
        report.source_text_logged,
        report.privacy_status.local_only
    ));
    output.push_str("## Usage\n\n");
    if report.usage.is_empty() {
        output.push_str("- No semantic usage sample requested.\n");
    } else {
        for usage in &report.usage {
            output.push_str(&format!(
                "- `{}` semantic `{}` candidates `{}` remote embeddings `{}`\n",
                usage.surface,
                usage.semantic_enabled,
                usage.semantic_candidate_count,
                usage.remote_embeddings_used
            ));
        }
    }
    output
}

fn render_policy_experiment_report(report: &RetrievalPolicyExperimentReport) -> String {
    let mut output = String::from("# ctxpack Retrieval Policy Experiments\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Task hash: `{}`\n- Source text logged: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.task_hash,
        report.source_text_logged,
        report.privacy_status.local_only
    ));
    output.push_str("## Rows\n\n");
    for row in &report.rows {
        output.push_str(&format!(
            "- `{}` semantic `{}` graph `{}` fileRecall@10 `{:?}` testRecall@10 `{:?}` graph nodes `{}` graph edges `{}` — {}\n",
            row.policy,
            row.semantic_enabled,
            row.graph_enabled,
            row.file_recall_at_10,
            row.test_recall_at_10,
            row.graph_node_count,
            row.graph_edge_count,
            row.note
        ));
    }
    if !report.diagnostics.is_empty() {
        output.push_str("\n## Diagnostics\n\n");
        print_diagnostics_to_string(&mut output, &report.diagnostics);
    }
    output
}

fn push_named_values(output: &mut String, title: &str, values: &[String]) {
    output.push_str(&format!("### {title}\n\n"));
    if values.is_empty() {
        output.push_str("_None._\n\n");
        return;
    }
    for value in values {
        output.push_str(&format!("- `{value}`\n"));
    }
    output.push('\n');
}

fn render_eval_checklist_with_gaps(
    traces: &[EvalTrace],
    retrieval_gap_summaries: &[ctxpack_compiler::RetrievalGapSummary],
) -> String {
    let mut output = String::from("# ctxpack Dogfood Checklist\n\n");
    output.push_str(
        "Use this checklist after an agent session to compare ctxpack recommendations with what the agent actually read, edited, and validated.\n\n",
    );

    output.push_str("## Grouped Retrieval Failures\n\n");
    push_retrieval_gap_summaries(&mut output, retrieval_gap_summaries);
    output.push('\n');

    if traces.is_empty() {
        output.push_str("No eval traces found for this repository.\n");
        return output;
    }

    for (index, trace) in traces.iter().enumerate() {
        output.push_str(&format!(
            "## Trace {}\n\n- Trace ID: `{}`\n- Task hash: `{}`\n- Task type: `{:?}`\n- Target agent: `{}`\n",
            index + 1,
            trace.id,
            trace.task_hash,
            trace.task_type,
            trace.target_agent
        ));
        if let Some(pack_id) = trace.pack_id {
            output.push_str(&format!("- Pack ID: `{pack_id}`\n"));
        }
        if let Some(budget) = &trace.budget {
            output.push_str(&format!("- Budget: `{:?}`\n", budget));
        }
        output.push_str(&format!(
            "- Created at: `{}`\n- Source text logged: `{}`\n\n",
            trace.created_at_unix_seconds, trace.source_text_logged
        ));

        output.push_str("### Recommended Files\n\n");
        push_checkbox_list(
            &mut output,
            &trace.recommended_files,
            "No files recommended.",
        );
        output.push_str("\n### Related Tests\n\n");
        push_checkbox_list(
            &mut output,
            &trace.recommended_tests,
            "No tests recommended.",
        );
        output.push_str("\n### Recommended Commands\n\n");
        push_checkbox_list(
            &mut output,
            &trace.recommended_commands,
            "No commands recommended.",
        );
        output.push_str("\n### Session Comparison\n\n");
        output.push_str("- [ ] Agent read every recommended target file before editing.\n");
        output.push_str(
            "- [ ] Agent edited only files justified by the task or follow-up evidence.\n",
        );
        output
            .push_str("- [ ] Agent ran the recommended command or documented why it could not.\n");
        output.push_str("- [ ] Missing file read by agent: `________________`\n");
        output.push_str("- [ ] Extra edited file needing explanation: `________________`\n");
        output.push_str("- [ ] Outcome recorded: pass / fail / blocked\n");
        output.push_str("- Notes: \n\n");
    }

    output
}

fn push_checkbox_list(output: &mut String, items: &[String], empty_message: &str) {
    if items.is_empty() {
        output.push_str(&format!("- {empty_message}\n"));
        return;
    }
    for item in items {
        output.push_str(&format!("- [ ] `{item}`\n"));
    }
}

fn render_historical_eval_report(report: &HistoricalEvalReport) -> String {
    let mut output = String::from("# ctxpack Historical Retrieval Eval\n\n");
    output.push_str("This source-free report replays recent commit subjects through `prepare_task` and compares recommended context paths with the safe files changed by each commit.\n\n");
    output.push_str(&format!(
        "- Eval range ID: `{}`\n- Repo ID: `{}`\n- Evaluated commits: `{}`\n- Budget: `{:?}`\n- Effective limit: `{}`\n- Ranking budget K: `{}`\n- Effective mode: `{:?}`\n- Effective target agent: `{}`\n- Semantic enabled: `{}`\n- Base: `{}`\n- Head: `{}`\n- File Recall@5: `{:.2}`\n- File Recall@10: `{:.2}`\n- Lexical Baseline Recall@5: `{:.2}`\n- Lexical Baseline Recall@10: `{:.2}`\n- ctxpack Lift@5: `{:+.2}`\n- ctxpack Lift@10: `{:+.2}`\n- Recall@K: `{:.2}`\n- Precision@K: `{:.2}`\n- MRR@K: `{:.2}`\n- Lexical Recall@K: `{:.2}`\n- No-context Recall@K: `{:.2}`\n- ctxpack Lift@K: `{:+.2}`\n- ctxpack Lift vs No-context@K: `{:+.2}`\n- Source Recall@5: `{:.2}`\n- Source Recall@10: `{:.2}`\n- Test Recall@5: `{:.2}`\n- Test Recall@10: `{:.2}`\n- Test recommendation rate: `{:.2}`\n- Average recommended context files: `{:.2}`\n- Low-information commits: `{}`\n- Privacy: local-only `{}`\n\n",
        report.eval_range_id,
        report.repo_id,
        report.evaluated_commits,
        report.budget,
        report.effective_filters.limit,
        report.effective_filters.ranking_budget,
        report.effective_filters.mode,
        report.effective_filters.target_agent,
        report.effective_filters.semantic_enabled,
        report.base.as_deref().unwrap_or("HEAD history"),
        report.head.as_deref().unwrap_or("HEAD"),
        report.file_recall_at_5,
        report.file_recall_at_10,
        report.lexical_baseline_recall_at_5,
        report.lexical_baseline_recall_at_10,
        report.ctxpack_lift_at_5,
        report.ctxpack_lift_at_10,
        report.ranking_comparison.combined.recall_at_k,
        report.ranking_comparison.combined.precision_at_k,
        report.ranking_comparison.combined.mrr_at_k,
        report.ranking_comparison.lexical_baseline.recall_at_k,
        report.ranking_comparison.no_context_baseline.recall_at_k,
        report.ranking_comparison.recall_lift_at_k,
        report.ranking_comparison.recall_lift_vs_no_context_at_k,
        report.source_recall_at_5,
        report.source_recall_at_10,
        report.test_recall_at_5,
        report.test_recall_at_10,
        report.test_recommendation_rate,
        report.average_recommended_context_files,
        report.low_information_commit_count,
        report.privacy_status.local_only
    ));

    if report.commits.is_empty() {
        output.push_str("No safe historical commits were available for evaluation.\n");
        return output;
    }

    output.push_str("## Top Retrieval Gaps\n\n");
    if report.top_missing_files.is_empty() {
        output.push_str("- No missing files at 10 across evaluated commits.\n\n");
    } else {
        for gap in &report.top_missing_files {
            output.push_str(&format!(
                "- `{}` ({:?}) missed `{}` time(s)\n",
                gap.path, gap.role, gap.missed_count
            ));
        }
        output.push('\n');
    }

    output.push_str("## Token ROI\n\n");
    if report.token_roi.is_empty() {
        output.push_str("- No token ROI rows available.\n\n");
    } else {
        for row in &report.token_roi {
            output.push_str(&format!(
                "- `{:?}`: cutoff `{}`, estimated tokens `{}`, useful targets `{}/{}`, useful targets per 1k tokens `{:.2}`, recall `{:.2}`, marginal useful targets `{:+}`, larger pack adds little value `{}`\n",
                row.budget,
                row.ranking_cutoff,
                row.estimated_tokens,
                row.useful_targets,
                row.safe_targets,
                row.useful_targets_per_1k_tokens,
                row.recall_at_cutoff,
                row.marginal_useful_targets_vs_previous_budget,
                row.larger_pack_adds_little_value
            ));
        }
        output.push('\n');
    }

    output.push_str("## Signal Ablations\n\n");
    for ablation in &report.signal_ablations {
        output.push_str(&format!(
            "- Disabled `{:?}` over range `{}` / `{}` commit(s): Recall@K `{:.2}`, Precision@K `{:.2}`, MRR@K `{:.2}`, lift vs lexical `{:+.2}`\n",
            ablation.disabled_signal,
            ablation.eval_range_id,
            ablation.evaluated_commits,
            ablation.metrics.recall_at_k,
            ablation.metrics.precision_at_k,
            ablation.metrics.mrr_at_k,
            ablation.recall_lift_vs_lexical_at_k
        ));
    }
    if report.signal_ablations.is_empty() {
        output.push_str("- No signal ablations available.\n");
    }
    output.push('\n');

    output.push_str("## Grouped Retrieval Failures\n\n");
    push_retrieval_gap_summaries(&mut output, &report.retrieval_gap_summaries);
    output.push('\n');

    output.push_str("## Commits\n\n");
    for commit in &report.commits {
        let short_sha = commit.sha.chars().take(12).collect::<String>();
        output.push_str(&format!(
            "### `{short_sha}`\n\n- Task hash: `{}`\n- Task type: `{:?}`\n- Target agent: `{}`\n- Confidence: `{:.2}`\n- Source text logged: `{}`\n- Low-information task: `{}`\n- Safe changed files: `{}`\n- Excluded changed files: `{}`\n- Hits@5: `{}`\n- Hits@10: `{}`\n- Lexical baseline hits@5/10: `{}/{}`\n- Source hits@5/10: `{}/{}` of `{}`\n- Test hits@5/10: `{}/{}` of `{}`\n",
            commit.task_hash,
            commit.task_type,
            commit.target_agent,
            commit.confidence,
            commit.source_text_logged,
            commit.low_information_task,
            commit.safe_changed_files.len(),
            commit.excluded_changed_file_count,
            commit.file_hits_at_5.len(),
            commit.file_hits_at_10.len(),
            commit.lexical_baseline_hits_at_5.len(),
            commit.lexical_baseline_hits_at_10.len(),
            commit.source_hits_at_5,
            commit.source_hits_at_10,
            commit.source_files_changed,
            commit.test_hits_at_5,
            commit.test_hits_at_10,
            commit.test_files_changed
        ));
        output.push_str("\nChanged files:\n");
        push_plain_path_list(
            &mut output,
            &commit.safe_changed_files,
            "No safe changed files.",
        );
        output.push_str("\nRecommended context files:\n");
        push_plain_path_list(
            &mut output,
            &commit.recommended_context_files,
            "No context files recommended.",
        );
        if !commit.missing_files_at_10.is_empty() {
            output.push_str("\nMissing changed files at 10:\n");
            push_plain_path_list(&mut output, &commit.missing_files_at_10, "None.");
        }
        output.push('\n');
    }

    output
}

fn push_retrieval_gap_summaries(
    output: &mut String,
    retrieval_gap_summaries: &[ctxpack_compiler::RetrievalGapSummary],
) {
    if retrieval_gap_summaries.is_empty() {
        output.push_str("- No grouped retrieval failures at 10.\n");
        return;
    }
    for gap in retrieval_gap_summaries {
        output.push_str(&format!(
            "- Role `{:?}`, signal gap `{}`, package `{}`, family `{}`, status `{:?}`, area `{:?}`: `{}` miss(es)",
            gap.role,
            gap.signal_gap,
            gap.package,
            gap.path_family,
            gap.target_status,
            gap.recommendation_area,
            gap.missed_count
        ));
        if !gap.example_paths.is_empty() {
            output.push_str(" examples ");
            for (index, path) in gap.example_paths.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("`{path}`"));
            }
        }
        output.push('\n');
    }
}

fn render_benchmark_suite_report(report: &BenchmarkSuiteReport) -> String {
    let mut output = String::from("# ctxpack Benchmark Suite\n\n");
    output.push_str(
        "This source-free report runs named real-repo historical eval suites and compares ctxpack retrieval against configured baselines without storing task text or source snippets.\n\n",
    );
    output.push_str(&format!(
        "- Suite: `{}`\n- Suite ID: `{}`\n- Repositories: `{}`\n- Evaluated repositories: `{}`\n- Evaluated commits: `{}`\n- Generated at: `{}`\n- Privacy: local-only `{}`\n\n",
        report.suite_name,
        report.suite_id,
        report.repository_count,
        report.evaluated_repository_count,
        report.evaluated_commit_count,
        report.generated_at_unix_seconds,
        report.privacy_status.local_only
    ));
    if let Some(description) = &report.description {
        output.push_str(&format!("## Description\n\n{description}\n\n"));
    }

    output.push_str("## Repository Results\n\n");
    if report.repositories.is_empty() {
        output.push_str("- No repositories configured.\n");
        return output;
    }

    for repo in &report.repositories {
        output.push_str(&format!("### `{}`\n\n", repo.name));
        output.push_str(&format!(
            "- Repo ID: `{}`\n- Evaluated commits: `{}`\n- Excluded changed files: `{}`\n- Skipped path labels: `{}`\n- Limit: `{}`\n- Ranking budget K: `{}`\n- Mode: `{:?}`\n- Target agent: `{}`\n- Base: `{}`\n- Head: `{}`\n- Role filters: `{}`\n- Privacy: local-only `{}`\n",
            repo.repo_id.as_deref().unwrap_or("unavailable"),
            repo.evaluated_commits,
            repo.excluded_changed_file_count,
            repo.skipped_path_count,
            repo.effective_config.limit,
            repo.effective_config.ranking_budget,
            repo.effective_config.mode,
            repo.effective_config.target_agent,
            repo.effective_config.base.as_deref().unwrap_or("HEAD history"),
            repo.effective_config.head.as_deref().unwrap_or("HEAD"),
            role_filter_label(&repo.effective_config.role_filters),
            repo.privacy_status.local_only
        ));

        if let Some(error) = &repo.error {
            output.push_str(&format!("- Error: `{error}`\n\n"));
            continue;
        }

        let Some(eval) = &repo.report else {
            output.push_str("- No historical eval report available.\n\n");
            continue;
        };
        output.push_str(&format!(
            "- File Recall@5: `{:.2}`\n- File Recall@10: `{:.2}`\n- Lexical Baseline Recall@5: `{:.2}`\n- Lexical Baseline Recall@10: `{:.2}`\n- No-context Recall@K: `{:.2}`\n- ctxpack Lift@5: `{:+.2}`\n- ctxpack Lift@10: `{:+.2}`\n- ctxpack Lift vs No-context@K: `{:+.2}`\n- Source Recall@10: `{:.2}`\n- Test Recall@10: `{:.2}`\n- Test recommendation rate: `{:.2}`\n- Average recommended context files: `{:.2}`\n\n",
            eval.file_recall_at_5,
            eval.file_recall_at_10,
            eval.lexical_baseline_recall_at_5,
            eval.lexical_baseline_recall_at_10,
            eval.ranking_comparison.no_context_baseline.recall_at_k,
            eval.ctxpack_lift_at_5,
            eval.ctxpack_lift_at_10,
            eval.ranking_comparison.recall_lift_vs_no_context_at_k,
            eval.source_recall_at_10,
            eval.test_recall_at_10,
            eval.test_recommendation_rate,
            eval.average_recommended_context_files
        ));

        output.push_str("#### Token ROI\n\n");
        if eval.token_roi.is_empty() {
            output.push_str("- No token ROI rows available.\n\n");
        } else {
            for row in &eval.token_roi {
                output.push_str(&format!(
                    "- `{:?}`: cutoff `{}`, estimated tokens `{}`, useful targets `{}/{}`, useful targets per 1k tokens `{:.2}`, larger pack adds little value `{}`\n",
                    row.budget,
                    row.ranking_cutoff,
                    row.estimated_tokens,
                    row.useful_targets,
                    row.safe_targets,
                    row.useful_targets_per_1k_tokens,
                    row.larger_pack_adds_little_value
                ));
            }
            output.push('\n');
        }

        output.push_str("#### Grouped Retrieval Failures\n\n");
        push_retrieval_gap_summaries(&mut output, &eval.retrieval_gap_summaries);
        output.push('\n');
    }

    output
}

fn render_benchmark_comparison_report(report: &BenchmarkComparisonReport) -> String {
    let mut output = String::from("# ctxpack Benchmark Comparison\n\n");
    output.push_str(
        "This source-free report compares two benchmark JSON reports and flags configured metric regressions.\n\n",
    );
    output.push_str(&format!(
        "- Base suite ID: `{}`\n- Head suite ID: `{}`\n- Repositories compared: `{}`\n- Passed thresholds: `{}`\n- Privacy: local-only `{}`\n\n",
        report.base_suite_id,
        report.head_suite_id,
        report.repository_count,
        report.passed,
        report.privacy_status.local_only
    ));

    output.push_str("## Metric Deltas\n\n");
    if report.metric_deltas.is_empty() {
        output.push_str("- No matching repository metrics to compare.\n\n");
    } else {
        for delta in &report.metric_deltas {
            output.push_str(&format!(
                "- `{}` `{}`: `{:.3}` -> `{:.3}` ({:+.3})\n",
                delta.repository, delta.metric, delta.base_value, delta.head_value, delta.delta
            ));
        }
        output.push('\n');
    }

    output.push_str("## Threshold Checks\n\n");
    if report.threshold_checks.is_empty() {
        output.push_str("- No thresholds configured.\n\n");
    } else {
        for check in &report.threshold_checks {
            output.push_str(&format!(
                "- `{}` `{}`: delta `{:+.3}`, max drop `{:.3}`, passed `{}`\n",
                check.repository, check.metric, check.delta, check.max_drop, check.passed
            ));
        }
        output.push('\n');
    }

    output.push_str("## Gap Family Deltas\n\n");
    if report.gap_family_deltas.is_empty() {
        output.push_str("- No grouped retrieval gap deltas.\n");
    } else {
        for gap in &report.gap_family_deltas {
            output.push_str(&format!(
                "- `{}` role `{:?}`, signal `{}`, package `{}`, family `{}`, status `{:?}`, area `{:?}`: `{}` -> `{}` ({:+})\n",
                gap.repository,
                gap.role,
                gap.signal_gap,
                gap.package,
                gap.path_family,
                gap.target_status,
                gap.recommendation_area,
                gap.base_missed_count,
                gap.head_missed_count,
                gap.delta
            ));
        }
    }
    output
}

fn render_product_proof_report(report: &ProductProofReport) -> String {
    let mut output = String::from("# ctxpack Product Proof\n\n");
    output.push_str(
        "This source-free report summarizes whether ctxpack improves repository context selection over fixed-budget baselines for a configured benchmark suite.\n\n",
    );
    output.push_str(&format!(
        "- Suite: `{}`\n- Suite ID: `{}`\n- Evaluated repositories: `{}`\n- Evaluated commits: `{}`\n- Privacy: local-only `{}`\n\n",
        report.suite_name,
        report.suite_id,
        report.evaluated_repository_count,
        report.evaluated_commit_count,
        report.privacy_status.local_only
    ));

    output.push_str("## Headline Metrics\n\n");
    for metric in &report.headline_metrics {
        output.push_str(&format!(
            "- `{}`: `{:.3}` `{}`\n",
            metric.label, metric.value, metric.unit
        ));
    }
    output.push('\n');

    output.push_str("## When It Helps\n\n");
    for item in &report.helps_when {
        output.push_str(&format!("- {item}\n"));
    }
    output.push('\n');

    output.push_str("## When It Does Not Help\n\n");
    for item in &report.does_not_help_when {
        output.push_str(&format!("- {item}\n"));
    }
    output.push('\n');

    output.push_str("## Limitations\n\n");
    for item in &report.limitations {
        output.push_str(&format!("- {item}\n"));
    }
    output.push('\n');

    output.push_str("## Future Work From Gaps\n\n");
    for item in &report.future_work {
        output.push_str(&format!("- {item}\n"));
    }
    output.push('\n');

    output.push_str("## Reproduce\n\n");
    output.push_str(
        "Run `ctxpack eval proof --config <suite.json>` or inspect the embedded source-free benchmark report in JSON output.\n",
    );
    output
}

fn role_filter_label(filters: &[ctxpack_core::FileRole]) -> String {
    if filters.is_empty() {
        return "all safe roles".to_string();
    }
    filters
        .iter()
        .map(|role| format!("{role:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_cards_report(report: &ContextCardsReport) -> String {
    let mut output = String::from("# ctxpack Context Cards\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Cards directory: `{}`\n- Cards generated: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.cards_dir.display(),
        report.cards.len(),
        report.privacy_status.local_only
    ));
    for card in &report.cards {
        output.push_str(&format!(
            "- `{}`: `{}` ({} bytes, memory `{}`)\n",
            card.name,
            card.path.display(),
            card.bytes,
            card.memory_card_id
        ));
    }
    print_diagnostics_to_string(&mut output, &report.diagnostics);
    output
}

fn render_memory_cards(cards: &[ctxpack_core::MemoryCard]) -> String {
    let mut output = String::from("# ctxpack Memory Cards\n\n");
    output.push_str(&format!("- Cards: `{}`\n\n", cards.len()));
    if cards.is_empty() {
        output.push_str("- No source-free memory cards are stored for this repo.\n");
        return output;
    }
    for card in cards {
        output.push_str(&format!(
            "- `{}` {:?} {:?} {:?} disabled `{}` confidence `{:.2}`\n",
            card.id, card.kind, card.freshness, card.review_status, card.disabled, card.confidence
        ));
        output.push_str(&format!("  - {}\n", card.summary));
        if !card.source_links.is_empty() {
            output.push_str(&format!(
                "  - Links: {}\n",
                card.source_links
                    .iter()
                    .take(6)
                    .map(|link| format!("`{link}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    output
}

fn render_experience_cards_report(report: &ExperienceCardsReport) -> String {
    let mut output = String::from("# ctxpack Experience Cards\n\n");
    output.push_str(&format!(
        "- Repo ID: `{}`\n- Cards generated: `{}`\n- Stored records: `{}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.cards.len(),
        report.stored_records,
        report.privacy_status.local_only
    ));
    output.push_str(&render_memory_cards(&report.cards));
    print_diagnostics_to_string(&mut output, &report.diagnostics);
    output
}

fn print_diagnostics_to_string(output: &mut String, diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    output.push_str("\n## Diagnostics\n");
    for diagnostic in diagnostics {
        output.push_str(&format!(
            "- `{:?}` `{}`: {}\n",
            diagnostic.severity, diagnostic.code, diagnostic.message
        ));
    }
}

fn push_plain_path_list(output: &mut String, items: &[String], empty_message: &str) {
    if items.is_empty() {
        output.push_str(&format!("- {empty_message}\n"));
        return;
    }
    for item in items {
        output.push_str(&format!("- `{item}`\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctxpack_core::PackBudget;
    use ctxpack_core::PrivacyStatus;
    use uuid::Uuid;

    #[test]
    fn eval_checklist_renders_manual_comparison_fields() {
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

        let checklist = render_eval_checklist_with_gaps(&[trace], &[]);

        assert!(checklist.contains("# ctxpack Dogfood Checklist"));
        assert!(checklist.contains("- [ ] `src/auth.ts`"));
        assert!(checklist.contains("- [ ] `tests/auth.test.ts`"));
        assert!(checklist.contains("- [ ] `pnpm test tests/auth.test.ts`"));
        assert!(checklist.contains("Agent read every recommended target file"));
        assert!(checklist.contains("Missing file read by agent"));
        assert!(checklist.contains("Source text logged: `false`"));
        assert!(!checklist.contains("source code"));
    }

    #[test]
    fn eval_checklist_includes_source_free_retrieval_gap_summaries() {
        let checklist = render_eval_checklist_with_gaps(
            &[],
            &[ctxpack_compiler::RetrievalGapSummary {
                role: ctxpack_core::FileRole::Test,
                signal_gap: "no_candidate_signal".to_string(),
                package: "tests".to_string(),
                path_family: "tests/auth/*.ts".to_string(),
                target_status: ctxpack_compiler::RetrievalGapTargetStatus::CurrentReachable,
                recommendation_area: ctxpack_compiler::RetrievalGapRecommendationArea::TestMapping,
                missed_count: 2,
                example_paths: vec!["tests/auth/session.test.ts".to_string()],
            }],
        );

        assert!(checklist.contains("Grouped Retrieval Failures"));
        assert!(checklist.contains("Role `Test`, signal gap `no_candidate_signal`"));
        assert!(checklist.contains("area `TestMapping`"));
        assert!(checklist.contains("family `tests/auth/*.ts`"));
        assert!(checklist.contains("`tests/auth/session.test.ts`"));
        assert!(!checklist.contains("commit subject"));
        assert!(!checklist.contains("source code"));
    }

    #[test]
    fn historical_eval_report_renders_source_free_metrics() {
        let report = HistoricalEvalReport {
            eval_range_id: "range-1".to_string(),
            repo_id: "repo-1".to_string(),
            evaluated_commits: 1,
            budget: PackBudget::Standard,
            effective_filters: ctxpack_compiler::HistoricalEvalEffectiveFilters {
                limit: 5,
                ranking_budget: 10,
                mode: TaskType::BugFix,
                target_agent: "codex".to_string(),
                budget: PackBudget::Standard,
                semantic_enabled: false,
            },
            refs: ctxpack_compiler::HistoricalEvalRefs {
                base: Some("abc000".to_string()),
                head: Some("def111".to_string()),
            },
            base: Some("abc000".to_string()),
            head: Some("def111".to_string()),
            ranking_comparison: ctxpack_compiler::EvalComparison {
                k: 10,
                combined: ctxpack_compiler::RankingMetrics {
                    k: 10,
                    recall_at_k: 1.0,
                    precision_at_k: 0.1,
                    mrr_at_k: 1.0,
                    role_recall: vec![ctxpack_compiler::RoleRecallMetric {
                        role: ctxpack_core::FileRole::Source,
                        recall_at_k: 1.0,
                        changed_file_count: 1,
                        hit_count: 1,
                    }],
                    test_recommendation_rate: 1.0,
                    average_recommended_context_files: 2.0,
                },
                lexical_baseline: ctxpack_compiler::RankingMetrics {
                    k: 10,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: vec![ctxpack_compiler::RoleRecallMetric {
                        role: ctxpack_core::FileRole::Source,
                        recall_at_k: 0.0,
                        changed_file_count: 1,
                        hit_count: 0,
                    }],
                    test_recommendation_rate: 1.0,
                    average_recommended_context_files: 1.0,
                },
                no_context_baseline: ctxpack_compiler::RankingMetrics {
                    k: 10,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: vec![ctxpack_compiler::RoleRecallMetric {
                        role: ctxpack_core::FileRole::Source,
                        recall_at_k: 0.0,
                        changed_file_count: 1,
                        hit_count: 0,
                    }],
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 0.0,
                },
                recall_lift_at_k: 1.0,
                precision_lift_at_k: 0.1,
                mrr_lift_at_k: 1.0,
                recall_lift_vs_no_context_at_k: 1.0,
                precision_lift_vs_no_context_at_k: 0.1,
                mrr_lift_vs_no_context_at_k: 1.0,
            },
            signal_ablations: Vec::new(),
            token_roi: vec![
                ctxpack_compiler::TokenRoiMetric {
                    budget: PackBudget::Brief,
                    ranking_cutoff: 5,
                    estimated_tokens: 4_000,
                    useful_targets: 1,
                    safe_targets: 1,
                    useful_targets_per_1k_tokens: 0.25,
                    recall_at_cutoff: 1.0,
                    marginal_useful_targets_vs_previous_budget: 1,
                    larger_pack_adds_little_value: false,
                },
                ctxpack_compiler::TokenRoiMetric {
                    budget: PackBudget::Standard,
                    ranking_cutoff: 10,
                    estimated_tokens: 24_000,
                    useful_targets: 1,
                    safe_targets: 1,
                    useful_targets_per_1k_tokens: 0.041666668,
                    recall_at_cutoff: 1.0,
                    marginal_useful_targets_vs_previous_budget: 0,
                    larger_pack_adds_little_value: true,
                },
                ctxpack_compiler::TokenRoiMetric {
                    budget: PackBudget::Deep,
                    ranking_cutoff: 10,
                    estimated_tokens: 100_000,
                    useful_targets: 1,
                    safe_targets: 1,
                    useful_targets_per_1k_tokens: 0.01,
                    recall_at_cutoff: 1.0,
                    marginal_useful_targets_vs_previous_budget: 0,
                    larger_pack_adds_little_value: true,
                },
            ],
            retrieval_gap_summaries: Vec::new(),
            low_information_commit_count: 1,
            file_recall_at_5: 1.0,
            file_recall_at_10: 1.0,
            lexical_baseline_recall_at_5: 0.5,
            lexical_baseline_recall_at_10: 0.5,
            ctxpack_lift_at_5: 0.5,
            ctxpack_lift_at_10: 0.5,
            source_recall_at_5: 1.0,
            source_recall_at_10: 1.0,
            test_recall_at_5: 0.0,
            test_recall_at_10: 0.0,
            test_recommendation_rate: 1.0,
            average_recommended_context_files: 2.0,
            top_missing_files: vec![ctxpack_compiler::HistoricalMissingFileSummary {
                path: "README.md".to_string(),
                role: ctxpack_core::FileRole::Docs,
                missed_count: 1,
            }],
            commits: vec![ctxpack_compiler::HistoricalCommitEval {
                sha: "abcdef1234567890".to_string(),
                task_hash: "hash-1".to_string(),
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                changed_path_labels: vec![ctxpack_compiler::HistoricalChangedPathLabel {
                    path: "src/auth.ts".to_string(),
                    old_path: None,
                    change_kind: ctxpack_index::ChangeKind::Modified,
                    role: ctxpack_core::FileRole::Source,
                    label_scope: ctxpack_index::LabelScope::Safe,
                    excluded_reason: None,
                }],
                safe_changed_files: vec!["src/auth.ts".to_string()],
                excluded_changed_file_count: 1,
                recommended_files: vec!["src/auth.ts".to_string()],
                recommended_tests: vec!["tests/auth.test.ts".to_string()],
                recommended_context_files: vec![
                    "src/auth.ts".to_string(),
                    "tests/auth.test.ts".to_string(),
                ],
                recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
                lexical_baseline_files: vec!["README.md".to_string()],
                file_hits_at_5: vec!["src/auth.ts".to_string()],
                file_hits_at_10: vec!["src/auth.ts".to_string()],
                lexical_baseline_hits_at_5: vec![],
                lexical_baseline_hits_at_10: vec![],
                missing_files_at_10: vec![],
                source_files_changed: 1,
                source_hits_at_5: 1,
                source_hits_at_10: 1,
                test_files_changed: 0,
                test_hits_at_5: 0,
                test_hits_at_10: 0,
                low_information_task: true,
                confidence: 0.85,
                source_text_logged: false,
            }],
            privacy_status: PrivacyStatus::local_only(),
        };

        let markdown = render_historical_eval_report(&report);

        assert!(markdown.contains("# ctxpack Historical Retrieval Eval"));
        assert!(markdown.contains("Eval range ID: `range-1`"));
        assert!(markdown.contains("Budget: `Standard`"));
        assert!(markdown.contains("Effective limit: `5`"));
        assert!(markdown.contains("Effective target agent: `codex`"));
        assert!(markdown.contains("Base: `abc000`"));
        assert!(markdown.contains("Head: `def111`"));
        assert!(markdown.contains("Low-information commits: `1`"));
        assert!(markdown.contains("File Recall@5: `1.00`"));
        assert!(markdown.contains("Lexical Baseline Recall@5: `0.50`"));
        assert!(markdown.contains("No-context Recall@K: `0.00`"));
        assert!(markdown.contains("ctxpack Lift@10: `+0.50`"));
        assert!(markdown.contains("ctxpack Lift vs No-context@K: `+1.00`"));
        assert!(markdown.contains("Source Recall@10: `1.00`"));
        assert!(markdown.contains("Test Recall@10: `0.00`"));
        assert!(markdown.contains("Token ROI"));
        assert!(markdown.contains("`Brief`: cutoff `5`, estimated tokens `4000`"));
        assert!(markdown.contains("larger pack adds little value `true`"));
        assert!(markdown.contains("Top Retrieval Gaps"));
        assert!(markdown.contains("`README.md` (Docs) missed `1` time"));
        assert!(markdown.contains("Lexical baseline hits@5/10: `0/0`"));
        assert!(markdown.contains("Low-information task: `true`"));
        assert!(markdown.contains("Source hits@5/10: `1/1` of `1`"));
        assert!(markdown.contains("`abcdef123456`"));
        assert!(markdown.contains("`src/auth.ts`"));
        assert!(markdown.contains("Source text logged: `false`"));
        assert!(!markdown.contains("source code"));
    }

    #[test]
    fn historical_eval_report_history_budget_arg_parses() {
        let cli = Cli::try_parse_from([
            "ctxpack", "eval", "history", "--limit", "3", "--budget", "4", "--format", "json",
        ])
        .unwrap();

        let Command::Eval(EvalArgs {
            command: EvalCommand::History(args),
        }) = cli.command
        else {
            panic!("expected eval history command");
        };
        assert_eq!(args.limit, 3);
        assert_eq!(args.budget, 4);

        let default_cli = Cli::try_parse_from(["ctxpack", "eval", "history"]).unwrap();
        let Command::Eval(EvalArgs {
            command: EvalCommand::History(default_args),
        }) = default_cli.command
        else {
            panic!("expected eval history command");
        };
        assert_eq!(default_args.budget, 10);
    }

    #[test]
    fn inspector_export_command_parses_static_formats() {
        let cli = Cli::try_parse_from([
            "ctxpack",
            "inspector",
            "export",
            "fix auth redirect",
            "--mode",
            "bug-fix",
            "--budget",
            "standard",
            "--target-agent",
            "codex",
            "--format",
            "html",
            "--output",
            "pack.html",
        ])
        .unwrap();

        let Command::Inspector(InspectorArgs {
            command: InspectorCommand::Export(args),
        }) = cli.command
        else {
            panic!("expected inspector export command");
        };
        assert_eq!(args.task, "fix auth redirect");
        assert!(matches!(args.mode, Mode::BugFix));
        assert!(matches!(args.budget, Budget::Standard));
        assert!(matches!(args.format, InspectorFormat::Html));
        assert_eq!(args.target_agent, "codex");
        assert_eq!(args.output, Some(PathBuf::from("pack.html")));
    }

    #[test]
    fn agent_preview_command_parses_agent_and_format() {
        let cli = Cli::try_parse_from([
            "ctxpack",
            "agent",
            "preview",
            "fix auth redirect",
            "--mode",
            "bug-fix",
            "--budget",
            "standard",
            "--target-agent",
            "claude-code",
            "--format",
            "json",
        ])
        .unwrap();

        let Command::Agent(AgentArgs {
            command: AgentCommand::Preview(args),
        }) = cli.command
        else {
            panic!("expected agent preview command");
        };
        assert_eq!(args.task, "fix auth redirect");
        assert!(matches!(args.mode, Mode::BugFix));
        assert!(matches!(args.budget, Budget::Standard));
        assert_eq!(args.target_agent, "claude-code");
        assert!(matches!(args.format, PackFormat::Json));
    }

    #[test]
    fn cards_report_renders_generated_paths() {
        let report = ContextCardsReport {
            repo_id: "repo-1".to_string(),
            cards_dir: PathBuf::from("/tmp/repo/.ctxpack/cards"),
            cards: vec![ctxpack_compiler::GeneratedContextCard {
                name: "repo-overview".to_string(),
                path: PathBuf::from("/tmp/repo/.ctxpack/cards/repo-overview.md"),
                title: "Repo Overview".to_string(),
                bytes: 123,
                memory_card_id: "domain:repo-overview".to_string(),
            }],
            diagnostics: Vec::new(),
            privacy_status: PrivacyStatus::local_only(),
        };

        let markdown = render_cards_report(&report);

        assert!(markdown.contains("# ctxpack Context Cards"));
        assert!(markdown.contains("Cards generated: `1`"));
        assert!(markdown.contains("repo-overview.md"));
        assert!(markdown.contains("Privacy: local-only `true`"));
    }
}
