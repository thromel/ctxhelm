use crate::planning::{
    normalized_target_agent, prepare_context_plan_with_paths,
    prepare_context_plan_with_paths_and_semantic,
};
use ctxpack_core::{
    ContextPack, ContextPlan, Diagnostic, DiagnosticSeverity, LineRange, PackBudget, PackSection,
    TaskType,
};
use ctxpack_index::{
    load_or_refresh_inventory, read_safe_source, repo_id_for_path, task_hash, InventoryError,
    InventoryOptions, RepoInventory, SourceReadStatus, SOURCE_READ_MAX_BYTES,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn compile_context_pack(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
) -> Result<ContextPack, InventoryError> {
    let (_, pack) =
        compile_context_pack_with_plan_for_agent(repo_root, task, task_type, budget, "generic")?;
    Ok(pack)
}

pub fn compile_context_pack_with_plan(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    compile_context_pack_with_plan_and_paths_for_agent(
        repo_root,
        task,
        task_type,
        budget,
        &[],
        "generic",
    )
}

pub fn compile_context_pack_with_plan_and_paths(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    anchor_paths: &[String],
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    compile_context_pack_with_plan_and_paths_for_agent(
        repo_root,
        task,
        task_type,
        budget,
        anchor_paths,
        "generic",
    )
}

pub fn compile_context_pack_with_plan_for_agent(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    target_agent: &str,
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    compile_context_pack_with_plan_and_paths_for_agent(
        repo_root,
        task,
        task_type,
        budget,
        &[],
        target_agent,
    )
}

pub fn compile_context_pack_with_plan_and_paths_for_agent(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    anchor_paths: &[String],
    target_agent: &str,
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    compile_context_pack_with_plan_and_paths_for_agent_and_semantic(
        repo_root,
        task,
        task_type,
        budget,
        anchor_paths,
        target_agent,
        false,
    )
}

pub fn compile_context_pack_with_plan_and_paths_for_agent_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    anchor_paths: &[String],
    target_agent: &str,
    semantic_enabled: bool,
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    let repo_root = repo_root.as_ref();
    let plan = if semantic_enabled {
        prepare_context_plan_with_paths_and_semantic(
            repo_root,
            task,
            task_type,
            anchor_paths,
            true,
        )?
    } else {
        prepare_context_plan_with_paths(repo_root, task, task_type, anchor_paths)?
    };
    let pack = compile_pack_from_plan(repo_root, task, &plan, budget, target_agent);
    Ok((plan, pack))
}

pub fn compile_context_pack_from_plan(
    repo_root: impl AsRef<Path>,
    task: &str,
    plan: &ContextPlan,
    budget: PackBudget,
) -> ContextPack {
    compile_context_pack_from_plan_for_agent(repo_root, task, plan, budget, "generic")
}

pub fn compile_context_pack_from_plan_for_agent(
    repo_root: impl AsRef<Path>,
    task: &str,
    plan: &ContextPlan,
    budget: PackBudget,
    target_agent: &str,
) -> ContextPack {
    compile_pack_from_plan(repo_root.as_ref(), task, plan, budget, target_agent)
}

fn compile_pack_from_plan(
    repo_root: &Path,
    task: &str,
    plan: &ContextPlan,
    budget: PackBudget,
    target_agent: &str,
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

    let mut diagnostics = plan.diagnostics.clone();
    let inventory_report = match load_or_refresh_inventory(repo_root, &InventoryOptions::default())
    {
        Ok(report) => {
            diagnostics.extend(report.diagnostics.clone());
            Some(report)
        }
        Err(error) => {
            diagnostics.push(Diagnostic {
                code: "source_revalidation_unavailable".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "Pack snippets could not be revalidated against current safe inventory: {error}"
                ),
                paths: Vec::new(),
                count: 0,
            });
            None
        }
    };

    let mut snippet_warnings = Vec::new();
    let source_snippets = render_target_snippets(
        repo_root,
        inventory_report.as_ref().map(|report| &report.inventory),
        plan,
        limits.target_files,
        limits.lines,
        &mut diagnostics,
        &mut snippet_warnings,
    );
    if !source_snippets.is_empty() {
        sections.push(section(
            "Target snippets",
            "target_snippets",
            source_snippets,
        ));
    }

    let test_snippets = render_test_snippets(
        repo_root,
        inventory_report.as_ref().map(|report| &report.inventory),
        plan,
        limits.test_files,
        limits.lines,
        &mut diagnostics,
        &mut snippet_warnings,
    );
    if !test_snippets.is_empty() {
        sections.push(section("Test snippets", "test_snippets", test_snippets));
    }

    sections.push(section(
        "Final checklist",
        "final_checklist",
        render_final_checklist(plan),
    ));

    let mut warnings = plan.missing_info_questions.clone();
    warnings.extend(snippet_warnings);
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
        repo_id: pack_repo_id(repo_root),
        task_hash: task_hash(task),
        task_type: plan.task_type.clone(),
        target_agent: normalized_target_agent(target_agent),
        budget,
        sections,
        token_estimate,
        confidence: plan.confidence,
        warnings,
        diagnostics,
        privacy_status: plan.privacy_status.clone(),
    }
}

pub(crate) fn pack_repo_id(repo_root: &Path) -> String {
    let canonical = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    repo_id_for_path(&canonical)
}

pub fn render_pack_markdown(pack: &ContextPack) -> String {
    let mut output = format!(
        "# Context Pack\n\n- Pack ID: `{}`\n- Task ID: `{}`\n- Repo ID: `{}`\n- Task hash: `{}`\n- Target agent: `{}`\n- Task type: `{:?}`\n- Budget: `{:?}`\n- Confidence: `{:.2}`\n- Estimated tokens: `{}`\n- Privacy: local-only `{}`\n\n",
        pack.id,
        pack.task_id,
        pack.repo_id,
        pack.task_hash,
        pack.target_agent,
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
    inventory: Option<&RepoInventory>,
    plan: &ContextPlan,
    max_files: usize,
    max_lines: usize,
    diagnostics: &mut Vec<Diagnostic>,
    warnings: &mut Vec<String>,
) -> String {
    plan.target_files
        .iter()
        .take(max_files)
        .filter_map(|file| {
            render_file_snippet(
                repo_root,
                inventory,
                &file.path,
                file.line_range.as_ref(),
                max_lines,
                diagnostics,
                warnings,
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_test_snippets(
    repo_root: &Path,
    inventory: Option<&RepoInventory>,
    plan: &ContextPlan,
    max_files: usize,
    max_lines: usize,
    diagnostics: &mut Vec<Diagnostic>,
    warnings: &mut Vec<String>,
) -> String {
    let mut seen = BTreeSet::new();
    plan.related_tests
        .iter()
        .take(max_files)
        .filter(|test| seen.insert(test.path.clone()))
        .filter_map(|test| {
            render_file_snippet(
                repo_root,
                inventory,
                &test.path,
                None,
                max_lines,
                diagnostics,
                warnings,
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_file_snippet(
    repo_root: &Path,
    inventory: Option<&RepoInventory>,
    path: &str,
    line_range: Option<&LineRange>,
    max_lines: usize,
    diagnostics: &mut Vec<Diagnostic>,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let Some(inventory) = inventory else {
        warnings.push(format!(
            "Snippet for `{path}` was skipped because safe inventory revalidation failed."
        ));
        return None;
    };
    let source = match read_safe_source(repo_root, inventory, path, SOURCE_READ_MAX_BYTES) {
        Ok(source) => source,
        Err(error) => {
            diagnostics.push(Diagnostic {
                code: "source_revalidation_failed".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!("Snippet for `{path}` was skipped: {error}"),
                paths: vec![path.to_string()],
                count: 1,
            });
            warnings.push(format!("Snippet for `{path}` was skipped: {error}"));
            return None;
        }
    };
    diagnostics.extend(source.diagnostics.clone());
    let SourceReadStatus::Read = source.status else {
        let reason = source
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.message.clone())
            .unwrap_or_else(|| "source could not be safely read".to_string());
        warnings.push(format!("Snippet for `{path}` was skipped: {reason}"));
        return None;
    };
    let content = source.text.unwrap_or_default();
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
