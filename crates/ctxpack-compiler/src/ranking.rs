use ctxpack_core::{
    Command, FileRole, LineRange, RelatedTest, RetrievalCandidate, RetrievalCandidateKind,
    RetrievalEvidence, RetrievalSignalKind, RetrievalSignalScore, TargetFile,
};
use ctxpack_index::{
    CoChangeHint, DependencyEdge, RelatedTestResult, SearchResult, SemanticSearchResult,
    SymbolSearchResult,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnchorCandidate {
    pub path: String,
    pub role: FileRole,
    pub current_diff: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RankingInput {
    pub anchors: Vec<AnchorCandidate>,
    pub lexical_results: Vec<SearchResult>,
    pub protected_lexical_limit: Option<usize>,
    pub semantic_results: Vec<SemanticSearchResult>,
    pub symbol_results: Vec<SymbolSearchResult>,
    pub related_tests: Vec<RelatedTestResult>,
    pub co_change_hints: Vec<CoChangeHint>,
    pub dependency_edges: Vec<DependencyEdge>,
    pub roles: BTreeMap<String, FileRole>,
    pub expansion_seeds: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct RankedSelection {
    pub retrieval_candidates: Vec<RetrievalCandidate>,
    pub target_files: Vec<TargetFile>,
    pub related_tests: Vec<RelatedTest>,
    pub recommended_commands: Vec<Command>,
}

pub(crate) fn rank_candidates(input: RankingInput) -> Vec<RankedCandidate> {
    let seed_paths = seed_paths(&input);
    let mut candidates = CandidateAccumulator::default();

    for anchor in input.anchors {
        let kind = candidate_kind_for_role(&anchor.role);
        let evidence_signal = if anchor.current_diff {
            RetrievalSignalKind::CurrentDiff
        } else {
            RetrievalSignalKind::Anchor
        };
        let reason_code = if anchor.current_diff {
            "current_diff_anchor"
        } else {
            "path_anchor"
        };
        candidates.add_path_signal(PathSignal {
            kind,
            path: anchor.path,
            role: anchor.role,
            signal: evidence_signal.clone(),
            score: 1.0,
            weight: signal_weight(&evidence_signal),
            reason_code,
            edge_label: None,
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range: None,
            command: None,
            lexical_rank: None,
        });
    }

    for (lexical_rank, result) in input.lexical_results.into_iter().enumerate() {
        let kind = candidate_kind_for_role(&result.role);
        let role = result.role;
        let path = result.path;
        let score = normalize_signal_score(result.score);
        let signal = if input
            .protected_lexical_limit
            .is_some_and(|limit| lexical_rank >= limit)
        {
            RetrievalSignalKind::LexicalExpansion
        } else {
            RetrievalSignalKind::Lexical
        };
        candidates.add_path_signal(PathSignal {
            kind,
            path,
            role,
            signal: signal.clone(),
            score,
            weight: signal_weight(&signal),
            reason_code: "lexical_match",
            edge_label: None,
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range: None,
            command: None,
            lexical_rank: Some(lexical_rank),
        });
    }

    for result in input.semantic_results {
        let kind = candidate_kind_for_role(&result.role);
        let facet_label = semantic_facet_label(&result);
        candidates.add_path_signal(PathSignal {
            kind,
            path: result.path,
            role: result.role,
            signal: RetrievalSignalKind::Semantic,
            score: result.score.clamp(0.05, 0.95),
            weight: signal_weight(&RetrievalSignalKind::Semantic),
            reason_code: "semantic_match",
            edge_label: Some(facet_label),
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range: None,
            command: None,
            lexical_rank: None,
        });
    }

    for result in input.symbol_results {
        let path = result.symbol.path;
        let role = role_for_path(&input.roles, &path);
        let score = normalize_signal_score(result.score);
        let line_range = Some(LineRange {
            start: result.symbol.start_line,
            end: result.symbol.end_line.max(result.symbol.start_line),
        });
        candidates.add_path_signal(PathSignal {
            kind: candidate_kind_for_role(&role),
            path: path.clone(),
            role: role.clone(),
            signal: RetrievalSignalKind::Symbol,
            score,
            weight: signal_weight(&RetrievalSignalKind::Symbol),
            reason_code: "symbol_match",
            edge_label: None,
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range: line_range.clone(),
            command: None,
            lexical_rank: None,
        });
        candidates.add_path_signal(PathSignal {
            kind: RetrievalCandidateKind::Symbol,
            path,
            role,
            signal: RetrievalSignalKind::Symbol,
            score,
            weight: signal_weight(&RetrievalSignalKind::Symbol),
            reason_code: "symbol_match",
            edge_label: None,
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range,
            command: None,
            lexical_rank: None,
        });
    }

    for edge in input.dependency_edges {
        let source_is_seed = seed_paths.contains(&edge.source_path);
        let target_is_seed = seed_paths.contains(&edge.target_path);
        if !source_is_seed && !target_is_seed {
            continue;
        }
        let path = if source_is_seed {
            edge.target_path
        } else {
            edge.source_path
        };
        let role = role_for_path(&input.roles, &path);
        candidates.add_path_signal(PathSignal {
            kind: candidate_kind_for_role(&role),
            path,
            role,
            signal: RetrievalSignalKind::Dependency,
            score: edge.confidence,
            weight: signal_weight(&RetrievalSignalKind::Dependency),
            reason_code: "dependency_neighbor",
            edge_label: Some(edge.kind),
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range: None,
            command: None,
            lexical_rank: None,
        });
    }

    for test in input.related_tests {
        let role = FileRole::Test;
        candidates.add_path_signal(PathSignal {
            kind: RetrievalCandidateKind::Test,
            path: test.path,
            role,
            signal: RetrievalSignalKind::RelatedTest,
            score: test.confidence,
            weight: signal_weight(&RetrievalSignalKind::RelatedTest),
            reason_code: "related_test",
            edge_label: None,
            commit_ids: Vec::new(),
            commit_count: 0,
            line_range: None,
            command: test.command,
            lexical_rank: None,
        });
    }

    let mut commit_ids = BTreeSet::new();
    let mut commit_count = 0;
    for hint in input.co_change_hints {
        let path = hint.path;
        let role = role_for_path(&input.roles, &path);
        let sample_commits = hint.sample_commits;
        commit_count += hint.commit_count as u32;
        commit_ids.extend(sample_commits.iter().cloned());
        candidates.add_path_signal(PathSignal {
            kind: candidate_kind_for_role(&role),
            path,
            role: role.clone(),
            signal: RetrievalSignalKind::CoChange,
            score: co_change_score_for_role(hint.confidence, &role),
            weight: signal_weight(&RetrievalSignalKind::CoChange),
            reason_code: "co_change_neighbor",
            edge_label: None,
            commit_ids: sample_commits,
            commit_count: hint.commit_count as u32,
            line_range: None,
            command: None,
            lexical_rank: None,
        });
    }

    if !commit_ids.is_empty() {
        candidates.add_commit_signal(
            commit_ids.into_iter().collect(),
            commit_count,
            RetrievalSignalKind::History,
        );
    }

    candidates.finish()
}

fn co_change_score_for_role(confidence: f32, role: &FileRole) -> f32 {
    if matches!(role, FileRole::Test) {
        (confidence + 0.35).min(0.95)
    } else {
        confidence
    }
}

pub(crate) fn rerank_with_local_metadata(
    mut candidates: Vec<RankedCandidate>,
) -> Vec<RankedCandidate> {
    candidates.sort_by(|left, right| {
        local_metadata_rerank_score(right)
            .total_cmp(&local_metadata_rerank_score(left))
            .then_with(|| right.rank_score.total_cmp(&left.rank_score))
            .then_with(|| left.candidate.path.cmp(&right.candidate.path))
    });
    candidates
}

fn local_metadata_rerank_score(candidate: &RankedCandidate) -> f32 {
    let evidence_score = candidate.candidate.evidence.len() as f32 * 0.05;
    let exact_score = candidate
        .candidate
        .signal_scores
        .iter()
        .filter(|score| {
            matches!(
                score.signal,
                RetrievalSignalKind::Anchor
                    | RetrievalSignalKind::CurrentDiff
                    | RetrievalSignalKind::Lexical
                    | RetrievalSignalKind::LexicalExpansion
                    | RetrievalSignalKind::Symbol
            )
        })
        .map(|score| score.score * score.weight)
        .sum::<f32>();
    candidate.rank_score + exact_score + evidence_score
}

fn semantic_facet_label(result: &SemanticSearchResult) -> String {
    let facets = result
        .matched_facets
        .iter()
        .map(|facet| facet.label.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(4)
        .collect::<Vec<_>>()
        .join(",");
    let precision = result
        .precision_status
        .as_ref()
        .map(|status| format!(":{status:?}").to_ascii_lowercase())
        .unwrap_or_default();
    if facets.is_empty() {
        format!(
            "{}:{}:{}{}",
            result.provider.provider,
            result.provider.model,
            result.provider.distance_metric,
            precision
        )
    } else {
        format!(
            "{}:{}:{}:{}{}",
            result.provider.provider,
            result.provider.model,
            result.provider.distance_metric,
            facets,
            precision
        )
    }
}

pub(crate) fn select_ranked_candidates(
    candidates: &[RankedCandidate],
    file_budget: usize,
    test_budget: usize,
) -> RankedSelection {
    let retrieval_candidates = candidates
        .iter()
        .map(|candidate| candidate.candidate.clone())
        .collect::<Vec<_>>();
    let target_files = select_target_files(candidates, file_budget);
    let related_tests = select_related_tests(candidates, test_budget);
    let mut command_set = BTreeSet::new();
    let recommended_commands = related_tests
        .iter()
        .filter_map(|test| test.command.clone())
        .filter(|command| command_set.insert(command.clone()))
        .map(|command| Command {
            command,
            reason: "targeted validation for related test".to_string(),
        })
        .collect::<Vec<_>>();

    RankedSelection {
        retrieval_candidates,
        target_files,
        related_tests,
        recommended_commands,
    }
}

fn select_related_tests(candidates: &[RankedCandidate], test_budget: usize) -> Vec<RelatedTest> {
    if test_budget == 0 {
        return Vec::new();
    }

    let mut selected = Vec::new();
    let mut selected_paths = BTreeSet::new();
    for candidate in candidates.iter().filter(|candidate| {
        candidate.related_test.is_some()
            && signal_score(&candidate.candidate, RetrievalSignalKind::CoChange).is_some()
    }) {
        push_related_test(candidate, &mut selected, &mut selected_paths, test_budget);
    }
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.related_test.is_some())
    {
        push_related_test(candidate, &mut selected, &mut selected_paths, test_budget);
    }
    selected
}

fn push_related_test(
    candidate: &RankedCandidate,
    selected: &mut Vec<RelatedTest>,
    selected_paths: &mut BTreeSet<String>,
    test_budget: usize,
) {
    if selected.len() >= test_budget {
        return;
    }
    let Some(test) = &candidate.related_test else {
        return;
    };
    if selected_paths.insert(test.path.clone()) {
        selected.push(test.clone());
    }
}

fn select_target_files(candidates: &[RankedCandidate], file_budget: usize) -> Vec<TargetFile> {
    if file_budget == 0 {
        return Vec::new();
    }

    let mut selected = Vec::new();
    let mut selected_paths = BTreeSet::new();
    let has_active_context = candidates.iter().any(|candidate| {
        signal_score(&candidate.candidate, RetrievalSignalKind::Anchor).is_some()
            || signal_score(&candidate.candidate, RetrievalSignalKind::CurrentDiff).is_some()
    });

    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.target_file.is_some())
        .filter(|candidate| {
            signal_score(&candidate.candidate, RetrievalSignalKind::Anchor).is_some()
                || signal_score(&candidate.candidate, RetrievalSignalKind::CurrentDiff).is_some()
        })
    {
        push_target(candidate, &mut selected, &mut selected_paths, file_budget);
    }

    if !has_active_context {
        for (_, candidate) in source_lexical_floor(candidates)
            .into_iter()
            .take(source_lexical_floor_limit(file_budget))
        {
            push_target(candidate, &mut selected, &mut selected_paths, file_budget);
        }
        for (_, candidate) in governance_doc_floor(candidates)
            .into_iter()
            .take(governance_doc_floor_limit(file_budget))
        {
            push_target(candidate, &mut selected, &mut selected_paths, file_budget);
        }
    }

    let mut lexical_floor = candidates
        .iter()
        .filter(|candidate| candidate.target_file.is_some())
        .filter_map(|candidate| {
            let lexical_score = signal_score(&candidate.candidate, RetrievalSignalKind::Lexical)?;
            let minimum_lexical_score = if has_active_context { 0.90 } else { 0.0 };
            if lexical_score < minimum_lexical_score {
                return None;
            }
            Some((lexical_score, candidate))
        })
        .collect::<Vec<_>>();
    lexical_floor.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left.lexical_rank.cmp(&right.lexical_rank))
            .then_with(|| right.rank_score.total_cmp(&left.rank_score))
            .then_with(|| left.candidate.path.cmp(&right.candidate.path))
    });

    let symbol_floor_reserve = if has_active_context
        || !has_symbol_target_candidates(candidates)
        || !has_archive_lexical_target_candidates(candidates)
    {
        0
    } else {
        symbol_floor_reserve_limit(file_budget)
    };
    let lexical_floor_limit = if has_active_context {
        7.min(file_budget)
    } else if symbol_floor_reserve == 0 {
        file_budget
    } else {
        file_budget
            .saturating_sub(symbol_floor_reserve + selected.len())
            .max(1)
    };
    for (_, candidate) in lexical_floor.into_iter().take(lexical_floor_limit) {
        push_target(candidate, &mut selected, &mut selected_paths, file_budget);
    }

    if selected.len() < file_budget {
        let mut symbol_floor = candidates
            .iter()
            .filter(|candidate| candidate.target_file.is_some())
            .filter_map(|candidate| {
                let symbol_score = signal_score(&candidate.candidate, RetrievalSignalKind::Symbol)?;
                Some((symbol_score, candidate))
            })
            .collect::<Vec<_>>();
        symbol_floor.sort_by(|(left_score, left), (right_score, right)| {
            right_score
                .total_cmp(left_score)
                .then_with(|| right.rank_score.total_cmp(&left.rank_score))
                .then_with(|| left.candidate.path.cmp(&right.candidate.path))
        });
        let symbol_floor_limit = symbol_floor_limit(file_budget, has_active_context);
        for (_, candidate) in symbol_floor.into_iter().take(symbol_floor_limit) {
            push_target(candidate, &mut selected, &mut selected_paths, file_budget);
        }
    }

    if !has_active_context && selected.len() < file_budget {
        for (_, candidate) in source_lexical_floor(candidates)
            .into_iter()
            .take(source_lexical_floor_limit(file_budget))
        {
            push_target(candidate, &mut selected, &mut selected_paths, file_budget);
        }
    }
    let mut history_floor = candidates
        .iter()
        .filter(|candidate| candidate.target_file.is_some())
        .filter_map(|candidate| {
            let history_score = signal_score(&candidate.candidate, RetrievalSignalKind::CoChange)?;
            if history_score < 0.50 {
                return None;
            }
            Some((history_score, candidate))
        })
        .collect::<Vec<_>>();
    history_floor.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| right.rank_score.total_cmp(&left.rank_score))
            .then_with(|| left.candidate.path.cmp(&right.candidate.path))
    });
    for (_, candidate) in history_floor.into_iter().take(4) {
        push_target(candidate, &mut selected, &mut selected_paths, file_budget);
    }
    for candidate in candidates {
        if selected.len() >= file_budget {
            break;
        }
        push_target(candidate, &mut selected, &mut selected_paths, file_budget);
    }

    selected
}

fn source_lexical_floor(candidates: &[RankedCandidate]) -> Vec<(f32, &RankedCandidate)> {
    let mut source_lexical_floor = candidates
        .iter()
        .filter(|candidate| candidate.target_file.is_some())
        .filter(|candidate| candidate.candidate.role == Some(FileRole::Source))
        .filter_map(|candidate| {
            let lexical_score = lexical_signal_score(&candidate.candidate)?;
            if lexical_score < 0.30 {
                return None;
            }
            Some((lexical_score, candidate))
        })
        .collect::<Vec<_>>();
    source_lexical_floor.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left.lexical_rank.cmp(&right.lexical_rank))
            .then_with(|| right.rank_score.total_cmp(&left.rank_score))
            .then_with(|| left.candidate.path.cmp(&right.candidate.path))
    });
    source_lexical_floor
}

fn source_lexical_floor_limit(file_budget: usize) -> usize {
    file_budget.div_ceil(2).clamp(1, 4)
}

fn has_symbol_target_candidates(candidates: &[RankedCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.target_file.is_some()
            && signal_score(&candidate.candidate, RetrievalSignalKind::Symbol).is_some()
    })
}

fn has_archive_lexical_target_candidates(candidates: &[RankedCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.target_file.is_some()
            && lexical_signal_score(&candidate.candidate).is_some()
            && candidate
                .candidate
                .path
                .as_deref()
                .is_some_and(is_archive_context_artifact_path)
    })
}

fn is_archive_context_artifact_path(path: &str) -> bool {
    path.starts_with(".planning/milestones/")
        || (path.starts_with(".planning/e2e/") && path.ends_with(".json"))
}

fn symbol_floor_limit(file_budget: usize, has_active_context: bool) -> usize {
    if has_active_context {
        3.min(file_budget)
    } else {
        file_budget.div_ceil(2).clamp(1, 4)
    }
}

fn symbol_floor_reserve_limit(file_budget: usize) -> usize {
    file_budget.div_ceil(5).clamp(1, 2)
}

fn governance_doc_floor(candidates: &[RankedCandidate]) -> Vec<(f32, &RankedCandidate)> {
    let has_project_planning_docs = candidates.iter().any(|candidate| {
        candidate
            .candidate
            .path
            .as_deref()
            .is_some_and(is_root_planning_doc_path)
    });
    if !has_project_planning_docs {
        return Vec::new();
    }

    let mut governance_floor = candidates
        .iter()
        .filter(|candidate| candidate.target_file.is_some())
        .filter(|candidate| {
            candidate
                .candidate
                .path
                .as_deref()
                .is_some_and(is_root_governance_doc_path)
        })
        .filter_map(|candidate| {
            let lexical_score = lexical_signal_score(&candidate.candidate)?;
            if lexical_score < 0.30 {
                return None;
            }
            Some((lexical_score, candidate))
        })
        .collect::<Vec<_>>();
    governance_floor.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left.lexical_rank.cmp(&right.lexical_rank))
            .then_with(|| left.candidate.path.cmp(&right.candidate.path))
    });
    governance_floor
}

fn governance_doc_floor_limit(file_budget: usize) -> usize {
    (file_budget / 4).clamp(1, 3)
}

fn is_root_governance_doc_path(path: &str) -> bool {
    matches!(
        path,
        ".planning/STATE.md"
            | ".planning/ROADMAP.md"
            | ".planning/MILESTONES.md"
            | ".planning/REQUIREMENTS.md"
            | ".planning/PROJECT.md"
            | "AGENTS.md"
            | "README.md"
            | "docs/benchmarking.md"
            | "docs/release.md"
            | "docs/semantic.md"
    )
}

fn is_root_planning_doc_path(path: &str) -> bool {
    matches!(
        path,
        ".planning/STATE.md"
            | ".planning/ROADMAP.md"
            | ".planning/MILESTONES.md"
            | ".planning/REQUIREMENTS.md"
            | ".planning/PROJECT.md"
    )
}

fn push_target(
    candidate: &RankedCandidate,
    selected: &mut Vec<TargetFile>,
    selected_paths: &mut BTreeSet<String>,
    file_budget: usize,
) {
    if selected.len() >= file_budget {
        return;
    }
    let Some(target) = candidate.target_file.clone() else {
        return;
    };
    if selected_paths.insert(target.path.clone()) {
        selected.push(target);
    }
}

fn signal_score(candidate: &RetrievalCandidate, signal: RetrievalSignalKind) -> Option<f32> {
    candidate
        .signal_scores
        .iter()
        .find(|score| score.signal == signal)
        .map(|score| score.score * score.weight)
}

fn lexical_signal_score(candidate: &RetrievalCandidate) -> Option<f32> {
    [
        RetrievalSignalKind::Lexical,
        RetrievalSignalKind::LexicalExpansion,
    ]
    .into_iter()
    .filter_map(|signal| signal_score(candidate, signal))
    .max_by(|left, right| left.total_cmp(right))
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RankedCandidate {
    pub candidate: RetrievalCandidate,
    pub target_file: Option<TargetFile>,
    pub related_test: Option<RelatedTest>,
    pub rank_score: f32,
    pub lexical_rank: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateKey {
    kind_rank: u8,
    path: Option<String>,
}

#[derive(Debug, Clone)]
struct CandidateBuilder {
    kind: RetrievalCandidateKind,
    path: Option<String>,
    role: Option<FileRole>,
    reason_code: String,
    signal_scores: Vec<RetrievalSignalScore>,
    evidence: Vec<RetrievalEvidence>,
    target_file: Option<TargetFile>,
    related_test: Option<RelatedTest>,
    rank_score: f32,
    lexical_rank: Option<usize>,
}

#[derive(Default)]
struct CandidateAccumulator {
    candidates: BTreeMap<CandidateKey, CandidateBuilder>,
}

struct PathSignal {
    kind: RetrievalCandidateKind,
    path: String,
    role: FileRole,
    signal: RetrievalSignalKind,
    score: f32,
    weight: f32,
    reason_code: &'static str,
    edge_label: Option<String>,
    commit_ids: Vec<String>,
    commit_count: u32,
    line_range: Option<LineRange>,
    command: Option<String>,
    lexical_rank: Option<usize>,
}

impl CandidateAccumulator {
    fn add_path_signal(&mut self, signal: PathSignal) {
        let weighted_score = signal.score * signal.weight;
        let key = CandidateKey {
            kind_rank: kind_rank(&signal.kind),
            path: Some(signal.path.clone()),
        };
        let builder = self
            .candidates
            .entry(key)
            .or_insert_with(|| CandidateBuilder {
                kind: signal.kind.clone(),
                path: Some(signal.path.clone()),
                role: Some(signal.role.clone()),
                reason_code: signal.reason_code.to_string(),
                signal_scores: Vec::new(),
                evidence: Vec::new(),
                target_file: target_file_for(&signal, weighted_score),
                related_test: related_test_for(&signal, weighted_score),
                rank_score: 0.0,
                lexical_rank: signal.lexical_rank,
            });
        builder.lexical_rank = match (builder.lexical_rank, signal.lexical_rank) {
            (Some(existing), Some(incoming)) => Some(existing.min(incoming)),
            (None, Some(incoming)) => Some(incoming),
            (existing, None) => existing,
        };
        builder.rank_score += weighted_score;
        merge_signal_score(
            &mut builder.signal_scores,
            signal.signal.clone(),
            signal.score,
            signal.weight,
        );
        builder.evidence.push(RetrievalEvidence {
            signal: signal.signal.clone(),
            score: signal.score,
            reason_code: signal.reason_code.to_string(),
            path: Some(signal.path.clone()),
            role: Some(signal.role.clone()),
            edge_label: signal.edge_label,
            commit_ids: signal.commit_ids.clone(),
            commit_count: signal.commit_count,
        });
        if matches!(
            builder.kind,
            RetrievalCandidateKind::Doc | RetrievalCandidateKind::Config
        ) {
            let boost_signal = match builder.kind {
                RetrievalCandidateKind::Doc => RetrievalSignalKind::Docs,
                RetrievalCandidateKind::Config => RetrievalSignalKind::Config,
                _ => unreachable!(),
            };
            merge_signal_score(
                &mut builder.signal_scores,
                boost_signal.clone(),
                1.0,
                signal_weight(&boost_signal),
            );
            if !builder.evidence.iter().any(|evidence| {
                evidence.signal == boost_signal && evidence.reason_code == "role_boost"
            }) {
                builder.evidence.push(RetrievalEvidence {
                    signal: boost_signal,
                    score: 1.0,
                    reason_code: "role_boost".to_string(),
                    path: Some(signal.path.clone()),
                    role: Some(signal.role.clone()),
                    edge_label: None,
                    commit_ids: Vec::new(),
                    commit_count: 0,
                });
            }
        }
        if let Some(target) = &mut builder.target_file {
            target.confidence = score_to_confidence(builder.rank_score);
            if target.line_range.is_none() {
                target.line_range = signal.line_range.clone();
            }
            target.attribution = builder.evidence.clone();
        }
        if let Some(test) = &mut builder.related_test {
            test.confidence = score_to_confidence(builder.rank_score);
            if test.command.is_none() {
                test.command = signal.command;
            }
            test.attribution = builder.evidence.clone();
        }
    }

    fn add_commit_signal(
        &mut self,
        commit_ids: Vec<String>,
        commit_count: u32,
        signal: RetrievalSignalKind,
    ) {
        let key = CandidateKey {
            kind_rank: kind_rank(&RetrievalCandidateKind::Commit),
            path: None,
        };
        let score = (commit_count as f32 / 5.0).clamp(0.1, 0.95);
        let weight = signal_weight(&signal);
        let builder = self
            .candidates
            .entry(key)
            .or_insert_with(|| CandidateBuilder {
                kind: RetrievalCandidateKind::Commit,
                path: None,
                role: None,
                reason_code: "history_commit".to_string(),
                signal_scores: Vec::new(),
                evidence: Vec::new(),
                target_file: None,
                related_test: None,
                rank_score: 0.0,
                lexical_rank: None,
            });
        builder.rank_score += score * weight;
        merge_signal_score(&mut builder.signal_scores, signal.clone(), score, weight);
        builder.evidence.push(RetrievalEvidence {
            signal,
            score,
            reason_code: "history_commit".to_string(),
            path: None,
            role: None,
            edge_label: None,
            commit_ids,
            commit_count,
        });
    }

    fn finish(self) -> Vec<RankedCandidate> {
        let mut candidates = self
            .candidates
            .into_values()
            .map(|mut builder| {
                sort_signal_scores(&mut builder.signal_scores);
                sort_evidence(&mut builder.evidence);
                if let Some(target) = &mut builder.target_file {
                    target.attribution = builder.evidence.clone();
                }
                if let Some(test) = &mut builder.related_test {
                    test.attribution = builder.evidence.clone();
                }
                RankedCandidate {
                    candidate: RetrievalCandidate {
                        kind: builder.kind,
                        path: builder.path,
                        role: builder.role,
                        reason_code: builder.reason_code,
                        confidence: score_to_confidence(builder.rank_score),
                        signal_scores: builder.signal_scores,
                        evidence: builder.evidence,
                    },
                    target_file: builder.target_file,
                    related_test: builder.related_test,
                    rank_score: builder.rank_score,
                    lexical_rank: builder.lexical_rank,
                }
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            right
                .rank_score
                .total_cmp(&left.rank_score)
                .then_with(|| {
                    kind_rank(&left.candidate.kind).cmp(&kind_rank(&right.candidate.kind))
                })
                .then_with(|| left.candidate.path.cmp(&right.candidate.path))
        });
        candidates
    }
}

fn target_file_for(signal: &PathSignal, weighted_score: f32) -> Option<TargetFile> {
    if !matches!(
        signal.kind,
        RetrievalCandidateKind::File | RetrievalCandidateKind::Doc | RetrievalCandidateKind::Config
    ) {
        return None;
    }
    Some(TargetFile {
        path: signal.path.clone(),
        reason: target_reason(signal.reason_code).to_string(),
        line_range: signal.line_range.clone(),
        confidence: score_to_confidence(weighted_score),
        attribution: Vec::new(),
    })
}

fn related_test_for(signal: &PathSignal, weighted_score: f32) -> Option<RelatedTest> {
    if signal.kind != RetrievalCandidateKind::Test {
        return None;
    }
    Some(RelatedTest {
        path: signal.path.clone(),
        reason: related_test_reason(signal.reason_code).to_string(),
        command: signal.command.clone(),
        confidence: score_to_confidence(weighted_score),
        attribution: Vec::new(),
    })
}

fn merge_signal_score(
    scores: &mut Vec<RetrievalSignalScore>,
    signal: RetrievalSignalKind,
    score: f32,
    weight: f32,
) {
    if let Some(existing) = scores.iter_mut().find(|existing| existing.signal == signal) {
        if score * weight > existing.score * existing.weight {
            existing.score = score;
            existing.weight = weight;
        }
    } else {
        scores.push(RetrievalSignalScore {
            signal,
            score,
            weight,
        });
    }
}

fn sort_signal_scores(scores: &mut [RetrievalSignalScore]) {
    scores.sort_by(|left, right| {
        (right.score * right.weight)
            .total_cmp(&(left.score * left.weight))
            .then_with(|| signal_rank(&left.signal).cmp(&signal_rank(&right.signal)))
    });
}

fn sort_evidence(evidence: &mut [RetrievalEvidence]) {
    evidence.sort_by(|left, right| {
        (right.score * signal_weight(&right.signal))
            .total_cmp(&(left.score * signal_weight(&left.signal)))
            .then_with(|| signal_rank(&left.signal).cmp(&signal_rank(&right.signal)))
            .then_with(|| left.path.cmp(&right.path))
    });
}

fn seed_paths(input: &RankingInput) -> BTreeSet<String> {
    let explicit = input
        .expansion_seeds
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if !explicit.is_empty() {
        return explicit;
    }
    input
        .anchors
        .iter()
        .map(|anchor| anchor.path.clone())
        .chain(
            input
                .lexical_results
                .iter()
                .map(|result| result.path.clone()),
        )
        .chain(
            input
                .symbol_results
                .iter()
                .map(|result| result.symbol.path.clone()),
        )
        .collect()
}

fn role_for_path(roles: &BTreeMap<String, FileRole>, path: &str) -> FileRole {
    roles.get(path).cloned().unwrap_or(FileRole::Source)
}

fn candidate_kind_for_role(role: &FileRole) -> RetrievalCandidateKind {
    match role {
        FileRole::Test => RetrievalCandidateKind::Test,
        FileRole::Docs => RetrievalCandidateKind::Doc,
        FileRole::Config | FileRole::Schema => RetrievalCandidateKind::Config,
        _ => RetrievalCandidateKind::File,
    }
}

fn normalize_signal_score(score: f32) -> f32 {
    (score / 20.0).clamp(0.15, 0.95)
}

fn score_to_confidence(score: f32) -> f32 {
    score.clamp(0.15, 0.98)
}

fn signal_weight(signal: &RetrievalSignalKind) -> f32 {
    match signal {
        RetrievalSignalKind::Anchor => 2.00,
        RetrievalSignalKind::CurrentDiff => 2.00,
        RetrievalSignalKind::Symbol => 1.05,
        RetrievalSignalKind::Lexical => 1.00,
        RetrievalSignalKind::LexicalExpansion => 0.80,
        RetrievalSignalKind::Semantic => 0.45,
        RetrievalSignalKind::Dependency => 0.45,
        RetrievalSignalKind::RelatedTest => 0.90,
        RetrievalSignalKind::CoChange => 1.35,
        RetrievalSignalKind::History => 0.65,
        RetrievalSignalKind::Config => 0.25,
        RetrievalSignalKind::Docs => 0.20,
        RetrievalSignalKind::Memory => 0.25,
    }
}

fn kind_rank(kind: &RetrievalCandidateKind) -> u8 {
    match kind {
        RetrievalCandidateKind::File => 0,
        RetrievalCandidateKind::Test => 1,
        RetrievalCandidateKind::Symbol => 2,
        RetrievalCandidateKind::Config => 3,
        RetrievalCandidateKind::Doc => 4,
        RetrievalCandidateKind::Commit => 5,
        RetrievalCandidateKind::Memory => 6,
    }
}

fn signal_rank(signal: &RetrievalSignalKind) -> u8 {
    match signal {
        RetrievalSignalKind::Anchor => 0,
        RetrievalSignalKind::CurrentDiff => 1,
        RetrievalSignalKind::Symbol => 2,
        RetrievalSignalKind::Lexical => 3,
        RetrievalSignalKind::LexicalExpansion => 4,
        RetrievalSignalKind::Semantic => 5,
        RetrievalSignalKind::Dependency => 6,
        RetrievalSignalKind::RelatedTest => 7,
        RetrievalSignalKind::CoChange => 8,
        RetrievalSignalKind::History => 9,
        RetrievalSignalKind::Config => 10,
        RetrievalSignalKind::Docs => 11,
        RetrievalSignalKind::Memory => 12,
    }
}

fn target_reason(reason_code: &str) -> &str {
    match reason_code {
        "path_anchor" | "current_diff_anchor" => "explicit path anchor from active context",
        "symbol_match" => "symbol match",
        "lexical_match" => "lexical match",
        "semantic_match" => "local semantic match",
        "dependency_neighbor" => "dependency neighbor",
        "co_change_neighbor" => "co-change neighbor",
        _ => reason_code,
    }
}

fn related_test_reason(reason_code: &str) -> &str {
    match reason_code {
        "related_test" => "related test",
        "lexical_match" => "lexical test match",
        "co_change_neighbor" => "co-change related test",
        _ => reason_code,
    }
}

#[allow(dead_code)]
fn evidence_signals(candidate: &RetrievalCandidate) -> Vec<RetrievalSignalKind> {
    candidate
        .evidence
        .iter()
        .map(|evidence| evidence.signal.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctxpack_core::{RetrievalCandidateKind, RetrievalSignalKind, RetrievalSignalScore};
    use ctxpack_index::{CodeSymbol, SemanticProviderConfig, SymbolKind};

    #[test]
    fn ranking_merges_multiple_signals_for_same_path() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![SearchResult {
                path: "src/auth/session.ts".to_string(),
                role: FileRole::Source,
                language: Some("typescript".to_string()),
                score: 12.0,
                reason: "term match".to_string(),
            }],
            symbol_results: vec![SymbolSearchResult {
                symbol: CodeSymbol {
                    name: "requireSession".to_string(),
                    kind: SymbolKind::Function,
                    path: "src/auth/session.ts".to_string(),
                    language: Some("typescript".to_string()),
                    start_line: 7,
                    end_line: 9,
                    signature: "export function requireSession()".to_string(),
                    exported: true,
                },
                score: 15.0,
                reason: "symbol name match".to_string(),
            }],
            roles: roles([("src/auth/session.ts", FileRole::Source)]),
            ..RankingInput::default()
        });

        let file = candidates
            .iter()
            .find(|candidate| {
                candidate.candidate.kind == RetrievalCandidateKind::File
                    && candidate.candidate.path.as_deref() == Some("src/auth/session.ts")
            })
            .unwrap();

        assert_signals(
            &file.candidate.signal_scores,
            &[RetrievalSignalKind::Symbol, RetrievalSignalKind::Lexical],
        );
        assert_signals_from_evidence(
            &file.candidate.evidence,
            &[RetrievalSignalKind::Symbol, RetrievalSignalKind::Lexical],
        );
        assert_eq!(
            file.target_file.as_ref().unwrap().line_range,
            Some(LineRange { start: 7, end: 9 })
        );
    }

    #[test]
    fn ranking_expands_one_hop_without_recursing_from_neighbors() {
        let candidates = rank_candidates(RankingInput {
            anchors: vec![AnchorCandidate {
                path: "src/a.ts".to_string(),
                role: FileRole::Source,
                current_diff: false,
            }],
            dependency_edges: vec![
                DependencyEdge {
                    source_path: "src/a.ts".to_string(),
                    target_path: "src/b.ts".to_string(),
                    kind: "imports".to_string(),
                    confidence: 0.8,
                    reason: "typescript import".to_string(),
                },
                DependencyEdge {
                    source_path: "src/b.ts".to_string(),
                    target_path: "src/c.ts".to_string(),
                    kind: "imports".to_string(),
                    confidence: 0.9,
                    reason: "typescript import".to_string(),
                },
            ],
            related_tests: vec![RelatedTestResult {
                path: "tests/a.test.ts".to_string(),
                command: Some("pnpm test tests/a.test.ts".to_string()),
                confidence: 0.9,
                reason: "imports source".to_string(),
            }],
            co_change_hints: vec![CoChangeHint {
                path: "src/history.ts".to_string(),
                commit_count: 2,
                confidence: 0.7,
                sample_commits: vec!["abc1234".to_string()],
                reason: "changed together".to_string(),
            }],
            roles: roles([
                ("src/a.ts", FileRole::Source),
                ("src/b.ts", FileRole::Source),
                ("src/c.ts", FileRole::Source),
                ("src/history.ts", FileRole::Source),
                ("tests/a.test.ts", FileRole::Test),
            ]),
            expansion_seeds: vec!["src/a.ts".to_string()],
            ..RankingInput::default()
        });

        assert!(candidate_paths(&candidates).contains(&"src/b.ts"));
        assert!(candidate_paths(&candidates).contains(&"tests/a.test.ts"));
        assert!(candidate_paths(&candidates).contains(&"src/history.ts"));
        assert!(!candidate_paths(&candidates).contains(&"src/c.ts"));
    }

    #[test]
    fn ranking_keeps_semantic_as_secondary_signal() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![lexical("src/auth/session.ts", 10.0)],
            semantic_results: vec![SemanticSearchResult {
                path: "src/auth/session.ts".to_string(),
                role: FileRole::Source,
                language: Some("typescript".to_string()),
                score: 0.91,
                reason: "local semantic similarity".to_string(),
                provider: SemanticProviderConfig::default(),
                document_id: Some("sem_doc_auth".to_string()),
                matched_facets: Vec::new(),
                precision_status: None,
            }],
            roles: roles([("src/auth/session.ts", FileRole::Source)]),
            ..RankingInput::default()
        });

        let file = candidates
            .iter()
            .find(|candidate| candidate.candidate.path.as_deref() == Some("src/auth/session.ts"))
            .unwrap();

        assert_signals(
            &file.candidate.signal_scores,
            &[RetrievalSignalKind::Lexical, RetrievalSignalKind::Semantic],
        );
    }

    #[test]
    fn selection_preserves_explicit_anchors_before_lexical_floor() {
        let lexical_results = (0..8)
            .map(|index| lexical(&format!("src/noisy-{index}.ts"), 24.0))
            .collect::<Vec<_>>();
        let candidates = rank_candidates(RankingInput {
            anchors: vec![AnchorCandidate {
                path: "src/active.ts".to_string(),
                role: FileRole::Source,
                current_diff: false,
            }],
            lexical_results,
            roles: roles([
                ("src/active.ts", FileRole::Source),
                ("src/noisy-0.ts", FileRole::Source),
                ("src/noisy-1.ts", FileRole::Source),
                ("src/noisy-2.ts", FileRole::Source),
                ("src/noisy-3.ts", FileRole::Source),
                ("src/noisy-4.ts", FileRole::Source),
                ("src/noisy-5.ts", FileRole::Source),
                ("src/noisy-6.ts", FileRole::Source),
                ("src/noisy-7.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 3, 0);

        assert_eq!(selection.target_files[0].path, "src/active.ts");
        assert_eq!(
            selection.target_files[0].reason,
            "explicit path anchor from active context"
        );
    }

    #[test]
    fn selection_uses_stronger_lexical_floor_without_active_context() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: (0..5)
                .map(|index| lexical(&format!("src/lexical-{index}.ts"), 8.0))
                .collect(),
            semantic_results: (0..5)
                .map(|index| SemanticSearchResult {
                    path: format!("src/semantic-{index}.ts"),
                    role: FileRole::Source,
                    language: Some("typescript".to_string()),
                    score: 0.95,
                    reason: "local semantic similarity".to_string(),
                    provider: SemanticProviderConfig::default(),
                    document_id: Some(format!("sem_doc_{index}")),
                    matched_facets: Vec::new(),
                    precision_status: None,
                })
                .collect(),
            roles: roles([
                ("src/lexical-0.ts", FileRole::Source),
                ("src/lexical-1.ts", FileRole::Source),
                ("src/lexical-2.ts", FileRole::Source),
                ("src/lexical-3.ts", FileRole::Source),
                ("src/lexical-4.ts", FileRole::Source),
                ("src/semantic-0.ts", FileRole::Source),
                ("src/semantic-1.ts", FileRole::Source),
                ("src/semantic-2.ts", FileRole::Source),
                ("src/semantic-3.ts", FileRole::Source),
                ("src/semantic-4.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 5, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            paths,
            vec![
                "src/lexical-0.ts",
                "src/lexical-1.ts",
                "src/lexical-2.ts",
                "src/lexical-3.ts",
                "src/lexical-4.ts",
            ]
        );
    }

    #[test]
    fn selection_uses_fixed_budgets_and_lexical_rank_order() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                lexical("src/b.ts", 10.0),
                lexical("src/a.ts", 10.0),
                lexical("src/c.ts", 9.0),
            ],
            related_tests: vec![
                RelatedTestResult {
                    path: "tests/b.test.ts".to_string(),
                    command: None,
                    confidence: 0.8,
                    reason: "related".to_string(),
                },
                RelatedTestResult {
                    path: "tests/a.test.ts".to_string(),
                    command: None,
                    confidence: 0.8,
                    reason: "related".to_string(),
                },
            ],
            roles: roles([
                ("src/a.ts", FileRole::Source),
                ("src/b.ts", FileRole::Source),
                ("src/c.ts", FileRole::Source),
                ("tests/a.test.ts", FileRole::Test),
                ("tests/b.test.ts", FileRole::Test),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 2, 1);

        assert_eq!(
            selection
                .target_files
                .iter()
                .map(|target| target.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/b.ts", "src/a.ts"]
        );
        assert_eq!(
            selection
                .related_tests
                .iter()
                .map(|test| test.path.as_str())
                .collect::<Vec<_>>(),
            vec!["tests/a.test.ts"]
        );
    }

    #[test]
    fn selection_preserves_strong_lexical_targets_when_symbols_dominate() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: "documentation/mcp.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 24.0,
                    reason: "strong doc match".to_string(),
                },
                lexical("src/target.ts", 24.0),
            ],
            symbol_results: (0..8)
                .map(|index| SymbolSearchResult {
                    symbol: CodeSymbol {
                        name: format!("Exact{index}"),
                        kind: SymbolKind::Function,
                        path: format!("src/noisy-{index}.ts"),
                        language: Some("typescript".to_string()),
                        start_line: 1,
                        end_line: 1,
                        signature: format!("function Exact{index}() {{}}"),
                        exported: true,
                    },
                    score: 25.0,
                    reason: "symbol name match".to_string(),
                })
                .collect(),
            roles: roles([
                ("documentation/mcp.md", FileRole::Docs),
                ("src/target.ts", FileRole::Source),
                ("src/noisy-0.ts", FileRole::Source),
                ("src/noisy-1.ts", FileRole::Source),
                ("src/noisy-2.ts", FileRole::Source),
                ("src/noisy-3.ts", FileRole::Source),
                ("src/noisy-4.ts", FileRole::Source),
                ("src/noisy-5.ts", FileRole::Source),
                ("src/noisy-6.ts", FileRole::Source),
                ("src/noisy-7.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 3, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"documentation/mcp.md"));
        assert!(paths.contains(&"src/target.ts"));
    }

    #[test]
    fn selection_reserves_symbol_floor_when_archive_docs_fill_budget() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: ".planning/milestones/v1/old-0.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 90.0,
                    reason: "archive doc match".to_string(),
                },
                SearchResult {
                    path: ".planning/e2e/run/proof.json".to_string(),
                    role: FileRole::Docs,
                    language: Some("json".to_string()),
                    score: 89.0,
                    reason: "archive json match".to_string(),
                },
                SearchResult {
                    path: "docs/current.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 88.0,
                    reason: "current doc match".to_string(),
                },
                lexical("src/exact.ts", 30.0),
            ],
            symbol_results: vec![SymbolSearchResult {
                symbol: CodeSymbol {
                    name: "ProtectedSymbol".to_string(),
                    kind: SymbolKind::Function,
                    path: "src/protected.ts".to_string(),
                    language: Some("typescript".to_string()),
                    start_line: 7,
                    end_line: 9,
                    signature: "function ProtectedSymbol()".to_string(),
                    exported: true,
                },
                score: 50.0,
                reason: "symbol name match".to_string(),
            }],
            roles: roles([
                (".planning/milestones/v1/old-0.md", FileRole::Docs),
                (".planning/e2e/run/proof.json", FileRole::Docs),
                ("docs/current.md", FileRole::Docs),
                ("src/exact.ts", FileRole::Source),
                ("src/protected.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 4, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"src/exact.ts"));
        assert!(paths.contains(&".planning/milestones/v1/old-0.md"));
        assert!(paths.contains(&"src/protected.ts"));
        assert_eq!(paths.len(), 4);
    }

    #[test]
    fn selection_reserves_budget_for_source_lexical_candidates_without_active_context() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: "docs/runtime.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 40.0,
                    reason: "doc match".to_string(),
                },
                SearchResult {
                    path: "Cargo.toml".to_string(),
                    role: FileRole::Config,
                    language: Some("toml".to_string()),
                    score: 38.0,
                    reason: "config match".to_string(),
                },
                SearchResult {
                    path: "src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java".to_string(),
                    role: FileRole::Source,
                    language: Some("java".to_string()),
                    score: 18.0,
                    reason: "source match".to_string(),
                },
                SearchResult {
                    path: "src/main/java/org/refactoringminer/astDiff/matchers/wrappers/BodyMapperMatcher.java".to_string(),
                    role: FileRole::Source,
                    language: Some("java".to_string()),
                    score: 16.0,
                    reason: "source match".to_string(),
                },
            ],
            protected_lexical_limit: Some(2),
            roles: roles([
                ("docs/runtime.md", FileRole::Docs),
                ("Cargo.toml", FileRole::Config),
                (
                    "src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java",
                    FileRole::Source,
                ),
                (
                    "src/main/java/org/refactoringminer/astDiff/matchers/wrappers/BodyMapperMatcher.java",
                    FileRole::Source,
                ),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 4, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(
            &"src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java"
        ));
        assert!(paths.contains(
            &"src/main/java/org/refactoringminer/astDiff/matchers/wrappers/BodyMapperMatcher.java"
        ));
        let source_signal = candidates
            .iter()
            .find(|candidate| {
                candidate.candidate.path.as_deref()
                    == Some(
                        "src/main/java/org/refactoringminer/astDiff/matchers/wrappers/MethodMatcher.java",
                    )
            })
            .and_then(|candidate| candidate.candidate.signal_scores.first())
            .map(|score| score.signal.clone());
        assert_eq!(source_signal, Some(RetrievalSignalKind::LexicalExpansion));
    }

    #[test]
    fn selection_applies_source_lexical_reserve_before_docs_fill_budget() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: "docs/old-0.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 60.0,
                    reason: "doc match".to_string(),
                },
                SearchResult {
                    path: "docs/old-1.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 59.0,
                    reason: "doc match".to_string(),
                },
                SearchResult {
                    path: "docs/old-2.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 58.0,
                    reason: "doc match".to_string(),
                },
                SearchResult {
                    path: "docs/old-3.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 57.0,
                    reason: "doc match".to_string(),
                },
                SearchResult {
                    path: "src/eval.rs".to_string(),
                    role: FileRole::Source,
                    language: Some("rust".to_string()),
                    score: 12.0,
                    reason: "source match".to_string(),
                },
            ],
            protected_lexical_limit: Some(4),
            roles: roles([
                ("docs/old-0.md", FileRole::Docs),
                ("docs/old-1.md", FileRole::Docs),
                ("docs/old-2.md", FileRole::Docs),
                ("docs/old-3.md", FileRole::Docs),
                ("src/eval.rs", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 4, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"src/eval.rs"));
        assert_eq!(paths.len(), 4);
    }

    #[test]
    fn selection_reserves_governance_docs_for_project_planning_candidates() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: "crates/ctxpack-compiler/src/eval.rs".to_string(),
                    role: FileRole::Source,
                    language: Some("rust".to_string()),
                    score: 18.0,
                    reason: "source match".to_string(),
                },
                SearchResult {
                    path: ".planning/milestones/v2.5/summary.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 80.0,
                    reason: "milestone match".to_string(),
                },
                SearchResult {
                    path: ".planning/e2e/old-proof.json".to_string(),
                    role: FileRole::Docs,
                    language: Some("json".to_string()),
                    score: 79.0,
                    reason: "proof match".to_string(),
                },
                SearchResult {
                    path: ".planning/STATE.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 16.0,
                    reason: "governance match".to_string(),
                },
            ],
            roles: roles([
                ("crates/ctxpack-compiler/src/eval.rs", FileRole::Source),
                (".planning/milestones/v2.5/summary.md", FileRole::Docs),
                (".planning/e2e/old-proof.json", FileRole::Docs),
                (".planning/STATE.md", FileRole::Docs),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 3, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"crates/ctxpack-compiler/src/eval.rs"));
        assert!(paths.contains(&".planning/STATE.md"));
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn selection_does_not_reserve_generic_governance_docs_without_planning_docs() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: "src/main.rs".to_string(),
                    role: FileRole::Source,
                    language: Some("rust".to_string()),
                    score: 18.0,
                    reason: "source match".to_string(),
                },
                SearchResult {
                    path: "docs/benchmarking.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 16.0,
                    reason: "generic doc match".to_string(),
                },
                SearchResult {
                    path: "docs/noisy.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 90.0,
                    reason: "higher doc match".to_string(),
                },
            ],
            roles: roles([
                ("src/main.rs", FileRole::Source),
                ("docs/benchmarking.md", FileRole::Docs),
                ("docs/noisy.md", FileRole::Docs),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 2, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(!paths.contains(&"docs/benchmarking.md"));
    }

    #[test]
    fn selection_does_not_let_source_reserve_displace_active_context_anchor() {
        let candidates = rank_candidates(RankingInput {
            anchors: vec![AnchorCandidate {
                path: "docs/active.md".to_string(),
                role: FileRole::Docs,
                current_diff: false,
            }],
            lexical_results: vec![
                lexical("src/high-0.ts", 24.0),
                lexical("src/high-1.ts", 24.0),
                lexical("src/high-2.ts", 24.0),
            ],
            roles: roles([
                ("docs/active.md", FileRole::Docs),
                ("src/high-0.ts", FileRole::Source),
                ("src/high-1.ts", FileRole::Source),
                ("src/high-2.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 2, 0);

        assert_eq!(selection.target_files[0].path, "docs/active.md");
    }

    #[test]
    fn selection_preserves_strong_cochange_targets() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                lexical("src/a.ts", 24.0),
                lexical("src/b.ts", 24.0),
                lexical("src/c.ts", 24.0),
                lexical("src/d.ts", 24.0),
                lexical("src/e.ts", 24.0),
            ],
            co_change_hints: vec![CoChangeHint {
                path: "src/historical.ts".to_string(),
                commit_count: 2,
                confidence: 0.8,
                sample_commits: vec!["abc1234".to_string(), "def5678".to_string()],
                reason: "changed together".to_string(),
            }],
            roles: roles([
                ("src/a.ts", FileRole::Source),
                ("src/b.ts", FileRole::Source),
                ("src/c.ts", FileRole::Source),
                ("src/d.ts", FileRole::Source),
                ("src/e.ts", FileRole::Source),
                ("src/historical.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 6, 0);
        let paths = selection
            .target_files
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"src/historical.ts"));
    }

    #[test]
    fn selection_promotes_cochanged_validation_tests() {
        let candidates = rank_candidates(RankingInput {
            related_tests: vec![
                RelatedTestResult {
                    path: "tests/generic_a.test.ts".to_string(),
                    command: None,
                    confidence: 0.95,
                    reason: "content mentions broad terms".to_string(),
                },
                RelatedTestResult {
                    path: "tests/generic_b.test.ts".to_string(),
                    command: None,
                    confidence: 0.95,
                    reason: "content mentions broad terms".to_string(),
                },
            ],
            co_change_hints: vec![CoChangeHint {
                path: "tests/targeted.test.ts".to_string(),
                commit_count: 2,
                confidence: 0.6,
                sample_commits: vec!["abc1234".to_string(), "def5678".to_string()],
                reason: "changed together".to_string(),
            }],
            roles: roles([
                ("tests/generic_a.test.ts", FileRole::Test),
                ("tests/generic_b.test.ts", FileRole::Test),
                ("tests/targeted.test.ts", FileRole::Test),
            ]),
            ..RankingInput::default()
        });

        let selection = select_ranked_candidates(&candidates, 0, 1);

        assert_eq!(selection.related_tests[0].path, "tests/targeted.test.ts");
        assert!(selection.related_tests[0]
            .attribution
            .iter()
            .any(|evidence| {
                evidence.signal == RetrievalSignalKind::CoChange
                    && evidence.reason_code == "co_change_neighbor"
            }));
    }

    #[test]
    fn ranking_materializes_doc_commit_and_config_candidates_source_free() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![
                SearchResult {
                    path: "README.md".to_string(),
                    role: FileRole::Docs,
                    language: Some("markdown".to_string()),
                    score: 6.0,
                    reason: "term match".to_string(),
                },
                SearchResult {
                    path: "Cargo.toml".to_string(),
                    role: FileRole::Config,
                    language: Some("toml".to_string()),
                    score: 6.0,
                    reason: "term match".to_string(),
                },
            ],
            co_change_hints: vec![CoChangeHint {
                path: "README.md".to_string(),
                commit_count: 3,
                confidence: 0.8,
                sample_commits: vec!["abc1234".to_string(), "def5678".to_string()],
                reason: "changed together".to_string(),
            }],
            roles: roles([
                ("README.md", FileRole::Docs),
                ("Cargo.toml", FileRole::Config),
            ]),
            expansion_seeds: vec!["src/lib.rs".to_string()],
            ..RankingInput::default()
        });

        assert!(has_kind_path(
            &candidates,
            RetrievalCandidateKind::Doc,
            Some("README.md")
        ));
        assert!(has_kind_path(
            &candidates,
            RetrievalCandidateKind::Config,
            Some("Cargo.toml")
        ));
        assert!(has_kind_path(
            &candidates,
            RetrievalCandidateKind::Commit,
            None
        ));
        let serialized = serde_json::to_string(
            &candidates
                .iter()
                .map(|candidate| &candidate.candidate)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert!(!serialized.contains("term match"));
        assert!(!serialized.contains("changed together"));
        assert!(serialized.contains("abc1234"));
    }

    #[test]
    fn local_metadata_reranker_is_deterministic_and_source_free() {
        let candidates = rank_candidates(RankingInput {
            lexical_results: vec![lexical("src/secondary.ts", 20.0)],
            symbol_results: vec![SymbolSearchResult {
                symbol: CodeSymbol {
                    name: "AuthService".to_string(),
                    kind: ctxpack_index::SymbolKind::Function,
                    path: "src/primary.ts".to_string(),
                    language: Some("typescript".to_string()),
                    signature: "AuthService()".to_string(),
                    start_line: 1,
                    end_line: 1,
                    exported: true,
                },
                score: 20.0,
                reason: "symbol name match".to_string(),
            }],
            roles: roles([
                ("src/primary.ts", FileRole::Source),
                ("src/secondary.ts", FileRole::Source),
            ]),
            ..RankingInput::default()
        });

        let reranked = rerank_with_local_metadata(candidates.clone());
        let reranked_again = rerank_with_local_metadata(candidates);

        assert_eq!(candidate_paths(&reranked), candidate_paths(&reranked_again));
        assert_eq!(candidate_paths(&reranked).first(), Some(&"src/primary.ts"));
        let serialized = serde_json::to_string(
            &reranked
                .iter()
                .map(|candidate| &candidate.candidate)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert!(!serialized.contains("symbol name match"));
        assert!(!serialized.contains("source code"));
    }

    fn lexical(path: &str, score: f32) -> SearchResult {
        SearchResult {
            path: path.to_string(),
            role: FileRole::Source,
            language: Some("typescript".to_string()),
            score,
            reason: "term match".to_string(),
        }
    }

    fn roles<const N: usize>(entries: [(&str, FileRole); N]) -> BTreeMap<String, FileRole> {
        entries
            .into_iter()
            .map(|(path, role)| (path.to_string(), role))
            .collect()
    }

    fn assert_signals(signal_scores: &[RetrievalSignalScore], expected: &[RetrievalSignalKind]) {
        let actual = signal_scores
            .iter()
            .map(|score| score.signal.clone())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    fn assert_signals_from_evidence(
        evidence: &[RetrievalEvidence],
        expected: &[RetrievalSignalKind],
    ) {
        let actual = evidence
            .iter()
            .map(|item| item.signal.clone())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    fn candidate_paths(candidates: &[RankedCandidate]) -> Vec<&str> {
        candidates
            .iter()
            .filter_map(|candidate| candidate.candidate.path.as_deref())
            .collect()
    }

    fn has_kind_path(
        candidates: &[RankedCandidate],
        kind: RetrievalCandidateKind,
        path: Option<&str>,
    ) -> bool {
        candidates.iter().any(|candidate| {
            candidate.candidate.kind == kind && candidate.candidate.path.as_deref() == path
        })
    }
}
