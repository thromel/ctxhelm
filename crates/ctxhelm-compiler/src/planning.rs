use crate::policy::{
    provider_policy_report, query_family_routed_reranker_enabled_for_family, reranker_decision,
    semantic_provider_decision,
};
use crate::ranking::{
    rank_candidates, rerank_with_local_metadata, select_ranked_candidates_for_scope,
    AnchorCandidate, MemoryPathCandidate, RankingInput,
};
use ctxhelm_core::{
    context_area_for_path, context_area_resource_uri, Command, ContextArea, ContextPlan,
    Diagnostic, DiagnosticSeverity, FileRole, FusionControlSummary, InspectionPressureBreakdown,
    MemoryCard, MemoryFreshness, MemoryReviewStatus, PackBudget, PackOption, PrivacyStatus,
    ProviderDecisionStatus, QueryConstructionTrace, QueryFacet, QueryFacetKind, RetrievalCandidate,
    RetrievalCandidateKind, RetrievalEvidence, RetrievalSignalKind, RetrievalSignalScore,
    RetrieverQuerySet, RiskFlag, SelectedMemory, TargetFile, TaskType,
};
use ctxhelm_index::{
    co_change_hints_report, current_diff_summary_report, lexical_search_report, list_memory_cards,
    load_or_refresh_inventory, mentioned_commit_changed_paths_report, normalized_provider,
    related_dependency_edges_report, related_tests_report, semantic_search_report,
    symbol_search_report, task_hash, test_map_report, CoChangeOptions, CurrentDiffOptions,
    DependencyOptions, InventoryError, InventoryOptions, RelatedTestResult, SearchOptions,
    SearchResult, SemanticOptions, SemanticProviderConfig, StoreConfig, SymbolOptions,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use uuid::Uuid;

pub(crate) const PREPARE_TASK_TARGET_LIMIT: usize = 10;
pub(crate) const PREPARE_TASK_TEST_LIMIT: usize = 10;
const SELECTED_MEMORY_INITIAL_READ_LIMIT: usize = 3;
const SEMANTIC_CANDIDATE_PATH_HINT_PATH_LIMIT: usize = 8;
const SEMANTIC_CANDIDATE_PATH_HINT_TERM_LIMIT: usize = 12;
const SEMANTIC_SIBLING_PATH_HINT_PATH_LIMIT: usize = 8;
const SEMANTIC_SIBLING_PATH_HINT_TERM_LIMIT: usize = 4;
const SEMANTIC_SIBLING_PATH_HINTS_PER_PATH_LIMIT: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryMode {
    Disabled,
    Full,
}

#[derive(Debug, Clone)]
pub(crate) struct PlannerSemanticOptions {
    pub enabled: bool,
    pub provider: SemanticProviderConfig,
    pub query_source_role_hints: bool,
    pub query_candidate_path_hints: bool,
    pub query_candidate_sibling_path_hints: bool,
}

impl PlannerSemanticOptions {
    fn plain(enabled: bool, provider: SemanticProviderConfig) -> Self {
        Self {
            enabled,
            provider,
            query_source_role_hints: false,
            query_candidate_path_hints: false,
            query_candidate_sibling_path_hints: false,
        }
    }
}

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
    prepare_context_plan_with_paths_and_semantic_provider(
        repo_root,
        task,
        task_type,
        anchor_paths,
        semantic_enabled,
        SemanticProviderConfig::default(),
    )
}

pub fn prepare_context_plan_with_paths_and_semantic_provider(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    semantic_enabled: bool,
    semantic_provider: SemanticProviderConfig,
) -> Result<ContextPlan, InventoryError> {
    prepare_context_plan_with_paths_history_and_semantic(
        repo_root,
        task,
        task_type,
        anchor_paths,
        true,
        PlannerSemanticOptions::plain(semantic_enabled, semantic_provider),
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
        PlannerSemanticOptions::plain(false, SemanticProviderConfig::default()),
    )
}

pub(crate) fn prepare_context_plan_with_paths_history_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    include_history: bool,
    semantic: PlannerSemanticOptions,
) -> Result<ContextPlan, InventoryError> {
    let history_mode = if include_history {
        HistoryMode::Full
    } else {
        HistoryMode::Disabled
    };
    prepare_context_plan_with_paths_history_mode_and_semantic(
        repo_root,
        task,
        task_type,
        anchor_paths,
        history_mode,
        semantic,
    )
}

pub(crate) fn prepare_context_plan_with_paths_history_mode_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    history_mode: HistoryMode,
    semantic: PlannerSemanticOptions,
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
    let multi_area_task = is_multi_area_task(task);
    if multi_area_task {
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "multi_area_task".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: "Task appears to span multiple implementation or validation areas; inspect the suggested files as a scoped starting point and run broad validation when available.".to_string(),
                paths: Vec::new(),
                count: 0,
            },
        );
        plan.risk_flags.push(RiskFlag {
            code: "multi_area_task".to_string(),
            message: "Prompt has broad workflow/eval/lint-style terms, so no small context pack can cover every changed area.".to_string(),
        });
    }

    let (mut roles, inventory_diagnostics) = inventory_roles(repo_root)?;
    extend_plan_diagnostics(&mut plan, inventory_diagnostics);
    let memory_cards = load_memory_cards_for_plan(repo_root, &mut plan);
    let memory_paths = memory_path_candidates(task, &memory_cards, &roles);
    let mut provider_policy = provider_policy_report(repo_root)?;
    extend_plan_diagnostics(&mut plan, provider_policy.diagnostics.clone());
    let mut query_trace = construct_query_trace(task, anchor_paths);
    let mut combined_anchor_paths = anchor_paths.to_vec();
    let mut mentioned_commit_path_set = BTreeSet::new();
    let mentioned_commit_paths =
        match mentioned_commit_changed_path_anchors(repo_root, task, &mut plan) {
            Ok(paths) => paths,
            Err(error) => {
                push_plan_diagnostic(
                    &mut plan,
                    Diagnostic {
                        code: "mentioned_commit_changed_paths_unavailable".to_string(),
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "Task-mentioned commit changed-path signal was unavailable: {error}."
                        ),
                        paths: Vec::new(),
                        count: 0,
                    },
                );
                Vec::new()
            }
        };
    for path in mentioned_commit_paths {
        push_query_facet(
            &mut query_trace.facets,
            QueryFacetKind::ExplicitPath,
            path.clone(),
            "mentioned_commit_changed_path".to_string(),
            1.0,
        );
        push_unique(
            &mut query_trace.retriever_queries.lexical_terms,
            path.clone(),
        );
        push_unique(&mut query_trace.retriever_queries.graph_seeds, path.clone());
        push_unique(&mut query_trace.retriever_queries.test_terms, path.clone());
        mentioned_commit_path_set.insert(path.clone());
        push_unique(&mut combined_anchor_paths, path);
    }
    for path in query_trace
        .facets
        .iter()
        .filter(|facet| {
            matches!(
                facet.kind,
                QueryFacetKind::ExplicitPath | QueryFacetKind::StackFrame
            )
        })
        .map(|facet| facet.value.clone())
    {
        if !combined_anchor_paths.contains(&path) {
            combined_anchor_paths.push(path);
        }
    }
    let lexical_query = query_text_or_task(&query_trace.retriever_queries.lexical_terms, task);
    let mut semantic_query = semantic_query_text_or_task(
        &query_trace.retriever_queries.semantic_phrases,
        task,
        &plan.task_type,
        &roles,
        semantic.query_source_role_hints,
    );
    if semantic.query_source_role_hints {
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "semantic_query_source_role_hints_applied".to_string(),
                severity: DiagnosticSeverity::Info,
                message:
                    "Semantic query text included source-free coding role and dominant language hints."
                        .to_string(),
                paths: Vec::new(),
                count: 1,
            },
        );
    }

    let symbol_results = if query_trace.retriever_queries.symbol_terms.is_empty() {
        Vec::new()
    } else {
        let symbol_query = query_trace.retriever_queries.symbol_terms.join(" ");
        let symbol_report = symbol_search_report(
            repo_root,
            &symbol_query,
            &SymbolOptions {
                limit: search_limit(&plan.task_type),
            },
        )?;
        extend_plan_diagnostics(&mut plan, symbol_report.diagnostics);
        symbol_report.results
    };

    let search_report = lexical_search_report(
        repo_root,
        &lexical_query,
        &SearchOptions {
            limit: lexical_candidate_limit(&plan.task_type),
        },
    )?;
    extend_plan_diagnostics(&mut plan, search_report.diagnostics);
    let mut search_results = search_report.results;
    extend_project_governance_docs(&mut search_results, &roles, task);
    for result in &search_results {
        roles.insert(result.path.clone(), result.role.clone());
    }
    if semantic.query_candidate_path_hints || semantic.query_candidate_sibling_path_hints {
        let hint_count = append_semantic_candidate_path_hints(&mut semantic_query, &search_results);
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "semantic_query_candidate_path_hints_applied".to_string(),
                severity: DiagnosticSeverity::Info,
                message:
                    "Semantic query text included bounded source-free aliases from lexical candidate paths."
                        .to_string(),
                paths: Vec::new(),
                count: hint_count,
            },
        );
    }
    if semantic.query_candidate_sibling_path_hints {
        let hint_count = append_semantic_candidate_sibling_path_hints(
            &mut semantic_query,
            &search_results,
            &roles,
        );
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "semantic_query_candidate_sibling_path_hints_applied".to_string(),
                severity: DiagnosticSeverity::Info,
                message: "Semantic query text included bounded source-free aliases from same-directory and mirrored-test inventory paths."
                    .to_string(),
                paths: Vec::new(),
                count: hint_count,
            },
        );
    }
    let semantic_provider = normalized_provider(&semantic.provider);
    let semantic_decision =
        semantic_provider_decision(&provider_policy, &semantic_provider, semantic.enabled);
    provider_policy.decisions.push(semantic_decision.clone());
    let semantic_results = if semantic.enabled
        && matches!(semantic_decision.status, ProviderDecisionStatus::Allowed)
    {
        let semantic_report = semantic_search_report(
            repo_root,
            &semantic_query,
            &SemanticOptions {
                limit: search_limit(&plan.task_type),
                enabled: true,
                provider: semantic_provider.clone(),
            },
        )?;
        extend_plan_diagnostics(&mut plan, semantic_report.diagnostics);
        for result in &semantic_report.results {
            roles.insert(result.path.clone(), result.role.clone());
        }
        semantic_report.results
    } else {
        if semantic.enabled {
            push_plan_diagnostic(
                &mut plan,
                Diagnostic {
                    code: "semantic_provider_policy_blocked".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    message: semantic_decision.reason,
                    paths: Vec::new(),
                    count: 1,
                },
            );
        }
        Vec::new()
    };
    let (anchor_targets, unavailable_anchors, anchor_diagnostics) =
        anchored_target_files(repo_root, &combined_anchor_paths)?;
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
    for path in &current_diff_paths {
        push_query_facet(
            &mut query_trace.facets,
            QueryFacetKind::CurrentDiffPath,
            path.clone(),
            "current_diff".to_string(),
            1.0,
        );
        push_unique(&mut query_trace.retriever_queries.graph_seeds, path.clone());
    }
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

    let mut has_history = false;
    let mut co_change_hints = Vec::new();
    if !matches!(history_mode, HistoryMode::Disabled) {
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

    let test_seed_paths = related_test_seed_paths(
        &source_seed_paths,
        &co_change_hints,
        &dependency_edges,
        &roles,
    );
    let test_report = related_tests_report(repo_root, &test_seed_paths)?;
    extend_plan_diagnostics(&mut plan, test_report.diagnostics);
    let mut test_results = test_report.results;
    let cochanged_test_diagnostics =
        extend_cochanged_test_results(repo_root, &mut test_results, &co_change_hints, &roles)?;
    extend_plan_diagnostics(&mut plan, cochanged_test_diagnostics);

    let ranking_co_change_hints = if matches!(history_mode, HistoryMode::Full) {
        co_change_hints.clone()
    } else {
        Vec::new()
    };

    let broad_target_selection = uses_broad_target_file_floors(task);

    if broad_target_selection {
        extend_broad_source_area_candidates(&mut search_results, &roles);
    }

    let ranked_candidates = rank_candidates(RankingInput {
        anchors,
        lexical_results: search_results,
        protected_lexical_limit: Some(search_limit(&plan.task_type)),
        semantic_results,
        symbol_results,
        related_tests: test_results,
        co_change_hints: ranking_co_change_hints,
        dependency_edges,
        roles: roles.clone(),
        expansion_seeds: expansion_seed_paths,
        memory_paths,
    });
    let reranker = reranker_decision(&provider_policy);
    provider_policy.decisions.push(reranker.clone());
    let query_family = planner_reranker_query_family(&plan.task_type, task, &query_trace);
    let full_reranker_enabled = provider_policy.policy.enable_local_metadata_reranker
        || provider_policy.policy.enable_local_fixture_reranker;
    let routed_reranker_enabled = provider_policy.policy.enable_query_family_routed_reranker
        && query_family_routed_reranker_enabled_for_family(&query_family);
    let ranked_candidates = if matches!(reranker.status, ProviderDecisionStatus::Allowed)
        && (full_reranker_enabled || routed_reranker_enabled)
    {
        let (code, message) = if routed_reranker_enabled && !full_reranker_enabled {
            (
                "query_family_routed_reranker_applied",
                format!(
                    "Applied deterministic local metadata reranker for route-safe query family `{query_family}`."
                ),
            )
        } else {
            (
                "local_metadata_reranker_applied",
                "Applied deterministic local metadata reranker using source-free candidate metadata."
                    .to_string(),
            )
        };
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: code.to_string(),
                severity: DiagnosticSeverity::Info,
                message,
                paths: Vec::new(),
                count: ranked_candidates.len(),
            },
        );
        rerank_with_local_metadata(ranked_candidates)
    } else {
        if matches!(reranker.status, ProviderDecisionStatus::Allowed)
            && provider_policy.policy.enable_query_family_routed_reranker
            && !full_reranker_enabled
        {
            push_plan_diagnostic(
                &mut plan,
                Diagnostic {
                    code: "query_family_routed_reranker_held".to_string(),
                    severity: DiagnosticSeverity::Info,
                    message: format!(
                        "Query-family routed reranker held for unproven query family `{query_family}`."
                    ),
                    paths: Vec::new(),
                    count: ranked_candidates.len(),
                },
            );
        }
        ranked_candidates
    };
    let mut selection = select_ranked_candidates_for_scope(
        &ranked_candidates,
        PREPARE_TASK_TARGET_LIMIT,
        PREPARE_TASK_TEST_LIMIT,
        broad_target_selection,
    );
    if is_commit_replay_task(task) && !mentioned_commit_path_set.is_empty() {
        narrow_selection_to_mentioned_commit_paths(
            &mut selection,
            &mentioned_commit_path_set,
            &mut plan,
        );
    }
    plan.target_files = selection.target_files.clone();
    plan.related_tests = selection.related_tests.clone();
    plan.recommended_commands = selection.recommended_commands.clone();
    plan.retrieval_candidates = selection.retrieval_candidates.clone();
    maybe_add_broad_validation_command(task, &mut plan);
    attach_selected_memory(task, &mut plan, &memory_cards);
    promote_selected_memory_initial_reads(&mut plan, &roles);
    selection.target_files = plan.target_files.clone();
    selection.related_tests = plan.related_tests.clone();
    selection.retrieval_candidates = plan.retrieval_candidates.clone();
    plan.context_areas = context_areas_for_plan(&selection, multi_area_task);
    plan.query_trace = Some(query_trace);
    plan.provider_policy = Some(provider_policy);

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

fn promote_selected_memory_initial_reads(
    plan: &mut ContextPlan,
    roles: &BTreeMap<String, FileRole>,
) {
    if plan.selected_memory.is_empty() {
        return;
    }

    let mut seen = plan
        .target_files
        .iter()
        .map(|target| target.path.clone())
        .collect::<BTreeSet<_>>();
    let mut promoted = Vec::new();

    'memory: for memory in &plan.selected_memory {
        for link in &memory.card.source_links {
            if promoted.len() >= SELECTED_MEMORY_INITIAL_READ_LIMIT {
                break 'memory;
            }
            if seen.contains(link) {
                continue;
            }
            let Some(role) = roles.get(link) else {
                continue;
            };
            if !memory_path_role_is_context(role) {
                continue;
            }
            seen.insert(link.clone());
            promoted.push(TargetFile {
                path: link.clone(),
                reason: "selected memory source link".to_string(),
                line_range: None,
                confidence: selected_memory_initial_read_confidence(memory),
                attribution: selected_memory_initial_read_evidence(memory, link, role),
            });
        }
    }

    if promoted.is_empty() {
        return;
    }

    let insert_at = plan
        .target_files
        .iter()
        .position(|target| !is_active_context_target(target))
        .unwrap_or(plan.target_files.len());
    let promoted_count = promoted.len();
    plan.target_files.splice(insert_at..insert_at, promoted);
    plan.target_files.truncate(PREPARE_TASK_TARGET_LIMIT);
    push_plan_diagnostic(
        plan,
        Diagnostic {
            code: "selected_memory_initial_read_promoted".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Promoted {promoted_count} selected-memory source link(s) into targetFiles for native first-read consumption."
            ),
            paths: plan
                .target_files
                .iter()
                .filter(|target| target.reason == "selected memory source link")
                .map(|target| target.path.clone())
                .collect(),
            count: promoted_count,
        },
    );
}

fn selected_memory_initial_read_evidence(
    memory: &SelectedMemory,
    path: &str,
    role: &FileRole,
) -> Vec<RetrievalEvidence> {
    let mut evidence = memory
        .evidence
        .iter()
        .filter(|evidence| evidence.path.as_deref() == Some(path))
        .cloned()
        .collect::<Vec<_>>();
    if !evidence
        .iter()
        .any(|evidence| evidence.reason_code == "selected_memory_initial_read")
    {
        evidence.push(RetrievalEvidence {
            signal: RetrievalSignalKind::Memory,
            score: selected_memory_initial_read_confidence(memory),
            reason_code: "selected_memory_initial_read".to_string(),
            path: Some(path.to_string()),
            role: Some(role.clone()),
            edge_label: Some(memory.card.id.clone()),
            commit_ids: Vec::new(),
            commit_count: 0,
        });
    }
    evidence
}

fn selected_memory_initial_read_confidence(memory: &SelectedMemory) -> f32 {
    memory.score.max(0.82).clamp(0.05, 0.95)
}

fn is_active_context_target(target: &TargetFile) -> bool {
    target.attribution.iter().any(|evidence| {
        matches!(
            evidence.signal,
            RetrievalSignalKind::Anchor | RetrievalSignalKind::CurrentDiff
        )
    }) || target.reason == "explicit path anchor from active context"
}

fn related_test_seed_paths(
    source_seed_paths: &[String],
    co_change_hints: &[ctxhelm_index::CoChangeHint],
    dependency_edges: &[ctxhelm_index::DependencyEdge],
    roles: &BTreeMap<String, FileRole>,
) -> Vec<String> {
    let mut seed_paths = Vec::new();
    for path in source_seed_paths {
        push_source_test_seed(&mut seed_paths, roles, path);
    }
    for hint in co_change_hints {
        push_source_test_seed(&mut seed_paths, roles, &hint.path);
    }
    for edge in dependency_edges {
        push_source_test_seed(&mut seed_paths, roles, &edge.source_path);
        push_source_test_seed(&mut seed_paths, roles, &edge.target_path);
    }
    seed_paths
}

fn context_areas_for_plan(
    selection: &crate::ranking::RankedSelection,
    multi_area_task: bool,
) -> Vec<ContextArea> {
    let selected_paths = selection
        .target_files
        .iter()
        .map(|target| target.path.as_str())
        .chain(
            selection
                .related_tests
                .iter()
                .map(|test| test.path.as_str()),
        )
        .collect::<BTreeSet<_>>();
    let mut areas = BTreeMap::<String, AreaAccumulator>::new();

    for candidate in &selection.retrieval_candidates {
        let Some(path) = candidate.path.as_deref() else {
            continue;
        };
        if !matches!(
            candidate.role,
            Some(
                FileRole::Source
                    | FileRole::Test
                    | FileRole::Config
                    | FileRole::Schema
                    | FileRole::Docs
            )
        ) {
            continue;
        }
        let area = context_area_for_path(path);
        let entry = areas.entry(area).or_default();
        entry.record_role(&candidate.role);
        entry.record_signals(candidate);
        if entry.unique_paths.insert(path.to_string()) {
            entry.candidate_count += 1;
            if entry.representative_paths.len() < 3 {
                entry.representative_paths.push(path.to_string());
            }
            if selected_paths.contains(path) {
                entry.selected_count += 1;
                entry.record_selected_role(&candidate.role);
            } else {
                entry.record_next_read_candidate(candidate, path);
            }
        }
    }

    let mut areas = areas
        .into_iter()
        .map(|(area, accumulator)| {
            let unselected_count = accumulator
                .candidate_count
                .saturating_sub(accumulator.selected_count);
            let context_area = ContextArea {
                area: area.clone(),
                reason: context_area_reason(&accumulator, multi_area_task),
                resource_uri: context_area_resource_uri(&area),
                representative_paths: accumulator.representative_paths.clone(),
                next_read_paths: accumulator.next_read_paths(),
                signal_counts: accumulator.signal_counts(),
                role_counts: accumulator.role_counts(),
                selected_role_counts: accumulator.selected_role_counts(),
                candidate_count: accumulator.candidate_count,
                selected_count: accumulator.selected_count,
                unselected_count,
                coverage_percent: context_area_coverage_percent(
                    accumulator.selected_count,
                    accumulator.candidate_count,
                ),
                inspection_pressure: context_area_inspection_pressure(&accumulator),
                inspection_pressure_breakdown: context_area_inspection_pressure_breakdown(
                    &accumulator,
                ),
            };
            RankedContextArea {
                context_area,
                accumulator,
            }
        })
        .collect::<Vec<_>>();
    if !multi_area_task {
        areas.retain(|area| {
            area.accumulator.selected_source_like_count() > 0
                && area.accumulator.source_like_count()
                    > area.accumulator.selected_source_like_count()
        });
    }
    areas.sort_by(|left, right| {
        context_area_priority(&left.context_area.area, &left.accumulator, multi_area_task)
            .cmp(&context_area_priority(
                &right.context_area.area,
                &right.accumulator,
                multi_area_task,
            ))
            .then_with(|| {
                right
                    .accumulator
                    .selected_source_like_count()
                    .cmp(&left.accumulator.selected_source_like_count())
            })
            .then_with(|| {
                right
                    .accumulator
                    .source_like_count()
                    .cmp(&left.accumulator.source_like_count())
            })
            .then_with(|| {
                right
                    .context_area
                    .selected_count
                    .cmp(&left.context_area.selected_count)
            })
            .then_with(|| {
                right
                    .context_area
                    .candidate_count
                    .cmp(&left.context_area.candidate_count)
            })
            .then_with(|| {
                right
                    .context_area
                    .inspection_pressure
                    .cmp(&left.context_area.inspection_pressure)
            })
            .then_with(|| left.context_area.area.cmp(&right.context_area.area))
    });
    let mut areas = areas
        .into_iter()
        .map(|area| area.context_area)
        .collect::<Vec<_>>();
    areas.truncate(MAX_CONTEXT_AREAS);
    areas
}

fn context_area_coverage_percent(selected_count: usize, candidate_count: usize) -> u8 {
    if candidate_count == 0 {
        return 0;
    }
    let percent = selected_count.saturating_mul(100) / candidate_count;
    percent.min(100) as u8
}

fn context_area_inspection_pressure(accumulator: &AreaAccumulator) -> usize {
    context_area_inspection_pressure_breakdown(accumulator).total
}

fn context_area_inspection_pressure_breakdown(
    accumulator: &AreaAccumulator,
) -> InspectionPressureBreakdown {
    let unselected_source_like = accumulator
        .source_like_count()
        .saturating_sub(accumulator.selected_source_like_count());
    let unselected_validation = accumulator
        .test_count
        .saturating_sub(accumulator.selected_test_count);
    let unselected_docs = accumulator
        .docs_count
        .saturating_sub(accumulator.selected_docs_count);
    let source_like_weight = 3usize;
    let validation_weight = 2usize;
    let docs_weight = 1usize;
    InspectionPressureBreakdown {
        source_like_unselected: unselected_source_like,
        validation_unselected: unselected_validation,
        docs_unselected: unselected_docs,
        source_like_weight,
        validation_weight,
        docs_weight,
        total: unselected_source_like
            .saturating_mul(source_like_weight)
            .saturating_add(unselected_validation.saturating_mul(validation_weight))
            .saturating_add(unselected_docs.saturating_mul(docs_weight)),
    }
}

fn context_area_reason(accumulator: &AreaAccumulator, multi_area_task: bool) -> String {
    let scope = if multi_area_task {
        "Broad task"
    } else {
        "Focused task"
    };
    format!(
        "{scope} candidate area with {} candidate path(s), {} selected path(s), and {} source-like next-read candidate(s).",
        accumulator.candidate_count,
        accumulator.selected_count,
        accumulator
            .source_like_count()
            .saturating_sub(accumulator.selected_source_like_count())
    )
}

struct RankedContextArea {
    context_area: ContextArea,
    accumulator: AreaAccumulator,
}

fn context_area_priority(area: &str, accumulator: &AreaAccumulator, multi_area_task: bool) -> u8 {
    if multi_area_task && accumulator.selected_test_count > 0 {
        return 0;
    }
    if area == "examples" || area.starts_with("examples/") {
        return 3;
    }
    if accumulator.source_count + accumulator.config_count + accumulator.schema_count > 0 {
        return 1;
    }
    if accumulator.docs_count > 0 {
        return 2;
    }
    if accumulator.test_count > 0 {
        return 3;
    }
    4
}

const MAX_CONTEXT_AREAS: usize = 16;

#[derive(Default)]
struct AreaAccumulator {
    unique_paths: BTreeSet<String>,
    representative_paths: Vec<String>,
    next_read_candidates: Vec<AreaNextReadCandidate>,
    candidate_count: usize,
    selected_count: usize,
    source_count: usize,
    test_count: usize,
    config_count: usize,
    schema_count: usize,
    docs_count: usize,
    selected_source_count: usize,
    selected_test_count: usize,
    selected_config_count: usize,
    selected_schema_count: usize,
    selected_docs_count: usize,
    signal_counts: BTreeMap<String, usize>,
}

impl AreaAccumulator {
    fn record_role(&mut self, role: &Option<FileRole>) {
        match role {
            Some(FileRole::Source) => self.source_count += 1,
            Some(FileRole::Test) => self.test_count += 1,
            Some(FileRole::Config) => self.config_count += 1,
            Some(FileRole::Schema) => self.schema_count += 1,
            Some(FileRole::Docs) => self.docs_count += 1,
            _ => {}
        }
    }

    fn record_selected_role(&mut self, role: &Option<FileRole>) {
        match role {
            Some(FileRole::Source) => self.selected_source_count += 1,
            Some(FileRole::Test) => self.selected_test_count += 1,
            Some(FileRole::Config) => self.selected_config_count += 1,
            Some(FileRole::Schema) => self.selected_schema_count += 1,
            Some(FileRole::Docs) => self.selected_docs_count += 1,
            _ => {}
        }
    }

    fn record_signals(&mut self, candidate: &RetrievalCandidate) {
        let mut seen = BTreeSet::new();
        for score in &candidate.signal_scores {
            seen.insert(retrieval_signal_key(&score.signal));
        }
        if seen.is_empty() {
            for evidence in &candidate.evidence {
                seen.insert(retrieval_signal_key(&evidence.signal));
            }
        }
        for signal in seen {
            *self.signal_counts.entry(signal.to_string()).or_insert(0) += 1;
        }
    }

    fn record_next_read_candidate(&mut self, candidate: &RetrievalCandidate, path: &str) {
        self.next_read_candidates.push(AreaNextReadCandidate {
            path: path.to_string(),
            role_priority: next_read_role_priority(&candidate.role),
            signal_priority: next_read_signal_priority(candidate),
            signal_score: next_read_signal_score(candidate),
            confidence: candidate.confidence,
            insertion_index: self.next_read_candidates.len(),
        });
    }

    fn next_read_paths(&self) -> Vec<String> {
        let mut candidates = self.next_read_candidates.clone();
        candidates.sort_by(|left, right| {
            left.role_priority
                .cmp(&right.role_priority)
                .then_with(|| left.signal_priority.cmp(&right.signal_priority))
                .then_with(|| right.signal_score.total_cmp(&left.signal_score))
                .then_with(|| right.confidence.total_cmp(&left.confidence))
                .then_with(|| left.insertion_index.cmp(&right.insertion_index))
                .then_with(|| left.path.cmp(&right.path))
        });
        let limit = self.next_read_limit();
        candidates
            .into_iter()
            .take(limit)
            .map(|candidate| candidate.path)
            .collect()
    }

    fn next_read_limit(&self) -> usize {
        let source_like_unselected = self
            .source_like_count()
            .saturating_sub(self.selected_source_like_count());
        let validation_unselected = self.test_count.saturating_sub(self.selected_test_count);

        if source_like_unselected >= 12 || validation_unselected >= 8 {
            8
        } else if source_like_unselected >= 6 || validation_unselected >= 4 {
            6
        } else {
            4
        }
    }

    fn source_like_count(&self) -> usize {
        self.source_count + self.config_count + self.schema_count
    }

    fn selected_source_like_count(&self) -> usize {
        self.selected_source_count + self.selected_config_count + self.selected_schema_count
    }

    fn role_counts(&self) -> BTreeMap<String, usize> {
        role_count_map([
            ("source", self.source_count),
            ("test", self.test_count),
            ("config", self.config_count),
            ("schema", self.schema_count),
            ("docs", self.docs_count),
        ])
    }

    fn selected_role_counts(&self) -> BTreeMap<String, usize> {
        role_count_map([
            ("source", self.selected_source_count),
            ("test", self.selected_test_count),
            ("config", self.selected_config_count),
            ("schema", self.selected_schema_count),
            ("docs", self.selected_docs_count),
        ])
    }

    fn signal_counts(&self) -> BTreeMap<String, usize> {
        self.signal_counts.clone()
    }
}

#[derive(Clone)]
struct AreaNextReadCandidate {
    path: String,
    role_priority: u8,
    signal_priority: u8,
    signal_score: f32,
    confidence: f32,
    insertion_index: usize,
}

fn next_read_role_priority(role: &Option<FileRole>) -> u8 {
    match role {
        Some(FileRole::Source | FileRole::Config | FileRole::Schema) => 0,
        Some(FileRole::Test) => 1,
        Some(FileRole::Docs) => 2,
        _ => 3,
    }
}

fn next_read_signal_priority(candidate: &RetrievalCandidate) -> u8 {
    candidate
        .signal_scores
        .iter()
        .map(|score| next_read_signal_kind_priority(&score.signal))
        .chain(
            candidate
                .evidence
                .iter()
                .map(|evidence| next_read_signal_kind_priority(&evidence.signal)),
        )
        .min()
        .unwrap_or(9)
}

fn next_read_signal_score(candidate: &RetrievalCandidate) -> f32 {
    candidate
        .signal_scores
        .iter()
        .map(|score| score.score * score.weight)
        .chain(candidate.evidence.iter().map(|evidence| evidence.score))
        .max_by(|left, right| left.total_cmp(right))
        .unwrap_or(candidate.confidence)
}

fn next_read_signal_kind_priority(signal: &RetrievalSignalKind) -> u8 {
    match signal {
        RetrievalSignalKind::Anchor | RetrievalSignalKind::CurrentDiff => 0,
        RetrievalSignalKind::Lexical => 1,
        RetrievalSignalKind::Symbol => 2,
        RetrievalSignalKind::CoChange => 3,
        RetrievalSignalKind::Dependency => 4,
        RetrievalSignalKind::LexicalExpansion => 5,
        RetrievalSignalKind::Memory => 6,
        RetrievalSignalKind::Semantic => 7,
        RetrievalSignalKind::RelatedTest => 8,
        RetrievalSignalKind::History | RetrievalSignalKind::Config | RetrievalSignalKind::Docs => 9,
    }
}

fn retrieval_signal_key(signal: &RetrievalSignalKind) -> &'static str {
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

fn role_count_map<const N: usize>(counts: [(&str, usize); N]) -> BTreeMap<String, usize> {
    counts
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(role, count)| (role.to_string(), count))
        .collect()
}

fn extend_broad_source_area_candidates(
    search_results: &mut Vec<SearchResult>,
    roles: &BTreeMap<String, FileRole>,
) {
    let existing_paths = search_results
        .iter()
        .map(|result| result.path.as_str())
        .collect::<BTreeSet<_>>();
    let roots = source_roots_for_broad_candidates(search_results, roles);
    if roots.is_empty() {
        return;
    }

    let mut by_area = BTreeMap::<String, Vec<String>>::new();
    for (path, role) in roles {
        if role != &FileRole::Source || existing_paths.contains(path.as_str()) {
            continue;
        }
        if path.starts_with("examples/") || path.starts_with("tests/") || path.starts_with('.') {
            continue;
        }
        let Some(root) = first_path_component(path) else {
            continue;
        };
        if !roots.contains(root) {
            continue;
        }
        by_area
            .entry(context_area_for_path(path))
            .or_default()
            .push(path.clone());
    }

    let mut added = 0usize;
    for paths in by_area.values_mut() {
        paths.sort_by(|left, right| {
            path_depth(left)
                .cmp(&path_depth(right))
                .then_with(|| left.cmp(right))
        });
        for path in paths
            .iter()
            .take(BROAD_SOURCE_AREA_REPRESENTATIVES_PER_AREA)
        {
            if added >= BROAD_SOURCE_AREA_CANDIDATE_LIMIT {
                return;
            }
            search_results.push(SearchResult {
                path: path.clone(),
                role: FileRole::Source,
                language: language_for_path(path),
                score: BROAD_SOURCE_AREA_CANDIDATE_SCORE,
                reason: "broad source area candidate for multi-area task".to_string(),
            });
            added += 1;
        }
    }
}

const BROAD_SOURCE_AREA_CANDIDATE_LIMIT: usize = 48;
const BROAD_SOURCE_AREA_REPRESENTATIVES_PER_AREA: usize = 4;
const BROAD_SOURCE_AREA_CANDIDATE_SCORE: f32 = 8.0;

fn source_roots_for_broad_candidates<'a>(
    search_results: &'a [SearchResult],
    roles: &'a BTreeMap<String, FileRole>,
) -> BTreeSet<&'a str> {
    let mut roots = search_results
        .iter()
        .filter(|result| result.role == FileRole::Source)
        .filter_map(|result| first_path_component(&result.path))
        .filter(|root| !is_auxiliary_source_root(root))
        .collect::<BTreeSet<_>>();
    if roots.is_empty() {
        let mut counts = BTreeMap::<&str, usize>::new();
        for (path, role) in roles {
            if role == &FileRole::Source {
                if let Some(root) = first_path_component(path) {
                    if !is_auxiliary_source_root(root) {
                        *counts.entry(root).or_insert(0) += 1;
                    }
                }
            }
        }
        if let Some((root, _)) = counts
            .into_iter()
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        {
            roots.insert(root);
        }
    }
    roots
}

fn first_path_component(path: &str) -> Option<&str> {
    path.split('/').find(|component| !component.is_empty())
}

fn is_auxiliary_source_root(root: &str) -> bool {
    matches!(
        root,
        "examples" | "tests" | "test" | "docs" | "doc" | "scripts"
    )
}

fn path_depth(path: &str) -> usize {
    path.split('/')
        .filter(|component| !component.is_empty())
        .count()
}

fn language_for_path(path: &str) -> Option<String> {
    let language = if path.ends_with(".py") {
        "python"
    } else if path.ends_with(".ts") || path.ends_with(".tsx") {
        "typescript"
    } else if path.ends_with(".js") || path.ends_with(".jsx") {
        "javascript"
    } else if path.ends_with(".rs") {
        "rust"
    } else if path.ends_with(".go") {
        "go"
    } else if path.ends_with(".java") {
        "java"
    } else if path.ends_with(".kt") || path.ends_with(".kts") {
        "kotlin"
    } else {
        return None;
    };
    Some(language.to_string())
}

fn extend_cochanged_test_results(
    repo_root: &Path,
    test_results: &mut Vec<RelatedTestResult>,
    co_change_hints: &[ctxhelm_index::CoChangeHint],
    roles: &BTreeMap<String, FileRole>,
) -> Result<Vec<Diagnostic>, InventoryError> {
    let cochanged_tests = co_change_hints
        .iter()
        .filter(|hint| {
            roles
                .get(&hint.path)
                .is_some_and(|role| matches!(role, FileRole::Test))
        })
        .collect::<Vec<_>>();
    if cochanged_tests.is_empty() {
        return Ok(Vec::new());
    }

    let test_map = test_map_report(repo_root)?;
    let commands_by_path = test_map
        .results
        .into_iter()
        .map(|test| (test.path, test.command))
        .collect::<BTreeMap<_, _>>();

    for hint in cochanged_tests {
        if let Some(existing) = test_results.iter_mut().find(|test| test.path == hint.path) {
            if existing.command.is_none() {
                existing.command = commands_by_path.get(&hint.path).cloned().flatten();
            }
            continue;
        }

        test_results.push(RelatedTestResult {
            path: hint.path.clone(),
            command: commands_by_path.get(&hint.path).cloned().flatten(),
            confidence: hint.confidence.clamp(0.55, 0.95),
            reason: format!("test co-changed with target files: {}", hint.reason),
        });
    }

    Ok(test_map.diagnostics)
}

fn push_source_test_seed(
    seed_paths: &mut Vec<String>,
    roles: &BTreeMap<String, FileRole>,
    path: &str,
) {
    if roles
        .get(path)
        .is_none_or(|role| matches!(role, FileRole::Source))
    {
        push_unique(seed_paths, path.to_string());
    }
}

fn maybe_add_broad_validation_command(task: &str, plan: &mut ContextPlan) {
    if !is_broad_validation_task(task, &plan.related_tests) {
        return;
    }
    let Some(command) = broad_validation_command(&plan.recommended_commands) else {
        return;
    };
    if plan
        .recommended_commands
        .iter()
        .any(|existing| existing.command == command)
    {
        return;
    }

    plan.recommended_commands.push(Command {
        command: command.clone(),
        reason: "broad validation fallback for multi-area smoke/workflow task".to_string(),
    });
    push_plan_diagnostic(
        plan,
        Diagnostic {
            code: "broad_validation_scope".to_string(),
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "Task appears to span multiple validation areas; run `{command}` after targeted tests."
            ),
            paths: plan
                .related_tests
                .iter()
                .map(|test| test.path.clone())
                .collect(),
            count: plan.related_tests.len(),
        },
    );
}

fn is_broad_validation_task(task: &str, related_tests: &[ctxhelm_core::RelatedTest]) -> bool {
    let test_area_count = related_tests
        .iter()
        .filter_map(|test| test.path.strip_prefix("tests/"))
        .filter_map(|path| path.split('/').next())
        .filter(|area| !area.is_empty())
        .collect::<BTreeSet<_>>()
        .len();
    let terms = terms(task);
    let broad_term_count = [
        "ci",
        "eval",
        "evaluation",
        "harden",
        "orchestration",
        "run",
        "runs",
        "smoke",
        "stabilize",
        "workflow",
    ]
    .into_iter()
    .filter(|term| terms.contains(*term))
    .count();

    test_area_count >= 3
        && (related_tests.len() >= PREPARE_TASK_TEST_LIMIT || broad_term_count >= 2)
}

fn broad_validation_command(commands: &[Command]) -> Option<String> {
    let command_strings = commands
        .iter()
        .map(|command| command.command.trim())
        .filter(|command| !command.is_empty())
        .collect::<Vec<_>>();

    if command_strings
        .iter()
        .any(|command| command.starts_with("pytest "))
    {
        return Some("pytest".to_string());
    }
    if command_strings
        .iter()
        .any(|command| command.starts_with("cargo test"))
    {
        return Some("cargo test".to_string());
    }
    if command_strings
        .iter()
        .any(|command| command.starts_with("./gradlew test"))
    {
        return Some("./gradlew test".to_string());
    }
    if command_strings
        .iter()
        .any(|command| command.starts_with("mvn -Dtest="))
    {
        return Some("mvn test".to_string());
    }
    for package_manager in ["pnpm", "npm", "yarn"] {
        if command_strings
            .iter()
            .any(|command| command.starts_with(&format!("{package_manager} vitest run ")))
        {
            return Some(format!("{package_manager} vitest run"));
        }
        if command_strings
            .iter()
            .any(|command| command.starts_with(&format!("{package_manager} jest ")))
        {
            return Some(format!("{package_manager} jest"));
        }
        if command_strings.iter().any(|command| {
            command.starts_with(&format!("{package_manager} test "))
                || command.starts_with(&format!("{package_manager} test -- "))
        }) {
            return Some(format!("{package_manager} test"));
        }
    }

    None
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

fn construct_query_trace(task: &str, anchor_paths: &[String]) -> QueryConstructionTrace {
    let mut facets = Vec::new();
    push_query_facet(
        &mut facets,
        QueryFacetKind::OriginalTask,
        format!("task_hash:{}", task_hash(task)),
        "task".to_string(),
        0.1,
    );
    for path in anchor_paths {
        push_query_facet(
            &mut facets,
            QueryFacetKind::ExplicitPath,
            normalize_query_path(path),
            "active_context".to_string(),
            1.0,
        );
    }
    for path in extract_path_facets(task) {
        push_query_facet(
            &mut facets,
            QueryFacetKind::ExplicitPath,
            path,
            "task_path".to_string(),
            1.0,
        );
    }
    for frame in extract_stack_frame_facets(task) {
        push_query_facet(
            &mut facets,
            QueryFacetKind::StackFrame,
            frame,
            "task_stack_frame".to_string(),
            0.95,
        );
    }
    for symbol in extract_symbol_facets(task) {
        push_query_facet(
            &mut facets,
            QueryFacetKind::Symbol,
            symbol,
            "task_symbol".to_string(),
            0.9,
        );
    }
    for error in extract_error_facets(task) {
        push_query_facet(
            &mut facets,
            QueryFacetKind::ErrorText,
            error,
            "task_error".to_string(),
            0.8,
        );
    }
    for term in extract_domain_terms(task) {
        push_query_facet(
            &mut facets,
            QueryFacetKind::DomainPhrase,
            term,
            "task_terms".to_string(),
            0.5,
        );
    }
    if looks_like_commit_subject(task) {
        for term in extract_domain_terms(task).into_iter().take(6) {
            push_query_facet(
                &mut facets,
                QueryFacetKind::CommitClue,
                term,
                "task_commit_clue".to_string(),
                0.45,
            );
        }
    }

    let retriever_queries = retriever_query_set(&facets);
    QueryConstructionTrace {
        task_hash: task_hash(task),
        facets,
        retriever_queries,
        fusion_controls: FusionControlSummary {
            anchor_dominance: true,
            exact_evidence_protected: true,
            semantic_candidate_cap: search_limit(&TaskType::Explain),
            semantic_weight: 0.7,
        },
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn mentioned_commit_changed_path_anchors(
    repo_root: &Path,
    task: &str,
    plan: &mut ContextPlan,
) -> Result<Vec<String>, InventoryError> {
    let shas = extract_mentioned_commit_shas(task);
    if shas.is_empty() {
        return Ok(Vec::new());
    }
    let report =
        mentioned_commit_changed_paths_report(repo_root, &shas, PREPARE_TASK_TARGET_LIMIT)?;
    extend_plan_diagnostics(plan, report.diagnostics);
    if report.excluded_changed_file_count > 0 {
        push_plan_diagnostic(
            plan,
            Diagnostic {
                code: "mentioned_commit_changed_paths_excluded".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "{} task-mentioned commit path(s) were excluded by safe inventory policy.",
                    report.excluded_changed_file_count
                ),
                paths: Vec::new(),
                count: report.excluded_changed_file_count,
            },
        );
    }
    Ok(report.safe_changed_files)
}

fn extract_mentioned_commit_shas(task: &str) -> Vec<String> {
    if !task.to_ascii_lowercase().contains("commit") {
        return Vec::new();
    }
    let mut shas = Vec::new();
    for token in
        task.split(|character: char| !(character.is_ascii_alphanumeric() || character == '^'))
    {
        let token = token.trim_end_matches('^');
        if (7..=40).contains(&token.len())
            && token.chars().all(|character| character.is_ascii_hexdigit())
        {
            push_unique(&mut shas, token.to_ascii_lowercase());
        }
    }
    shas
}

fn is_commit_replay_task(task: &str) -> bool {
    let lower = task.to_ascii_lowercase();
    lower.contains("recreate the behavior from public commit")
        || lower.contains("recreate the behavior from commit")
        || lower.contains("restore the intended behavior")
}

fn narrow_selection_to_mentioned_commit_paths(
    selection: &mut crate::ranking::RankedSelection,
    mentioned_paths: &BTreeSet<String>,
    plan: &mut ContextPlan,
) {
    let original_target_count = selection.target_files.len();
    let original_test_count = selection.related_tests.len();
    selection
        .target_files
        .retain(|target| mentioned_paths.contains(&target.path));
    selection
        .related_tests
        .retain(|test| mentioned_paths.contains(&test.path));
    let target_paths = selection
        .target_files
        .iter()
        .map(|target| target.path.clone())
        .chain(selection.related_tests.iter().map(|test| test.path.clone()))
        .collect::<BTreeSet<_>>();
    selection.retrieval_candidates.retain(|candidate| {
        candidate
            .path
            .as_ref()
            .is_some_and(|path| target_paths.contains(path))
    });
    selection.recommended_commands = selection
        .related_tests
        .iter()
        .filter_map(|test| test.command.clone())
        .map(|command| Command {
            command,
            reason: "targeted validation from task-mentioned commit".to_string(),
        })
        .collect();
    push_plan_diagnostic(
        plan,
        Diagnostic {
            code: "commit_replay_changed_path_queue".to_string(),
            severity: DiagnosticSeverity::Info,
            message: format!(
                "Narrowed commit-replay recommendations from {} target(s) and {} test(s) to {} task-mentioned changed path(s).",
                original_target_count,
                original_test_count,
                target_paths.len()
            ),
            paths: target_paths.into_iter().collect(),
            count: mentioned_paths.len(),
        },
    );
}

fn planner_reranker_query_family(
    task_type: &TaskType,
    task: &str,
    query_trace: &QueryConstructionTrace,
) -> String {
    if is_multi_area_task(task) {
        return "broad_scope".to_string();
    }
    if is_low_information_task(task) {
        return "low_information".to_string();
    }
    if query_trace.facets.iter().any(|facet| {
        matches!(
            facet.kind,
            QueryFacetKind::ExplicitPath | QueryFacetKind::CurrentDiffPath
        )
    }) {
        return "explicit_path".to_string();
    }
    if query_trace.facets.iter().any(|facet| {
        matches!(
            facet.kind,
            QueryFacetKind::StackFrame | QueryFacetKind::ErrorText
        )
    }) {
        return "stack_or_error".to_string();
    }
    if query_trace
        .facets
        .iter()
        .any(|facet| matches!(facet.kind, QueryFacetKind::Symbol))
    {
        return "symbol_identifier".to_string();
    }
    if query_trace
        .facets
        .iter()
        .any(|facet| matches!(facet.kind, QueryFacetKind::CommitClue))
    {
        return "commit_clue".to_string();
    }
    if query_trace
        .facets
        .iter()
        .any(|facet| matches!(facet.kind, QueryFacetKind::DomainPhrase))
    {
        return "domain_phrase".to_string();
    }
    format!("task_type_{task_type:?}").to_ascii_lowercase()
}

fn push_query_facet(
    facets: &mut Vec<QueryFacet>,
    kind: QueryFacetKind,
    value: String,
    source: String,
    weight: f32,
) {
    let value = bounded_query_value(&value);
    if value.is_empty()
        || facets
            .iter()
            .any(|facet| facet.kind == kind && facet.value == value)
    {
        return;
    }
    facets.push(QueryFacet {
        kind,
        value,
        origin: source,
        weight,
    });
}

fn retriever_query_set(facets: &[QueryFacet]) -> RetrieverQuerySet {
    let mut query_set = RetrieverQuerySet {
        lexical_terms: Vec::new(),
        semantic_phrases: Vec::new(),
        symbol_terms: Vec::new(),
        graph_seeds: Vec::new(),
        history_terms: Vec::new(),
        test_terms: Vec::new(),
    };
    for facet in facets {
        match facet.kind {
            QueryFacetKind::ExplicitPath | QueryFacetKind::StackFrame => {
                push_unique(&mut query_set.lexical_terms, facet.value.clone());
                push_unique(&mut query_set.graph_seeds, facet.value.clone());
                push_unique(&mut query_set.test_terms, facet.value.clone());
            }
            QueryFacetKind::CurrentDiffPath => {
                push_unique(&mut query_set.lexical_terms, facet.value.clone());
                push_unique(&mut query_set.graph_seeds, facet.value.clone());
            }
            QueryFacetKind::Symbol => {
                push_unique(&mut query_set.symbol_terms, facet.value.clone());
                push_unique(&mut query_set.lexical_terms, facet.value.clone());
                push_unique(&mut query_set.semantic_phrases, facet.value.clone());
                push_unique(&mut query_set.test_terms, facet.value.clone());
            }
            QueryFacetKind::ErrorText => {
                push_unique(&mut query_set.lexical_terms, facet.value.clone());
                push_unique(&mut query_set.history_terms, facet.value.clone());
            }
            QueryFacetKind::DomainPhrase => {
                push_unique(&mut query_set.lexical_terms, facet.value.clone());
                push_unique(&mut query_set.semantic_phrases, facet.value.clone());
                push_unique(&mut query_set.history_terms, facet.value.clone());
            }
            QueryFacetKind::CommitClue => {
                push_unique(&mut query_set.history_terms, facet.value.clone());
            }
            QueryFacetKind::OriginalTask => {}
        }
    }
    query_set
}

fn query_text_or_task(values: &[String], task: &str) -> String {
    if values.is_empty() {
        task.to_string()
    } else {
        values.join(" ")
    }
}

fn semantic_query_text_or_task(
    values: &[String],
    task: &str,
    task_type: &TaskType,
    roles: &BTreeMap<String, FileRole>,
    source_role_hints: bool,
) -> String {
    let mut query = query_text_or_task(values, task);
    if !source_role_hints || !semantic_source_role_hints_apply(task_type) {
        return query;
    }

    let mut hints = vec![
        "source".to_string(),
        "code".to_string(),
        "implementation".to_string(),
        "module".to_string(),
    ];
    for language in dominant_source_languages(roles) {
        push_unique(&mut hints, language);
    }
    for hint in hints {
        if !query
            .split_whitespace()
            .any(|term| term.eq_ignore_ascii_case(&hint))
        {
            if !query.is_empty() {
                query.push(' ');
            }
            query.push_str(&hint);
        }
    }
    query
}

fn append_semantic_candidate_path_hints(
    query: &mut String,
    search_results: &[SearchResult],
) -> usize {
    let mut hint_count = 0usize;
    for result in search_results
        .iter()
        .take(SEMANTIC_CANDIDATE_PATH_HINT_PATH_LIMIT)
    {
        for hint in semantic_candidate_path_hint_terms(&result.path) {
            if hint_count >= SEMANTIC_CANDIDATE_PATH_HINT_TERM_LIMIT {
                return hint_count;
            }
            if query
                .split_whitespace()
                .any(|term| term.eq_ignore_ascii_case(&hint))
            {
                continue;
            }
            if !query.is_empty() {
                query.push(' ');
            }
            query.push_str(&hint);
            hint_count += 1;
        }
    }
    hint_count
}

fn append_semantic_candidate_sibling_path_hints(
    query: &mut String,
    search_results: &[SearchResult],
    roles: &BTreeMap<String, FileRole>,
) -> usize {
    let mut hint_count = 0usize;
    let mut sibling_paths = Vec::new();
    for result in search_results
        .iter()
        .take(SEMANTIC_SIBLING_PATH_HINT_PATH_LIMIT)
    {
        for path in semantic_candidate_sibling_paths(&result.path, roles) {
            if !sibling_paths.contains(&path) {
                sibling_paths.push(path);
            }
        }
    }
    for path in sibling_paths {
        let mut path_hint_count = 0usize;
        for hint in semantic_candidate_sibling_path_hint_terms(&path) {
            if hint_count >= SEMANTIC_SIBLING_PATH_HINT_TERM_LIMIT {
                return hint_count;
            }
            if path_hint_count >= SEMANTIC_SIBLING_PATH_HINTS_PER_PATH_LIMIT {
                break;
            }
            if query
                .split_whitespace()
                .any(|term| term.eq_ignore_ascii_case(&hint))
            {
                continue;
            }
            if !query.is_empty() {
                query.push(' ');
            }
            query.push_str(&hint);
            hint_count += 1;
            path_hint_count += 1;
        }
    }
    hint_count
}

fn semantic_candidate_sibling_path_hint_terms(path: &str) -> Vec<String> {
    let mut terms = Vec::new();
    if let Some(stem) = path_stem(path) {
        if semantic_candidate_compound_path_hint_allowed(&stem) {
            terms.push(stem);
        }
    }
    for term in semantic_candidate_path_hint_terms(path) {
        if !terms.contains(&term) {
            terms.push(term);
        }
    }
    terms
}

fn semantic_candidate_sibling_paths(
    candidate_path: &str,
    roles: &BTreeMap<String, FileRole>,
) -> Vec<String> {
    let mut paths = Vec::new();
    let Some(parent) = candidate_path.rsplit_once('/').map(|(parent, _)| parent) else {
        return paths;
    };
    let candidate_stem = path_stem(candidate_path);
    for (path, role) in roles {
        if path == candidate_path {
            continue;
        }
        if !matches!(role, FileRole::Source | FileRole::Test) {
            continue;
        }
        let same_directory = path
            .rsplit_once('/')
            .map(|(path_parent, _)| path_parent == parent)
            .unwrap_or(false);
        let mirrored_test = matches!(role, FileRole::Test)
            && candidate_stem
                .as_deref()
                .is_some_and(|stem| mirrored_test_path_matches(path, stem));
        if (same_directory || mirrored_test) && !paths.contains(path) {
            paths.push(path.clone());
        }
        if paths.len() >= SEMANTIC_SIBLING_PATH_HINT_PATH_LIMIT {
            break;
        }
    }
    paths
}

fn path_stem(path: &str) -> Option<String> {
    let file = path.rsplit('/').next()?;
    let stem = file.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(file);
    (!stem.is_empty()).then(|| stem.to_ascii_lowercase())
}

fn mirrored_test_path_matches(path: &str, source_stem: &str) -> bool {
    let Some(test_stem) = path_stem(path) else {
        return false;
    };
    let normalized = test_stem
        .strip_prefix("test_")
        .or_else(|| test_stem.strip_suffix("_test"))
        .unwrap_or(&test_stem);
    normalized == source_stem
}

fn semantic_candidate_path_hint_terms(path: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for raw in path.split(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
    }) {
        let raw = raw.trim_matches(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        });
        if raw.is_empty() {
            continue;
        }
        for part in raw.split(['_', '-']) {
            let part = part.to_ascii_lowercase();
            if semantic_candidate_path_hint_term_allowed(&part) && !terms.contains(&part) {
                terms.push(part);
            }
        }
    }
    terms
}

fn semantic_candidate_path_hint_term_allowed(term: &str) -> bool {
    if term.len() < 3 {
        return false;
    }
    !matches!(
        term,
        "src"
            | "lib"
            | "main"
            | "mod"
            | "index"
            | "init"
            | "test"
            | "tests"
            | "schema"
            | "agent"
            | "agents"
            | "python"
            | "source"
            | "docs"
            | "doc"
            | "readme"
    )
}

fn semantic_candidate_compound_path_hint_allowed(term: &str) -> bool {
    if term.len() < 3 {
        return false;
    }
    term.split(['_', '-'])
        .any(semantic_candidate_path_hint_term_allowed)
}

fn semantic_source_role_hints_apply(task_type: &TaskType) -> bool {
    matches!(
        task_type,
        TaskType::BugFix | TaskType::Feature | TaskType::Refactor | TaskType::Test
    )
}

fn dominant_source_languages(roles: &BTreeMap<String, FileRole>) -> Vec<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for (path, role) in roles {
        if !matches!(role, FileRole::Source) {
            continue;
        }
        if let Some(language) = language_for_path(path) {
            *counts.entry(language).or_default() += 1;
        }
    }
    let mut languages = counts.into_iter().collect::<Vec<_>>();
    languages.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    languages
        .into_iter()
        .take(2)
        .map(|(language, _)| language)
        .collect()
}

fn extract_path_facets(task: &str) -> Vec<String> {
    task.split_whitespace()
        .filter_map(|token| {
            let token = clean_query_token(token);
            let looks_like_path = token.contains('/')
                && [
                    ".ts", ".tsx", ".js", ".jsx", ".py", ".rs", ".go", ".java", ".kt", ".md",
                ]
                .iter()
                .any(|extension| token.ends_with(extension));
            looks_like_path.then_some(normalize_query_path(&token))
        })
        .collect()
}

fn extract_stack_frame_facets(task: &str) -> Vec<String> {
    let mut frames = Vec::new();
    for token in task.split_whitespace().map(clean_query_token) {
        let Some((path, line)) = token.rsplit_once(':') else {
            continue;
        };
        if path.contains('/') && path.contains('.') && line.chars().all(|c| c.is_ascii_digit()) {
            push_unique(&mut frames, normalize_query_path(path));
        }
    }
    frames
}

fn extract_symbol_facets(task: &str) -> Vec<String> {
    let mut symbols = task
        .split(|character: char| {
            !(character.is_ascii_alphanumeric()
                || character == '_'
                || character == ':'
                || character == '.')
        })
        .map(clean_query_token)
        .filter(|token| is_precise_symbol_facet(token))
        .take(16)
        .collect::<Vec<_>>();
    for alias in hyphenated_symbol_aliases(task) {
        if is_precise_symbol_facet(&alias) {
            push_unique(&mut symbols, alias);
        }
    }
    symbols.truncate(16);
    symbols
}

fn is_precise_symbol_facet(token: &str) -> bool {
    if token.len() < 3 {
        return false;
    }
    if token.contains("::") || token.contains('.') || token.contains('_') {
        return true;
    }
    let mut chars = token.chars();
    let _ = chars.next();
    let has_inner_uppercase = chars.any(|character| character.is_ascii_uppercase());
    let has_lowercase = token
        .chars()
        .any(|character| character.is_ascii_lowercase());
    has_inner_uppercase && has_lowercase
}

fn extract_error_facets(task: &str) -> Vec<String> {
    task.lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("error")
                || lower.contains("exception")
                || lower.contains("failed")
                || lower.contains("panic")
        })
        .map(clean_query_token)
        .filter(|line| !line.is_empty())
        .take(8)
        .collect()
}

fn extract_domain_terms(task: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for token in
        task.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
    {
        let token = clean_query_token(token).to_ascii_lowercase();
        if token.len() < 3 || QUERY_STOP_WORDS.contains(&token.as_str()) {
            continue;
        }
        push_unique(&mut terms, token);
        if terms.len() >= 24 {
            break;
        }
    }
    terms
}

fn hyphenated_symbol_aliases(task: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    for token in task.split(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
    }) {
        let token = clean_query_token(token)
            .trim_matches('-')
            .to_ascii_lowercase();
        if !token.contains('-') {
            continue;
        }
        let parts = token
            .split('-')
            .filter(|part| part.len() >= 2 && !QUERY_STOP_WORDS.contains(part))
            .collect::<Vec<_>>();
        if parts.len() < 2 || parts.len() > 5 {
            continue;
        }
        let alias = parts.join("_");
        if !aliases.contains(&alias) {
            aliases.push(alias);
        }
    }
    aliases
}

fn looks_like_commit_subject(task: &str) -> bool {
    let lower = task.trim().to_ascii_lowercase();
    [
        "fix ",
        "add ",
        "update ",
        "remove ",
        "refactor ",
        "support ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn clean_query_token(token: &str) -> String {
    let mut cleaned = token
        .trim_matches(|character: char| {
            character.is_ascii_punctuation()
                && !matches!(character, '/' | '.' | '_' | '-' | ':' | '\\')
        })
        .replace('\\', "/");
    while cleaned.ends_with(':') && !cleaned.ends_with("::") {
        cleaned.pop();
    }
    cleaned
}

fn normalize_query_path(path: &str) -> String {
    clean_query_token(path).trim_start_matches("./").to_string()
}

fn bounded_query_value(value: &str) -> String {
    let value = value.trim();
    if value.len() <= 160 {
        value.to_string()
    } else {
        value.chars().take(160).collect()
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}

fn extend_project_governance_docs(
    search_results: &mut Vec<SearchResult>,
    roles: &BTreeMap<String, FileRole>,
    task: &str,
) {
    let project_governance_task = is_project_governance_task(task);
    let agent_outcome_task = is_agent_outcome_task(task);
    let agent_guidance_task = is_agent_guidance_task(task);
    let graph_rag_task = is_graph_rag_rnd_task(task);
    if !project_governance_task && !agent_outcome_task && !agent_guidance_task && !graph_rag_task {
        return;
    }

    let mut seen = search_results
        .iter()
        .map(|result| result.path.clone())
        .collect::<BTreeSet<_>>();
    let mut paths = Vec::new();
    if project_governance_task {
        paths.extend(project_governance_doc_paths());
        if is_release_proof_task(task) {
            paths.extend(release_proof_artifact_paths());
        }
    }
    if agent_outcome_task {
        paths.extend(agent_outcome_doc_paths());
    }
    for (index, path) in paths.into_iter().enumerate() {
        let Some(role) = roles.get(path).cloned() else {
            continue;
        };
        if !matches!(
            role,
            FileRole::Docs | FileRole::Config | FileRole::Source | FileRole::Unknown
        ) {
            continue;
        }
        if !seen.insert(path.to_string()) {
            continue;
        }
        let is_agent_outcome_doc = agent_outcome_task && agent_outcome_doc_paths().contains(&path);
        let is_release_proof_artifact = project_governance_task
            && release_proof_artifact_paths().contains(&path)
            && is_release_proof_task(task);
        search_results.push(SearchResult {
            path: path.to_string(),
            role: project_governance_candidate_role(path, &role),
            language: language_for_governance_doc(path),
            score: if is_agent_outcome_doc {
                26.0 - index.min(10) as f32
            } else if is_release_proof_artifact {
                30.0 - index.min(10) as f32
            } else {
                16.0 - index.min(10) as f32
            },
            reason: if is_agent_outcome_doc {
                "agent-run workflow documentation for outcome-eval task".to_string()
            } else if is_release_proof_artifact {
                "release proof/governor artifact for project governance task".to_string()
            } else {
                "project governance artifact for planning/eval/release task".to_string()
            },
        });
    }

    if agent_guidance_task {
        extend_agent_guidance_source_candidates(search_results, roles, &mut seen);
    }
    if graph_rag_task {
        extend_graph_rag_source_candidates(search_results, roles, &mut seen);
    }
}

fn is_project_governance_task(task: &str) -> bool {
    let task_terms = terms(task);
    task_terms.iter().any(|term| {
        matches!(
            term.as_str(),
            "benchmark"
                | "benchmarks"
                | "eval"
                | "evaluate"
                | "evaluation"
                | "gate"
                | "gates"
                | "historical"
                | "measure"
                | "measured"
                | "metric"
                | "metrics"
                | "milestone"
                | "milestones"
                | "phase"
                | "phases"
                | "planning"
                | "proof"
                | "promote"
                | "promotion"
                | "recall"
                | "release"
                | "retrieval"
                | "roadmap"
                | "validation"
        )
    })
}

fn is_release_proof_task(task: &str) -> bool {
    let task_terms = terms(task);
    let has_release_or_gate = task_terms
        .iter()
        .any(|term| matches!(term.as_str(), "gate" | "gates" | "release"));
    let has_proof_or_governor = task_terms.iter().any(|term| {
        matches!(
            term.as_str(),
            "governor" | "governors" | "proof" | "decision" | "decisions"
        )
    });
    has_release_or_gate && has_proof_or_governor
}

fn is_agent_outcome_task(task: &str) -> bool {
    let task_terms = terms(task);
    let has_agent_run = task_terms
        .iter()
        .any(|term| term == "agent_run" || term == "agent-run")
        || (task_terms.iter().any(|term| term == "agent")
            && task_terms.iter().any(|term| term == "run"));
    has_agent_run
        || task_terms.iter().any(|term| {
            matches!(
                term.as_str(),
                "outcome" | "outcomes" | "lane" | "lanes" | "matrix" | "paired"
            )
        }) && task_terms.iter().any(|term| {
            matches!(
                term.as_str(),
                "agent" | "agents" | "claude" | "codex" | "client" | "clients"
            )
        })
}

fn is_agent_guidance_task(task: &str) -> bool {
    let task_terms = terms(task);
    let guidance_terms = [
        "consume",
        "consumes",
        "consumption",
        "e2e",
        "guidance",
        "harness",
        "instruction",
        "instructions",
        "native",
        "read",
        "reading",
        "setup",
    ];
    let agent_surface_terms = [
        "agent",
        "agents",
        "claude",
        "codex",
        "cursor",
        "mcp",
        "memory",
        "opencode",
        "selected",
        "selectedmemory",
    ];
    task_terms
        .iter()
        .any(|term| guidance_terms.contains(&term.as_str()))
        && task_terms
            .iter()
            .any(|term| agent_surface_terms.contains(&term.as_str()))
}

fn is_graph_rag_rnd_task(task: &str) -> bool {
    let task_terms = terms(task);
    let graph_terms = [
        "dependency",
        "dependencies",
        "edge",
        "edges",
        "graph",
        "graphrag",
        "neighborhood",
        "neighborhoods",
    ];
    let graph_quality_terms = [
        "ablation",
        "ablations",
        "allocation",
        "budget",
        "budgets",
        "family",
        "families",
        "profile",
        "profiles",
        "ranking",
        "retrieval",
    ];
    task_terms
        .iter()
        .any(|term| graph_terms.contains(&term.as_str()))
        && task_terms
            .iter()
            .any(|term| graph_quality_terms.contains(&term.as_str()))
}

fn extend_agent_guidance_source_candidates(
    search_results: &mut Vec<SearchResult>,
    roles: &BTreeMap<String, FileRole>,
    seen: &mut BTreeSet<String>,
) {
    let mut additions = Vec::new();
    for (index, path) in agent_guidance_source_paths().into_iter().enumerate() {
        let Some(role) = roles.get(path).cloned() else {
            continue;
        };
        if !is_agent_guidance_implementation_role(path, &role) {
            continue;
        }

        let boosted_score = 32.0 - index.min(10) as f32;
        let candidate_role = agent_guidance_candidate_role(path, &role);
        if let Some(position) = search_results.iter().position(|result| result.path == path) {
            let mut result = search_results.remove(position);
            result.role = candidate_role;
            result.score = result.score.max(boosted_score);
            result.reason = "agent-native guidance implementation surface".to_string();
            additions.push(result);
            continue;
        }

        if !seen.insert(path.to_string()) {
            continue;
        }

        additions.push(SearchResult {
            path: path.to_string(),
            role: candidate_role,
            language: language_for_path(path),
            score: boosted_score,
            reason: "agent-native guidance implementation surface".to_string(),
        });
    }
    search_results.splice(0..0, additions);
}

fn extend_graph_rag_source_candidates(
    search_results: &mut Vec<SearchResult>,
    roles: &BTreeMap<String, FileRole>,
    seen: &mut BTreeSet<String>,
) {
    let mut additions = Vec::new();
    for (index, path) in graph_rag_source_paths().into_iter().enumerate() {
        let Some(role) = roles.get(path).cloned() else {
            continue;
        };
        if !matches!(role, FileRole::Source) {
            continue;
        }

        let boosted_score = 34.0 - index.min(10) as f32;
        if let Some(position) = search_results.iter().position(|result| result.path == path) {
            let mut result = search_results.remove(position);
            result.role = FileRole::Source;
            result.score = result.score.max(boosted_score);
            result.reason =
                "GraphRAG implementation surface for graph-edge retrieval task".to_string();
            additions.push(result);
            continue;
        }

        if !seen.insert(path.to_string()) {
            continue;
        }

        additions.push(SearchResult {
            path: path.to_string(),
            role: FileRole::Source,
            language: language_for_path(path),
            score: boosted_score,
            reason: "GraphRAG implementation surface for graph-edge retrieval task".to_string(),
        });
    }
    search_results.splice(0..0, additions);
}

fn is_agent_guidance_implementation_role(path: &str, role: &FileRole) -> bool {
    matches!(role, FileRole::Source)
        || (matches!(role, FileRole::Unknown)
            && (path.ends_with(".sh") || path.ends_with(".bash") || path.ends_with(".zsh")))
}

fn agent_guidance_candidate_role(path: &str, role: &FileRole) -> FileRole {
    if matches!(role, FileRole::Unknown)
        && (path.ends_with(".sh") || path.ends_with(".bash") || path.ends_with(".zsh"))
    {
        FileRole::Source
    } else {
        role.clone()
    }
}

fn project_governance_candidate_role(path: &str, role: &FileRole) -> FileRole {
    if path == "scripts/smoke-governor.sh"
        && matches!(role, FileRole::Unknown)
        && (path.ends_with(".sh") || path.ends_with(".bash") || path.ends_with(".zsh"))
    {
        FileRole::Source
    } else {
        role.clone()
    }
}

fn project_governance_doc_paths() -> [&'static str; 10] {
    [
        ".planning/STATE.md",
        ".planning/ROADMAP.md",
        ".planning/MILESTONES.md",
        ".planning/REQUIREMENTS.md",
        ".planning/PROJECT.md",
        "AGENTS.md",
        "README.md",
        "docs/benchmarking.md",
        "docs/release.md",
        "docs/semantic.md",
    ]
}

fn release_proof_artifact_paths() -> [&'static str; 4] {
    [
        "docs/context-governor.md",
        "crates/ctxhelm-core/src/contracts.rs",
        "scripts/smoke-governor.sh",
        "scripts/release-gate.sh",
    ]
}

fn agent_outcome_doc_paths() -> [&'static str; 2] {
    ["docs/feedback.md", "docs/agent-setup.md"]
}

fn agent_guidance_source_paths() -> [&'static str; 5] {
    [
        "crates/ctxhelm-mcp/src/tools.rs",
        "crates/ctxhelm-core/src/init.rs",
        "crates/ctxhelm-compiler/src/packs.rs",
        "crates/ctxhelm-compiler/src/planning.rs",
        "scripts/e2e-agent-run-codex.sh",
    ]
}

fn graph_rag_source_paths() -> [&'static str; 3] {
    [
        "crates/ctxhelm-compiler/src/eval.rs",
        "crates/ctxhelm-compiler/src/ranking.rs",
        "crates/ctxhelm-index/src/dependencies.rs",
    ]
}

fn language_for_governance_doc(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .and_then(|(_, extension)| match extension {
            "md" => Some("markdown".to_string()),
            "toml" => Some("toml".to_string()),
            _ => None,
        })
}

const QUERY_STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "that", "this", "from", "into", "when", "then", "fix", "add",
    "update", "remove", "refactor", "support", "test", "task", "code", "file",
];

fn base_plan(task_type: TaskType) -> ContextPlan {
    let task_id = Uuid::new_v4();
    ContextPlan {
        task_id,
        task_type,
        confidence: 0.0,
        target_files: Vec::new(),
        related_tests: Vec::new(),
        context_areas: Vec::new(),
        recommended_commands: Vec::new(),
        pack_options: vec![
            PackOption {
                budget: PackBudget::Brief,
                resource_uri: format!("ctxhelm://pack/{task_id}/brief"),
            },
            PackOption {
                budget: PackBudget::Standard,
                resource_uri: format!("ctxhelm://pack/{task_id}/standard"),
            },
            PackOption {
                budget: PackBudget::Deep,
                resource_uri: format!("ctxhelm://pack/{task_id}/deep"),
            },
        ],
        missing_info_questions: Vec::new(),
        risk_flags: Vec::new(),
        diagnostics: Vec::new(),
        retrieval_candidates: Vec::new(),
        selected_memory: Vec::new(),
        query_trace: None,
        provider_policy: None,
        privacy_status: PrivacyStatus::local_only(),
    }
}

fn load_memory_cards_for_plan(repo_root: &Path, plan: &mut ContextPlan) -> Vec<MemoryCard> {
    match list_memory_cards(repo_root, &StoreConfig::default(), false) {
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
            Vec::new()
        }
    }
}

fn memory_path_candidates(
    task: &str,
    cards: &[MemoryCard],
    roles: &BTreeMap<String, FileRole>,
) -> Vec<MemoryPathCandidate> {
    const MAX_MEMORY_CONTEXT_LINKS_PER_CARD: usize = 3;

    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    let task_terms = terms(task);
    if task_terms.is_empty() {
        return candidates;
    }
    for card in cards.iter().filter(|card| memory_card_pack_eligible(card)) {
        let Some(score) = memory_task_overlap_score(&task_terms, card) else {
            continue;
        };
        let mut card_link_count = 0usize;
        for link in &card.source_links {
            let Some(role) = roles.get(link) else {
                continue;
            };
            if !memory_path_role_is_context(role) {
                continue;
            }
            if card_link_count >= MAX_MEMORY_CONTEXT_LINKS_PER_CARD {
                break;
            }
            if seen.insert((card.id.clone(), link.clone())) {
                candidates.push(MemoryPathCandidate {
                    path: link.clone(),
                    role: role.clone(),
                    score,
                    card_id: card.id.clone(),
                });
                card_link_count += 1;
            }
        }
    }
    candidates
}

fn memory_path_role_is_context(role: &FileRole) -> bool {
    matches!(
        role,
        FileRole::Source | FileRole::Config | FileRole::Schema | FileRole::Docs
    )
}

fn memory_task_overlap_score(task_terms: &BTreeSet<String>, card: &MemoryCard) -> Option<f32> {
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
    if overlap == 0 {
        return None;
    }
    Some(((overlap as f32 * 0.16).min(0.64) * card.confidence.max(0.1)).min(0.95))
}

fn attach_selected_memory(task: &str, plan: &mut ContextPlan, cards: &[MemoryCard]) {
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

pub(crate) fn is_multi_area_task(task: &str) -> bool {
    let terms = terms(task);
    if terms.is_empty() {
        return false;
    }
    let broad_term_count = [
        "archive",
        "archives",
        "artifact",
        "artifacts",
        "ci",
        "channel",
        "doc",
        "docs",
        "eval",
        "evaluate",
        "evaluation",
        "harden",
        "historical",
        "lint",
        "measure",
        "measured",
        "metric",
        "metrics",
        "orchestration",
        "product",
        "promote",
        "promotion",
        "proof",
        "retrieval",
        "run",
        "runs",
        "smoke",
        "stabilize",
        "target",
        "targets",
        "workflow",
    ]
    .into_iter()
    .filter(|term| terms.contains(*term))
    .count();
    broad_term_count >= 2
        || terms.contains("workflow")
            && (terms.contains("lint") || terms.contains("smoke") || terms.contains("eval"))
        || terms.contains("orchestration") && (terms.contains("run") || terms.contains("eval"))
}

fn uses_broad_target_file_floors(task: &str) -> bool {
    let terms = terms(task);
    if terms.is_empty() {
        return false;
    }
    let broad_term_count = [
        "ci",
        "channel",
        "eval",
        "evaluate",
        "evaluation",
        "harden",
        "historical",
        "lint",
        "measure",
        "measured",
        "metric",
        "metrics",
        "orchestration",
        "product",
        "promote",
        "promotion",
        "proof",
        "run",
        "runs",
        "smoke",
        "stabilize",
        "target",
        "targets",
        "workflow",
    ]
    .into_iter()
    .filter(|term| terms.contains(*term))
    .count();
    broad_term_count >= 2
        || terms.contains("workflow")
            && (terms.contains("lint") || terms.contains("smoke") || terms.contains("eval"))
        || terms.contains("orchestration") && (terms.contains("run") || terms.contains("eval"))
}

fn search_limit(task_type: &TaskType) -> usize {
    match task_type {
        TaskType::Explain => 8,
        TaskType::Review | TaskType::Refactor => 12,
        TaskType::BugFix | TaskType::Feature | TaskType::Test => 10,
    }
}

fn lexical_candidate_limit(task_type: &TaskType) -> usize {
    match task_type {
        TaskType::Explain => search_limit(task_type),
        TaskType::Review | TaskType::Refactor => 18,
        TaskType::BugFix | TaskType::Feature | TaskType::Test => 24,
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
    symbol_results: &[ctxhelm_index::SymbolSearchResult],
    search_results: &[ctxhelm_index::SearchResult],
    semantic_results: &[ctxhelm_index::SemanticSearchResult],
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
    {
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }
    if !paths.is_empty() {
        return paths;
    }
    for path in semantic_results.iter().map(|result| result.path.clone()) {
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
type AnchoredTargetFiles =
    Result<(Vec<AnchoredTarget>, Vec<String>, Vec<Diagnostic>), InventoryError>;

fn anchored_target_files(repo_root: &Path, anchor_paths: &[String]) -> AnchoredTargetFiles {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command as ProcessCommand;

    #[test]
    fn query_trace_extracts_source_free_facets() {
        let trace = construct_query_trace(
            "fix src/auth/session.ts\nsrc/auth/session.ts:42\nError: getSession failed in AuthService",
            &["tests/auth/session.test.ts".to_string()],
        );

        assert!(!trace.source_text_logged);
        assert!(trace.privacy_status.local_only);
        assert!(trace
            .facets
            .iter()
            .any(|facet| facet.kind == QueryFacetKind::ExplicitPath
                && facet.value == "src/auth/session.ts"));
        assert!(trace
            .facets
            .iter()
            .any(|facet| facet.kind == QueryFacetKind::Symbol && facet.value == "AuthService"));
        assert!(trace
            .facets
            .iter()
            .any(|facet| facet.kind == QueryFacetKind::ErrorText));
        assert!(trace
            .retriever_queries
            .semantic_phrases
            .iter()
            .any(|phrase| phrase == "AuthService"));
        assert!(trace.fusion_controls.anchor_dominance);
    }

    #[test]
    fn semantic_query_source_role_hints_add_dominant_source_languages() {
        let mut roles = BTreeMap::new();
        roles.insert("schema_agent/core/router.py".to_string(), FileRole::Source);
        roles.insert("schema_agent/core/tools.py".to_string(), FileRole::Source);
        roles.insert("docs/architecture.md".to_string(), FileRole::Docs);

        let query = semantic_query_text_or_task(
            &["schema".to_string(), "agent".to_string()],
            "fix schema agent routing",
            &TaskType::BugFix,
            &roles,
            true,
        );

        assert!(query.contains("schema agent"));
        assert!(query.contains("source"));
        assert!(query.contains("implementation"));
        assert!(query.contains("python"));
    }

    #[test]
    fn semantic_query_candidate_path_hints_add_bounded_path_aliases() {
        let mut query = "workflow vocabulary".to_string();
        let results = vec![
            SearchResult {
                path: "schema_agent/core/validated_workflow.py".to_string(),
                role: FileRole::Source,
                language: Some("python".to_string()),
                score: 1.0,
                reason: "test".to_string(),
            },
            SearchResult {
                path: "tests/agents/test_normalization_fds.py".to_string(),
                role: FileRole::Test,
                language: Some("python".to_string()),
                score: 0.8,
                reason: "test".to_string(),
            },
        ];

        let hint_count = append_semantic_candidate_path_hints(&mut query, &results);

        assert!(hint_count > 0);
        assert!(query.contains("validated"));
        assert!(query.contains("normalization"));
        assert!(query.contains("fds"));
        assert!(!query.contains("schema_agent"));
        assert!(!query.contains(" tests "));
    }

    #[test]
    fn semantic_query_candidate_sibling_path_hints_add_source_free_neighbor_aliases() {
        let mut query = "phase workflow".to_string();
        let results = vec![SearchResult {
            path: "schema_agent/core/state_validator.py".to_string(),
            role: FileRole::Source,
            language: Some("python".to_string()),
            score: 1.0,
            reason: "test".to_string(),
        }];
        let roles = BTreeMap::from([
            ("schema_agent/core/state.py".to_string(), FileRole::Source),
            (
                "schema_agent/core/state_validator.py".to_string(),
                FileRole::Source,
            ),
            (
                "schema_agent/nlp/transformer_ner.py".to_string(),
                FileRole::Source,
            ),
        ]);

        let hint_count = append_semantic_candidate_sibling_path_hints(&mut query, &results, &roles);

        assert!(hint_count > 0);
        assert!(query.contains("state"));
        assert!(!query.contains("schema_agent"));
    }

    #[test]
    fn semantic_query_candidate_sibling_path_hints_add_mirrored_test_aliases() {
        let mut query = "nlp transformer".to_string();
        let results = vec![SearchResult {
            path: "schema_agent/nlp/transformer_ner.py".to_string(),
            role: FileRole::Source,
            language: Some("python".to_string()),
            score: 1.0,
            reason: "test".to_string(),
        }];
        let roles = BTreeMap::from([
            (
                "schema_agent/nlp/transformer_ner.py".to_string(),
                FileRole::Source,
            ),
            (
                "tests/nlp/test_transformer_ner.py".to_string(),
                FileRole::Test,
            ),
        ]);

        let hint_count = append_semantic_candidate_sibling_path_hints(&mut query, &results, &roles);

        assert!(hint_count > 0);
        assert!(query.contains("test_transformer_ner"));
        assert!(!query.contains(" tests "));
    }

    #[test]
    fn query_trace_does_not_treat_commit_subject_acronyms_as_symbol_facets() {
        let trace = construct_query_trace("Default MCP repository path to working directory", &[]);

        assert!(trace.retriever_queries.symbol_terms.is_empty());
        assert!(trace
            .retriever_queries
            .lexical_terms
            .iter()
            .any(|term| term == "mcp"));
        assert!(trace
            .facets
            .iter()
            .any(|facet| facet.kind == QueryFacetKind::DomainPhrase && facet.value == "mcp"));
    }

    #[test]
    fn query_trace_adds_hyphenated_identifier_symbol_aliases() {
        let trace = construct_query_trace("Improve agent-run report attribution", &[]);

        assert!(trace
            .retriever_queries
            .symbol_terms
            .iter()
            .any(|term| term == "agent_run"));
        assert!(trace
            .retriever_queries
            .lexical_terms
            .iter()
            .any(|term| term == "agent_run"));
    }

    #[test]
    fn prepare_plan_promotes_task_mentioned_commit_changed_paths() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("src/main/java/gui")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/gui/webdiff/viewers/spv")).unwrap();
        fs::create_dir_all(repo.join("src/main/java/core")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/main/java/gui/MarkAsViewed.java"),
            "class MarkAsViewed {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/java/gui/webdiff/viewers/spv/AbstractSinglePageView.java"),
            "class AbstractSinglePageView {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/java/core/UMLModelDiff.java"),
            "class UMLModelDiff {}\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "initial"]);
        fs::write(
            repo.join("src/main/java/gui/MarkAsViewed.java"),
            "class MarkAsViewed { boolean viewed; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main/java/gui/webdiff/viewers/spv/AbstractSinglePageView.java"),
            "class AbstractSinglePageView { boolean viewed; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(
            &repo,
            &["commit", "-m", "Polish viewed file integration cleanup"],
        );
        let sha = run_git_stdout(&repo, &["rev-parse", "--short=12", "HEAD"]);

        let plan = prepare_context_plan_with_paths_history_and_semantic(
            &repo,
            &format!(
                "Recreate the behavior from public commit {sha}: Polish viewed file integration cleanup."
            ),
            TaskType::BugFix,
            &[],
            false,
            PlannerSemanticOptions::plain(false, SemanticProviderConfig::default()),
        )
        .unwrap();
        let target_paths = plan
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(target_paths.len(), 2);
        assert!(target_paths.contains(&"src/main/java/gui/MarkAsViewed.java"));
        assert!(target_paths
            .contains(&"src/main/java/gui/webdiff/viewers/spv/AbstractSinglePageView.java"));
        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "mentioned_commit_changed_paths"
            && diagnostic.count == 2));
        assert!(!serde_json::to_string(&plan)
            .unwrap()
            .contains("boolean viewed"));
    }

    #[test]
    fn prepare_plan_selects_renderer_for_hyphenated_agent_run_task() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("crates/ctxhelm/src")).unwrap();
        fs::create_dir_all(repo.join("scripts")).unwrap();
        fs::create_dir_all(repo.join("docs")).unwrap();
        fs::write(
            repo.join("crates/ctxhelm/src/main.rs"),
            "fn render_agent_run_report() {}\nfn render_feedback_report() {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("scripts/e2e-agent-run.sh"),
            "#!/usr/bin/env bash\n# agent-run report attribution harness\n",
        )
        .unwrap();
        fs::write(
            repo.join("crates/ctxhelm/src/other.rs"),
            "fn render_unrelated_report() {}\n",
        )
        .unwrap();
        fs::write(
            repo.join("docs/feedback.md"),
            "paired agent-run feedback and outcome eval docs\n",
        )
        .unwrap();

        let plan = prepare_context_plan_with_paths_and_history(
            repo,
            "Improve agent-run report attribution",
            TaskType::Explain,
            &[],
            false,
        )
        .unwrap();
        let paths = plan
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"crates/ctxhelm/src/main.rs"));
        assert!(paths.contains(&"docs/feedback.md"));
    }

    #[test]
    fn prepare_plan_promotes_task_path_to_anchor() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function getSession() { return null; }\n",
        )
        .unwrap();

        let plan = prepare_context_plan_with_paths_history_and_semantic(
            repo,
            "fix src/auth/session.ts getSession",
            TaskType::BugFix,
            &[],
            false,
            PlannerSemanticOptions::plain(false, SemanticProviderConfig::default()),
        )
        .unwrap();

        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert!(plan
            .target_files
            .first()
            .unwrap()
            .attribution
            .iter()
            .any(|evidence| evidence.signal == RetrievalSignalKind::Anchor));
        let trace = plan.query_trace.expect("query trace");
        assert!(trace
            .facets
            .iter()
            .any(|facet| facet.kind == QueryFacetKind::ExplicitPath
                && facet.value == "src/auth/session.ts"));
    }

    #[test]
    fn expansion_seeds_use_semantic_only_when_exact_seeds_are_absent() {
        let lexical = ctxhelm_index::SearchResult {
            path: "src/exact.ts".to_string(),
            role: FileRole::Source,
            language: Some("typescript".to_string()),
            score: 12.0,
            reason: "term match".to_string(),
        };
        let semantic = ctxhelm_index::SemanticSearchResult {
            path: "src/semantic.ts".to_string(),
            role: FileRole::Source,
            language: Some("typescript".to_string()),
            score: 0.95,
            reason: "local semantic similarity".to_string(),
            provider: SemanticProviderConfig::default(),
            document_id: Some("semantic_doc".to_string()),
            matched_facets: Vec::new(),
            precision_status: None,
        };

        let exact_seeded =
            expansion_seed_paths(&[], &[], &[lexical], std::slice::from_ref(&semantic));
        assert_eq!(exact_seeded, vec!["src/exact.ts".to_string()]);

        let semantic_seeded = expansion_seed_paths(&[], &[], &[], &[semantic]);
        assert_eq!(semantic_seeded, vec!["src/semantic.ts".to_string()]);
    }

    #[test]
    fn lexical_candidate_limit_is_wider_than_symbol_limit_for_coding_tasks() {
        assert_eq!(search_limit(&TaskType::BugFix), 10);
        assert_eq!(lexical_candidate_limit(&TaskType::BugFix), 24);
        assert_eq!(lexical_candidate_limit(&TaskType::Feature), 24);
        assert_eq!(lexical_candidate_limit(&TaskType::Test), 24);
        assert_eq!(lexical_candidate_limit(&TaskType::Review), 18);
        assert_eq!(
            lexical_candidate_limit(&TaskType::Explain),
            search_limit(&TaskType::Explain)
        );
    }

    #[test]
    fn governance_tasks_add_root_planning_docs_as_candidates() {
        let mut search_results = vec![SearchResult {
            path: "crates/ctxhelm-compiler/src/eval.rs".to_string(),
            role: FileRole::Source,
            language: Some("rust".to_string()),
            score: 20.0,
            reason: "source match".to_string(),
        }];
        let roles = [
            (".planning/STATE.md".to_string(), FileRole::Docs),
            (".planning/ROADMAP.md".to_string(), FileRole::Docs),
            (".planning/MILESTONES.md".to_string(), FileRole::Docs),
            ("docs/benchmarking.md".to_string(), FileRole::Docs),
            ("docs/feedback.md".to_string(), FileRole::Docs),
            ("src/auth.ts".to_string(), FileRole::Source),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(
            &mut search_results,
            &roles,
            "evaluate retrievable historical targets",
        );
        let paths = search_results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&".planning/STATE.md"));
        assert!(paths.contains(&".planning/ROADMAP.md"));
        assert!(paths.contains(&".planning/MILESTONES.md"));
        assert!(paths.contains(&"docs/benchmarking.md"));
        assert!(!paths.contains(&"docs/context-governor.md"));
        assert!(!paths.contains(&"scripts/smoke-governor.sh"));
        assert!(!paths.contains(&"docs/feedback.md"));
        assert!(!paths.contains(&"src/auth.ts"));
    }

    #[test]
    fn release_proof_tasks_add_governor_docs_and_smoke_artifacts() {
        let mut search_results = vec![SearchResult {
            path: "crates/ctxhelm/src/main.rs".to_string(),
            role: FileRole::Source,
            language: Some("rust".to_string()),
            score: 20.0,
            reason: "source match".to_string(),
        }];
        let roles = [
            (".planning/STATE.md".to_string(), FileRole::Docs),
            ("docs/context-governor.md".to_string(), FileRole::Docs),
            (
                "crates/ctxhelm-core/src/contracts.rs".to_string(),
                FileRole::Source,
            ),
            ("docs/release.md".to_string(), FileRole::Docs),
            ("scripts/smoke-governor.sh".to_string(), FileRole::Unknown),
            ("scripts/release-gate.sh".to_string(), FileRole::Unknown),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(
            &mut search_results,
            &roles,
            "improve context governor decision reports and release-gate proof",
        );
        let result_by_path = search_results
            .iter()
            .map(|result| (result.path.as_str(), result))
            .collect::<BTreeMap<_, _>>();

        assert!(result_by_path.contains_key("docs/context-governor.md"));
        assert!(result_by_path.contains_key("crates/ctxhelm-core/src/contracts.rs"));
        assert_eq!(
            result_by_path["crates/ctxhelm-core/src/contracts.rs"].role,
            FileRole::Source
        );
        assert_eq!(
            result_by_path["scripts/smoke-governor.sh"].role,
            FileRole::Source
        );
        assert_eq!(
            result_by_path["scripts/release-gate.sh"].role,
            FileRole::Unknown
        );
        assert!(result_by_path["docs/context-governor.md"].score >= 20.0);
        assert_eq!(
            result_by_path["docs/context-governor.md"].reason,
            "release proof/governor artifact for project governance task"
        );
    }

    #[test]
    fn graph_rag_tasks_add_edge_retrieval_implementation_surfaces() {
        let mut search_results = vec![SearchResult {
            path: ".planning/ROADMAP.md".to_string(),
            role: FileRole::Docs,
            language: Some("markdown".to_string()),
            score: 20.0,
            reason: "planning match".to_string(),
        }];
        let roles = [
            (
                "crates/ctxhelm-compiler/src/eval.rs".to_string(),
                FileRole::Source,
            ),
            (
                "crates/ctxhelm-compiler/src/ranking.rs".to_string(),
                FileRole::Source,
            ),
            (
                "crates/ctxhelm-index/src/dependencies.rs".to_string(),
                FileRole::Source,
            ),
            (".planning/ROADMAP.md".to_string(), FileRole::Docs),
            ("docs/graph.md".to_string(), FileRole::Docs),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(
            &mut search_results,
            &roles,
            "graph edge profiles ablations and dependency edge-family budget allocation",
        );
        let result_by_path = search_results
            .iter()
            .map(|result| (result.path.as_str(), result))
            .collect::<BTreeMap<_, _>>();

        assert!(result_by_path.contains_key("crates/ctxhelm-compiler/src/eval.rs"));
        assert!(result_by_path.contains_key("crates/ctxhelm-compiler/src/ranking.rs"));
        assert!(result_by_path.contains_key("crates/ctxhelm-index/src/dependencies.rs"));
        assert!(!result_by_path.contains_key("docs/graph.md"));
        assert_eq!(
            result_by_path["crates/ctxhelm-compiler/src/ranking.rs"].reason,
            "GraphRAG implementation surface for graph-edge retrieval task"
        );
    }

    #[test]
    fn agent_outcome_tasks_add_feedback_docs_as_candidates() {
        let mut search_results = vec![SearchResult {
            path: "scripts/e2e-agent-run.sh".to_string(),
            role: FileRole::Unknown,
            language: Some("bash".to_string()),
            score: 20.0,
            reason: "source match".to_string(),
        }];
        let roles = [
            ("docs/feedback.md".to_string(), FileRole::Docs),
            ("docs/agent-setup.md".to_string(), FileRole::Docs),
            ("docs/benchmarking.md".to_string(), FileRole::Docs),
            ("src/auth.ts".to_string(), FileRole::Source),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(
            &mut search_results,
            &roles,
            "Improve paired agent-run lane matrix",
        );
        let paths = search_results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"docs/feedback.md"));
        assert!(paths.contains(&"docs/agent-setup.md"));
        assert!(!paths.contains(&"docs/benchmarking.md"));
        assert!(!paths.contains(&"src/auth.ts"));
    }

    #[test]
    fn agent_guidance_tasks_add_implementation_surfaces_as_candidates() {
        let mut search_results = vec![
            SearchResult {
                path: "crates/ctxhelm-core/src/init.rs".to_string(),
                role: FileRole::Source,
                language: Some("rust".to_string()),
                score: 20.0,
                reason: "source match".to_string(),
            },
            SearchResult {
                path: "scripts/e2e-agent-run-codex.sh".to_string(),
                role: FileRole::Unknown,
                language: Some("bash".to_string()),
                score: 18.0,
                reason: "late lexical match".to_string(),
            },
        ];
        let roles = [
            (
                "crates/ctxhelm-mcp/src/tools.rs".to_string(),
                FileRole::Source,
            ),
            (
                "crates/ctxhelm-core/src/init.rs".to_string(),
                FileRole::Source,
            ),
            (
                "crates/ctxhelm-compiler/src/packs.rs".to_string(),
                FileRole::Source,
            ),
            (
                "crates/ctxhelm-compiler/src/planning.rs".to_string(),
                FileRole::Source,
            ),
            (
                "scripts/e2e-agent-run-codex.sh".to_string(),
                FileRole::Unknown,
            ),
            ("docs/agent-setup.md".to_string(), FileRole::Docs),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(
            &mut search_results,
            &roles,
            "Improve Codex agent-run harness",
        );
        let paths = search_results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths
            .iter()
            .take(5)
            .any(|path| *path == "scripts/e2e-agent-run-codex.sh"));
        assert!(paths.contains(&"crates/ctxhelm-mcp/src/tools.rs"));
        assert!(paths.contains(&"crates/ctxhelm-compiler/src/planning.rs"));
        assert!(paths.contains(&"scripts/e2e-agent-run-codex.sh"));
        assert_eq!(
            paths
                .iter()
                .filter(|path| **path == "scripts/e2e-agent-run-codex.sh")
                .count(),
            1
        );
    }

    #[test]
    fn non_governance_tasks_do_not_add_root_planning_docs() {
        let mut search_results = Vec::new();
        let roles = [(".planning/STATE.md".to_string(), FileRole::Docs)]
            .into_iter()
            .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(&mut search_results, &roles, "fix auth session cookie");

        assert!(search_results.is_empty());
    }

    #[test]
    fn product_proof_and_historical_eval_tasks_are_multi_area() {
        assert!(is_multi_area_task("promote channel aware product proof"));
        assert!(is_multi_area_task(
            "evaluate retrievable historical targets"
        ));
        assert!(is_multi_area_task("dampen archive artifacts retrieval"));
        assert!(is_multi_area_task(
            "MCP docs record real client mcp proof refresh"
        ));
        assert!(!is_multi_area_task("fix auth session cookie"));
    }

    #[test]
    fn docs_and_archive_multi_area_tasks_do_not_spend_target_file_source_floors() {
        assert!(uses_broad_target_file_floors(
            "promote channel aware product proof"
        ));
        assert!(uses_broad_target_file_floors(
            "measure recall via validation channel"
        ));
        assert!(!uses_broad_target_file_floors(
            "dampen archive artifacts retrieval"
        ));
        assert!(!uses_broad_target_file_floors(
            "MCP docs record real client mcp proof refresh"
        ));
    }

    #[test]
    fn context_areas_include_docs_and_next_read_paths() {
        let selection = crate::ranking::RankedSelection {
            retrieval_candidates: vec![
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/workflow.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "lexical_match".to_string(),
                    confidence: 0.9,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::Dependency,
                        score: 0.8,
                        weight: 0.2,
                    }],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::Doc,
                    path: Some("docs/release.md".to_string()),
                    role: Some(FileRole::Docs),
                    reason_code: "lexical_match".to_string(),
                    confidence: 0.8,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::Lexical,
                        score: 0.8,
                        weight: 0.35,
                    }],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::Doc,
                    path: Some("docs/benchmarking.md".to_string()),
                    role: Some(FileRole::Docs),
                    reason_code: "lexical_match".to_string(),
                    confidence: 0.7,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::CoChange,
                        score: 0.7,
                        weight: 0.2,
                    }],
                    evidence: vec![],
                },
            ],
            target_files: vec![TargetFile {
                path: "src/workflow.ts".to_string(),
                reason: "selected source".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: vec![],
            }],
            related_tests: vec![],
            recommended_commands: vec![],
        };

        let areas = context_areas_for_plan(&selection, true);
        let docs_area = areas.iter().find(|area| area.area == "docs").unwrap();

        assert_eq!(docs_area.candidate_count, 2);
        assert_eq!(docs_area.selected_count, 0);
        assert_eq!(docs_area.unselected_count, 2);
        assert_eq!(docs_area.coverage_percent, 0);
        assert_eq!(
            docs_area.inspection_pressure, 2,
            "docs-only zero-selected areas should carry bounded inspection pressure"
        );
        assert_eq!(docs_area.inspection_pressure_breakdown.docs_unselected, 2);
        assert_eq!(docs_area.inspection_pressure_breakdown.docs_weight, 1);
        assert_eq!(docs_area.inspection_pressure_breakdown.total, 2);
        assert_eq!(
            docs_area.role_counts.get("docs").copied(),
            Some(2),
            "context areas should expose source-free role mix"
        );
        assert_eq!(
            docs_area.signal_counts.get("lexical").copied(),
            Some(1),
            "context areas should expose source-free signal mix"
        );
        assert_eq!(
            docs_area.signal_counts.get("co_change").copied(),
            Some(1),
            "context areas should preserve history pressure"
        );
        assert!(
            docs_area.selected_role_counts.is_empty(),
            "zero-selected areas should make selected role pressure explicit"
        );
        assert_eq!(
            docs_area.next_read_paths,
            vec![
                "docs/release.md".to_string(),
                "docs/benchmarking.md".to_string()
            ]
        );
        assert_eq!(docs_area.resource_uri, "ctxhelm://repo/context-area/docs");
    }

    #[test]
    fn focused_context_areas_keep_selected_source_area_hints_only() {
        let selection = crate::ranking::RankedSelection {
            retrieval_candidates: vec![
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/session.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "lexical_match".to_string(),
                    confidence: 0.9,
                    signal_scores: vec![],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/cookies.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "dependency_match".to_string(),
                    confidence: 0.7,
                    signal_scores: vec![],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::Doc,
                    path: Some("docs/auth.md".to_string()),
                    role: Some(FileRole::Docs),
                    reason_code: "lexical_match".to_string(),
                    confidence: 0.6,
                    signal_scores: vec![],
                    evidence: vec![],
                },
            ],
            target_files: vec![TargetFile {
                path: "src/auth/session.ts".to_string(),
                reason: "selected source".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: vec![],
            }],
            related_tests: vec![],
            recommended_commands: vec![],
        };

        let areas = context_areas_for_plan(&selection, false);

        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].area, "src/auth");
        assert_eq!(areas[0].selected_count, 1);
        assert_eq!(areas[0].next_read_paths, vec!["src/auth/cookies.ts"]);
        assert!(areas[0].reason.starts_with("Focused task"));
    }

    #[test]
    fn context_area_next_reads_order_by_source_free_signal_strength() {
        let selection = crate::ranking::RankedSelection {
            retrieval_candidates: vec![
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/session.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "selected source".to_string(),
                    confidence: 0.9,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::Lexical,
                        score: 0.9,
                        weight: 1.0,
                    }],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/semantic.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "semantic_match".to_string(),
                    confidence: 0.95,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::Semantic,
                        score: 0.95,
                        weight: 0.45,
                    }],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/cochange.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "co_change_neighbor".to_string(),
                    confidence: 0.7,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::CoChange,
                        score: 0.7,
                        weight: 1.35,
                    }],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/symbol.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "symbol_match".to_string(),
                    confidence: 0.8,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::Symbol,
                        score: 0.8,
                        weight: 1.05,
                    }],
                    evidence: vec![],
                },
                RetrievalCandidate {
                    kind: RetrievalCandidateKind::File,
                    path: Some("src/auth/lexical.ts".to_string()),
                    role: Some(FileRole::Source),
                    reason_code: "lexical_match".to_string(),
                    confidence: 0.75,
                    signal_scores: vec![RetrievalSignalScore {
                        signal: RetrievalSignalKind::Lexical,
                        score: 0.75,
                        weight: 1.0,
                    }],
                    evidence: vec![],
                },
            ],
            target_files: vec![TargetFile {
                path: "src/auth/session.ts".to_string(),
                reason: "selected source".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: vec![],
            }],
            related_tests: vec![],
            recommended_commands: vec![],
        };

        let areas = context_areas_for_plan(&selection, false);

        assert_eq!(areas.len(), 1);
        assert_eq!(
            areas[0].next_read_paths,
            vec![
                "src/auth/lexical.ts".to_string(),
                "src/auth/symbol.ts".to_string(),
                "src/auth/cochange.ts".to_string(),
                "src/auth/semantic.ts".to_string(),
            ],
            "next reads should use source-free signal priority before insertion order"
        );
    }

    #[test]
    fn context_area_next_reads_expand_for_high_pressure_source_areas() {
        let mut retrieval_candidates = vec![RetrievalCandidate {
            kind: RetrievalCandidateKind::File,
            path: Some("src/auth/session.ts".to_string()),
            role: Some(FileRole::Source),
            reason_code: "selected source".to_string(),
            confidence: 0.9,
            signal_scores: vec![RetrievalSignalScore {
                signal: RetrievalSignalKind::Lexical,
                score: 0.9,
                weight: 1.0,
            }],
            evidence: vec![],
        }];
        for index in 0..12 {
            retrieval_candidates.push(RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some(format!("src/auth/neighbor_{index}.ts")),
                role: Some(FileRole::Source),
                reason_code: "co_change_neighbor".to_string(),
                confidence: 0.7,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::CoChange,
                    score: 0.7,
                    weight: 1.35,
                }],
                evidence: vec![],
            });
        }
        let selection = crate::ranking::RankedSelection {
            retrieval_candidates,
            target_files: vec![TargetFile {
                path: "src/auth/session.ts".to_string(),
                reason: "selected source".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: vec![],
            }],
            related_tests: vec![],
            recommended_commands: vec![],
        };

        let areas = context_areas_for_plan(&selection, false);

        assert_eq!(areas.len(), 1);
        assert_eq!(
            areas[0].next_read_paths.len(),
            8,
            "high-pressure source areas should expose a larger progressive read budget"
        );
        assert_eq!(areas[0].next_read_paths[0], "src/auth/neighbor_0.ts");
    }

    #[test]
    fn broad_context_areas_reserve_selected_validation_areas() {
        let mut retrieval_candidates = Vec::new();
        for index in 0..MAX_CONTEXT_AREAS + 4 {
            retrieval_candidates.push(RetrievalCandidate {
                kind: RetrievalCandidateKind::File,
                path: Some(format!("src/module_{index}/workflow.ts")),
                role: Some(FileRole::Source),
                reason_code: "lexical_match".to_string(),
                confidence: 0.7,
                signal_scores: vec![RetrievalSignalScore {
                    signal: RetrievalSignalKind::LexicalExpansion,
                    score: 0.7,
                    weight: 0.8,
                }],
                evidence: vec![],
            });
        }
        retrieval_candidates.push(RetrievalCandidate {
            kind: RetrievalCandidateKind::Test,
            path: Some("tests/agents/test_base_agent_openai_compatible.py".to_string()),
            role: Some(FileRole::Test),
            reason_code: "related_test".to_string(),
            confidence: 0.95,
            signal_scores: vec![RetrievalSignalScore {
                signal: RetrievalSignalKind::RelatedTest,
                score: 0.95,
                weight: 0.9,
            }],
            evidence: vec![],
        });
        retrieval_candidates.push(RetrievalCandidate {
            kind: RetrievalCandidateKind::Test,
            path: Some("tests/agents/test_requirement_analyzer.py".to_string()),
            role: Some(FileRole::Test),
            reason_code: "related_test".to_string(),
            confidence: 0.85,
            signal_scores: vec![RetrievalSignalScore {
                signal: RetrievalSignalKind::RelatedTest,
                score: 0.85,
                weight: 0.9,
            }],
            evidence: vec![],
        });

        let selection = crate::ranking::RankedSelection {
            retrieval_candidates,
            target_files: vec![TargetFile {
                path: "src/module_0/workflow.ts".to_string(),
                reason: "selected source".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: vec![],
            }],
            related_tests: vec![ctxhelm_core::RelatedTest {
                path: "tests/agents/test_base_agent_openai_compatible.py".to_string(),
                reason: "selected validation".to_string(),
                command: Some(
                    "pytest tests/agents/test_base_agent_openai_compatible.py".to_string(),
                ),
                confidence: 0.95,
                attribution: vec![],
            }],
            recommended_commands: vec![],
        };

        let areas = context_areas_for_plan(&selection, true);
        let validation_area = areas
            .iter()
            .find(|area| area.area == "tests/agents")
            .unwrap();

        assert_eq!(validation_area.selected_count, 1);
        assert_eq!(
            validation_area.selected_role_counts.get("test").copied(),
            Some(1)
        );
        assert_eq!(
            validation_area.next_read_paths,
            vec!["tests/agents/test_requirement_analyzer.py".to_string()],
            "broad packs should preserve validation next-read hints even with many source areas"
        );
    }

    #[test]
    fn broad_source_area_candidates_cover_sibling_source_areas_without_existing_hits() {
        let mut search_results = vec![SearchResult {
            path: "schema_agent/core/workflow.py".to_string(),
            role: FileRole::Source,
            language: Some("python".to_string()),
            score: 20.0,
            reason: "lexical match".to_string(),
        }];
        let roles = [
            (
                "schema_agent/core/workflow.py".to_string(),
                FileRole::Source,
            ),
            (
                "schema_agent/nlp/dependency_parser.py".to_string(),
                FileRole::Source,
            ),
            (
                "schema_agent/nlp/entity_extractor.py".to_string(),
                FileRole::Source,
            ),
            (
                "schema_agent/text2r/db_executor.py".to_string(),
                FileRole::Source,
            ),
            (
                "schema_agent/text2r/normalizers/currency_normalizer.py".to_string(),
                FileRole::Source,
            ),
            (
                "examples/full_workflow_example.py".to_string(),
                FileRole::Source,
            ),
            ("tests/test_workflow.py".to_string(), FileRole::Test),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_broad_source_area_candidates(&mut search_results, &roles);
        let paths = search_results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"schema_agent/nlp/dependency_parser.py"));
        assert!(paths.contains(&"schema_agent/nlp/entity_extractor.py"));
        assert!(paths.contains(&"schema_agent/text2r/db_executor.py"));
        assert!(!paths.contains(&"examples/full_workflow_example.py"));
        assert_eq!(
            search_results
                .iter()
                .find(|result| result.path == "schema_agent/nlp/dependency_parser.py")
                .unwrap()
                .reason,
            "broad source area candidate for multi-area task"
        );
    }

    #[test]
    fn broad_source_area_candidates_enter_source_lexical_floor_at_controlled_score() {
        let weighted_score = (BROAD_SOURCE_AREA_CANDIDATE_SCORE / 20.0).clamp(0.15, 0.95) * 0.80;

        assert!(weighted_score >= 0.30);
    }

    #[test]
    fn prepare_plan_attaches_default_provider_policy_and_disabled_reranker() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.ts"), "export function auth() {}\n").unwrap();

        let plan = prepare_context_plan_with_paths_history_and_semantic(
            repo,
            "fix src/lib.ts auth",
            TaskType::BugFix,
            &[],
            false,
            PlannerSemanticOptions::plain(true, SemanticProviderConfig::default()),
        )
        .unwrap();
        let policy = plan.provider_policy.expect("provider policy");

        assert!(!policy.policy.allow_cloud_embeddings);
        assert!(!policy.policy.allow_cloud_reranking);
        assert!(!policy.policy.allow_source_transfer);
        assert!(policy.decisions.iter().any(|decision| decision.status
            == ProviderDecisionStatus::Disabled
            && decision.provider == "local_metadata"));
        assert!(policy
            .decisions
            .iter()
            .any(|decision| decision.provider == "local_hash"
                && decision.status == ProviderDecisionStatus::Allowed));
    }

    #[test]
    fn prepare_plan_applies_policy_enabled_local_metadata_reranker() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join(".ctxhelm")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.ts"), "export function auth() {}\n").unwrap();
        fs::write(
            repo.join(".ctxhelm/provider-policy.json"),
            r#"{
  "schemaVersion": 1,
  "name": "test-local-metadata-reranker",
  "allowLocalProviders": true,
  "allowCloudEmbeddings": false,
  "allowCloudReranking": false,
  "allowSourceTransfer": false,
  "enableLocalMetadataReranker": true,
  "sourceTextLogged": false
}"#,
        )
        .unwrap();

        let plan = prepare_context_plan_with_paths_history_and_semantic(
            repo,
            "fix src/lib.ts auth",
            TaskType::BugFix,
            &[],
            false,
            PlannerSemanticOptions::plain(true, SemanticProviderConfig::default()),
        )
        .unwrap();
        let policy = plan.provider_policy.as_ref().expect("provider policy");

        assert!(policy.decisions.iter().any(|decision| decision.status
            == ProviderDecisionStatus::Allowed
            && decision.provider == "local_metadata"));
        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "local_metadata_reranker_applied"
            && diagnostic.severity == DiagnosticSeverity::Info));
    }

    #[test]
    fn prepare_plan_applies_policy_enabled_query_family_routed_reranker_for_commit_clues() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join(".ctxhelm")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/lib.ts"),
            "export function paymentRetry() {}\n",
        )
        .unwrap();
        fs::write(
            repo.join(".ctxhelm/provider-policy.json"),
            r#"{
  "schemaVersion": 1,
  "name": "test-routed-local-metadata-reranker",
  "allowLocalProviders": true,
  "allowCloudEmbeddings": false,
  "allowCloudReranking": false,
  "allowSourceTransfer": false,
  "enableQueryFamilyRoutedReranker": true,
  "sourceTextLogged": false
}"#,
        )
        .unwrap();

        let plan = prepare_context_plan_with_paths_history_and_semantic(
            repo,
            "update payment retry behavior",
            TaskType::BugFix,
            &[],
            false,
            PlannerSemanticOptions::plain(true, SemanticProviderConfig::default()),
        )
        .unwrap();
        let policy = plan.provider_policy.as_ref().expect("provider policy");

        assert!(policy.decisions.iter().any(|decision| decision.status
            == ProviderDecisionStatus::Allowed
            && decision.provider == "local_metadata_routed"));
        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "query_family_routed_reranker_applied"
            && diagnostic.message.contains("commit_clue")));
        assert!(!plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "local_metadata_reranker_applied"));
    }

    #[test]
    fn prepare_plan_holds_policy_enabled_query_family_routed_reranker_for_unproven_families() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join(".ctxhelm")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.ts"), "export class AuthService {}\n").unwrap();
        fs::write(
            repo.join(".ctxhelm/provider-policy.json"),
            r#"{
  "schemaVersion": 1,
  "name": "test-routed-local-metadata-reranker",
  "allowLocalProviders": true,
  "allowCloudEmbeddings": false,
  "allowCloudReranking": false,
  "allowSourceTransfer": false,
  "enableQueryFamilyRoutedReranker": true,
  "sourceTextLogged": false
}"#,
        )
        .unwrap();

        let plan = prepare_context_plan_with_paths_history_and_semantic(
            repo,
            "fix AuthService redirect failure",
            TaskType::BugFix,
            &[],
            false,
            PlannerSemanticOptions::plain(true, SemanticProviderConfig::default()),
        )
        .unwrap();

        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "query_family_routed_reranker_held"
            && diagnostic.message.contains("symbol_identifier")));
        assert!(!plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "query_family_routed_reranker_applied"));
        assert!(!plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "local_metadata_reranker_applied"));
    }

    #[test]
    fn broad_validation_tasks_add_suite_fallback_command() {
        let mut plan = base_plan(TaskType::BugFix);
        plan.related_tests = [
            "tests/evaluation/test_pvldb_eval_runner.py",
            "tests/agents/test_base_agent_openai_compatible.py",
            "tests/core/test_retry_routing.py",
            "tests/gates/test_quality_gate_system_singleton.py",
        ]
        .into_iter()
        .map(|path| ctxhelm_core::RelatedTest {
            path: path.to_string(),
            reason: "fixture".to_string(),
            command: Some(format!("pytest {path}")),
            confidence: 0.8,
            attribution: Vec::new(),
        })
        .collect();
        plan.recommended_commands = plan
            .related_tests
            .iter()
            .filter_map(|test| test.command.clone())
            .map(|command| Command {
                command,
                reason: "targeted validation".to_string(),
            })
            .collect();

        maybe_add_broad_validation_command("fix(eval): harden qwen3.5 smoke workflow", &mut plan);

        assert!(plan.recommended_commands.iter().any(|command| {
            command.command == "pytest" && command.reason.contains("broad validation fallback")
        }));
        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "broad_validation_scope"));
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.code == "broad_validation_scope"));
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

    fn run_git_stdout(repo: &Path, args: &[&str]) -> String {
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
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}
