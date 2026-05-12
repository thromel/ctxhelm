use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctxpack_compiler::{
    compile_context_pack_with_plan_and_paths_for_agent, eval_trace_for_pack, eval_trace_for_plan,
    evaluate_historical_commits, generate_context_cards, prepare_context_plan_with_paths,
    render_pack_markdown, ContextCardsOptions, ContextCardsReport, HistoricalEvalOptions,
    HistoricalEvalReport,
};
use ctxpack_core::{
    run_init, AgentAdapter, EvalTrace, InitAction, InitOptions, InitReport, PackBudget, RepoRoot,
    TaskType,
};
use ctxpack_index::{
    append_eval_trace, co_change_hints, current_diff_summary, dependency_edges, extract_symbols,
    lexical_search, list_eval_traces, related_dependency_edges, related_tests, symbol_search,
    write_inventory, CoChangeOptions, CurrentDiffOptions, DependencyOptions, InventoryOptions,
    InventoryReport, SearchOptions, SymbolOptions,
};
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "ctxpack")]
#[command(about = "Agent-native context packs for coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init(InitArgs),
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
    #[arg(long, value_enum, default_value_t = Mode::BugFix)]
    mode: Mode,
    #[arg(long, default_value = "generic")]
    target_agent: String,
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
            let plan =
                prepare_context_plan_with_paths(&repo.path, &args.task, args.mode.into(), &paths)?;
            let trace = eval_trace_for_plan(&repo.path, &args.task, &args.target_agent, &plan);
            append_eval_trace(&repo.path, &trace)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Command::GetPack(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let paths = context_anchor_paths(&repo.path, args.paths, args.include_current_diff)?;
            let (plan, pack) = compile_context_pack_with_plan_and_paths_for_agent(
                &repo.path,
                &args.task,
                args.mode.into(),
                args.budget.into(),
                &paths,
                &args.target_agent,
            )?;
            let trace =
                eval_trace_for_pack(&repo.path, &args.task, &args.target_agent, &plan, &pack);
            append_eval_trace(&repo.path, &trace)?;
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
                println!("{}", render_eval_checklist(&traces));
            }
            EvalCommand::History(args) => {
                let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
                let repo = RepoRoot::discover_from(&start)?;
                let report = evaluate_historical_commits(
                    &repo.path,
                    &HistoricalEvalOptions {
                        limit: args.limit,
                        task_type: args.mode.into(),
                        target_agent: args.target_agent,
                    },
                )?;
                match args.format {
                    PackFormat::Markdown => println!("{}", render_historical_eval_report(&report)),
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
    let mut adapters = Vec::new();
    if args.cursor {
        adapters.push(AgentAdapter::Cursor);
    }
    if args.claude {
        adapters.push(AgentAdapter::Claude);
    }
    if args.opencode {
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
        };
        println!("- {action}: {}", file.path.display());
    }
    println!();
    println!("{}", report.codex_mcp_setup);
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

fn render_eval_checklist(traces: &[EvalTrace]) -> String {
    let mut output = String::from("# ctxpack Dogfood Checklist\n\n");
    output.push_str(
        "Use this checklist after an agent session to compare ctxpack recommendations with what the agent actually read, edited, and validated.\n\n",
    );

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
        "- Repo ID: `{}`\n- Evaluated commits: `{}`\n- File Recall@5: `{:.2}`\n- File Recall@10: `{:.2}`\n- Test recommendation rate: `{:.2}`\n- Average recommended context files: `{:.2}`\n- Privacy: local-only `{}`\n\n",
        report.repo_id,
        report.evaluated_commits,
        report.file_recall_at_5,
        report.file_recall_at_10,
        report.test_recommendation_rate,
        report.average_recommended_context_files,
        report.privacy_status.local_only
    ));

    if report.commits.is_empty() {
        output.push_str("No safe historical commits were available for evaluation.\n");
        return output;
    }

    output.push_str("## Commits\n\n");
    for commit in &report.commits {
        let short_sha = commit.sha.chars().take(12).collect::<String>();
        output.push_str(&format!(
            "### `{short_sha}`\n\n- Task hash: `{}`\n- Task type: `{:?}`\n- Target agent: `{}`\n- Confidence: `{:.2}`\n- Source text logged: `{}`\n- Safe changed files: `{}`\n- Excluded changed files: `{}`\n- Hits@5: `{}`\n- Hits@10: `{}`\n",
            commit.task_hash,
            commit.task_type,
            commit.target_agent,
            commit.confidence,
            commit.source_text_logged,
            commit.safe_changed_files.len(),
            commit.excluded_changed_file_count,
            commit.file_hits_at_5.len(),
            commit.file_hits_at_10.len()
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

        let checklist = render_eval_checklist(&[trace]);

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
    fn historical_eval_report_renders_source_free_metrics() {
        let report = HistoricalEvalReport {
            repo_id: "repo-1".to_string(),
            evaluated_commits: 1,
            file_recall_at_5: 1.0,
            file_recall_at_10: 1.0,
            test_recommendation_rate: 1.0,
            average_recommended_context_files: 2.0,
            commits: vec![ctxpack_compiler::HistoricalCommitEval {
                sha: "abcdef1234567890".to_string(),
                task_hash: "hash-1".to_string(),
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                safe_changed_files: vec!["src/auth.ts".to_string()],
                excluded_changed_file_count: 1,
                recommended_files: vec!["src/auth.ts".to_string()],
                recommended_tests: vec!["tests/auth.test.ts".to_string()],
                recommended_context_files: vec![
                    "src/auth.ts".to_string(),
                    "tests/auth.test.ts".to_string(),
                ],
                recommended_commands: vec!["pnpm test tests/auth.test.ts".to_string()],
                file_hits_at_5: vec!["src/auth.ts".to_string()],
                file_hits_at_10: vec!["src/auth.ts".to_string()],
                missing_files_at_10: vec![],
                confidence: 0.85,
                source_text_logged: false,
            }],
            privacy_status: PrivacyStatus::local_only(),
        };

        let markdown = render_historical_eval_report(&report);

        assert!(markdown.contains("# ctxpack Historical Retrieval Eval"));
        assert!(markdown.contains("File Recall@5: `1.00`"));
        assert!(markdown.contains("`abcdef123456`"));
        assert!(markdown.contains("`src/auth.ts`"));
        assert!(markdown.contains("Source text logged: `false`"));
        assert!(!markdown.contains("source code"));
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
            privacy_status: PrivacyStatus::local_only(),
        };

        let markdown = render_cards_report(&report);

        assert!(markdown.contains("# ctxpack Context Cards"));
        assert!(markdown.contains("Cards generated: `1`"));
        assert!(markdown.contains("repo-overview.md"));
        assert!(markdown.contains("Privacy: local-only `true`"));
    }
}
