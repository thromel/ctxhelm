use crate::policy::{provider_policy_report, reranker_decision, semantic_provider_decision};
use crate::ranking::{
    rank_candidates, rerank_with_local_metadata, select_ranked_candidates_for_scope,
    AnchorCandidate, RankingInput,
};
use ctxpack_core::{
    Command, ContextArea, ContextPlan, Diagnostic, DiagnosticSeverity, FileRole,
    FusionControlSummary, MemoryCard, MemoryFreshness, MemoryReviewStatus, PackBudget, PackOption,
    PrivacyStatus, ProviderDecisionStatus, QueryConstructionTrace, QueryFacet, QueryFacetKind,
    RetrievalCandidate, RetrievalCandidateKind, RetrievalEvidence, RetrievalSignalKind,
    RetrievalSignalScore, RetrieverQuerySet, RiskFlag, SelectedMemory, TargetFile, TaskType,
};
use ctxpack_index::{
    co_change_hints_report, current_diff_summary_report, lexical_search_report, list_memory_cards,
    load_or_refresh_inventory, normalized_provider, related_dependency_edges_report,
    related_tests_report, semantic_search_report, symbol_search_report, task_hash, test_map_report,
    CoChangeOptions, CurrentDiffOptions, DependencyOptions, InventoryError, InventoryOptions,
    RelatedTestResult, SearchOptions, SearchResult, SemanticOptions, SemanticProviderConfig,
    StoreConfig, SymbolOptions,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use uuid::Uuid;

pub(crate) const PREPARE_TASK_TARGET_LIMIT: usize = 10;
pub(crate) const PREPARE_TASK_TEST_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryMode {
    Disabled,
    Full,
    ValidationOnly,
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
        semantic_enabled,
        semantic_provider,
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
        SemanticProviderConfig::default(),
    )
}

pub(crate) fn prepare_context_plan_with_paths_history_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    include_history: bool,
    semantic_enabled: bool,
    semantic_provider: SemanticProviderConfig,
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
        semantic_enabled,
        semantic_provider,
    )
}

pub(crate) fn prepare_context_plan_with_paths_history_mode_and_semantic(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    anchor_paths: &[String],
    history_mode: HistoryMode,
    semantic_enabled: bool,
    semantic_provider: SemanticProviderConfig,
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
    let mut provider_policy = provider_policy_report(repo_root)?;
    extend_plan_diagnostics(&mut plan, provider_policy.diagnostics.clone());
    let mut query_trace = construct_query_trace(task, anchor_paths);
    let mut combined_anchor_paths = anchor_paths.to_vec();
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
    let symbol_query = query_text_or_task(&query_trace.retriever_queries.symbol_terms, task);
    let lexical_query = query_text_or_task(&query_trace.retriever_queries.lexical_terms, task);
    let semantic_query = query_text_or_task(&query_trace.retriever_queries.semantic_phrases, task);

    let symbol_report = symbol_search_report(
        repo_root,
        &symbol_query,
        &SymbolOptions {
            limit: search_limit(&plan.task_type),
        },
    )?;
    extend_plan_diagnostics(&mut plan, symbol_report.diagnostics);
    let symbol_results = symbol_report.results;

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
    let semantic_provider = normalized_provider(&semantic_provider);
    let semantic_decision =
        semantic_provider_decision(&provider_policy, &semantic_provider, semantic_enabled);
    provider_policy.decisions.push(semantic_decision.clone());
    let semantic_results = if semantic_enabled
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
        if semantic_enabled {
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

    if multi_area_task {
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
        roles,
        expansion_seeds: expansion_seed_paths,
    });
    let reranker = reranker_decision(&provider_policy);
    provider_policy.decisions.push(reranker.clone());
    let ranked_candidates = if matches!(reranker.status, ProviderDecisionStatus::Allowed) {
        push_plan_diagnostic(
            &mut plan,
            Diagnostic {
                code: "local_metadata_reranker_applied".to_string(),
                severity: DiagnosticSeverity::Info,
                message: "Applied deterministic local metadata reranker using source-free candidate metadata.".to_string(),
                paths: Vec::new(),
                count: ranked_candidates.len(),
            },
        );
        rerank_with_local_metadata(ranked_candidates)
    } else {
        ranked_candidates
    };
    let selection = select_ranked_candidates_for_scope(
        &ranked_candidates,
        PREPARE_TASK_TARGET_LIMIT,
        PREPARE_TASK_TEST_LIMIT,
        multi_area_task,
    );
    plan.context_areas = context_areas_for_plan(&selection, multi_area_task);
    plan.target_files = selection.target_files;
    plan.related_tests = selection.related_tests;
    plan.recommended_commands = selection.recommended_commands;
    plan.retrieval_candidates = selection.retrieval_candidates;
    maybe_add_broad_validation_command(task, &mut plan);
    attach_selected_memory(repo_root, task, &mut plan);
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

fn related_test_seed_paths(
    source_seed_paths: &[String],
    co_change_hints: &[ctxpack_index::CoChangeHint],
    dependency_edges: &[ctxpack_index::DependencyEdge],
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
    if !multi_area_task {
        return Vec::new();
    }

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
            Some(FileRole::Source | FileRole::Test | FileRole::Config | FileRole::Schema)
        ) {
            continue;
        }
        let area = context_area_for_path(path);
        let entry = areas.entry(area).or_default();
        entry.record_role(&candidate.role);
        if entry.unique_paths.insert(path.to_string()) {
            entry.candidate_count += 1;
            if entry.representative_paths.len() < 3 {
                entry.representative_paths.push(path.to_string());
            }
            if selected_paths.contains(path) {
                entry.selected_count += 1;
                entry.record_selected_role(&candidate.role);
            }
        }
    }

    let mut areas = areas
        .into_iter()
        .map(|(area, accumulator)| {
            let context_area = ContextArea {
                area: area.clone(),
                reason: format!(
                    "Broad task candidate area with {} candidate path(s) and {} selected path(s).",
                    accumulator.candidate_count, accumulator.selected_count
                ),
                representative_paths: accumulator.representative_paths.clone(),
                candidate_count: accumulator.candidate_count,
                selected_count: accumulator.selected_count,
            };
            RankedContextArea {
                context_area,
                accumulator,
            }
        })
        .collect::<Vec<_>>();
    areas.sort_by(|left, right| {
        context_area_priority(&left.context_area.area, &left.accumulator)
            .cmp(&context_area_priority(
                &right.context_area.area,
                &right.accumulator,
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
            .then_with(|| left.context_area.area.cmp(&right.context_area.area))
    });
    let mut areas = areas
        .into_iter()
        .map(|area| area.context_area)
        .collect::<Vec<_>>();
    areas.truncate(MAX_CONTEXT_AREAS);
    areas
}

struct RankedContextArea {
    context_area: ContextArea,
    accumulator: AreaAccumulator,
}

fn context_area_priority(area: &str, accumulator: &AreaAccumulator) -> u8 {
    if area == "examples" || area.starts_with("examples/") {
        return 2;
    }
    if accumulator.source_count + accumulator.config_count + accumulator.schema_count > 0 {
        return 0;
    }
    if accumulator.test_count > 0 {
        return 1;
    }
    3
}

const MAX_CONTEXT_AREAS: usize = 16;

#[derive(Default)]
struct AreaAccumulator {
    unique_paths: BTreeSet<String>,
    representative_paths: Vec<String>,
    candidate_count: usize,
    selected_count: usize,
    source_count: usize,
    test_count: usize,
    config_count: usize,
    schema_count: usize,
    selected_source_count: usize,
    selected_config_count: usize,
    selected_schema_count: usize,
}

impl AreaAccumulator {
    fn record_role(&mut self, role: &Option<FileRole>) {
        match role {
            Some(FileRole::Source) => self.source_count += 1,
            Some(FileRole::Test) => self.test_count += 1,
            Some(FileRole::Config) => self.config_count += 1,
            Some(FileRole::Schema) => self.schema_count += 1,
            _ => {}
        }
    }

    fn record_selected_role(&mut self, role: &Option<FileRole>) {
        match role {
            Some(FileRole::Source) => self.selected_source_count += 1,
            Some(FileRole::Config) => self.selected_config_count += 1,
            Some(FileRole::Schema) => self.selected_schema_count += 1,
            _ => {}
        }
    }

    fn source_like_count(&self) -> usize {
        self.source_count + self.config_count + self.schema_count
    }

    fn selected_source_like_count(&self) -> usize {
        self.selected_source_count + self.selected_config_count + self.selected_schema_count
    }
}

pub(crate) fn context_area_for_path(path: &str) -> String {
    let mut components = path
        .split('/')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.is_empty() {
        return ".".to_string();
    }
    if components[0].starts_with('.') {
        if components.len() >= 2 {
            return format!("{}/{}", components[0], components[1]);
        }
        return components[0].to_string();
    }
    if components.len() == 1 {
        return ".".to_string();
    }
    if components.len() == 2 && components[1].contains('.') {
        return components[0].to_string();
    }
    components.truncate(2);
    components.join("/")
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
    co_change_hints: &[ctxpack_index::CoChangeHint],
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

fn is_broad_validation_task(task: &str, related_tests: &[ctxpack_core::RelatedTest]) -> bool {
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

    related_tests.len() >= PREPARE_TASK_TEST_LIMIT && test_area_count >= 3
        || broad_term_count >= 2 && test_area_count >= 3
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
    task.split(|character: char| {
        !(character.is_ascii_alphanumeric()
            || character == '_'
            || character == ':'
            || character == '.')
    })
    .map(clean_query_token)
    .filter(|token| {
        token.len() >= 3
            && (token.contains("::")
                || token.contains('.')
                || token
                    .chars()
                    .any(|character| character.is_ascii_uppercase()))
    })
    .take(16)
    .collect()
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
    if !is_project_governance_task(task) {
        return;
    }

    let mut seen = search_results
        .iter()
        .map(|result| result.path.clone())
        .collect::<BTreeSet<_>>();
    for (index, path) in project_governance_doc_paths().into_iter().enumerate() {
        let Some(role) = roles.get(path).cloned() else {
            continue;
        };
        if !matches!(role, FileRole::Docs | FileRole::Config | FileRole::Unknown) {
            continue;
        }
        if !seen.insert(path.to_string()) {
            continue;
        }
        search_results.push(SearchResult {
            path: path.to_string(),
            role,
            language: language_for_governance_doc(path),
            score: 16.0 - index.min(10) as f32,
            reason: "project governance artifact for planning/eval/release task".to_string(),
        });
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
                | "evaluation"
                | "gate"
                | "gates"
                | "milestone"
                | "milestones"
                | "phase"
                | "phases"
                | "planning"
                | "proof"
                | "recall"
                | "release"
                | "retrieval"
                | "roadmap"
                | "validation"
        )
    })
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
        query_trace: None,
        provider_policy: None,
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

pub(crate) fn is_multi_area_task(task: &str) -> bool {
    let terms = terms(task);
    if terms.is_empty() {
        return false;
    }
    let broad_term_count = [
        "ci",
        "eval",
        "evaluation",
        "harden",
        "lint",
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
            false,
            SemanticProviderConfig::default(),
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
        let lexical = ctxpack_index::SearchResult {
            path: "src/exact.ts".to_string(),
            role: FileRole::Source,
            language: Some("typescript".to_string()),
            score: 12.0,
            reason: "term match".to_string(),
        };
        let semantic = ctxpack_index::SemanticSearchResult {
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
            path: "crates/ctxpack-compiler/src/eval.rs".to_string(),
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
            ("src/auth.ts".to_string(), FileRole::Source),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        extend_project_governance_docs(
            &mut search_results,
            &roles,
            "measure retrieval recall in release proof",
        );
        let paths = search_results
            .iter()
            .map(|result| result.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&".planning/STATE.md"));
        assert!(paths.contains(&".planning/ROADMAP.md"));
        assert!(paths.contains(&".planning/MILESTONES.md"));
        assert!(paths.contains(&"docs/benchmarking.md"));
        assert!(!paths.contains(&"src/auth.ts"));
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
            true,
            SemanticProviderConfig::default(),
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
    fn broad_validation_tasks_add_suite_fallback_command() {
        let mut plan = base_plan(TaskType::BugFix);
        plan.related_tests = [
            "tests/evaluation/test_pvldb_eval_runner.py",
            "tests/agents/test_base_agent_openai_compatible.py",
            "tests/core/test_retry_routing.py",
            "tests/gates/test_quality_gate_system_singleton.py",
        ]
        .into_iter()
        .map(|path| ctxpack_core::RelatedTest {
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
}
