use crate::ranking::{rank_candidates, select_ranked_candidates, AnchorCandidate, RankingInput};
use ctxpack_core::{
    ContextPlan, Diagnostic, DiagnosticSeverity, FileRole, MemoryCard, MemoryFreshness,
    MemoryReviewStatus, PackBudget, PackOption, PrivacyStatus, RetrievalCandidate,
    RetrievalCandidateKind, RetrievalEvidence, RetrievalSignalKind, RetrievalSignalScore, RiskFlag,
    SelectedMemory, TargetFile, TaskType,
};
use ctxpack_index::{
    co_change_hints_report, current_diff_summary_report, lexical_search_report, list_memory_cards,
    load_or_refresh_inventory, related_dependency_edges_report, related_tests_report,
    semantic_search_report, symbol_search_report, CoChangeOptions, CurrentDiffOptions,
    DependencyOptions, InventoryError, InventoryOptions, SearchOptions, SemanticOptions,
    StoreConfig, SymbolOptions,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use uuid::Uuid;

pub(crate) const PREPARE_TASK_TARGET_LIMIT: usize = 8;

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
    prepare_context_plan_with_paths_and_history(repo_root, task, task_type, anchor_paths, true)
}

pub fn prepare_context_plan_with_paths_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    semantic_enabled: bool,
) -> Result<ContextPlan, InventoryError> {
    prepare_context_plan_with_paths_history_and_semantic(
        repo_root,
        task,
        task_type,
        anchor_paths,
        true,
        semantic_enabled,
    )
}

pub(crate) fn prepare_context_plan_with_paths_and_history(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    include_history: bool,
) -> Result<ContextPlan, InventoryError> {
    prepare_context_plan_with_paths_history_and_semantic(
        repo_root,
        task,
        task_type,
        anchor_paths,
        include_history,
        false,
    )
}

pub(crate) fn prepare_context_plan_with_paths_history_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    include_history: bool,
    semantic_enabled: bool,
) -> Result<ContextPlan, InventoryError> {
    let repo_root = repo_root.as_ref();
    let mut plan = base_plan(task_type);
    let task = task.trim();
    if task.is_empty() {
        plan.missing_info_questions
            .push("Describe the code task or failure to prepare repository context.".to_string());
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "low_information_task".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: "Task description is too sparse to prepare strong repository context."
                    .to_string(),
                paths: Vec::new(),
                count: 0,
            },
        );
        return Ok(plan);
    }

    if is_low_information_task(task) {
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "low_information_task".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: "Task description has low information; provide a file path, symbol, stack trace, or error text for stronger context.".to_string(),
                paths: Vec::new(),
                count: 0,
            },
        );
        plan.missing_info_questions.push(
            "Provide a file path, symbol name, stack trace, or concrete error text for stronger context.".to_string(),
        );
    }

    let (mut roles, inventory_diagnostics) = inventory_roles(repo_root)?;
    extend_plan_diagnostics(&mut plan, inventory_diagnostics);

    let symbol_report = symbol_search_report(
        repo_root,
        task,
        &SymbolOptions {
            limit: search_limit(&plan.task_type),
        },
    )?;
    extend_plan_diagnostics(&mut plan, symbol_report.diagnostics);
    let symbol_results = symbol_report.results;

    let search_report = lexical_search_report(
        repo_root,
        task,
        &SearchOptions {
            limit: search_limit(&plan.task_type),
        },
    )?;
    extend_plan_diagnostics(&mut plan, search_report.diagnostics);
    let search_results = search_report.results;
    for result in &search_results {
        roles.insert(result.path.clone(), result.role.clone());
    }
    let semantic_results = if semantic_enabled {
        let semantic_report = semantic_search_report(
            repo_root,
            task,
            &SemanticOptions {
                limit: search_limit(&plan.task_type),
                enabled: true,
                ..SemanticOptions::default()
            },
        )?;
        extend_plan_diagnostics(&mut plan, semantic_report.diagnostics);
        for result in &semantic_report.results {
            roles.insert(result.path.clone(), result.role.clone());
        }
        semantic_report.results
    } else {
        Vec::new()
    };
    let (anchor_targets, unavailable_anchors, anchor_diagnostics) =
        anchored_target_files(repo_root, anchor_paths)?;
    extend_plan_diagnostics(&mut plan, anchor_diagnostics);
    for unavailable in unavailable_anchors {
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "anchor_unavailable".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "Active context path `{unavailable}` was not included because it is ignored, generated, sensitive, outside the repo, or not inventoried."
                ),
                paths: vec![unavailable],
                count: 1,
            },
        );
    }
    let current_diff_paths = current_diff_anchor_paths(repo_root, &anchor_targets);
    let anchors = anchor_targets
        .into_iter()
        .map(|(target, role)| {
            roles.insert(target.path.clone(), role.clone());
            AnchorCandidate {
                current_diff: current_diff_paths.contains(&target.path),
                path: target.path,
                role,
            }
        })
        .collect::<Vec<_>>();
    let expansion_seed_paths = expansion_seed_paths(
        &anchors,
        &symbol_results,
        &search_results,
        &semantic_results,
    );
    let source_seed_paths = expansion_seed_paths
        .iter()
        .filter(|path| {
            roles
                .get(*path)
                .is_none_or(|role| matches!(role, FileRole::Source))
        })
        .cloned()
        .collect::<Vec<_>>();

    let test_report = related_tests_report(repo_root, &source_seed_paths)?;
    extend_plan_diagnostics(&mut plan, test_report.diagnostics);
    let test_results = test_report.results;

    let mut has_history = false;
    let mut co_change_hints = Vec::new();
    if include_history {
        let co_change_report = co_change_hints_report(
            repo_root,
            &source_seed_paths,
            &CoChangeOptions {
                limit: co_change_limit(&plan.task_type),
            },
        )?;
        let history_unavailable = co_change_report.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.code.as_str(),
                "git_missing" | "git_timeout" | "git_unavailable" | "history_partial"
            )
        });
        extend_plan_diagnostics(&mut plan, co_change_report.diagnostics);
        has_history = !co_change_report.hints.is_empty();
        if history_unavailable && !has_history {
            plan.risk_flags.push(RiskFlag {
                code: "co_change_unavailable".to_string(),
                message:
                    "Local git co-change hints were unavailable; continuing without history signal."
                        .to_string(),
            });
        }
        co_change_hints = co_change_report.hints;
        for hint in co_change_hints.iter().take(5) {
            plan.risk_flags.push(RiskFlag {
                code: "co_change_hint".to_string(),
                message: format!(
                    "{} changed with target files in {} local commit(s): {}",
                    hint.path, hint.commit_count, hint.reason
                ),
            });
        }
    }

    let dependency_report = related_dependency_edges_report(
        repo_root,
        &source_seed_paths,
        &DependencyOptions {
            limit: co_change_limit(&plan.task_type),
        },
    )?;
    extend_plan_diagnostics(&mut plan, dependency_report.diagnostics);
    let has_graph = !dependency_report.edges.is_empty();
    let source_targets = source_seed_paths.iter().cloned().collect::<BTreeSet<_>>();
    let dependency_edges = dependency_report.edges;
    for edge in dependency_edges.iter().take(5) {
        let direction = if source_targets.contains(&edge.source_path) {
            "target imports"
        } else {
            "imports target"
        };
        plan.risk_flags.push(RiskFlag {
            code: "dependency_edge".to_string(),
            message: format!(
                "{} `{}` -> `{}`: {}",
                direction, edge.source_path, edge.target_path, edge.reason
            ),
        });
    }

    let ranked_candidates = rank_candidates(RankingInput {
        anchors,
        lexical_results: search_results,
        semantic_results,
        symbol_results,
        related_tests: test_results,
        co_change_hints,
        dependency_edges,
        roles,
        expansion_seeds: expansion_seed_paths,
    });
    let selection = select_ranked_candidates(&ranked_candidates, PREPARE_TASK_TARGET_LIMIT, 5);
    plan.target_files = selection.target_files;
    plan.related_tests = selection.related_tests;
    plan.recommended_commands = selection.recommended_commands;
    plan.retrieval_candidates = selection.retrieval_candidates;
    attach_selected_memory(repo_root, task, &mut plan);

    if plan.target_files.is_empty() {
        plan.missing_info_questions.push(
            "No safe inventoried files matched the task. Provide a file path, symbol, or error text."
                .to_string(),
        );
    }

    plan.confidence = plan_confidence(
        !plan.target_files.is_empty(),
        !plan.related_tests.is_empty(),
        has_history,
        has_graph,
    );

    Ok(plan)
}

fn extend_plan_diagnostics(plan: &mut ContextPlan, diagnostics: Vec<Diagnostic>) {
    for diagnostic in diagnostics {
        push_plan_diagnostic(plan, diagnostic);
    }
}

fn push_plan_diagnostic(plan: &mut ContextPlan, diagnostic: Diagnostic) {
    let risk_flag = if matches!(
        diagnostic.severity,
        DiagnosticSeverity::Warning | DiagnosticSeverity::Error
    ) {
        Some(RiskFlag {
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
        })
    } else {
        None
    };

    let duplicate_diagnostic = plan.diagnostics.iter().any(|existing| {
        existing.code == diagnostic.code
            && existing.message == diagnostic.message
            && existing.paths == diagnostic.paths
    });
    if !duplicate_diagnostic {
        plan.diagnostics.push(diagnostic);
    }

    if let Some(risk_flag) = risk_flag {
        let duplicate_risk = plan.risk_flags.iter().any(|existing| {
            existing.code == risk_flag.code && existing.message == risk_flag.message
        });
        if !duplicate_risk {
            plan.risk_flags.push(risk_flag);
        }
    }
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
            PackOption {
                budget: PackBudget::Deep,
                resource_uri: format!("ctxpack://pack/{task_id}/deep"),
            },
        ],
        missing_info_questions: Vec::new(),
        risk_flags: Vec::new(),
        diagnostics: Vec::new(),
        retrieval_candidates: Vec::new(),
        selected_memory: Vec::new(),
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn attach_selected_memory(repo_root: &Path, task: &str, plan: &mut ContextPlan) {
    let cards = match list_memory_cards(repo_root, &StoreConfig::default(), false) {
        Ok(cards) => cards,
        Err(error) => {
            push_plan_diagnostic(
                plan,
                Diagnostic {
                    code: "memory_unavailable".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: format!("Local memory cards were unavailable: {error}"),
                    paths: Vec::new(),
                    count: 0,
                },
            );
            return;
        }
    };
    if cards.is_empty() {
        return;
    }

    let mut blocked = 0usize;
    let mut selected = cards
        .iter()
        .filter_map(|card| {
            if !memory_card_pack_eligible(card) {
                blocked += 1;
                return None;
            }
            score_memory_card(task, plan, card)
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.card.id.cmp(&right.card.id))
    });
    selected.truncate(3);
    if !selected.is_empty() {
        for memory in &selected {
            plan.retrieval_candidates.push(RetrievalCandidate {
                kind: RetrievalCandidateKind::Memory,
                path: memory.card.source_links.first().cloned(),
                role: None,
                reason_code: "selected_memory".to_string(),
                confidence: memory.score,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::Memory,
                    score: memory.score,
                    weight: 0.25,
                }],
                evidence: memory.evidence.clone(),
            });
        }
        plan.selected_memory = selected;
    }
    if blocked > 0 {
        push_plan_diagnostic(
            plan,
            Diagnostic {
                code: "memory_cards_blocked".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "{blocked} memory card(s) were stale, degraded, disabled, rejected, or pending review and were not selected."
                ),
                paths: Vec::new(),
                count: blocked,
            },
        );
    }
}

fn memory_card_pack_eligible(card: &MemoryCard) -> bool {
    !card.disabled
        && matches!(card.freshness, MemoryFreshness::Fresh)
        && matches!(
            card.review_status,
            MemoryReviewStatus::Deterministic | MemoryReviewStatus::Approved
        )
}

fn score_memory_card(task: &str, plan: &ContextPlan, card: &MemoryCard) -> Option<SelectedMemory> {
    let task_terms = terms(task);
    let mut score = 0.0f32;
    let mut evidence = Vec::new();
    for link in &card.source_links {
        if plan.target_files.iter().any(|target| target.path == *link)
            || plan.related_tests.iter().any(|test| test.path == *link)
        {
            score += 0.55;
            evidence.push(RetrievalEvidence {
                signal: RetrievalSignalKind::Memory,
                score: 0.55,
                reason_code: "memory_source_link_selected".to_string(),
                path: Some(link.clone()),
                role: None,
                edge_label: Some("source_link".to_string()),
                commit_ids: Vec::new(),
                commit_count: 0,
            });
            break;
        }
    }
    let haystack = format!(
        "{} {} {}",
        card.title,
        card.summary,
        card.source_links.join(" ")
    )
    .to_ascii_lowercase();
    let overlap = task_terms
        .iter()
        .filter(|term| haystack.contains(term.as_str()))
        .count();
    if overlap > 0 {
        let overlap_score = (overlap as f32 * 0.12).min(0.36);
        score += overlap_score;
        evidence.push(RetrievalEvidence {
            signal: RetrievalSignalKind::Memory,
            score: overlap_score,
            reason_code: "memory_task_overlap".to_string(),
            path: card.source_links.first().cloned(),
            role: None,
            edge_label: Some("task_overlap".to_string()),
            commit_ids: Vec::new(),
            commit_count: 0,
        });
    }
    if score <= 0.0 {
        return None;
    }
    let score = score.min(1.0) * card.confidence.max(0.1);
    Some(SelectedMemory {
        card: card.clone(),
        score,
        reason: "Selected from fresh local memory with task or target-file overlap.".to_string(),
        evidence,
    })
}

fn terms(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| term.len() >= 3)
        .collect()
}

pub(crate) fn normalized_target_agent(target_agent: &str) -> String {
    let target_agent = target_agent.trim();
    if target_agent.is_empty() {
        "generic".to_string()
    } else {
        target_agent.to_string()
    }
}

pub(crate) fn is_low_information_task(task: &str) -> bool {
    let information_score = task
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter_map(|term| {
            let term = term.trim();
            let normalized = term.to_ascii_lowercase();
            if normalized.len() < 3
                || normalized
                    .chars()
                    .all(|character| character.is_ascii_digit())
            {
                return None;
            }
            if matches!(
                normalized.as_str(),
                "fix"
                    | "fixes"
                    | "fixed"
                    | "close"
                    | "closes"
                    | "closed"
                    | "issue"
                    | "bug"
                    | "for"
                    | "the"
                    | "and"
                    | "with"
                    | "from"
            ) {
                return None;
            }
            let has_lower = term.chars().any(|character| character.is_ascii_lowercase());
            let has_upper = term.chars().any(|character| character.is_ascii_uppercase());
            let identifier_like = term.contains('_') || has_lower && has_upper;
            Some(if identifier_like { 2 } else { 1 })
        })
        .sum::<usize>();

    information_score < 2
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
        TaskType::BugFix | TaskType::Refactor | TaskType::Review => 30,
        TaskType::Feature | TaskType::Test | TaskType::Explain => 5,
    }
}

pub(crate) fn plan_confidence(
    has_targets: bool,
    has_tests: bool,
    has_history: bool,
    has_graph: bool,
) -> f32 {
    let mut confidence: f32 = if has_targets { 0.45 } else { 0.05 };
    if has_tests {
        confidence += 0.25;
    }
    if has_history {
        confidence += 0.15;
    }
    if has_graph {
        confidence += 0.10;
    }
    confidence.min(0.95)
}

fn inventory_roles(
    repo_root: &Path,
) -> Result<(BTreeMap<String, FileRole>, Vec<Diagnostic>), InventoryError> {
    let report = load_or_refresh_inventory(repo_root, &InventoryOptions::default())?;
    let roles = report
        .inventory
        .files
        .into_iter()
        .filter(|file| !file.ignored && !file.generated && file.role != FileRole::Sensitive)
        .map(|file| (file.path, file.role))
        .collect::<BTreeMap<_, _>>();
    Ok((roles, report.diagnostics))
}

fn expansion_seed_paths(
    anchors: &[AnchorCandidate],
    symbol_results: &[ctxpack_index::SymbolSearchResult],
    search_results: &[ctxpack_index::SearchResult],
    semantic_results: &[ctxpack_index::SemanticSearchResult],
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut paths = Vec::new();
    for path in anchors
        .iter()
        .map(|anchor| anchor.path.clone())
        .chain(
            symbol_results
                .iter()
                .map(|result| result.symbol.path.clone()),
        )
        .chain(search_results.iter().map(|result| result.path.clone()))
        .chain(semantic_results.iter().map(|result| result.path.clone()))
    {
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }
    paths
}

fn current_diff_anchor_paths(repo_root: &Path, anchors: &[AnchoredTarget]) -> BTreeSet<String> {
    if anchors.is_empty() {
        return BTreeSet::new();
    }
    let Ok(report) = current_diff_summary_report(
        repo_root,
        &CurrentDiffOptions {
            include_untracked: true,
        },
    ) else {
        return BTreeSet::new();
    };
    report
        .summary
        .staged
        .into_iter()
        .chain(report.summary.unstaged)
        .chain(report.summary.untracked)
        .collect()
}

type AnchoredTarget = (TargetFile, FileRole);

fn anchored_target_files(
    repo_root: &Path,
    anchor_paths: &[String],
) -> Result<(Vec<AnchoredTarget>, Vec<String>, Vec<Diagnostic>), InventoryError> {
    if anchor_paths.is_empty() {
        return Ok((Vec::new(), Vec::new(), Vec::new()));
    }

    let inventory_report = load_or_refresh_inventory(repo_root, &InventoryOptions::default())?;
    let files_by_path = inventory_report
        .inventory
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
                    attribution: Vec::new(),
                },
                role.clone(),
            ));
        }
    }

    Ok((targets, unavailable, inventory_report.diagnostics))
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
