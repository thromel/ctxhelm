use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctxpack_compiler::{
    compile_context_pack_with_plan, eval_trace_for_pack, eval_trace_for_plan, prepare_context_plan,
    render_pack_markdown,
};
use ctxpack_core::{
    run_init, AgentAdapter, EvalTrace, InitAction, InitOptions, InitReport, PackBudget, RepoRoot,
    TaskType,
};
use ctxpack_index::{
    append_eval_trace, co_change_hints, extract_symbols, lexical_search, list_eval_traces,
    related_tests, symbol_search, write_inventory, CoChangeOptions, InventoryOptions,
    InventoryReport, SearchOptions, SymbolOptions,
};
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
struct PrepareTaskArgs {
    task: String,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Mode::Explain)]
    mode: Mode,
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
}

#[derive(Debug, Args)]
struct EvalTracesArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
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
            let plan = prepare_context_plan(&repo.path, &args.task, args.mode.into())?;
            let trace = eval_trace_for_plan(&repo.path, &args.task, &args.target_agent, &plan);
            append_eval_trace(&repo.path, &trace)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Command::GetPack(args) => {
            let start = args.repo.clone().unwrap_or(std::env::current_dir()?);
            let repo = RepoRoot::discover_from(&start)?;
            let (plan, pack) = compile_context_pack_with_plan(
                &repo.path,
                &args.task,
                args.mode.into(),
                args.budget.into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ctxpack_core::PackBudget;
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
}
