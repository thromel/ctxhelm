use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctxpack_compiler::{
    build_product_proof_report, compare_benchmark_suite_reports,
    compile_context_pack_with_plan_and_paths_for_agent, eval_trace_for_pack, eval_trace_for_plan,
    evaluate_historical_commits, generate_context_cards, load_benchmark_suite_report,
    prepare_context_plan_with_paths, render_pack_markdown, run_benchmark_suite,
    BenchmarkComparisonReport, BenchmarkRegressionThreshold, BenchmarkSuiteReport,
    ContextCardsOptions, ContextCardsReport, HistoricalEvalOptions, HistoricalEvalReport,
    ProductProofReport,
};
use ctxpack_core::{
    run_init, run_setup_check, AgentAdapter, Diagnostic, DiagnosticSeverity, EvalTrace, InitAction,
    InitOptions, InitReport, PackBudget, RepoRoot, SetupCheckReport, SetupCheckStatus, TaskType,
};
use ctxpack_index::{
    co_change_hints, current_diff_summary, dependency_edges, extract_symbols, lexical_search,
    list_eval_traces, related_dependency_edges, related_tests, symbol_search,
    try_append_eval_trace, write_inventory, CoChangeOptions, CurrentDiffOptions, DependencyOptions,
    InventoryOptions, InventoryReport, SearchOptions, SymbolOptions,
};
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

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
    Index(IndexArgs),
    PrepareTask(PrepareTaskArgs),
    GetPack(GetPackArgs),
    Search {
        query: String,
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Symbols(SymbolsArgs),
    RelatedTests(RelatedTestsArgs),
    CoChanges(CoChangesArgs),
    Dependencies(DependenciesArgs),
    Cards(CardsArgs),
    Eval(EvalArgs),
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
struct IndexArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    include_generated: bool,
    #[arg(long)]
    include_sensitive: bool,
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
        help = "Disable local eval trace recording for this read command."
    )]
    no_trace: bool,
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
    History(EvalHistoryArgs),
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
        }
        Command::PrepareTask(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let paths = context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
            let mut plan =
                prepare_context_plan_with_paths(&repo.path, &args.task, args.mode.into(), &paths)?;
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
            let (plan, mut pack) = compile_context_pack_with_plan_and_paths_for_agent(
                &repo.path,
                &args.task,
                args.mode.into(),
                args.budget.into(),
                &paths,
                &args.target_agent,
            )?;
            if args.no_trace {
                pack.diagnostics.push(trace_disabled_diagnostic());
            } else {
                let trace =
                    eval_trace_for_pack(&repo.path, &args.task, &args.target_agent, &plan, &pack);
                pack.diagnostics
                    .extend(try_append_eval_trace(&repo.path, &trace).diagnostics);
            }
            match args.format {
                PackFormat::Markdown => println!("{}", render_pack_markdown(&pack)),
                PackFormat::Json => println!("{}", serde_json::to_string_pretty(&pack)?),
            }
        }
        Command::Search { query, repo, limit } => {
            let start = repo.unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let results = lexical_search(&repo.path, &query, &SearchOptions { limit })?;
            println!("{}", serde_json::to_string_pretty(&results)?);
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
                    },
                )
                .map(|report| report.retrieval_gap_summaries)
                .unwrap_or_default();
                println!(
                    "{}",
                    render_eval_checklist_with_gaps(&traces, &retrieval_gap_summaries)
                );
            }
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
                    },
                )?;
                match args.format {
                    PackFormat::Markdown => println!("{}", render_historical_eval_report(&report)),
                    PackFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                }
            }
            EvalCommand::Benchmark(args) => {
                let report = run_benchmark_suite(&args.config)?;
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
                let benchmark = run_benchmark_suite(&args.config)?;
                let report = build_product_proof_report(benchmark);
                match args.format {
                    PackFormat::Markdown => println!("{}", render_product_proof_report(&report)),
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
        "- Eval range ID: `{}`\n- Repo ID: `{}`\n- Evaluated commits: `{}`\n- Budget: `{:?}`\n- Effective limit: `{}`\n- Ranking budget K: `{}`\n- Effective mode: `{:?}`\n- Effective target agent: `{}`\n- Base: `{}`\n- Head: `{}`\n- File Recall@5: `{:.2}`\n- File Recall@10: `{:.2}`\n- Lexical Baseline Recall@5: `{:.2}`\n- Lexical Baseline Recall@10: `{:.2}`\n- ctxpack Lift@5: `{:+.2}`\n- ctxpack Lift@10: `{:+.2}`\n- Recall@K: `{:.2}`\n- Precision@K: `{:.2}`\n- MRR@K: `{:.2}`\n- Lexical Recall@K: `{:.2}`\n- No-context Recall@K: `{:.2}`\n- ctxpack Lift@K: `{:+.2}`\n- ctxpack Lift vs No-context@K: `{:+.2}`\n- Source Recall@5: `{:.2}`\n- Source Recall@10: `{:.2}`\n- Test Recall@5: `{:.2}`\n- Test Recall@10: `{:.2}`\n- Test recommendation rate: `{:.2}`\n- Average recommended context files: `{:.2}`\n- Low-information commits: `{}`\n- Privacy: local-only `{}`\n\n",
        report.eval_range_id,
        report.repo_id,
        report.evaluated_commits,
        report.budget,
        report.effective_filters.limit,
        report.effective_filters.ranking_budget,
        report.effective_filters.mode,
        report.effective_filters.target_agent,
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
            "- `{}`: `{}` ({} bytes)\n",
            card.name,
            card.path.display(),
            card.bytes
        ));
    }
    output
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
    fn cards_report_renders_generated_paths() {
        let report = ContextCardsReport {
            repo_id: "repo-1".to_string(),
            cards_dir: PathBuf::from("/tmp/repo/.ctxpack/cards"),
            cards: vec![ctxpack_compiler::GeneratedContextCard {
                name: "repo-overview".to_string(),
                path: PathBuf::from("/tmp/repo/.ctxpack/cards/repo-overview.md"),
                title: "Repo Overview".to_string(),
                bytes: 123,
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
