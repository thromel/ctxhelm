use crate::planning::{normalized_target_agent, prepare_context_plan_with_paths};
use ctxpack_core::{
    ContextPack, ContextPlan, Diagnostic, DiagnosticSeverity, InspectorCandidateView,
    InspectorContentKind, InspectorMemoryView, InspectorRelatedTestView, InspectorSectionView,
    InspectorTargetFileView, LineRange, PackBudget, PackInspectorView, PackSection, TaskType,
};
use ctxpack_index::{
    load_or_refresh_inventory, read_safe_source, repo_id_for_path, task_hash, InventoryError,
    InventoryOptions, RepoInventory, SemanticProviderConfig, SourceReadStatus,
    SOURCE_READ_MAX_BYTES,
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
    compile_context_pack_with_plan_and_paths_for_agent_and_semantic_provider(
        repo_root,
        task,
        task_type,
        budget,
        anchor_paths,
        target_agent,
        semantic_enabled,
        SemanticProviderConfig::default(),
    )
}

pub fn compile_context_pack_with_plan_and_paths_for_agent_and_semantic_provider(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    budget: PackBudget,
    anchor_paths: &[String],
    target_agent: &str,
    semantic_enabled: bool,
    semantic_provider: SemanticProviderConfig,
) -> Result<(ContextPlan, ContextPack), InventoryError> {
    let repo_root = repo_root.as_ref();
    let plan = if semantic_enabled {
        crate::planning::prepare_context_plan_with_paths_and_semantic_provider(
            repo_root,
            task,
            task_type,
            anchor_paths,
            true,
            semantic_provider,
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
    if !plan.context_areas.is_empty() {
        sections.push(section(
            "Context areas",
            "context_areas",
            render_context_areas(plan),
        ));
    }
    if !plan.selected_memory.is_empty() {
        sections.push(section(
            "Selected memory",
            "selected_memory",
            render_selected_memory(plan, memory_limit(&budget)),
        ));
    }
    if let Some(policy) = &plan.provider_policy {
        sections.push(section(
            "Provider policy",
            "provider_policy",
            render_provider_policy(policy),
        ));
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
        provider_policy: plan.provider_policy.clone(),
        privacy_status: plan.privacy_status.clone(),
    }
}

pub(crate) fn pack_repo_id(repo_root: &Path) -> String {
    let canonical = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    repo_id_for_path(&canonical)
}

fn render_provider_policy(policy: &ctxpack_core::ProviderPolicyReport) -> String {
    let mut output = format!(
        "Policy: `{}`\nLocal providers: `{}`\nCloud embeddings: `{}`\nCloud reranking: `{}`\nSource transfer: `{}`\n\n",
        policy.policy_path.as_deref().unwrap_or("safe defaults"),
        policy.policy.allow_local_providers,
        policy.policy.allow_cloud_embeddings,
        policy.policy.allow_cloud_reranking,
        policy.policy.allow_source_transfer
    );
    for decision in &policy.decisions {
        output.push_str(&format!(
            "- `{:?}` provider `{}` status `{:?}` remoteAllowed `{}` sourceAllowed `{}`: {}\n",
            decision.capability,
            decision.provider,
            decision.status,
            decision.remote_allowed,
            decision.source_text_allowed,
            decision.reason
        ));
    }
    output
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

pub fn compile_pack_inspector_view(plan: &ContextPlan, pack: &ContextPack) -> PackInspectorView {
    let sections = pack
        .sections
        .iter()
        .map(|section| {
            let source_bearing = is_source_bearing_section(&section.kind);
            InspectorSectionView {
                title: section.title.clone(),
                kind: section.kind.clone(),
                content_kind: if source_bearing {
                    InspectorContentKind::SourceBearing
                } else {
                    InspectorContentKind::SourceFree
                },
                source_bearing,
                token_estimate: estimate_section_tokens(section),
            }
        })
        .collect::<Vec<_>>();
    let source_bearing_section_count = sections
        .iter()
        .filter(|section| section.source_bearing)
        .count();

    PackInspectorView {
        pack_id: pack.id,
        task_id: pack.task_id,
        repo_id: pack.repo_id.clone(),
        workspace_id: None,
        task_hash: pack.task_hash.clone(),
        task_type: pack.task_type.clone(),
        target_agent: pack.target_agent.clone(),
        budget: pack.budget.clone(),
        token_estimate: pack.token_estimate,
        confidence: pack.confidence,
        source_text_logged: false,
        source_bearing_section_count,
        privacy_status: pack.privacy_status.clone(),
        warnings: pack.warnings.clone(),
        diagnostics: pack.diagnostics.clone(),
        sections,
        target_files: plan
            .target_files
            .iter()
            .map(|target| InspectorTargetFileView {
                path: target.path.clone(),
                reason: target.reason.clone(),
                line_range: target.line_range.clone(),
                confidence: target.confidence,
                attribution: target.attribution.clone(),
            })
            .collect(),
        related_tests: plan
            .related_tests
            .iter()
            .map(|test| InspectorRelatedTestView {
                path: test.path.clone(),
                reason: test.reason.clone(),
                command: test.command.clone(),
                confidence: test.confidence,
                attribution: test.attribution.clone(),
            })
            .collect(),
        validation_commands: plan.recommended_commands.clone(),
        selected_memory: plan
            .selected_memory
            .iter()
            .map(|memory| InspectorMemoryView {
                id: memory.card.id.clone(),
                kind: memory.card.kind.clone(),
                title: memory.card.title.clone(),
                freshness: memory.card.freshness.clone(),
                review_status: memory.card.review_status.clone(),
                disabled: memory.card.disabled,
                confidence: memory.card.confidence,
                score: memory.score,
                reason: memory.reason.clone(),
                source_links: memory.card.source_links.clone(),
                evidence: memory.evidence.clone(),
                privacy_status: memory.card.privacy_status.clone(),
            })
            .collect(),
        retrieval_candidates: plan
            .retrieval_candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| InspectorCandidateView {
                id: inspector_candidate_id(
                    index,
                    candidate.path.as_deref(),
                    &candidate.reason_code,
                ),
                kind: candidate.kind.clone(),
                path: candidate.path.clone(),
                role: candidate.role.clone(),
                reason_code: candidate.reason_code.clone(),
                confidence: candidate.confidence,
                signal_scores: candidate.signal_scores.clone(),
                evidence: candidate.evidence.clone(),
            })
            .collect(),
    }
}

pub fn render_pack_inspector_markdown(view: &PackInspectorView) -> String {
    let mut output = format!(
        "# Pack Inspector\n\n- Pack ID: `{}`\n- Task ID: `{}`\n- Repo ID: `{}`\n- Task hash: `{}`\n- Target agent: `{}`\n- Task type: `{:?}`\n- Budget: `{:?}`\n- Confidence: `{:.2}`\n- Estimated pack tokens: `{}`\n- Source text logged: `{}`\n- Source-bearing sections: `{}`\n- Privacy: local-only `{}`\n\n",
        view.pack_id,
        view.task_id,
        view.repo_id,
        view.task_hash,
        view.target_agent,
        view.task_type,
        view.budget,
        view.confidence,
        view.token_estimate,
        view.source_text_logged,
        view.source_bearing_section_count,
        view.privacy_status.local_only
    );

    if !view.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &view.warnings {
            output.push_str(&format!("- {warning}\n"));
        }
        output.push('\n');
    }

    output.push_str("## Sections\n\n");
    for section in &view.sections {
        output.push_str(&format!(
            "- `{}` ({}) — {:?}, source-bearing `{}`, estimated tokens `{}`\n",
            section.title,
            section.kind,
            section.content_kind,
            section.source_bearing,
            section.token_estimate
        ));
    }
    output.push('\n');

    output.push_str("## Target Files\n\n");
    if view.target_files.is_empty() {
        output.push_str("No target files selected.\n\n");
    } else {
        for target in &view.target_files {
            let line_range = target
                .line_range
                .as_ref()
                .map(|range| format!(" lines {}-{}", range.start, range.end))
                .unwrap_or_default();
            output.push_str(&format!(
                "- `{}`{} — {:.2}: {}\n",
                target.path, line_range, target.confidence, target.reason
            ));
        }
        output.push('\n');
    }

    output.push_str("## Related Tests\n\n");
    if view.related_tests.is_empty() {
        output.push_str("No related tests selected.\n\n");
    } else {
        for test in &view.related_tests {
            let command = test
                .command
                .as_ref()
                .map(|command| format!(" command `{command}`"))
                .unwrap_or_default();
            output.push_str(&format!(
                "- `{}` — {:.2}: {}{}\n",
                test.path, test.confidence, test.reason, command
            ));
        }
        output.push('\n');
    }

    output.push_str("## Validation Commands\n\n");
    if view.validation_commands.is_empty() {
        output.push_str("No validation command inferred.\n\n");
    } else {
        for command in &view.validation_commands {
            output.push_str(&format!("- `{}` — {}\n", command.command, command.reason));
        }
        output.push('\n');
    }

    output.push_str("## Retrieval Candidates\n\n");
    if view.retrieval_candidates.is_empty() {
        output.push_str("No retrieval candidates recorded.\n\n");
    } else {
        for candidate in view.retrieval_candidates.iter().take(20) {
            let path = candidate.path.as_deref().unwrap_or("none");
            output.push_str(&format!(
                "- `{}` {:?} `{}` — {:.2} ({})\n",
                candidate.id, candidate.kind, path, candidate.confidence, candidate.reason_code
            ));
        }
        if view.retrieval_candidates.len() > 20 {
            output.push_str(&format!(
                "- ... {} more candidate(s)\n",
                view.retrieval_candidates.len() - 20
            ));
        }
        output.push('\n');
    }

    output
}

pub fn render_pack_inspector_html(view: &PackInspectorView) -> String {
    let mut sections = String::new();
    for section in &view.sections {
        sections.push_str(&format!(
            "<tr class=\"inspect-row\" data-kind=\"section\" data-source-bearing=\"{}\" data-filter-text=\"{}\"><td>{}</td><td><code>{}</code></td><td>{:?}</td><td>{}</td><td>{}</td></tr>",
            section.source_bearing,
            escape_html(&format!("{} {}", section.title, section.kind)),
            escape_html(&section.title),
            escape_html(&section.kind),
            section.content_kind,
            section.source_bearing,
            section.token_estimate
        ));
    }

    let mut targets = String::new();
    for target in &view.target_files {
        let lines = target
            .line_range
            .as_ref()
            .map(|range| format!("{}-{}", range.start, range.end))
            .unwrap_or_else(|| "n/a".to_string());
        let filter_text = format!("{} {} target file", target.path, target.reason);
        targets.push_str(&format!(
            "<tr class=\"inspect-row\" data-kind=\"target\" data-source-bearing=\"false\" data-filter-text=\"{}\"><td><code>{}</code></td><td>{}</td><td>{:.2}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&filter_text),
            escape_html(&target.path),
            escape_html(&lines),
            target.confidence,
            escape_html(&target.reason),
            render_signal_badges_from_evidence(&target.attribution)
        ));
    }

    let mut tests = String::new();
    for test in &view.related_tests {
        let filter_text = format!(
            "{} {} {} test",
            test.path,
            test.reason,
            test.command.as_deref().unwrap_or("")
        );
        tests.push_str(&format!(
            "<tr class=\"inspect-row\" data-kind=\"test\" data-source-bearing=\"false\" data-filter-text=\"{}\"><td><code>{}</code></td><td>{:.2}</td><td>{}</td><td><code>{}</code></td><td>{}</td></tr>",
            escape_html(&filter_text),
            escape_html(&test.path),
            test.confidence,
            escape_html(&test.reason),
            escape_html(test.command.as_deref().unwrap_or("")),
            render_signal_badges_from_evidence(&test.attribution)
        ));
    }

    let mut commands = String::new();
    for command in &view.validation_commands {
        commands.push_str(&format!(
            "<li class=\"inspect-row\" data-kind=\"command\" data-source-bearing=\"false\" data-filter-text=\"{} {}\"><code>{}</code><span>{}</span></li>",
            escape_html(&command.command),
            escape_html(&command.reason),
            escape_html(&command.command),
            escape_html(&command.reason)
        ));
    }

    let mut warnings = String::new();
    for warning in &view.warnings {
        warnings.push_str(&format!(
            "<li class=\"inspect-row\" data-kind=\"warning\" data-source-bearing=\"false\" data-filter-text=\"{}\">{}</li>",
            escape_html(warning),
            escape_html(warning)
        ));
    }
    if warnings.is_empty() {
        warnings.push_str("<li>No warnings.</li>");
    }

    let mut diagnostics = String::new();
    for diagnostic in &view.diagnostics {
        diagnostics.push_str(&format!(
            "<tr class=\"inspect-row\" data-kind=\"diagnostic\" data-source-bearing=\"false\" data-filter-text=\"{} {} {}\"><td><code>{}</code></td><td>{:?}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&diagnostic.code),
            escape_html(&diagnostic.message),
            escape_html(&diagnostic.paths.join(" ")),
            escape_html(&diagnostic.code),
            diagnostic.severity,
            escape_html(&diagnostic.message),
            diagnostic.count
        ));
    }
    if diagnostics.is_empty() {
        diagnostics.push_str("<tr><td colspan=\"4\">No diagnostics.</td></tr>");
    }

    let mut candidates = String::new();
    for candidate in &view.retrieval_candidates {
        let path = candidate.path.as_deref().unwrap_or("none");
        let filter_text = format!(
            "{} {:?} {} {}",
            candidate.id, candidate.kind, path, candidate.reason_code
        );
        candidates.push_str(&format!(
            "<tr class=\"inspect-row\" data-kind=\"candidate\" data-source-bearing=\"false\" data-filter-text=\"{}\"><td><code>{}</code></td><td>{:?}</td><td><code>{}</code></td><td>{:.2}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&filter_text),
            escape_html(&candidate.id),
            candidate.kind,
            escape_html(path),
            candidate.confidence,
            escape_html(&candidate.reason_code),
            render_signal_badges_from_scores(&candidate.signal_scores)
        ));
    }
    if candidates.is_empty() {
        candidates.push_str("<tr><td colspan=\"6\">No retrieval candidates recorded.</td></tr>");
    }

    let mut memory = String::new();
    for item in &view.selected_memory {
        let filter_text = format!(
            "{} {:?} {} {}",
            item.title,
            item.kind,
            item.reason,
            item.source_links.join(" ")
        );
        memory.push_str(&format!(
            "<tr class=\"inspect-row\" data-kind=\"memory\" data-source-bearing=\"false\" data-filter-text=\"{}\"><td><code>{}</code></td><td>{}</td><td>{:?}</td><td>{:?}</td><td>{:.2}</td><td>{}</td></tr>",
            escape_html(&filter_text),
            escape_html(&item.id),
            escape_html(&item.title),
            item.kind,
            item.freshness,
            item.score,
            escape_html(&item.reason)
        ));
    }
    if memory.is_empty() {
        memory.push_str("<tr><td colspan=\"6\">No selected memory.</td></tr>");
    }

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>ctxpack Pack Inspector</title>
  <style>
    * {{ box-sizing: border-box; }}
    body {{ margin: 0; font: 14px/1.45 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: #1f2933; background: #f6f7f9; }}
    main {{ max-width: 1240px; margin: 0 auto; padding: 28px 20px 44px; }}
    header {{ display: grid; gap: 12px; margin-bottom: 20px; }}
    h1 {{ margin: 0; font-size: 28px; letter-spacing: 0; }}
    h2 {{ margin: 28px 0 10px; font-size: 18px; letter-spacing: 0; }}
    .meta {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(190px, 1fr)); gap: 8px; }}
    .pill {{ border: 1px solid #d8dee7; background: #fff; border-radius: 6px; padding: 8px 10px; min-width: 0; overflow-wrap: anywhere; }}
    .toolbar {{ position: sticky; top: 0; z-index: 2; display: grid; grid-template-columns: minmax(220px, 1fr) 180px 170px; gap: 10px; align-items: end; padding: 10px; border: 1px solid #cad2dd; background: #ffffff; border-radius: 6px; box-shadow: 0 1px 0 rgba(31,41,51,.04); }}
    label {{ display: grid; gap: 4px; font-size: 12px; font-weight: 650; color: #4a5565; }}
    input, select {{ width: 100%; min-height: 34px; border: 1px solid #b9c3d0; border-radius: 4px; padding: 6px 8px; font: inherit; background: #fff; }}
    .check {{ display: flex; gap: 8px; align-items: center; min-height: 34px; }}
    .check input {{ width: 16px; min-height: 16px; }}
    .table-wrap {{ overflow-x: auto; border: 1px solid #d8dee7; background: #fff; }}
    table {{ width: 100%; min-width: 760px; border-collapse: collapse; background: #fff; }}
    th, td {{ text-align: left; vertical-align: top; border-bottom: 1px solid #e6eaf0; padding: 8px 10px; overflow-wrap: anywhere; }}
    th {{ background: #edf1f5; font-weight: 650; white-space: nowrap; }}
    code {{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 12px; }}
    ul {{ background: #fff; border: 1px solid #d8dee7; margin: 0; padding: 10px 10px 10px 28px; }}
    li {{ margin: 4px 0; overflow-wrap: anywhere; }}
    li span {{ margin-left: 8px; }}
    .badge {{ display: inline-flex; align-items: center; min-height: 20px; margin: 0 4px 4px 0; padding: 2px 6px; border: 1px solid #c7d2df; border-radius: 999px; background: #f4f6f8; font-size: 12px; white-space: nowrap; }}
    .hidden-by-filter {{ display: none; }}
    .empty-state {{ display: none; padding: 10px; border: 1px solid #d8dee7; background: #fff; }}
    @media (max-width: 720px) {{
      main {{ padding: 18px 12px 32px; }}
      h1 {{ font-size: 24px; }}
      .toolbar {{ position: static; grid-template-columns: 1fr; }}
      .meta {{ grid-template-columns: 1fr; }}
      table {{ min-width: 680px; }}
    }}
  </style>
</head>
<body data-inspector-source-free="true">
<main>
  <header>
    <h1>ctxpack Pack Inspector</h1>
    <div class="meta">
      <div class="pill"><strong>Pack</strong><br><code>{}</code></div>
      <div class="pill"><strong>Task</strong><br><code>{}</code></div>
      <div class="pill"><strong>Repo</strong><br><code>{}</code></div>
      <div class="pill"><strong>Agent</strong><br><code>{}</code></div>
      <div class="pill"><strong>Budget</strong><br>{:?}</div>
      <div class="pill"><strong>Confidence</strong><br>{:.2}</div>
      <div class="pill"><strong>Source text logged</strong><br>{}</div>
      <div class="pill"><strong>Source-bearing sections</strong><br>{}</div>
    </div>
  </header>
  <section class="toolbar" aria-label="Inspector filters">
    <label>Filter text<input id="filterText" type="search" placeholder="path, reason, signal, warning"></label>
    <label>Category<select id="kindFilter"><option value="all">All categories</option><option value="target">Targets</option><option value="test">Tests</option><option value="candidate">Candidates</option><option value="memory">Memory</option><option value="warning">Warnings</option><option value="diagnostic">Diagnostics</option><option value="section">Sections</option><option value="command">Commands</option></select></label>
    <label class="check"><input id="sourceOnly" type="checkbox">Source-bearing only</label>
  </section>
  <h2>Sections</h2>
  <div class="table-wrap"><table><thead><tr><th>Title</th><th>Kind</th><th>Content</th><th>Source-bearing</th><th>Tokens</th></tr></thead><tbody>{}</tbody></table></div>
  <h2>Target Files</h2>
  <div class="table-wrap"><table><thead><tr><th>Path</th><th>Lines</th><th>Confidence</th><th>Reason</th><th>Evidence</th></tr></thead><tbody>{}</tbody></table></div>
  <h2>Related Tests</h2>
  <div class="table-wrap"><table><thead><tr><th>Path</th><th>Confidence</th><th>Reason</th><th>Command</th><th>Evidence</th></tr></thead><tbody>{}</tbody></table></div>
  <h2>Validation Commands</h2>
  <ul>{}</ul>
  <h2>Warnings</h2>
  <ul>{}</ul>
  <h2>Diagnostics</h2>
  <div class="table-wrap"><table><thead><tr><th>Code</th><th>Severity</th><th>Message</th><th>Count</th></tr></thead><tbody>{}</tbody></table></div>
  <h2>Retrieval Candidates</h2>
  <div class="table-wrap"><table><thead><tr><th>ID</th><th>Kind</th><th>Path</th><th>Confidence</th><th>Reason</th><th>Signals</th></tr></thead><tbody>{}</tbody></table></div>
  <h2>Selected Memory</h2>
  <div class="table-wrap"><table><thead><tr><th>ID</th><th>Title</th><th>Kind</th><th>Freshness</th><th>Score</th><th>Reason</th></tr></thead><tbody>{}</tbody></table></div>
  <p id="emptyState" class="empty-state">No inspector rows match the current filters.</p>
</main>
<script>
(() => {{
  const textInput = document.getElementById('filterText');
  const kindFilter = document.getElementById('kindFilter');
  const sourceOnly = document.getElementById('sourceOnly');
  const emptyState = document.getElementById('emptyState');
  const rows = Array.from(document.querySelectorAll('.inspect-row'));
  function applyFilters() {{
    const needle = textInput.value.trim().toLowerCase();
    const kind = kindFilter.value;
    const requireSource = sourceOnly.checked;
    let visible = 0;
    for (const row of rows) {{
      const rowKind = row.dataset.kind || '';
      const text = row.dataset.filterText || '';
      const sourceBearing = row.dataset.sourceBearing === 'true';
      const matchesText = !needle || text.toLowerCase().includes(needle);
      const matchesKind = kind === 'all' || rowKind === kind;
      const matchesSource = !requireSource || sourceBearing;
      const show = matchesText && matchesKind && matchesSource;
      row.classList.toggle('hidden-by-filter', !show);
      if (show) visible += 1;
    }}
    emptyState.style.display = visible === 0 ? 'block' : 'none';
  }}
  textInput.addEventListener('input', applyFilters);
  kindFilter.addEventListener('change', applyFilters);
  sourceOnly.addEventListener('change', applyFilters);
  applyFilters();
}})();
</script>
</body>
</html>
"#,
        view.pack_id,
        view.task_id,
        escape_html(&view.repo_id),
        escape_html(&view.target_agent),
        view.budget,
        view.confidence,
        view.source_text_logged,
        view.source_bearing_section_count,
        sections,
        targets,
        tests,
        commands,
        warnings,
        diagnostics,
        candidates,
        memory
    )
}

fn render_signal_badges_from_evidence(evidence: &[ctxpack_core::RetrievalEvidence]) -> String {
    if evidence.is_empty() {
        return "<span class=\"badge\">no attribution</span>".to_string();
    }
    evidence
        .iter()
        .take(8)
        .map(|item| {
            format!(
                "<span class=\"badge\">{:?} {:.2}</span>",
                item.signal, item.score
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_signal_badges_from_scores(scores: &[ctxpack_core::RetrievalSignalScore]) -> String {
    if scores.is_empty() {
        return "<span class=\"badge\">no scores</span>".to_string();
    }
    scores
        .iter()
        .take(8)
        .map(|item| {
            format!(
                "<span class=\"badge\">{:?} {:.2} x {:.2}</span>",
                item.signal, item.score, item.weight
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn is_source_bearing_section(kind: &str) -> bool {
    matches!(kind, "target_snippets" | "test_snippets")
}

fn inspector_candidate_id(index: usize, path: Option<&str>, reason_code: &str) -> String {
    let anchor = path.unwrap_or("candidate");
    format!("{index}:{anchor}:{reason_code}")
}

fn estimate_section_tokens(section: &PackSection) -> usize {
    let words =
        section.title.split_whitespace().count() + section.content.split_whitespace().count();
    words.saturating_mul(4).div_ceil(3)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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

fn render_context_areas(plan: &ContextPlan) -> String {
    let mut output = String::from(
        "Use these source-free area hints for progressive native reads when the target-file list is too narrow for the task. Inspect representative paths from zero-selected areas first, then ask for a deeper pack only if those reads do not explain the change.\n\n",
    );
    let zero_selected = plan
        .context_areas
        .iter()
        .filter(|area| area.selected_count == 0)
        .collect::<Vec<_>>();
    if !zero_selected.is_empty() {
        output.push_str("Zero-selected areas to inspect next:\n");
        output.push_str(
            &zero_selected
                .iter()
                .take(6)
                .map(|area| {
                    let representatives = if area.representative_paths.is_empty() {
                        "no representative paths".to_string()
                    } else {
                        area.representative_paths
                            .iter()
                            .map(|path| format!("`{path}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    format!("- `{}`: {}", area.area, representatives)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        output.push_str("\n\nAll surfaced areas:\n");
    }

    output.push_str(
        &plan
            .context_areas
            .iter()
            .map(|area| {
                let representatives = if area.representative_paths.is_empty() {
                    "none".to_string()
                } else {
                    area.representative_paths
                        .iter()
                        .map(|path| format!("`{path}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                format!(
                    "- `{}`: {} Representative paths: {}",
                    area.area, area.reason, representatives
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );
    output
}

fn render_selected_memory(plan: &ContextPlan, limit: usize) -> String {
    plan.selected_memory
        .iter()
        .take(limit)
        .map(|memory| {
            let links = if memory.card.source_links.is_empty() {
                "none".to_string()
            } else {
                memory
                    .card
                    .source_links
                    .iter()
                    .take(6)
                    .map(|link| format!("`{link}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            format!(
                "- `{}` ({:?}, score {:.2})\n  - Summary: {}\n  - Reason: {}\n  - Source links: {}",
                memory.card.title,
                memory.card.kind,
                memory.score,
                memory.card.summary,
                memory.reason,
                links
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn memory_limit(budget: &PackBudget) -> usize {
    match budget {
        PackBudget::Brief => 1,
        PackBudget::Standard => 2,
        PackBudget::Deep => 3,
    }
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
