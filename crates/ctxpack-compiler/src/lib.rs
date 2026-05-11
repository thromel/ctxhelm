use ctxpack_core::{
    Command, ContextPack, ContextPlan, EvalTrace, FileRole, LineRange, PackBudget, PackOption,
    PackSection, PrivacyStatus, RelatedTest, RiskFlag, TargetFile, TaskType,
};
use ctxpack_index::{
    co_change_hints, lexical_search, load_or_build_inventory, related_tests, repo_id_for_path,
    symbol_search, task_hash, CoChangeOptions, InventoryError, InventoryOptions, SearchOptions,
    SymbolOptions,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn empty_plan_for_task(task_type: TaskType) -> ContextPlan {
    base_plan(task_type)
}

pub fn prepare_context_plan(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
) -> Result<ContextPlan, InventoryError> {
    prepare_context_plan_with_paths(repo_root, task, task_type, &[])
}

pub fn prepare_context_plan_with_paths(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
) -> Result<ContextPlan, InventoryError> {
    let repo_root = repo_root.as_ref();
    let mut plan = base_plan(task_type);
    let task = task.trim();
    if task.is_empty() {
        plan.missing_info_questions
            .push("Describe the code task or failure to prepare repository context.".to_string());
        return Ok(plan);
    }

    let symbol_results = symbol_search(
        repo_root,
        task,
        &SymbolOptions {
            limit: search_limit(&plan.task_type),
        },
    )?;
    let search_results = lexical_search(
        repo_root,
        task,
        &SearchOptions {
            limit: search_limit(&plan.task_type),
        },
    )?;
    let mut roles = BTreeMap::new();
    for result in &search_results {
        roles.insert(result.path.clone(), result.role.clone());
    }
    let mut target_files = Vec::new();
    let mut seen_paths = BTreeSet::new();
    let (anchor_targets, unavailable_anchors) = anchored_target_files(repo_root, anchor_paths)?;
    for unavailable in unavailable_anchors {
        plan.risk_flags.push(RiskFlag {
            code: "anchor_unavailable".to_string(),
            message: format!(
                "Active context path `{unavailable}` was not included because it is ignored, generated, sensitive, outside the repo, or not inventoried."
            ),
        });
    }
    for (target, role) in anchor_targets {
        roles.insert(target.path.clone(), role);
        if seen_paths.insert(target.path.clone()) {
            target_files.push(target);
        }
    }
    for result in symbol_results.iter().take(5) {
        if seen_paths.insert(result.symbol.path.clone()) {
            target_files.push(TargetFile {
                path: result.symbol.path.clone(),
                reason: format!(
                    "symbol match `{}` ({:?}): {}",
                    result.symbol.name, result.symbol.kind, result.reason
                ),
                line_range: Some(symbol_line_range(
                    result.symbol.start_line,
                    result.symbol.end_line,
                )),
                confidence: normalize_score(result.score),
            });
        }
    }
    for result in search_results.iter().take(5) {
        if seen_paths.insert(result.path.clone()) {
            target_files.push(TargetFile {
                path: result.path.clone(),
                reason: format!("lexical match: {}", result.reason),
                line_range: None,
                confidence: normalize_score(result.score),
            });
        }
    }
    target_files.truncate(5);
    let target_paths = target_files
        .iter()
        .map(|target| target.path.clone())
        .collect::<Vec<_>>();
    let source_target_paths = target_paths
        .iter()
        .filter(|path| {
            roles
                .get(*path)
                .is_none_or(|role| matches!(role, FileRole::Source))
        })
        .cloned()
        .collect::<Vec<_>>();

    plan.target_files = target_files;

    if target_paths.is_empty() {
        plan.missing_info_questions.push(
            "No safe inventoried files matched the task. Provide a file path, symbol, or error text."
                .to_string(),
        );
    }

    let test_results = related_tests(repo_root, &source_target_paths)?;
    let mut command_set = BTreeSet::new();
    plan.related_tests = test_results
        .iter()
        .take(5)
        .map(|test| {
            if let Some(command) = &test.command {
                command_set.insert(command.clone());
            }
            RelatedTest {
                path: test.path.clone(),
                reason: test.reason.clone(),
                command: test.command.clone(),
                confidence: test.confidence,
            }
        })
        .collect();

    plan.recommended_commands = command_set
        .into_iter()
        .map(|command| Command {
            command,
            reason: "targeted validation for related test".to_string(),
        })
        .collect();

    let mut has_history = false;
    match co_change_hints(
        repo_root,
        &source_target_paths,
        &CoChangeOptions {
            limit: co_change_limit(&plan.task_type),
        },
    ) {
        Ok(co_changes) => {
            has_history = !co_changes.is_empty();
            for hint in co_changes.into_iter().take(5) {
                plan.risk_flags.push(RiskFlag {
                    code: "co_change_hint".to_string(),
                    message: format!(
                        "{} changed with target files in {} local commit(s): {}",
                        hint.path, hint.commit_count, hint.reason
                    ),
                });
            }
        }
        Err(error) => {
            plan.risk_flags.push(RiskFlag {
                code: "co_change_unavailable".to_string(),
                message: format!(
                    "Local git co-change hints were unavailable; continuing without history signal: {error}"
                ),
            });
        }
    }

    plan.confidence = plan_confidence(
        !plan.target_files.is_empty(),
        !plan.related_tests.is_empty(),
        has_history,
    );

    Ok(plan)
}

pub fn compile_context_pack(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
) -> Result<ContextPack, InventoryError> {
    let (_, pack) = compile_context_pack_with_plan(repo_root, task, task_type, budget)?;
    Ok(pack)
}

pub fn compile_context_pack_with_plan(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    compile_context_pack_with_plan_and_paths(repo_root, task, task_type, budget, &[])
}

pub fn compile_context_pack_with_plan_and_paths(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    anchor_paths: &[String],
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    let repo_root = repo_root.as_ref();
    let plan = prepare_context_plan_with_paths(repo_root, task, task_type, anchor_paths)?;
    let pack = compile_pack_from_plan(repo_root, task, &plan, budget);
    Ok((plan, pack))
}

fn compile_pack_from_plan(
    repo_root: &Path,
    task: &str,
    plan: &ContextPlan,
    budget: PackBudget,
) -> ContextPack {
    let limits = pack_limits(&budget);

    let mut sections = vec![
        section(
            "Task",
            "task",
            format!(
                "Task: {}\nTask type: {:?}\nPlan confidence: {:.2}",
                task.trim(),
                plan.task_type,
                plan.confidence
            ),
        ),
        section(
            "High-confidence target files",
            "target_files",
            render_target_files(plan),
        ),
        section("Validation", "validation", render_validation(plan)),
    ];

    if !plan.risk_flags.is_empty() {
        sections.push(section("Risk flags", "risk_flags", render_risk_flags(plan)));
    }

    let source_snippets =
        render_target_snippets(repo_root, plan, limits.target_files, limits.lines);
    if !source_snippets.is_empty() {
        sections.push(section(
            "Target snippets",
            "target_snippets",
            source_snippets,
        ));
    }

    let test_snippets = render_test_snippets(repo_root, plan, limits.test_files, limits.lines);
    if !test_snippets.is_empty() {
        sections.push(section("Test snippets", "test_snippets", test_snippets));
    }

    sections.push(section(
        "Final checklist",
        "final_checklist",
        render_final_checklist(plan),
    ));

    let mut warnings = plan.missing_info_questions.clone();
    if plan.target_files.len() > limits.target_files {
        warnings.push(format!(
            "Pack budget limited target snippets to {} file(s).",
            limits.target_files
        ));
    }
    if plan.related_tests.len() > limits.test_files {
        warnings.push(format!(
            "Pack budget limited test snippets to {} file(s).",
            limits.test_files
        ));
    }

    let token_estimate = estimate_tokens(&sections);
    ContextPack {
        id: Uuid::new_v4(),
        task_id: plan.task_id,
        task_type: plan.task_type.clone(),
        budget,
        sections,
        token_estimate,
        confidence: plan.confidence,
        warnings,
        privacy_status: plan.privacy_status.clone(),
    }
}

pub fn render_pack_markdown(pack: &ContextPack) -> String {
    let mut output = format!(
        "# Context Pack\n\n- Pack ID: `{}`\n- Task ID: `{}`\n- Task type: `{:?}`\n- Budget: `{:?}`\n- Confidence: `{:.2}`\n- Estimated tokens: `{}`\n- Privacy: local-only `{}`\n\n",
        pack.id,
        pack.task_id,
        pack.task_type,
        pack.budget,
        pack.confidence,
        pack.token_estimate,
        pack.privacy_status.local_only
    );

    if !pack.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &pack.warnings {
            output.push_str(&format!("- {warning}\n"));
        }
        output.push('\n');
    }

    for section in &pack.sections {
        output.push_str(&format!("## {}\n\n{}\n\n", section.title, section.content));
    }

    output
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

fn base_plan(task_type: TaskType) -> ContextPlan {
    let task_id = Uuid::new_v4();
    ContextPlan {
        task_id,
        task_type,
        confidence: 0.0,
        target_files: Vec::new(),
        related_tests: Vec::new(),
        recommended_commands: Vec::new(),
        pack_options: vec![
            PackOption {
                budget: PackBudget::Brief,
                resource_uri: format!("ctxpack://pack/{task_id}/brief"),
            },
            PackOption {
                budget: PackBudget::Standard,
                resource_uri: format!("ctxpack://pack/{task_id}/standard"),
            },
        ],
        missing_info_questions: Vec::new(),
        risk_flags: Vec::new(),
        privacy_status: PrivacyStatus::local_only(),
    }
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

fn normalized_target_agent(target_agent: &str) -> String {
    let target_agent = target_agent.trim();
    if target_agent.is_empty() {
        "generic".to_string()
    } else {
        target_agent.to_string()
    }
}

fn search_limit(task_type: &TaskType) -> usize {
    match task_type {
        TaskType::Explain => 8,
        TaskType::Review | TaskType::Refactor => 12,
        TaskType::BugFix | TaskType::Feature | TaskType::Test => 10,
    }
}

fn co_change_limit(task_type: &TaskType) -> usize {
    match task_type {
        TaskType::BugFix | TaskType::Refactor | TaskType::Review => 8,
        TaskType::Feature | TaskType::Test | TaskType::Explain => 5,
    }
}

fn normalize_score(score: f32) -> f32 {
    (score / 20.0).clamp(0.15, 0.95)
}

fn symbol_line_range(start_line: u32, end_line: u32) -> LineRange {
    LineRange {
        start: start_line,
        end: end_line.max(start_line),
    }
}

fn plan_confidence(has_targets: bool, has_tests: bool, has_history: bool) -> f32 {
    let mut confidence = if has_targets { 0.45 } else { 0.05 };
    if has_tests {
        confidence += 0.25;
    }
    if has_history {
        confidence += 0.15;
    }
    confidence
}

type AnchoredTarget = (TargetFile, FileRole);

fn anchored_target_files(
    repo_root: &Path,
    anchor_paths: &[String],
) -> Result<(Vec<AnchoredTarget>, Vec<String>), InventoryError> {
    if anchor_paths.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let inventory = load_or_build_inventory(repo_root, &InventoryOptions::default())?;
    let files_by_path = inventory
        .files
        .into_iter()
        .filter(|file| !file.ignored && !file.generated && file.role != FileRole::Sensitive)
        .map(|file| (file.path.clone(), file.role))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut targets = Vec::new();
    let mut unavailable = Vec::new();

    for input in anchor_paths {
        let Some(path) = normalize_anchor_path(repo_root, input) else {
            unavailable.push(input.clone());
            continue;
        };
        let Some(role) = files_by_path.get(&path) else {
            unavailable.push(input.clone());
            continue;
        };
        if seen.insert(path.clone()) {
            targets.push((
                TargetFile {
                    path,
                    reason: "explicit path anchor from active context".to_string(),
                    line_range: None,
                    confidence: 0.98,
                },
                role.clone(),
            ));
        }
    }

    Ok((targets, unavailable))
}

fn normalize_anchor_path(repo_root: &Path, input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let path = Path::new(input);
    let relative = if path.is_absolute() {
        path.strip_prefix(repo_root).ok()?
    } else {
        path
    };
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

struct PackLimits {
    target_files: usize,
    test_files: usize,
    lines: usize,
}

fn pack_limits(budget: &PackBudget) -> PackLimits {
    match budget {
        PackBudget::Brief => PackLimits {
            target_files: 3,
            test_files: 2,
            lines: 80,
        },
        PackBudget::Standard => PackLimits {
            target_files: 5,
            test_files: 4,
            lines: 180,
        },
        PackBudget::Deep => PackLimits {
            target_files: 8,
            test_files: 6,
            lines: 320,
        },
    }
}

fn section(title: &str, kind: &str, content: String) -> PackSection {
    PackSection {
        title: title.to_string(),
        kind: kind.to_string(),
        content,
    }
}

fn render_target_files(plan: &ContextPlan) -> String {
    if plan.target_files.is_empty() {
        return "No high-confidence target files were found.".to_string();
    }

    plan.target_files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let line_hint = file
                .line_range
                .as_ref()
                .map(|range| format!("\n   - Lines: {}-{}", range.start, range.end))
                .unwrap_or_default();
            format!(
                "{}. `{}`\n   - Reason: {}\n   - Confidence: {:.2}",
                index + 1,
                file.path,
                file.reason,
                file.confidence
            ) + &line_hint
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_validation(plan: &ContextPlan) -> String {
    if plan.recommended_commands.is_empty() {
        return "No targeted validation command was inferred.".to_string();
    }

    plan.recommended_commands
        .iter()
        .map(|command| format!("- `{}`\n  - Reason: {}", command.command, command.reason))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_risk_flags(plan: &ContextPlan) -> String {
    plan.risk_flags
        .iter()
        .map(|flag| format!("- `{}`: {}", flag.code, flag.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_target_snippets(
    repo_root: &Path,
    plan: &ContextPlan,
    max_files: usize,
    max_lines: usize,
) -> String {
    plan.target_files
        .iter()
        .take(max_files)
        .filter_map(|file| {
            render_file_snippet(repo_root, &file.path, file.line_range.as_ref(), max_lines)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_test_snippets(
    repo_root: &Path,
    plan: &ContextPlan,
    max_files: usize,
    max_lines: usize,
) -> String {
    let mut seen = BTreeSet::new();
    plan.related_tests
        .iter()
        .take(max_files)
        .filter(|test| seen.insert(test.path.clone()))
        .filter_map(|test| render_file_snippet(repo_root, &test.path, None, max_lines))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_file_snippet(
    repo_root: &Path,
    path: &str,
    line_range: Option<&LineRange>,
    max_lines: usize,
) -> Option<String> {
    let absolute_path = repo_root.join(path);
    let content = fs::read_to_string(absolute_path).ok()?;
    let total_lines = content.lines().count();
    let (start_line, end_line) = snippet_bounds(line_range, max_lines, total_lines);
    let mut snippet = String::new();
    for (index, line) in content
        .lines()
        .enumerate()
        .skip(start_line.saturating_sub(1))
        .take(end_line.saturating_sub(start_line).saturating_add(1))
    {
        snippet.push_str(&format!("{:>4}: {}\n", index + 1, line));
    }
    if start_line > 1 {
        snippet.insert_str(0, &format!("... omitted lines 1-{}\n", start_line - 1));
    }
    if end_line < total_lines {
        snippet.push_str(&format!(
            "... omitted lines {}-{}\n",
            end_line + 1,
            total_lines
        ));
    }

    Some(format!("### `{path}`\n\n```text\n{snippet}```"))
}

fn snippet_bounds(
    line_range: Option<&LineRange>,
    max_lines: usize,
    total_lines: usize,
) -> (usize, usize) {
    let max_lines = max_lines.max(1);
    if total_lines == 0 {
        return (1, 0);
    }
    let Some(range) = line_range else {
        return (1, total_lines.min(max_lines));
    };
    let requested_start = usize::try_from(range.start).unwrap_or(1).max(1);
    let requested_end = usize::try_from(range.end).unwrap_or(requested_start);
    let context_before = 5usize;
    let mut start = requested_start.saturating_sub(context_before).max(1);
    let mut end = requested_end.min(total_lines);
    if end < start {
        end = start;
    }
    let current_len = end.saturating_sub(start).saturating_add(1);
    if current_len < max_lines {
        end = (end + (max_lines - current_len)).min(total_lines);
    }
    if end.saturating_sub(start).saturating_add(1) > max_lines {
        start = end.saturating_sub(max_lines).saturating_add(1);
    }
    (start, end)
}

fn render_final_checklist(plan: &ContextPlan) -> String {
    let mut lines = vec![
        "- Read the high-confidence target files before editing.".to_string(),
        "- Keep changes scoped to the task and preserve existing public behavior unless evidence says otherwise.".to_string(),
    ];

    if !plan.related_tests.is_empty() {
        lines.push("- Use the related tests as the first validation path.".to_string());
    }
    if !plan.risk_flags.is_empty() {
        lines.push("- Review risk flags before broadening the edit scope.".to_string());
    }
    if !plan.missing_info_questions.is_empty() {
        lines.push("- Resolve missing information before making a broad change.".to_string());
    }

    lines.join("\n")
}

fn estimate_tokens(sections: &[PackSection]) -> usize {
    let words = sections
        .iter()
        .map(|section| {
            section.title.split_whitespace().count() + section.content.split_whitespace().count()
        })
        .sum::<usize>();
    words.saturating_mul(4).div_ceil(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command as ProcessCommand;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn empty_plan_includes_brief_and_standard_pack_options() {
        let plan = empty_plan_for_task(TaskType::Explain);
        assert_eq!(plan.pack_options.len(), 2);
        assert!(plan.pack_options[0].resource_uri.ends_with("/brief"));
        assert!(plan.pack_options[1].resource_uri.ends_with("/standard"));
    }

    #[test]
    fn prepare_context_plan_fuses_search_tests_and_history() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();

        assert_eq!(plan.task_type, TaskType::BugFix);
        assert!(plan.confidence > 0.5);
        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert_eq!(plan.target_files[0].line_range.as_ref().unwrap().start, 1);
        assert_eq!(plan.related_tests[0].path, "tests/auth/session.test.ts");
        assert_eq!(
            plan.recommended_commands[0].command,
            "pnpm test tests/auth/session.test.ts"
        );
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.message.contains("tests/auth/session.test.ts")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn context_pack_snippets_focus_around_symbol_line_ranges() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        let header = (1..=30)
            .map(|index| format!("const filler{index} = {index};"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(
            repo.join("src/auth/session.ts"),
            format!("{header}\nexport function requireSession() {{\n  return true;\n}}\n"),
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let (plan, pack) = compile_context_pack_with_plan(
            &repo,
            "fix requireSession bug",
            TaskType::BugFix,
            PackBudget::Brief,
        )
        .unwrap();
        let markdown = render_pack_markdown(&pack);

        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert_eq!(plan.target_files[0].line_range.as_ref().unwrap().start, 31);
        assert!(markdown.contains("- Lines: 31-31"));
        assert!(markdown.contains("31: export function requireSession"));
        assert!(markdown.contains("... omitted lines 1-"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_context_plan_degrades_when_git_history_is_unavailable() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();

        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert_eq!(plan.related_tests[0].path, "tests/auth/session.test.ts");
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.code == "co_change_unavailable"));
        assert!(plan
            .risk_flags
            .iter()
            .all(|flag| flag.code != "co_change_hint"));
        assert_eq!(plan.confidence, plan_confidence(true, true, false));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_context_plan_prefers_explicit_path_anchors() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("src/billing")).unwrap();
        fs::create_dir_all(repo.join("tests/billing")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/billing/invoice.ts"),
            "export function issueInvoice() { return 'invoice'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/billing/invoice.test.ts"),
            "import { issueInvoice } from '../../src/billing/invoice';\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add billing"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let anchors = vec!["src/billing/invoice.ts".to_string()];
        let plan =
            prepare_context_plan_with_paths(&repo, "fix session bug", TaskType::BugFix, &anchors)
                .unwrap();

        assert_eq!(plan.target_files[0].path, "src/billing/invoice.ts");
        assert_eq!(
            plan.target_files[0].reason,
            "explicit path anchor from active context"
        );
        assert_eq!(plan.target_files[0].confidence, 0.98);
        assert_eq!(plan.related_tests[0].path, "tests/billing/invoice.test.ts");

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_context_plan_reports_unavailable_path_anchors() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("dist/generated.js"),
            "export const generated = true;\n",
        )
        .unwrap();
        std::env::set_var("CTXPACK_HOME", &home);

        let anchors = vec![
            "../outside.ts".to_string(),
            "dist/generated.js".to_string(),
            "src/auth/session.ts".to_string(),
        ];
        let plan =
            prepare_context_plan_with_paths(&repo, "fix session bug", TaskType::BugFix, &anchors)
                .unwrap();

        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert_eq!(
            plan.risk_flags
                .iter()
                .filter(|flag| flag.code == "anchor_unavailable")
                .count(),
            2
        );
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.message.contains("../outside.ts")));
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.message.contains("dist/generated.js")));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn prepare_context_plan_reports_missing_info_for_empty_task() {
        let temp = tempfile::tempdir().unwrap();
        let plan = prepare_context_plan(temp.path(), " ", TaskType::Explain).unwrap();

        assert_eq!(plan.confidence, 0.0);
        assert!(plan.target_files.is_empty());
        assert_eq!(plan.missing_info_questions.len(), 1);
    }

    #[test]
    fn compile_context_pack_materializes_plan_snippets_and_validation() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() {\n  return true;\n}\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\ntest('session', () => requireSession());\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let pack = compile_context_pack(
            &repo,
            "fix requireSession bug",
            TaskType::BugFix,
            PackBudget::Brief,
        )
        .unwrap();
        let markdown = render_pack_markdown(&pack);

        assert_eq!(pack.budget, PackBudget::Brief);
        assert!(pack.token_estimate > 0);
        assert!(pack.sections.iter().any(|section| section.kind == "task"));
        assert!(markdown.contains("src/auth/session.ts"));
        assert!(markdown.contains("export function requireSession"));
        assert!(markdown.contains("tests/auth/session.test.ts"));
        assert!(markdown.contains("pnpm test tests/auth/session.test.ts"));
        assert!(markdown.contains("Final checklist"));

        std::env::remove_var("CTXPACK_HOME");
    }

    #[test]
    fn compile_context_pack_with_plan_supports_source_free_eval_trace() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxpack-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxpack@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxpack"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        std::env::set_var("CTXPACK_HOME", &home);

        let (plan, pack) = compile_context_pack_with_plan(
            &repo,
            "fix requireSession bug",
            TaskType::BugFix,
            PackBudget::Brief,
        )
        .unwrap();
        let trace = eval_trace_for_pack(&repo, "fix requireSession bug", "codex", &plan, &pack);
        let value = serde_json::to_value(&trace).unwrap();

        assert_eq!(trace.pack_id, Some(pack.id));
        assert_eq!(trace.budget, Some(PackBudget::Brief));
        assert_eq!(trace.target_agent, "codex");
        assert!(trace
            .recommended_files
            .contains(&"src/auth/session.ts".to_string()));
        assert_eq!(trace.recommended_tests, vec!["tests/auth/session.test.ts"]);
        assert_eq!(value["sourceTextLogged"], false);
        assert!(value.get("task").is_none());

        std::env::remove_var("CTXPACK_HOME");
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = ProcessCommand::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
