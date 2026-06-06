mod agent_preview;
mod cards;
mod eval;
mod graph;
mod packs;
mod planning;
mod policy;
mod ranking;
mod workspace;

pub use agent_preview::{build_agent_preview_report, build_agent_preview_report_with_provider};
pub use cards::{
    generate_context_cards, generate_experience_cards, generate_fallback_cards,
    ContextCardsOptions, ContextCardsReport, ExperienceCardsOptions, ExperienceCardsReport,
    FallbackCardsOptions, FallbackCardsReport, GeneratedContextCard,
};
pub use eval::{
    build_product_proof_report, build_retrieval_health_report, compare_benchmark_suite_reports,
    compare_candidate_feature_exports, compare_lexical_backends_on_corpus,
    delete_candidate_feature_export, eval_trace_for_pack, eval_trace_for_plan,
    evaluate_historical_commits, export_candidate_features_for_task,
    list_candidate_feature_exports, load_benchmark_suite_config, load_benchmark_suite_report,
    load_candidate_feature_export, paired_baseline_analysis_report, run_benchmark_suite,
    run_benchmark_suite_config, semantic_precision_gate_report,
    semantic_precision_gate_report_with_provider, write_candidate_feature_export,
    BenchmarkComparisonReport, BenchmarkDefaults, BenchmarkGapFamilyDelta, BenchmarkMetricDelta,
    BenchmarkRegressionThreshold, BenchmarkRepoBaseline, BenchmarkRepoBaselineStatus,
    BenchmarkRepoConfig, BenchmarkRepoEffectiveConfig, BenchmarkRepoReport, BenchmarkSuiteConfig,
    BenchmarkSuiteReport, BenchmarkThresholdCheck, CandidateCoverageAreaSummary,
    CandidateCoverageSummary, CandidateFeatureComparisonReport, CandidateFeatureKindDelta,
    CandidateMissedFileProfile, ContextAreaNextReadSummary, ContextAreaPressurePeak,
    ContextAreaPressureSummary, EvalComparison, GraphEdgeAblationResult, GraphEdgeProfile,
    HistoricalChangedPathLabel, HistoricalCommitEval, HistoricalEvalEffectiveFilters,
    HistoricalEvalOptions, HistoricalEvalRefs, HistoricalEvalReport, HistoricalEvalRuntimeSummary,
    HistoricalMissingFileSummary, HistoricalProtectedEvidenceFile, HistoricalSignalRanking,
    HistoricalSlowCommitSummary, LexicalBackendCommitRow, LexicalBackendComparison,
    LexicalBackendCorpusOptions, LexicalBackendCorpusReport, LexicalBackendMetrics,
    LexicalBackendRuntimeSummary, MemoryReuseSummary, PairedBaselineAnalysisReport,
    PairedBaselineFamily, PairedBaselineRow, PairedBaselineVerdict, ProductProofCorpusStatus,
    ProductProofCorpusVerdict, ProductProofLexicalBackendComparison, ProductProofLexicalClaim,
    ProductProofLexicalComparison, ProductProofMetric, ProductProofReleaseGate, ProductProofReport,
    ProtectedEvidenceSignalSummary, ProtectedEvidenceSummary, RankingMetrics,
    RecommendedResearchAction, RetrievalGapRecommendationArea, RetrievalGapSummary,
    RetrievalGapTargetStatus, RoleRecallMetric, SemanticContributionSummary,
    SemanticMissedTargetGapFamily, SemanticPrecisionGateDecision, SemanticPrecisionGateReport,
    SemanticPrecisionNamedCase, SemanticPrecisionVariant, SemanticPrecisionVariantStatus,
    SignalAblationResult, SignalSaturationMetric, TokenRoiMetric,
};
pub use graph::build_graph_neighborhood_report;
pub use packs::{
    compile_context_pack, compile_context_pack_from_plan, compile_context_pack_from_plan_for_agent,
    compile_context_pack_with_plan, compile_context_pack_with_plan_and_paths,
    compile_context_pack_with_plan_and_paths_for_agent,
    compile_context_pack_with_plan_and_paths_for_agent_and_semantic,
    compile_context_pack_with_plan_and_paths_for_agent_and_semantic_provider,
    compile_context_pack_with_plan_for_agent, compile_pack_inspector_view,
    render_pack_inspector_html, render_pack_inspector_markdown, render_pack_markdown,
};
pub use planning::{
    empty_plan_for_task, prepare_context_plan, prepare_context_plan_with_paths,
    prepare_context_plan_with_paths_and_semantic,
    prepare_context_plan_with_paths_and_semantic_provider,
};
pub use policy::{
    provider_policy_report, retrieval_policy_experiment_report, semantic_provider_status_report,
    semantic_provider_status_report_with_provider,
};
pub use workspace::{compile_workspace_context_pack, prepare_workspace_context_plan};

#[cfg(test)]
use ctxhelm_core::{
    FileRole, MemoryCard, MemoryCardKind, MemoryFreshness, MemoryReviewStatus, PackBudget,
    PrivacyStatus, RetrievalCandidateKind, RetrievalSignalKind, TaskType,
};
#[cfg(test)]
use ctxhelm_index::{
    persist_memory_card_records, repo_id_for_path, task_hash, write_inventory, InventoryOptions,
    SemanticProviderConfig, StorageMemoryCardRecord, StoreConfig,
};
#[cfg(test)]
use eval::HistoricalEvalWorktree;
#[cfg(test)]
use planning::{
    is_low_information_task, is_multi_area_task, plan_confidence, PREPARE_TASK_TARGET_LIMIT,
    PREPARE_TASK_TEST_LIMIT,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;
    use std::process::Command as ProcessCommand;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn empty_plan_includes_progressive_pack_options() {
        let plan = empty_plan_for_task(TaskType::Explain);
        assert_eq!(plan.pack_options.len(), 3);
        assert!(plan.pack_options[0].resource_uri.ends_with("/brief"));
        assert!(plan.pack_options[1].resource_uri.ends_with("/standard"));
        assert!(plan.pack_options[2].resource_uri.ends_with("/deep"));
    }

    #[test]
    fn low_information_task_detection_flags_issue_only_titles() {
        assert!(is_low_information_task("Fixes #1061"));
        assert!(is_low_information_task("fix bug"));
        assert!(!is_low_information_task("fix requireSession bug"));
        assert!(!is_low_information_task(
            "Improve signature for nested functions in TypeScript"
        ));
        assert!(!is_low_information_task("Add test for #1060"));
    }

    #[test]
    fn multi_area_task_detection_flags_broad_workflow_prompts() {
        assert!(is_multi_area_task("stabilize lint workflow"));
        assert!(is_multi_area_task("qwen3.5 eval harden smoke workflow"));
        assert!(is_multi_area_task("pvldb run orchestration"));
        assert!(!is_multi_area_task("fix requireSession bug"));
        assert!(!is_multi_area_task(
            "Improve signature for nested functions in TypeScript"
        ));
    }

    #[test]
    fn prepare_context_plan_fuses_search_tests_and_history() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        std::env::set_var("CTXHELM_HOME", &home);

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

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_keeps_supporting_targets_past_top_five() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        for index in 0..10 {
            fs::write(
                repo.join(format!("src/auth/session_{index}.ts")),
                format!("export function session{index}() {{ return 'auth session redirect'; }}\n"),
            )
            .unwrap();
        }
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add auth session files"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let plan =
            prepare_context_plan(&repo, "fix auth session redirect", TaskType::BugFix).unwrap();

        assert_eq!(plan.target_files.len(), PREPARE_TASK_TARGET_LIMIT);
        assert!(plan
            .target_files
            .iter()
            .any(|target| target.path.as_str() >= "src/auth/session_5.ts"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_keeps_validation_tests_up_to_recall_at_ten() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function sessionFlow() { return 'auth session'; }\n",
        )
        .unwrap();
        for index in 0..10 {
            fs::write(
                repo.join(format!("tests/auth/session_flow_{index}.test.ts")),
                "import '../src/auth/session';\nit('covers auth session flow', () => {});\n",
            )
            .unwrap();
        }
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add auth session tests"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix auth session flow", TaskType::BugFix).unwrap();

        assert_eq!(plan.related_tests.len(), PREPARE_TASK_TEST_LIMIT);

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_uses_cochange_sources_for_related_tests() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/middleware.ts"),
            "export function authMiddleware() { return 'middleware'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function getSession() { return 'session'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { getSession } from '../../src/auth/session';\nit('checks getSession', () => {});\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add auth files"]);
        fs::write(
            repo.join("src/auth/middleware.ts"),
            "export function authMiddleware() { return 'middleware redirect'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function getSession() { return 'session redirect'; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "change middleware with session"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let plan =
            prepare_context_plan(&repo, "fix authMiddleware redirect", TaskType::BugFix).unwrap();

        assert!(plan
            .related_tests
            .iter()
            .any(|test| test.path == "tests/auth/session.test.ts"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn context_pack_snippets_focus_around_symbol_line_ranges() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        std::env::set_var("CTXHELM_HOME", &home);

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
        assert!(markdown.contains("- Lines: 31-33"));
        assert!(markdown.contains("31: export function requireSession"));
        assert!(markdown.contains("... omitted lines 1-"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn pack_inspector_view_keeps_source_snippets_out_of_metadata() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() {\n  return 'TOP_SECRET_SOURCE_SENTINEL';\n}\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let (plan, pack) = compile_context_pack_with_plan(
            &repo,
            "fix requireSession sentinel",
            TaskType::BugFix,
            PackBudget::Brief,
        )
        .unwrap();
        let pack_markdown = render_pack_markdown(&pack);
        let view = compile_pack_inspector_view(&plan, &pack);
        let view_json = serde_json::to_string_pretty(&view).unwrap();
        let view_markdown = render_pack_inspector_markdown(&view);
        let view_html = render_pack_inspector_html(&view);

        assert!(pack_markdown.contains("TOP_SECRET_SOURCE_SENTINEL"));
        assert!(!view_json.contains("TOP_SECRET_SOURCE_SENTINEL"));
        assert!(!view_markdown.contains("TOP_SECRET_SOURCE_SENTINEL"));
        assert!(!view_html.contains("TOP_SECRET_SOURCE_SENTINEL"));
        assert!(!view.source_text_logged);
        assert!(view.source_bearing_section_count > 0);
        assert!(view
            .sections
            .iter()
            .any(|section| section.kind == "target_snippets" && section.source_bearing));
        assert_eq!(view.target_files[0].path, "src/auth/session.ts");
        assert!(view_html.contains("data-inspector-source-free=\"true\""));
        assert!(view_html.contains("id=\"filterText\""));
        assert!(view_html.contains("id=\"kindFilter\""));
        assert!(view_html.contains("id=\"sourceOnly\""));
        assert!(view_html.contains("data-kind=\"target\""));
        assert!(view_html.contains("data-source-bearing=\"true\""));
        assert!(view_html.contains("Retrieval Candidates"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_degrades_when_git_history_is_unavailable() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
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
        std::env::set_var("CTXHELM_HOME", &home);

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
        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "git_unavailable"));
        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "history_partial"));
        assert_eq!(plan.confidence, plan_confidence(true, true, false, true));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_adds_dependency_edges_as_graph_evidence() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() { return parseCookie(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();

        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert!(plan.risk_flags.iter().any(|flag| {
            flag.code == "dependency_edge" && flag.message.contains("src/auth/cookies.ts")
        }));
        assert_eq!(plan.confidence, plan_confidence(true, true, false, true));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_can_select_dependency_neighbor_without_lexical_match() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() { return parseCookie(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();
        let dependency_target = plan
            .target_files
            .iter()
            .find(|target| target.path == "src/auth/cookies.ts")
            .expect("dependency neighbor should consume target budget");

        assert!(dependency_target
            .attribution
            .iter()
            .any(|evidence| evidence.signal == RetrievalSignalKind::Dependency));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_recommends_related_test_with_attribution_and_command() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
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
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();
        let related_test = plan
            .related_tests
            .iter()
            .find(|test| test.path == "tests/auth/session.test.ts")
            .expect("related test should be projected separately from target files");

        assert!(!plan
            .target_files
            .iter()
            .any(|target| target.path == "tests/auth/session.test.ts"));
        assert_eq!(
            related_test.command.as_deref(),
            Some("pnpm test tests/auth/session.test.ts")
        );
        assert!(related_test
            .attribution
            .iter()
            .any(|evidence| evidence.signal == RetrievalSignalKind::RelatedTest));
        assert!(plan
            .recommended_commands
            .iter()
            .any(|command| command.command == "pnpm test tests/auth/session.test.ts"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_attributes_current_diff_anchors() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return false; }\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let anchors = vec!["src/auth/session.ts".to_string()];
        let plan = prepare_context_plan_with_paths(
            &repo,
            "fix unrelated behavior",
            TaskType::BugFix,
            &anchors,
        )
        .unwrap();

        assert_eq!(plan.target_files[0].path, "src/auth/session.ts");
        assert!(plan.target_files[0]
            .attribution
            .iter()
            .any(|evidence| evidence.signal == RetrievalSignalKind::CurrentDiff));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_projects_all_typed_candidates_source_free() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        fs::write(
            repo.join("README.md"),
            "requireSession documentation should stay source free\n",
        )
        .unwrap();
        fs::write(
            repo.join("Cargo.toml"),
            "[package]\nname = \"requireSession-config\"\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session docs and config"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return false; }\n",
        )
        .unwrap();
        fs::write(repo.join("README.md"), "requireSession docs updated\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "update session docs"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let first = prepare_context_plan(
            &repo,
            "fix requireSession documentation config",
            TaskType::BugFix,
        )
        .unwrap();
        let second = prepare_context_plan(
            &repo,
            "fix requireSession documentation config",
            TaskType::BugFix,
        )
        .unwrap();
        let kinds = first
            .retrieval_candidates
            .iter()
            .map(|candidate| candidate.kind.clone())
            .collect::<Vec<_>>();
        let serialized = serde_json::to_string(&first.retrieval_candidates).unwrap();

        assert!(kinds.contains(&RetrievalCandidateKind::File));
        assert!(kinds.contains(&RetrievalCandidateKind::Test));
        assert!(kinds.contains(&RetrievalCandidateKind::Symbol));
        assert!(kinds.contains(&RetrievalCandidateKind::Doc));
        assert!(kinds.contains(&RetrievalCandidateKind::Commit));
        assert!(kinds.contains(&RetrievalCandidateKind::Config));
        assert_eq!(
            serialized,
            serde_json::to_string(&second.retrieval_candidates).unwrap()
        );
        assert!(!serialized.contains("export function requireSession"));
        assert!(!serialized.contains("documentation should stay source free"));
        assert!(first
            .risk_flags
            .iter()
            .any(|flag| flag.code == "co_change_hint"));
        assert!(first
            .risk_flags
            .iter()
            .any(|flag| flag.code == "dependency_edge"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn candidate_feature_export_persists_source_free_rows() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return 'CTXHELM_FEATURE_SOURCE_SENTINEL'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        fs::write(
            repo.join("README.md"),
            "CTXHELM_FEATURE_DOC_SENTINEL requireSession documentation\n",
        )
        .unwrap();
        fs::write(
            repo.join("Cargo.toml"),
            "[package]\nname = \"requireSession-config\"\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session docs and config"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return false; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "fix requireSession behavior"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let export = export_candidate_features_for_task(
            &repo,
            "fix requireSession documentation config",
            TaskType::BugFix,
            "codex",
            100,
            false,
        )
        .unwrap();
        let kinds = export
            .rows
            .iter()
            .map(|row| row.candidate_kind.clone())
            .collect::<Vec<_>>();
        let serialized = serde_json::to_string(&export).unwrap();

        assert_eq!(export.schema_version, 1);
        assert_eq!(export.target_agent.as_deref(), Some("codex"));
        assert_eq!(export.row_count, export.rows.len());
        assert!(kinds.contains(&RetrievalCandidateKind::File));
        assert!(kinds.contains(&RetrievalCandidateKind::Test));
        assert!(kinds.contains(&RetrievalCandidateKind::Symbol));
        assert!(kinds.contains(&RetrievalCandidateKind::Doc));
        assert!(kinds.contains(&RetrievalCandidateKind::Commit));
        assert!(kinds.contains(&RetrievalCandidateKind::Config));
        assert!(export.rows.iter().any(|row| row
            .labels
            .contains(&ctxhelm_core::CandidateFeatureLabel::Selected)));
        assert!(export.rows.iter().any(|row| row.lexical_score > 0.0));
        assert!(export.rows.iter().any(|row| row.history_commit_count > 0));
        assert!(!export.source_text_logged);
        assert!(!serialized.contains("CTXHELM_FEATURE_SOURCE_SENTINEL"));
        assert!(!serialized.contains("CTXHELM_FEATURE_DOC_SENTINEL"));
        assert!(!serialized.contains("export function"));

        let path = write_candidate_feature_export(&repo, &export).unwrap();
        assert!(path.exists());
        let listed = list_candidate_feature_exports(&repo).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].export_id, export.export_id);
        let loaded = load_candidate_feature_export(&repo, &export.export_id.to_string()).unwrap();
        let comparison = compare_candidate_feature_exports(&export, &loaded);
        assert_eq!(comparison.row_count_delta, 0);
        assert!(!comparison.source_text_logged);
        let deleted =
            delete_candidate_feature_export(&repo, &export.export_id.to_string()).unwrap();
        assert_eq!(deleted, path);
        assert!(list_candidate_feature_exports(&repo).unwrap().is_empty());

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_prefers_explicit_path_anchors() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("src/billing")).unwrap();
        fs::create_dir_all(repo.join("tests/billing")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        std::env::set_var("CTXHELM_HOME", &home);

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

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_reports_unavailable_path_anchors() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
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
        std::env::set_var("CTXHELM_HOME", &home);

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
        assert_eq!(
            plan.diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "anchor_unavailable")
                .count(),
            2
        );

        std::env::remove_var("CTXHELM_HOME");
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
    fn prepare_context_plan_reports_low_information_task_diagnostics() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "export const session = true;\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "Fixes #1061", TaskType::BugFix).unwrap();

        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "low_information_task"));
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.code == "low_information_task"));
        assert!(plan.missing_info_questions.iter().any(|question| {
            question.contains("file path")
                || question.contains("symbol")
                || question.contains("error")
        }));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_reports_multi_area_task_diagnostics() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("docs")).unwrap();
        fs::write(
            repo.join("src/workflow.ts"),
            "export const workflow = true;\n",
        )
        .unwrap();
        fs::write(
            repo.join("docs/release.md"),
            "workflow release documentation\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let plan =
            prepare_context_plan(&repo, "stabilize lint workflow", TaskType::BugFix).unwrap();

        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "multi_area_task"));
        assert!(plan
            .risk_flags
            .iter()
            .any(|flag| flag.code == "multi_area_task"));
        assert!(plan
            .context_areas
            .iter()
            .any(|area| area.area == "src" && area.selected_count > 0));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_context_plan_diagnostics_stay_source_free_for_unreadable_inputs() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/broken.ts"),
            [
                0xff, b'u', b'n', b's', b'a', b'f', b'e', b'_', b'p', b'a', b'y', b'l', b'o', b'a',
                b'd',
            ],
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "brokenPayload", TaskType::BugFix).unwrap();
        let diagnostics_json = serde_json::to_string(&plan.diagnostics).unwrap();

        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source_non_utf8"));
        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "parse_gap"));
        assert!(!diagnostics_json.contains("unsafe_payload"));
        assert!(diagnostics_json.contains("src/broken.ts"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn compile_context_pack_materializes_plan_snippets_and_validation() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        std::env::set_var("CTXHELM_HOME", &home);

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
        assert!(pack
            .sections
            .iter()
            .any(|section| section.kind == "consumption_guidance"));
        assert!(pack
            .sections
            .iter()
            .any(|section| section.kind == "related_test_evidence"));
        assert!(markdown.contains("src/auth/session.ts"));
        assert!(markdown.contains("export function requireSession"));
        assert!(markdown.contains("## Consumption guidance"));
        assert!(markdown.contains(
            "discovering a path or seeing a pack snippet is not the same as consuming the current file"
        ));
        assert!(markdown
            .contains("including docs, config, schema, and script targets when they appear"));
        assert!(markdown.contains("## Related test evidence"));
        assert!(markdown.contains("Area: `tests/auth`"));
        assert!(markdown.contains(
            "They are validation evidence, so some may not be repeated in context-area next-read lists."
        ));
        assert!(markdown.contains("tests/auth/session.test.ts"));
        assert!(markdown.contains("pnpm test tests/auth/session.test.ts"));
        assert!(markdown.contains("Final checklist"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn compile_context_pack_renders_context_areas() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let mut plan = empty_plan_for_task(TaskType::BugFix);
        plan.context_areas = vec![
            ctxhelm_core::ContextArea {
                area: "src".to_string(),
                reason:
                    "Broad task candidate area with 3 candidate path(s) and 1 selected path(s)."
                        .to_string(),
                resource_uri: "ctxhelm://repo/context-area/src".to_string(),
                representative_paths: vec!["src/lib.rs".to_string()],
                next_read_paths: vec!["src/compiler.rs".to_string()],
                signal_counts: BTreeMap::from([
                    ("dependency".to_string(), 2),
                    ("lexical".to_string(), 1),
                ]),
                role_counts: BTreeMap::from([("source".to_string(), 3)]),
                selected_role_counts: BTreeMap::from([("source".to_string(), 1)]),
                candidate_count: 3,
                selected_count: 1,
                unselected_count: 2,
                coverage_percent: 33,
                inspection_pressure: 6,
                inspection_pressure_breakdown: ctxhelm_core::InspectionPressureBreakdown {
                    source_like_unselected: 2,
                    validation_unselected: 0,
                    docs_unselected: 0,
                    source_like_weight: 3,
                    validation_weight: 2,
                    docs_weight: 1,
                    total: 6,
                },
            },
            ctxhelm_core::ContextArea {
                area: "tests".to_string(),
                reason:
                    "Broad task candidate area with 2 candidate path(s) and 0 selected path(s)."
                        .to_string(),
                resource_uri: "ctxhelm://repo/context-area/tests".to_string(),
                representative_paths: vec!["tests/lib_test.rs".to_string()],
                next_read_paths: vec!["tests/lib_test.rs".to_string()],
                signal_counts: BTreeMap::from([("related_test".to_string(), 2)]),
                role_counts: BTreeMap::from([("test".to_string(), 2)]),
                selected_role_counts: BTreeMap::new(),
                candidate_count: 2,
                selected_count: 0,
                unselected_count: 2,
                coverage_percent: 0,
                inspection_pressure: 4,
                inspection_pressure_breakdown: ctxhelm_core::InspectionPressureBreakdown {
                    source_like_unselected: 0,
                    validation_unselected: 2,
                    docs_unselected: 0,
                    source_like_weight: 3,
                    validation_weight: 2,
                    docs_weight: 1,
                    total: 4,
                },
            },
        ];

        let pack = compile_context_pack_from_plan(
            &repo,
            "stabilize lint workflow",
            &plan,
            PackBudget::Brief,
        );
        let markdown = render_pack_markdown(&pack);

        assert!(pack
            .sections
            .iter()
            .any(|section| section.kind == "context_areas"));
        assert!(markdown.contains("Context areas"));
        assert!(markdown.contains("Zero-selected areas to inspect next"));
        assert!(markdown.contains("`src`"));
        assert!(markdown.contains("`src/lib.rs`"));
        assert!(markdown.contains("`src/compiler.rs`"));
        assert!(markdown.contains("`tests/lib_test.rs`"));
        assert!(markdown.contains("Next reads"));
        assert!(markdown.contains("Coverage: 33% selected, pressure 6"));
        assert!(markdown.contains("source_like=2x3, validation=0x2, docs=0x1"));
        assert!(markdown.contains("validation=2x2"));
        assert!(markdown
            .contains("pressure 4 (source_like=0x3, validation=2x2, docs=0x1), coverage 0%"));
        assert!(markdown.contains("Signals: dependency=2, lexical=1"));
        assert!(markdown.contains("Signals: related_test=2"));
        assert!(markdown.contains("Role counts: source=3"));
        assert!(markdown.contains("Selected roles: source=1"));
        assert!(markdown.contains("Role counts: test=2"));
        assert!(markdown.contains("Selected roles: none"));
        assert!(markdown.contains("ctxhelm://repo/context-area/tests"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn pack_revalidates_plan_snippet_paths_against_current_inventory() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() {\n  return 'safe';\n}\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let mut plan =
            prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();
        plan.target_files[0].path = ".env".to_string();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();

        let pack = compile_context_pack_from_plan(
            &repo,
            "fix requireSession bug",
            &plan,
            PackBudget::Brief,
        );
        let markdown = render_pack_markdown(&pack);

        assert!(!markdown.contains("TOKEN=secret"));
        assert!(pack
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source_policy_excluded"));
        assert!(pack
            .warnings
            .iter()
            .any(|warning| warning.contains(".env") && warning.contains("skipped")));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn pack_reports_deleted_generated_oversized_and_non_utf8_snippet_candidates() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        fs::write(
            repo.join("src/deleted.ts"),
            "export const deleted = true;\n",
        )
        .unwrap();
        fs::write(
            repo.join("dist/generated.ts"),
            "export const generated = true;\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/binary.ts"),
            [0xff, b'u', b'n', b's', b'a', b'f', b'e'],
        )
        .unwrap();
        fs::write(repo.join("src/huge.ts"), "x".repeat(1_000_001)).unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let mut plan = empty_plan_for_task(TaskType::BugFix);
        plan.target_files = vec![
            ctxhelm_core::TargetFile {
                path: "src/deleted.ts".to_string(),
                reason: "fixture".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: Vec::new(),
            },
            ctxhelm_core::TargetFile {
                path: "dist/generated.ts".to_string(),
                reason: "fixture".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: Vec::new(),
            },
            ctxhelm_core::TargetFile {
                path: "src/binary.ts".to_string(),
                reason: "fixture".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: Vec::new(),
            },
            ctxhelm_core::TargetFile {
                path: "src/huge.ts".to_string(),
                reason: "fixture".to_string(),
                line_range: None,
                confidence: 0.9,
                attribution: Vec::new(),
            },
        ];
        fs::remove_file(repo.join("src/deleted.ts")).unwrap();

        let pack = compile_context_pack_from_plan(&repo, "fix snippets", &plan, PackBudget::Deep);
        let markdown = render_pack_markdown(&pack);
        let diagnostic_codes = pack
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();

        assert!(!markdown.contains("generated = true"));
        assert!(!markdown.contains("unsafe"));
        assert!(!markdown.contains(&"x".repeat(80)));
        assert!(diagnostic_codes.contains(&"source_policy_excluded"));
        assert!(diagnostic_codes.contains(&"source_non_utf8"));
        assert!(diagnostic_codes.contains(&"source_oversized"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn compile_context_pack_with_plan_supports_source_free_eval_trace() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        std::env::set_var("CTXHELM_HOME", &home);

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
        assert_eq!(
            pack.repo_id,
            repo_id_for_path(&fs::canonicalize(&repo).unwrap())
        );
        assert_eq!(pack.task_hash, task_hash("fix requireSession bug"));
        assert_eq!(pack.target_agent, "generic");
        assert!(trace
            .recommended_files
            .contains(&"src/auth/session.ts".to_string()));
        assert_eq!(trace.recommended_tests, vec!["tests/auth/session.test.ts"]);
        assert_eq!(value["sourceTextLogged"], false);
        assert!(value.get("task").is_none());

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn ranking_metrics_historical_eval_reports_fixed_budget_without_source_text() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        fs::write(repo.join("dist/generated.min.js"), "generated\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "fix requireSession bug"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let report = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 5,
                ranking_budget: 10,
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            },
        )
        .unwrap();
        let value = serde_json::to_value(&report).unwrap();

        assert_eq!(report.evaluated_commits, 1);
        assert_eq!(report.eval_range_id.len(), 64);
        assert_eq!(report.budget, PackBudget::Standard);
        assert_eq!(report.effective_filters.limit, 5);
        assert_eq!(report.effective_filters.ranking_budget, 10);
        assert_eq!(report.effective_filters.mode, TaskType::BugFix);
        assert_eq!(report.effective_filters.budget, PackBudget::Standard);
        assert_eq!(report.ranking_comparison.k, 10);
        assert_eq!(report.ranking_comparison.combined.recall_at_k, 1.0);
        assert_eq!(report.ranking_comparison.combined.precision_at_k, 0.2);
        assert_eq!(report.ranking_comparison.combined.mrr_at_k, 1.0);
        assert_eq!(report.ranking_comparison.lexical_baseline.recall_at_k, 1.0);
        assert_eq!(
            report.ranking_comparison.no_context_baseline.recall_at_k,
            0.0
        );
        assert_eq!(report.ranking_comparison.recall_lift_at_k, 0.0);
        assert_eq!(
            report.ranking_comparison.recall_lift_vs_no_context_at_k,
            1.0
        );
        assert_eq!(
            report
                .ranking_comparison
                .combined
                .role_recall
                .iter()
                .find(|metric| metric.role == FileRole::Source)
                .unwrap()
                .recall_at_k,
            1.0
        );
        assert!(report.commits[0].recommended_context_files.len() <= report.ranking_comparison.k);
        assert_eq!(report.refs.base, None);
        assert_eq!(report.refs.head, None);
        assert_eq!(report.base, None);
        assert_eq!(report.head, None);
        assert_eq!(report.low_information_commit_count, 0);
        assert_eq!(report.commits[0].target_agent, "codex");
        assert_eq!(
            report.commits[0].safe_changed_files,
            vec!["src/auth/session.ts", "tests/auth/session.test.ts"]
        );
        assert!(report.commits[0]
            .recommended_context_files
            .contains(&"src/auth/session.ts".to_string()));
        assert!(report.commits[0]
            .recommended_context_files
            .contains(&"tests/auth/session.test.ts".to_string()));
        assert_eq!(report.commits[0].file_hits_at_5.len(), 2);
        assert_eq!(report.file_recall_at_5, 1.0);
        assert_eq!(report.file_recall_at_10, 1.0);
        assert_eq!(report.lexical_baseline_recall_at_5, 1.0);
        assert_eq!(report.lexical_baseline_recall_at_10, 1.0);
        assert_eq!(report.ctxhelm_lift_at_5, 0.0);
        assert_eq!(report.ctxhelm_lift_at_10, 0.0);
        assert_eq!(report.source_recall_at_5, 1.0);
        assert_eq!(report.source_recall_at_10, 1.0);
        assert_eq!(report.test_recall_at_5, 1.0);
        assert_eq!(report.test_recall_at_10, 1.0);
        assert_eq!(report.validation_command_recall, 0.0);
        assert_eq!(report.effective_validation_recall_at_10, 1.0);
        assert!(report.top_missing_files.is_empty());
        assert!(report.commits[0]
            .lexical_baseline_files
            .contains(&"src/auth/session.ts".to_string()));
        assert_eq!(report.commits[0].lexical_baseline_hits_at_5.len(), 2);
        assert_eq!(report.commits[0].lexical_baseline_hits_at_10.len(), 2);
        assert_eq!(report.commits[0].source_files_changed, 1);
        assert_eq!(report.commits[0].source_hits_at_5, 1);
        assert_eq!(report.commits[0].source_hits_at_10, 1);
        assert_eq!(report.commits[0].test_files_changed, 1);
        assert_eq!(report.commits[0].test_hits_at_5, 1);
        assert_eq!(report.commits[0].test_hits_at_10, 1);
        assert!(report.commits[0].changed_path_labels.iter().any(|label| {
            label.path == "src/auth/session.ts"
                && label.role == FileRole::Source
                && label.change_kind == ctxhelm_index::ChangeKind::Added
                && label.label_scope == ctxhelm_index::LabelScope::Safe
                && label.excluded_reason.is_none()
        }));
        assert!(report.commits[0].changed_path_labels.iter().any(|label| {
            label.path == "dist/generated.min.js"
                && label.role == FileRole::Generated
                && label.change_kind == ctxhelm_index::ChangeKind::Added
                && label.label_scope == ctxhelm_index::LabelScope::Generated
                && label.excluded_reason
                    == Some(ctxhelm_index::HistoricalPathExclusionReason::Generated)
        }));
        assert!(!report.commits[0].low_information_task);
        assert!(report.test_recommendation_rate > 0.0);
        assert_eq!(report.token_roi.len(), 3);
        assert_eq!(report.token_roi[0].budget, PackBudget::Brief);
        assert_eq!(report.token_roi[0].ranking_cutoff, 5);
        assert_eq!(report.token_roi[0].useful_targets, 2);
        assert!(report.token_roi[0].useful_targets_per_1k_tokens > 0.0);
        assert!(report.token_roi[2].larger_pack_adds_little_value);
        assert!(!report.commits[0].source_text_logged);
        assert!(value.get("commits").is_some());
        assert!(value.get("evalRangeId").is_some());
        assert_eq!(value["budget"], "standard");
        assert_eq!(value["effectiveFilters"]["limit"], 5);
        assert_eq!(value["effectiveFilters"]["rankingBudget"], 10);
        assert_eq!(value["effectiveFilters"]["mode"], "bug_fix");
        assert_eq!(value["effectiveFilters"]["budget"], "standard");
        assert_eq!(value["rankingComparison"]["k"], 10);
        assert_eq!(value["rankingComparison"]["combined"]["recallAtK"], 1.0);
        assert_eq!(
            value["rankingComparison"]["noContextBaseline"]["recallAtK"],
            0.0
        );
        assert_eq!(value["rankingComparison"]["recallLiftVsNoContextAtK"], 1.0);
        assert_eq!(value["tokenRoi"][0]["budget"], "brief");
        assert_eq!(value["tokenRoi"][0]["rankingCutoff"], 5);
        assert_eq!(value["tokenRoi"][0]["usefulTargets"], 2);
        assert!(
            (value["rankingComparison"]["combined"]["precisionAtK"]
                .as_f64()
                .unwrap()
                - 0.2)
                .abs()
                < 0.000001
        );
        assert_eq!(value["rankingComparison"]["combined"]["mrrAtK"], 1.0);
        assert_eq!(value["rankingComparison"]["recallLiftAtK"], 0.0);
        assert_eq!(value["refs"]["base"], serde_json::Value::Null);
        assert_eq!(value["refs"]["head"], serde_json::Value::Null);
        assert_eq!(
            value["commits"][0]["changedPathLabels"][0]["changeKind"],
            "added"
        );
        assert_eq!(value["lexicalBaselineRecallAt5"], 1.0);
        assert_eq!(value["ctxhelmLiftAt10"], 0.0);
        assert_eq!(value["sourceRecallAt5"], 1.0);
        assert_eq!(value["testRecallAt10"], 1.0);
        assert!(value["privacyStatus"]["localOnly"].as_bool().unwrap());
        assert!(value["commits"][0].get("title").is_none());
        assert!(value["commits"][0].get("task").is_none());
        assert!(!serde_json::to_string(&report)
            .unwrap()
            .contains("export function"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn historical_eval_separates_new_files_from_retrievable_context_targets() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("docs")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return false; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "seed auth session"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        fs::write(repo.join("docs/auth-session.md"), "auth session docs\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "fix auth session docs"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let report = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 1,
                ranking_budget: 10,
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            },
        )
        .unwrap();

        let commit = &report.commits[0];
        assert!(commit
            .safe_changed_files
            .contains(&"docs/auth-session.md".to_string()));
        assert_eq!(
            commit.retrieval_target_files,
            vec!["src/auth/session.ts".to_string()]
        );
        assert_eq!(
            commit.file_hits_at_10,
            vec!["src/auth/session.ts".to_string()]
        );
        assert_eq!(report.file_recall_at_10, 1.0);
        assert!(report
            .top_missing_files
            .iter()
            .all(|missing| missing.path != "docs/auth-session.md"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn paired_baseline_analysis_reports_variant_verdicts_without_source_text() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
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
        run_git(&repo, &["commit", "-m", "fix requireSession bug"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let historical = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 5,
                ranking_budget: 10,
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            },
        )
        .unwrap();
        let report = paired_baseline_analysis_report(&historical, 0.03, 0.03);
        let variants = report
            .rows
            .iter()
            .map(|row| row.variant.as_str())
            .collect::<Vec<_>>();

        for expected in [
            "ctxhelm_default",
            "lexical_baseline",
            "no_context",
            "semantic_only",
            "graph_only",
            "history_only",
            "test_only",
            "memory_only",
            "feedback_weighted",
        ] {
            assert!(variants.contains(&expected), "missing {expected}");
        }
        assert!(variants
            .iter()
            .any(|variant| variant.starts_with("without_")));
        assert_eq!(report.evaluated_commits, 1);
        assert_eq!(report.k, 10);
        assert_eq!(report.token_roi.len(), 3);
        assert!(report.signal_saturation.iter().any(|row| {
            row.signal == RetrievalSignalKind::RelatedTest && row.commits_with_signal >= 1
        }));
        assert_eq!(report.privacy_status, PrivacyStatus::local_only());
        assert!(!report.source_text_logged);
        assert_eq!(
            report
                .rows
                .iter()
                .find(|row| row.variant == "feedback_weighted")
                .unwrap()
                .verdict,
            PairedBaselineVerdict::InsufficientEvidence
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("lexical_baseline"));
        assert!(!json.contains("export function"));
        assert!(!json.contains("fix requireSession bug"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn historical_eval_reuses_source_free_cache_and_parallelism_metadata() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "fix requireSession bug"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let options = HistoricalEvalOptions {
            limit: 5,
            ranking_budget: 10,
            task_type: TaskType::BugFix,
            target_agent: "codex".to_string(),
            base: None,
            head: None,
            semantic_enabled: false,
            semantic_provider: SemanticProviderConfig::default(),
            local_metadata_reranker: false,
            query_family_routed_reranker: false,
            cache_enabled: true,
            force_refresh: false,
            parallelism: 2,
        };

        let first = evaluate_historical_commits(&repo, &options).unwrap();
        let second = evaluate_historical_commits(&repo, &options).unwrap();
        let forced = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                force_refresh: true,
                ..options
            },
        )
        .unwrap();

        assert_eq!(first.evaluated_commits, 1);
        assert_eq!(first.runtime.cache_hits, 0);
        assert_eq!(first.runtime.cache_misses, 1);
        assert_eq!(first.runtime.parallelism, 1);
        assert_eq!(second.eval_range_id, first.eval_range_id);
        assert_eq!(second.runtime.cache_hits, 1);
        assert_eq!(second.runtime.cache_misses, 0);
        assert_eq!(second.runtime.parallelism, 1);
        assert_eq!(forced.eval_range_id, first.eval_range_id);
        assert_eq!(forced.runtime.cache_hits, 0);
        assert_eq!(forced.runtime.cache_misses, 1);

        let cache_root = home
            .join("repos")
            .join(&first.repo_id)
            .join("eval-cache")
            .join(format!("{}.json", first.eval_range_id));
        assert!(cache_root.exists());
        let json = fs::read_to_string(cache_root).unwrap();
        assert!(json.contains("\"sourceTextLogged\": false"));
        assert!(!json.contains("return true"));
        assert!(!json.contains("fix requireSession bug"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn historical_eval_report_public_json_shape_is_stable() {
        let report = HistoricalEvalReport {
            eval_range_id: "range-1".to_string(),
            repo_id: "repo-1".to_string(),
            evaluated_commits: 1,
            budget: PackBudget::Standard,
            effective_filters: HistoricalEvalEffectiveFilters {
                limit: 1,
                ranking_budget: 10,
                mode: TaskType::BugFix,
                target_agent: "codex".to_string(),
                budget: PackBudget::Standard,
                semantic_enabled: false,
                semantic_provider: None,
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
            },
            refs: HistoricalEvalRefs {
                base: Some("base".to_string()),
                head: Some("head".to_string()),
            },
            base: Some("base".to_string()),
            head: Some("head".to_string()),
            ranking_comparison: EvalComparison {
                k: 10,
                combined: RankingMetrics {
                    k: 10,
                    recall_at_k: 1.0,
                    precision_at_k: 0.1,
                    mrr_at_k: 1.0,
                    role_recall: vec![RoleRecallMetric {
                        role: FileRole::Source,
                        recall_at_k: 1.0,
                        changed_file_count: 1,
                        hit_count: 1,
                    }],
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 1.0,
                },
                lexical_baseline: RankingMetrics {
                    k: 10,
                    recall_at_k: 1.0,
                    precision_at_k: 0.1,
                    mrr_at_k: 1.0,
                    role_recall: vec![RoleRecallMetric {
                        role: FileRole::Source,
                        recall_at_k: 1.0,
                        changed_file_count: 1,
                        hit_count: 1,
                    }],
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 1.0,
                },
                no_context_baseline: RankingMetrics {
                    k: 10,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: vec![RoleRecallMetric {
                        role: FileRole::Source,
                        recall_at_k: 0.0,
                        changed_file_count: 1,
                        hit_count: 0,
                    }],
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 0.0,
                },
                recall_lift_at_k: 0.0,
                precision_lift_at_k: 0.0,
                mrr_lift_at_k: 0.0,
                recall_lift_vs_no_context_at_k: 1.0,
                precision_lift_vs_no_context_at_k: 0.1,
                mrr_lift_vs_no_context_at_k: 1.0,
            },
            signal_ablations: Vec::new(),
            graph_edge_ablations: Vec::new(),
            token_roi: vec![
                TokenRoiMetric {
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
                TokenRoiMetric {
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
                TokenRoiMetric {
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
            graph_edge_profiles: Vec::new(),
            runtime: HistoricalEvalRuntimeSummary {
                total_millis: 120,
                commit_millis: 120,
                overhead_millis: 0,
                average_commit_millis: 120.0,
                cache_hits: 0,
                cache_misses: 1,
                parallelism: 1,
                git_sample_millis: 0,
                ranking_millis: 0,
                pack_compiler_millis: 0,
                slow_commits: vec![HistoricalSlowCommitSummary {
                    sha: "abc123".to_string(),
                    elapsed_millis: 120,
                    safe_changed_file_count: 1,
                    recommended_context_file_count: 1,
                    missing_file_count_at_10: 0,
                    low_information_task: false,
                }],
            },
            low_information_commit_count: 0,
            broad_scope_commit_count: 0,
            broad_context_area_recall: 1.0,
            context_area_pressure_summary: ContextAreaPressureSummary {
                context_area_count: 1,
                zero_selected_area_count: 0,
                total_inspection_pressure: 0,
                source_like_unselected: 0,
                validation_unselected: 0,
                docs_unselected: 0,
                source_like_pressure: 0,
                validation_pressure: 0,
                docs_pressure: 0,
                highest_pressure_area: Some(ContextAreaPressurePeak {
                    area: "src".to_string(),
                    resource_uri: "ctxhelm://repo/context-area/src".to_string(),
                    inspection_pressure: 0,
                    coverage_percent: 100,
                    unselected_count: 0,
                }),
                source_text_logged: false,
            },
            context_area_next_read_summary: ContextAreaNextReadSummary {
                missed_file_count_at_10: 3,
                next_read_recoverable_count: 2,
                agent_evidence_recoverable_count: 2,
                agent_evidence_only_count: 0,
                agent_evidence_only_role_counts: BTreeMap::new(),
                top_agent_evidence_only_areas: Vec::new(),
                top_pressure_next_read_recoverable_count: 1,
                zero_selected_area_recoverable_count: 1,
                source_text_logged: false,
            },
            candidate_coverage_summary: CandidateCoverageSummary {
                missed_file_count_at_10: 3,
                candidate_recoverable_count: 2,
                no_candidate_count: 1,
                candidate_recoverable_role_counts: BTreeMap::from([
                    ("source".to_string(), 1),
                    ("test".to_string(), 1),
                ]),
                candidate_recoverable_signal_counts: BTreeMap::from([
                    ("co_change".to_string(), 1),
                    ("dependency".to_string(), 1),
                ]),
                no_candidate_role_counts: BTreeMap::from([("docs".to_string(), 1)]),
                top_candidate_recoverable_areas: vec![CandidateCoverageAreaSummary {
                    context_area: "src/auth".to_string(),
                    missed_count: 2,
                }],
                source_text_logged: false,
            },
            memory_reuse_summary: MemoryReuseSummary {
                commits_with_memory_candidates: 1,
                memory_candidate_count: 2,
                memory_selected_at_10_count: 1,
                memory_target_hit_at_10_count: 1,
                memory_target_missed_at_10_count: 0,
                memory_unique_target_hit_count: 1,
                memory_unique_non_target_count: 1,
                memory_unique_target_hit_with_current_support_count: 1,
                memory_unique_target_hit_without_current_support_count: 0,
                memory_unique_non_target_with_current_support_count: 0,
                memory_unique_non_target_without_current_support_count: 1,
                memory_unique_target_hit_current_support_signal_counts: BTreeMap::from([(
                    "dependency".to_string(),
                    1,
                )]),
                memory_unique_non_target_current_support_signal_counts: BTreeMap::new(),
                selected_role_counts: BTreeMap::from([("source".to_string(), 1)]),
                source_text_logged: false,
            },
            recommended_research_actions: vec![RecommendedResearchAction {
                action: "improve_candidate_generation".to_string(),
                priority: 1,
                origin: "historical_eval".to_string(),
                reason: "Some missed retrieval targets have no generated source-free candidate."
                    .to_string(),
            }],
            file_recall_at_5: 0.5,
            file_recall_at_10: 1.0,
            lexical_baseline_recall_at_5: 0.25,
            lexical_baseline_recall_at_10: 0.75,
            ctxhelm_lift_at_5: 0.25,
            ctxhelm_lift_at_10: 0.25,
            source_recall_at_5: 1.0,
            source_recall_at_10: 1.0,
            test_recall_at_5: 0.0,
            test_recall_at_10: 0.0,
            validation_command_recall: 0.0,
            effective_validation_recall_at_10: 0.0,
            test_recommendation_rate: 0.0,
            average_recommended_context_files: 3.0,
            protected_evidence: ProtectedEvidenceSummary::default(),
            top_missing_files: vec![HistoricalMissingFileSummary {
                path: "src/missing.rs".to_string(),
                role: FileRole::Source,
                missed_count: 2,
            }],
            commits: vec![HistoricalCommitEval {
                sha: "abc123".to_string(),
                task_hash: "hash-1".to_string(),
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                changed_path_labels: vec![HistoricalChangedPathLabel {
                    path: "src/lib.rs".to_string(),
                    old_path: None,
                    role: FileRole::Source,
                    change_kind: ctxhelm_index::ChangeKind::Modified,
                    label_scope: ctxhelm_index::LabelScope::Safe,
                    excluded_reason: None,
                }],
                safe_changed_files: vec!["src/lib.rs".to_string()],
                retrieval_target_files: vec!["src/lib.rs".to_string()],
                excluded_changed_file_count: 0,
                recommended_files: vec!["src/lib.rs".to_string()],
                recommended_tests: Vec::new(),
                recommended_context_files: vec!["src/lib.rs".to_string()],
                recommended_commands: Vec::new(),
                lexical_baseline_files: vec!["src/lib.rs".to_string()],
                signal_baseline_files: Vec::new(),
                selected_signal_profiles: Vec::new(),
                protected_evidence: Vec::new(),
                graph_edge_profiles: Vec::new(),
                file_hits_at_5: vec!["src/lib.rs".to_string()],
                file_hits_at_10: vec!["src/lib.rs".to_string()],
                lexical_baseline_hits_at_5: vec!["src/lib.rs".to_string()],
                lexical_baseline_hits_at_10: vec!["src/lib.rs".to_string()],
                missing_files_at_10: Vec::new(),
                candidate_missed_files_at_10: Vec::new(),
                candidate_missed_file_profiles_at_10: Vec::new(),
                source_files_changed: 1,
                source_hits_at_5: 1,
                source_hits_at_10: 1,
                test_files_changed: 0,
                test_hits_at_5: 0,
                test_hits_at_10: 0,
                validation_command_hits: 0,
                effective_validation_hits_at_10: 0,
                low_information_task: false,
                broad_scope_task: false,
                changed_context_areas: vec!["src".to_string()],
                context_area_hits: vec!["src".to_string()],
                context_areas: vec![ctxhelm_core::ContextArea {
                    area: "src".to_string(),
                    reason: "source area".to_string(),
                    resource_uri: "ctxhelm://repo/context-area/src".to_string(),
                    representative_paths: vec!["src/lib.rs".to_string()],
                    next_read_paths: vec![],
                    signal_counts: BTreeMap::from([("lexical".to_string(), 1)]),
                    role_counts: BTreeMap::from([("source".to_string(), 1)]),
                    selected_role_counts: BTreeMap::from([("source".to_string(), 1)]),
                    candidate_count: 1,
                    selected_count: 1,
                    unselected_count: 0,
                    coverage_percent: 100,
                    inspection_pressure: 0,
                    inspection_pressure_breakdown:
                        ctxhelm_core::InspectionPressureBreakdown::default(),
                }],
                confidence: 0.8,
                query_trace: None,
                elapsed_millis: 120,
                source_text_logged: false,
            }],
            privacy_status: PrivacyStatus::local_only(),
        };

        let value = serde_json::to_value(&report).unwrap();
        let object = value.as_object().unwrap();

        for key in [
            "evalRangeId",
            "repoId",
            "evaluatedCommits",
            "budget",
            "effectiveFilters",
            "refs",
            "base",
            "head",
            "rankingComparison",
            "signalAblations",
            "tokenRoi",
            "retrievalGapSummaries",
            "runtime",
            "lowInformationCommitCount",
            "broadScopeCommitCount",
            "broadContextAreaRecall",
            "contextAreaPressureSummary",
            "contextAreaNextReadSummary",
            "candidateCoverageSummary",
            "fileRecallAt5",
            "fileRecallAt10",
            "lexicalBaselineRecallAt5",
            "lexicalBaselineRecallAt10",
            "ctxhelmLiftAt5",
            "ctxhelmLiftAt10",
            "sourceRecallAt5",
            "sourceRecallAt10",
            "testRecallAt5",
            "testRecallAt10",
            "validationCommandRecall",
            "effectiveValidationRecallAt10",
            "testRecommendationRate",
            "averageRecommendedContextFiles",
            "protectedEvidence",
            "topMissingFiles",
            "commits",
            "privacyStatus",
        ] {
            assert!(object.contains_key(key), "missing public field {key}");
        }
        assert_eq!(value["evaluatedCommits"], 1);
        assert_eq!(value["evalRangeId"], "range-1");
        assert_eq!(value["budget"], "standard");
        assert_eq!(value["effectiveFilters"]["targetAgent"], "codex");
        assert_eq!(value["refs"]["base"], "base");
        assert_eq!(value["rankingComparison"]["combined"]["mrrAtK"], 1.0);
        assert_eq!(
            value["rankingComparison"]["noContextBaseline"]["recallAtK"],
            0.0
        );
        assert_eq!(value["rankingComparison"]["recallLiftVsNoContextAtK"], 1.0);
        assert_eq!(value["runtime"]["totalMillis"], 120);
        assert_eq!(value["runtime"]["commitMillis"], 120);
        assert_eq!(value["runtime"]["overheadMillis"], 0);
        assert_eq!(value["runtime"]["slowCommits"][0]["sha"], "abc123");
        assert_eq!(value["contextAreaPressureSummary"]["contextAreaCount"], 1);
        assert_eq!(
            value["contextAreaPressureSummary"]["highestPressureArea"]["area"],
            "src"
        );
        assert_eq!(
            value["contextAreaPressureSummary"]["sourceTextLogged"],
            false
        );
        assert_eq!(
            value["contextAreaNextReadSummary"]["missedFileCountAt10"],
            3
        );
        assert_eq!(
            value["contextAreaNextReadSummary"]["nextReadRecoverableCount"],
            2
        );
        assert_eq!(
            value["contextAreaNextReadSummary"]["agentEvidenceRecoverableCount"],
            2
        );
        assert_eq!(
            value["contextAreaNextReadSummary"]["topPressureNextReadRecoverableCount"],
            1
        );
        assert_eq!(
            value["contextAreaNextReadSummary"]["zeroSelectedAreaRecoverableCount"],
            1
        );
        assert_eq!(
            value["contextAreaNextReadSummary"]["sourceTextLogged"],
            false
        );
        assert_eq!(value["candidateCoverageSummary"]["missedFileCountAt10"], 3);
        assert_eq!(
            value["candidateCoverageSummary"]["candidateRecoverableCount"],
            2
        );
        assert_eq!(value["candidateCoverageSummary"]["noCandidateCount"], 1);
        assert_eq!(
            value["candidateCoverageSummary"]["candidateRecoverableRoleCounts"]["source"],
            1
        );
        assert_eq!(
            value["candidateCoverageSummary"]["candidateRecoverableSignalCounts"]["dependency"],
            1
        );
        assert_eq!(
            value["candidateCoverageSummary"]["noCandidateRoleCounts"]["docs"],
            1
        );
        assert_eq!(
            value["candidateCoverageSummary"]["topCandidateRecoverableAreas"][0]["contextArea"],
            "src/auth"
        );
        assert_eq!(value["candidateCoverageSummary"]["sourceTextLogged"], false);
        assert_eq!(
            value["memoryReuseSummary"]["commitsWithMemoryCandidates"],
            1
        );
        assert_eq!(value["memoryReuseSummary"]["memoryCandidateCount"], 2);
        assert_eq!(value["memoryReuseSummary"]["memoryUniqueTargetHitCount"], 1);
        assert_eq!(
            value["memoryReuseSummary"]["selectedRoleCounts"]["source"],
            1
        );
        assert_eq!(value["memoryReuseSummary"]["sourceTextLogged"], false);
        assert_eq!(
            value["recommendedResearchActions"][0]["action"],
            "improve_candidate_generation"
        );
        assert_eq!(value["recommendedResearchActions"][0]["priority"], 1);
        assert_eq!(
            value["recommendedResearchActions"][0]["origin"],
            "historical_eval"
        );
        assert!(value["signalAblations"].as_array().unwrap().is_empty());
        assert_eq!(value["tokenRoi"][0]["budget"], "brief");
        assert_eq!(value["tokenRoi"][1]["largerPackAddsLittleValue"], true);
        assert!(value["retrievalGapSummaries"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(value["fileRecallAt5"], 0.5);
        assert_eq!(value["lexicalBaselineRecallAt5"], 0.25);
        assert_eq!(value["sourceRecallAt5"], 1.0);
        assert_eq!(value["testRecallAt5"], 0.0);
        assert_eq!(value["topMissingFiles"][0]["path"], "src/missing.rs");
        assert_eq!(value["privacyStatus"]["localOnly"], true);
        assert_eq!(value["commits"][0]["elapsedMillis"], 120);
        assert_eq!(value["commits"][0]["sourceTextLogged"], false);
        assert_eq!(
            value["commits"][0]["changedPathLabels"][0]["changeKind"],
            "modified"
        );

        assert!(value.get("repo_id").is_none());
        assert!(value.get("evaluated_commits").is_none());
        assert!(value.get("eval_range_id").is_none());
        assert!(value.get("file_recall_at_5").is_none());
        assert!(value.get("lexical_baseline_recall_at_5").is_none());
        assert!(value.get("top_missing_files").is_none());
        assert!(value.get("privacy_status").is_none());
        assert!(serde_json::to_string(&report)
            .unwrap()
            .contains("\"sourceTextLogged\":false"));
    }

    #[test]
    fn historical_eval_uses_parent_snapshot_without_future_context() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/future")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(repo.join("README.md"), "# Repo\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "initial repo"]);
        fs::write(
            repo.join("src/future/widget.ts"),
            "export function FutureWidget() { return true; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add FutureWidget support"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let report = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 1,
                ranking_budget: 10,
                task_type: TaskType::Feature,
                target_agent: "codex".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            },
        )
        .unwrap();

        assert_eq!(report.evaluated_commits, 1);
        assert_eq!(
            report.commits[0].safe_changed_files,
            vec!["src/future/widget.ts"]
        );
        assert!(report.commits[0].retrieval_target_files.is_empty());
        assert_eq!(
            report.commits[0].changed_path_labels[0].label_scope,
            ctxhelm_index::LabelScope::Safe
        );
        assert!(!report.commits[0]
            .recommended_context_files
            .contains(&"src/future/widget.ts".to_string()));
        assert_eq!(report.file_recall_at_10, 0.0);
        assert_eq!(report.lexical_baseline_recall_at_10, 0.0);
        assert!(report.top_missing_files.is_empty());

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn historical_eval_uses_parent_bounded_sidecar_for_cochanged_tests() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("tests/qwen35")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/dispatcher.ts"),
            "export function dispatchRetry() { return 'initial'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/qwen35/test_workflow_smoke.py"),
            "def test_workflow_smoke():\n    assert True\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add dispatcher workflow"]);
        fs::write(
            repo.join("src/dispatcher.ts"),
            "export function dispatchRetry() { return 'fixed'; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/qwen35/test_workflow_smoke.py"),
            "def test_workflow_smoke():\n    assert 'fixed'\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "fix dispatcher retry"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let report = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 1,
                ranking_budget: 10,
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            },
        )
        .unwrap();

        let commit = &report.commits[0];
        assert!(commit
            .retrieval_target_files
            .contains(&"src/dispatcher.ts".to_string()));
        assert!(commit
            .retrieval_target_files
            .contains(&"tests/qwen35/test_workflow_smoke.py".to_string()));
        assert_eq!(commit.test_files_changed, 1);
        assert_eq!(commit.test_hits_at_10, 1);
        assert!(commit
            .recommended_tests
            .contains(&"tests/qwen35/test_workflow_smoke.py".to_string()));
        assert!(commit
            .recommended_commands
            .contains(&"pytest tests/qwen35/test_workflow_smoke.py".to_string()));
        assert!(serde_json::to_string(&report)
            .unwrap()
            .contains("\"sourceTextLogged\":false"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn ablation_historical_eval_groups_source_free_retrieval_gaps() {
        let report = HistoricalEvalReport {
            eval_range_id: "range-1".to_string(),
            repo_id: "repo-1".to_string(),
            evaluated_commits: 1,
            budget: PackBudget::Standard,
            effective_filters: HistoricalEvalEffectiveFilters {
                limit: 1,
                ranking_budget: 2,
                mode: TaskType::BugFix,
                target_agent: "codex".to_string(),
                budget: PackBudget::Standard,
                semantic_enabled: false,
                semantic_provider: None,
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
            },
            refs: HistoricalEvalRefs {
                base: None,
                head: None,
            },
            base: None,
            head: None,
            ranking_comparison: EvalComparison {
                k: 2,
                combined: RankingMetrics {
                    k: 2,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 1.0,
                },
                lexical_baseline: RankingMetrics {
                    k: 2,
                    recall_at_k: 1.0,
                    precision_at_k: 0.5,
                    mrr_at_k: 1.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 1.0,
                },
                no_context_baseline: RankingMetrics {
                    k: 2,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 0.0,
                },
                recall_lift_at_k: -1.0,
                precision_lift_at_k: -0.5,
                mrr_lift_at_k: -1.0,
                recall_lift_vs_no_context_at_k: 0.0,
                precision_lift_vs_no_context_at_k: 0.0,
                mrr_lift_vs_no_context_at_k: 0.0,
            },
            signal_ablations: vec![SignalAblationResult {
                eval_range_id: "range-1".to_string(),
                disabled_signal: RetrievalSignalKind::Symbol,
                evaluated_commits: 1,
                metrics: RankingMetrics {
                    k: 2,
                    recall_at_k: 0.0,
                    precision_at_k: 0.0,
                    mrr_at_k: 0.0,
                    role_recall: Vec::new(),
                    test_recommendation_rate: 0.0,
                    average_recommended_context_files: 1.0,
                },
                recall_lift_vs_lexical_at_k: -1.0,
            }],
            graph_edge_ablations: Vec::new(),
            token_roi: Vec::new(),
            retrieval_gap_summaries: vec![RetrievalGapSummary {
                role: FileRole::Source,
                signal_gap: "lexical_only_miss".to_string(),
                package: "src".to_string(),
                path_family: "src/auth/*.ts".to_string(),
                context_area: "src/auth".to_string(),
                context_area_resource_uri: "ctxhelm://repo/context-area/src%2Fauth".to_string(),
                context_area_signal_counts: BTreeMap::from([("lexical".to_string(), 2)]),
                context_area_role_counts: BTreeMap::from([("source".to_string(), 3)]),
                context_area_selected_role_counts: BTreeMap::from([("source".to_string(), 1)]),
                context_area_unselected_count: 2,
                context_area_inspection_pressure_breakdown:
                    ctxhelm_core::InspectionPressureBreakdown {
                        source_like_unselected: 2,
                        validation_unselected: 0,
                        docs_unselected: 0,
                        source_like_weight: 3,
                        validation_weight: 2,
                        docs_weight: 1,
                        total: 6,
                    },
                target_status: RetrievalGapTargetStatus::CurrentReachable,
                recommendation_area: RetrievalGapRecommendationArea::LexicalRanking,
                missed_count: 2,
                example_paths: vec!["src/auth/session.ts".to_string()],
                next_read_paths: vec!["src/auth/session.ts".to_string()],
            }],
            graph_edge_profiles: Vec::new(),
            runtime: HistoricalEvalRuntimeSummary {
                total_millis: 0,
                commit_millis: 0,
                overhead_millis: 0,
                average_commit_millis: 0.0,
                cache_hits: 0,
                cache_misses: 1,
                parallelism: 1,
                git_sample_millis: 0,
                ranking_millis: 0,
                pack_compiler_millis: 0,
                slow_commits: Vec::new(),
            },
            low_information_commit_count: 0,
            broad_scope_commit_count: 0,
            broad_context_area_recall: 0.0,
            context_area_pressure_summary: ContextAreaPressureSummary::default(),
            context_area_next_read_summary: ContextAreaNextReadSummary::default(),
            candidate_coverage_summary: CandidateCoverageSummary::default(),
            memory_reuse_summary: MemoryReuseSummary::default(),
            recommended_research_actions: Vec::new(),
            file_recall_at_5: 0.0,
            file_recall_at_10: 0.0,
            lexical_baseline_recall_at_5: 1.0,
            lexical_baseline_recall_at_10: 1.0,
            ctxhelm_lift_at_5: -1.0,
            ctxhelm_lift_at_10: -1.0,
            source_recall_at_5: 0.0,
            source_recall_at_10: 0.0,
            test_recall_at_5: 0.0,
            test_recall_at_10: 0.0,
            validation_command_recall: 0.0,
            effective_validation_recall_at_10: 0.0,
            test_recommendation_rate: 0.0,
            average_recommended_context_files: 1.0,
            protected_evidence: ProtectedEvidenceSummary::default(),
            top_missing_files: Vec::new(),
            commits: Vec::new(),
            privacy_status: PrivacyStatus::local_only(),
        };
        let value = serde_json::to_value(&report).unwrap();

        assert_eq!(value["signalAblations"][0]["evalRangeId"], "range-1");
        assert_eq!(value["signalAblations"][0]["evaluatedCommits"], 1);
        assert_eq!(value["signalAblations"][0]["disabledSignal"], "symbol");
        assert_eq!(
            value["retrievalGapSummaries"][0]["signalGap"],
            "lexical_only_miss"
        );
        assert_eq!(value["retrievalGapSummaries"][0]["package"], "src");
        assert_eq!(
            value["retrievalGapSummaries"][0]["targetStatus"],
            "currentReachable"
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["recommendationArea"],
            "lexicalRanking"
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["pathFamily"],
            "src/auth/*.ts"
        );
        assert_eq!(value["retrievalGapSummaries"][0]["contextArea"], "src/auth");
        assert_eq!(
            value["retrievalGapSummaries"][0]["contextAreaResourceUri"],
            "ctxhelm://repo/context-area/src%2Fauth"
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["contextAreaSignalCounts"]["lexical"],
            2
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["contextAreaRoleCounts"]["source"],
            3
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["contextAreaSelectedRoleCounts"]["source"],
            1
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["contextAreaUnselectedCount"],
            2
        );
        assert_eq!(
            value["retrievalGapSummaries"][0]["nextReadPaths"][0],
            "src/auth/session.ts"
        );
        assert!(!serde_json::to_string(&report).unwrap().contains("fix auth"));
    }

    #[test]
    fn historical_eval_labels_deletes_as_historical_only() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(repo.join("src/legacy.ts"), "export const legacy = true;\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add legacy file"]);
        fs::remove_file(repo.join("src/legacy.ts")).unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "delete legacy file"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let report = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 1,
                ranking_budget: 10,
                task_type: TaskType::BugFix,
                target_agent: "codex".to_string(),
                base: None,
                head: None,
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
            },
        )
        .unwrap();

        assert_eq!(report.evaluated_commits, 1);
        assert!(report.commits[0].safe_changed_files.is_empty());
        assert_eq!(report.commits[0].changed_path_labels.len(), 1);
        assert_eq!(
            report.commits[0].changed_path_labels[0].change_kind,
            ctxhelm_index::ChangeKind::Deleted
        );
        assert_eq!(
            report.commits[0].changed_path_labels[0].label_scope,
            ctxhelm_index::LabelScope::HistoricalOnly
        );
        assert_eq!(report.file_recall_at_10, 0.0);
        assert!(!serde_json::to_string(&report)
            .unwrap()
            .contains("export const legacy"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn historical_eval_parent_snapshot_extracts_only_indexable_paths() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("target/generated")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(repo.join("src/lib.ts"), "export const visible = true;\n").unwrap();
        fs::write(
            repo.join("target/generated/huge.ts"),
            "export const generated = true;\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "initial repo"]);
        let revision = git_stdout(&repo, &["rev-parse", "HEAD"]);

        let snapshot = HistoricalEvalWorktree::for_parent(
            &repo,
            Some(revision.trim()),
            &["src/lib.ts".to_string()],
        )
        .unwrap();

        assert!(snapshot.path().join("src/lib.ts").exists());
        assert!(!snapshot.path().join("target/generated/huge.ts").exists());
        assert!(!snapshot.path().join(".git").exists());
    }

    #[test]
    fn generate_context_cards_writes_source_free_repo_cards() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        fs::create_dir_all(repo.join("dist")).unwrap();
        run_git(&repo, &["init"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "import { parseCookie } from './cookies';\nexport function requireSession() { return parseCookie(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/auth/cookies.ts"),
            "export function parseCookie() { return true; }\n",
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            "import { requireSession } from '../../src/auth/session';\n",
        )
        .unwrap();
        fs::write(repo.join("dist/generated.min.js"), "generated\n").unwrap();
        fs::write(repo.join(".env"), "TOKEN=secret\n").unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let report = generate_context_cards(&repo, &ContextCardsOptions { limit: 20 }).unwrap();

        assert_eq!(report.cards.len(), 5);
        assert_eq!(
            report.cards_dir,
            fs::canonicalize(&repo).unwrap().join(".ctxhelm/cards")
        );
        let overview = fs::read_to_string(repo.join(".ctxhelm/cards/repo-overview.md")).unwrap();
        let testing = fs::read_to_string(repo.join(".ctxhelm/cards/testing.md")).unwrap();
        let dependencies =
            fs::read_to_string(repo.join(".ctxhelm/cards/dependency-graph.md")).unwrap();
        let domain_src = fs::read_to_string(repo.join(".ctxhelm/cards/domain-src.md")).unwrap();

        assert!(overview.contains("# Repo Overview"));
        assert!(overview.contains("`src/auth/session.ts`"));
        assert!(overview.contains("`requireSession`"));
        assert!(testing.contains("# Testing"));
        assert!(testing.contains("`tests/auth/session.test.ts`"));
        assert!(testing.contains("pnpm test tests/auth/session.test.ts"));
        assert!(dependencies.contains("# Dependency Graph"));
        assert!(dependencies.contains("`src/auth/session.ts` -> `src/auth/cookies.ts`"));
        assert!(domain_src.contains("Memory card ID"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "memory_cards_persisted"));
        for content in [&overview, &testing, &dependencies, &domain_src] {
            assert!(content.contains("Source snippets included: `false`"));
            assert!(!content.contains("return parseCookie"));
            assert!(!content.contains("TOKEN=secret"));
            assert!(!content.contains("generated.min.js"));
        }

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn prepare_and_pack_select_fresh_memory_cards() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        generate_context_cards(&repo, &ContextCardsOptions { limit: 20 }).unwrap();
        let (plan, pack) = compile_context_pack_with_plan(
            &repo,
            "fix requireSession auth session bug",
            TaskType::BugFix,
            PackBudget::Brief,
        )
        .unwrap();
        let markdown = render_pack_markdown(&pack);

        assert!(!plan.selected_memory.is_empty());
        assert!(plan
            .retrieval_candidates
            .iter()
            .any(|candidate| candidate.kind == RetrievalCandidateKind::Memory));
        assert!(markdown.contains("## Selected memory"));
        assert!(markdown.contains("Native-read source links"));
        assert!(markdown.contains("selected-memory source/evidence path"));
        assert!(!markdown.contains("sourceText"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn approved_experience_memory_promotes_source_linked_targets() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/payments")).unwrap();
        fs::write(
            repo.join("src/payments/handler.ts"),
            "export function handlePayment() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let task = "fix checkout signature regression";
        let without_memory = prepare_context_plan(&repo, task, TaskType::BugFix).unwrap();
        assert!(
            !without_memory
                .target_files
                .iter()
                .any(|target| target.path == "src/payments/handler.ts"),
            "memory-free task should not find the hidden source link"
        );

        persist_memory_card_records(
            &repo,
            &StoreConfig::default(),
            &[StorageMemoryCardRecord {
                card: MemoryCard {
                    id: "experience:checkout-signature".to_string(),
                    kind: MemoryCardKind::Experience,
                    title: "Experience: checkout signature regression".to_string(),
                    summary: "A prior approved run for checkout signature work used the payments handler."
                        .to_string(),
                    source_links: vec!["src/payments/handler.ts".to_string()],
                    input_hashes: vec!["task-hash".to_string()],
                    freshness: MemoryFreshness::Fresh,
                    review_status: MemoryReviewStatus::Approved,
                    disabled: false,
                    confidence: 0.90,
                    reason: "test approved source-free experience card".to_string(),
                    privacy_status: PrivacyStatus::local_only(),
                },
            }],
        )
        .unwrap();

        let with_memory = prepare_context_plan(&repo, task, TaskType::BugFix).unwrap();
        let target = with_memory
            .target_files
            .iter()
            .find(|target| target.path == "src/payments/handler.ts")
            .expect("approved memory should promote source-linked file into target files");
        assert_eq!(target.reason, "approved memory source link");
        assert!(target.attribution.iter().any(|evidence| {
            evidence.signal == RetrievalSignalKind::Memory
                && evidence.reason_code == "memory_source_link"
        }));
        assert!(with_memory.selected_memory.iter().any(|memory| {
            memory.card.id == "experience:checkout-signature"
                && memory
                    .evidence
                    .iter()
                    .any(|evidence| evidence.reason_code == "memory_task_overlap")
        }));
        assert!(with_memory.retrieval_candidates.iter().any(|candidate| {
            candidate.path.as_deref() == Some("src/payments/handler.ts")
                && candidate
                    .signal_scores
                    .iter()
                    .any(|score| score.signal == RetrievalSignalKind::Memory)
        }));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn selected_memory_source_links_displace_tail_targets_for_initial_reads() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/payments")).unwrap();
        fs::create_dir_all(repo.join("src/noisy")).unwrap();
        fs::write(
            repo.join("src/payments/handler.ts"),
            "export function handlePayment() { return true; }\n",
        )
        .unwrap();
        for index in 0..12 {
            fs::write(
                repo.join(format!("src/noisy/candidate_{index}.ts")),
                "export const note = 'checkout signature regression search hit';\n",
            )
            .unwrap();
        }
        std::env::set_var("CTXHELM_HOME", &home);

        persist_memory_card_records(
            &repo,
            &StoreConfig::default(),
            &[StorageMemoryCardRecord {
                card: MemoryCard {
                    id: "experience:checkout-signature-tail".to_string(),
                    kind: MemoryCardKind::Experience,
                    title: "Experience: checkout signature regression".to_string(),
                    summary:
                        "The payments handler was the useful source link for a prior checkout fix."
                            .to_string(),
                    source_links: vec!["src/payments/handler.ts".to_string()],
                    input_hashes: vec!["task-hash".to_string()],
                    freshness: MemoryFreshness::Fresh,
                    review_status: MemoryReviewStatus::Approved,
                    disabled: false,
                    confidence: 0.90,
                    reason: "test approved source-free experience card".to_string(),
                    privacy_status: PrivacyStatus::local_only(),
                },
            }],
        )
        .unwrap();

        let plan =
            prepare_context_plan(&repo, "fix checkout signature regression", TaskType::BugFix)
                .unwrap();
        let promoted_index = plan
            .target_files
            .iter()
            .position(|target| target.path == "src/payments/handler.ts")
            .expect("selected memory source link should be promoted into targetFiles");
        let promoted = &plan.target_files[promoted_index];
        assert!(
            promoted_index < 5,
            "selected memory source link must land in the native first-read window"
        );
        assert_eq!(promoted.reason, "selected memory source link");
        assert!(promoted.attribution.iter().any(|evidence| {
            evidence.signal == RetrievalSignalKind::Memory
                && evidence.reason_code == "selected_memory_initial_read"
        }));
        assert!(plan.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "selected_memory_initial_read_promoted"
                && diagnostic
                    .paths
                    .contains(&"src/payments/handler.ts".to_string())
        }));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn experience_cards_preserve_recommended_file_order_before_tests() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join(".git")).unwrap();
        std::env::set_var("CTXHELM_HOME", &home);
        let repo_id = repo_id_for_path(&fs::canonicalize(&repo).unwrap());
        let task_hash = task_hash("fix checkout signature regression");

        ctxhelm_index::append_eval_trace(
            &repo,
            &ctxhelm_core::EvalTrace {
                id: uuid::Uuid::nil(),
                repo_id,
                task_hash,
                task_type: TaskType::BugFix,
                pack_id: None,
                target_agent: "claude-code".to_string(),
                budget: Some(PackBudget::Brief),
                recommended_files: vec![
                    "src/primary.ts".to_string(),
                    "src/secondary.ts".to_string(),
                    "src/primary.ts".to_string(),
                ],
                recommended_tests: vec![
                    "tests/primary.test.ts".to_string(),
                    "src/secondary.ts".to_string(),
                ],
                recommended_commands: vec!["pnpm test tests/primary.test.ts".to_string()],
                created_at_unix_seconds: 1,
                source_text_logged: false,
            },
        )
        .unwrap();

        let report =
            generate_experience_cards(&repo, &ExperienceCardsOptions { limit: 1 }).unwrap();
        assert_eq!(report.cards.len(), 1);
        assert_eq!(
            report.cards[0].source_links,
            vec![
                "src/primary.ts",
                "src/secondary.ts",
                "tests/primary.test.ts"
            ]
        );

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn memory_path_candidates_cap_context_links_and_skip_tests() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        for path in [
            "src/one.ts",
            "src/two.ts",
            "src/three.ts",
            "src/four.ts",
            "tests/one.test.ts",
        ] {
            fs::create_dir_all(repo.join(Path::new(path).parent().unwrap())).unwrap();
            fs::write(repo.join(path), "export const value = true;\n").unwrap();
        }
        std::env::set_var("CTXHELM_HOME", &home);

        persist_memory_card_records(
            &repo,
            &StoreConfig::default(),
            &[StorageMemoryCardRecord {
                card: MemoryCard {
                    id: "experience:many-links".to_string(),
                    kind: MemoryCardKind::Experience,
                    title: "Experience: checkout signature regression".to_string(),
                    summary: "A prior approved run selected several files and tests.".to_string(),
                    source_links: vec![
                        "src/one.ts".to_string(),
                        "src/two.ts".to_string(),
                        "src/three.ts".to_string(),
                        "src/four.ts".to_string(),
                        "tests/one.test.ts".to_string(),
                    ],
                    input_hashes: vec!["task-hash".to_string()],
                    freshness: MemoryFreshness::Fresh,
                    review_status: MemoryReviewStatus::Approved,
                    disabled: false,
                    confidence: 0.90,
                    reason: "test approved source-free experience card".to_string(),
                    privacy_status: PrivacyStatus::local_only(),
                },
            }],
        )
        .unwrap();

        let plan =
            prepare_context_plan(&repo, "fix checkout signature regression", TaskType::BugFix)
                .unwrap();
        let memory_paths = plan
            .retrieval_candidates
            .iter()
            .filter(|candidate| {
                candidate.reason_code == "memory_source_link"
                    && candidate
                        .signal_scores
                        .iter()
                        .any(|score| score.signal == RetrievalSignalKind::Memory)
            })
            .filter_map(|candidate| candidate.path.as_deref())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            memory_paths,
            ["src/one.ts", "src/two.ts", "src/three.ts"]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
        assert!(!memory_paths.contains(&"src/four.ts"));
        assert!(!memory_paths.contains(&"tests/one.test.ts"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn generate_fallback_cards_writes_disconnected_agent_guide() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return 'CTXHELM_FALLBACK_SOURCE_SENTINEL'; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add session"]);
        std::env::set_var("CTXHELM_HOME", &home);

        let report = generate_fallback_cards(
            &repo,
            &FallbackCardsOptions {
                limit: 20,
                target_agent: "codex-cli".to_string(),
            },
        )
        .unwrap();
        let guide = fs::read_to_string(&report.guide_path).unwrap();
        let serialized = serde_json::to_string(&report).unwrap();

        assert_eq!(report.target_agent, "codex");
        assert!(report.card_count >= 3);
        assert!(guide.contains("ctxhelm Disconnected Fallback"));
        assert!(guide.contains("AGENTS.md"));
        assert!(guide.contains("Codex cloud or isolated runs"));
        assert!(!guide.contains("CTXHELM_FALLBACK_SOURCE_SENTINEL"));
        assert!(!serialized.contains("CTXHELM_FALLBACK_SOURCE_SENTINEL"));
        assert!(!serialized.contains("\"sourceText\""));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn cards_use_fresh_inventory_and_report_degraded_inputs_without_source() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/session.ts"),
            "export function requireSession() { return 'do-not-copy'; }\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);
        write_inventory(&repo, &InventoryOptions::default()).unwrap();
        fs::rename(repo.join("src/session.ts"), repo.join(".env")).unwrap();
        fs::write(
            repo.join("src/broken.ts"),
            [0xff, b'b', b'r', b'o', b'k', b'e', b'n'],
        )
        .unwrap();

        let report = generate_context_cards(&repo, &ContextCardsOptions { limit: 20 }).unwrap();
        let overview = fs::read_to_string(repo.join(".ctxhelm/cards/repo-overview.md")).unwrap();
        let report_json = serde_json::to_string(&report).unwrap();

        assert!(!overview.contains("src/session.ts"));
        assert!(!overview.contains("do-not-copy"));
        assert!(!overview.contains("TOKEN"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventory_stale"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source_non_utf8"));
        assert!(!report_json.contains("do-not-copy"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn historical_eval_projects_source_memory_into_parent_snapshots() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/internal")).unwrap();
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(&repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/internal/handler.ts"),
            "export function internalHandler() { return true; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "add internal handler"]);
        let base = git_stdout(&repo, &["rev-parse", "HEAD"]).trim().to_string();
        fs::write(
            repo.join("src/internal/handler.ts"),
            "export function internalHandler() { return false; }\n",
        )
        .unwrap();
        run_git(&repo, &["add", "."]);
        run_git(
            &repo,
            &["commit", "-m", "fix checkout signature regression"],
        );
        let head = git_stdout(&repo, &["rev-parse", "HEAD"]).trim().to_string();
        std::env::set_var("CTXHELM_HOME", &home);

        persist_memory_card_records(
            &repo,
            &StoreConfig::default(),
            &[StorageMemoryCardRecord {
                card: MemoryCard {
                    id: "experience:parent-snapshot-checkout".to_string(),
                    kind: MemoryCardKind::Experience,
                    title: "Experience: checkout signature regression".to_string(),
                    summary:
                        "A prior approved run selected one source file for checkout signature work."
                            .to_string(),
                    source_links: vec!["src/internal/handler.ts".to_string()],
                    input_hashes: vec!["task-hash".to_string()],
                    freshness: MemoryFreshness::Fresh,
                    review_status: MemoryReviewStatus::Approved,
                    disabled: false,
                    confidence: 0.90,
                    reason: "test approved source-free experience card".to_string(),
                    privacy_status: PrivacyStatus::local_only(),
                },
            }],
        )
        .unwrap();

        let report = evaluate_historical_commits(
            &repo,
            &HistoricalEvalOptions {
                limit: 1,
                ranking_budget: 10,
                task_type: TaskType::BugFix,
                target_agent: "claude-code".to_string(),
                base: Some(base),
                head: Some(head),
                semantic_enabled: false,
                semantic_provider: SemanticProviderConfig::default(),
                local_metadata_reranker: false,
                query_family_routed_reranker: false,
                cache_enabled: false,
                force_refresh: true,
                parallelism: 1,
            },
        )
        .unwrap();

        assert_eq!(report.evaluated_commits, 1);
        assert_eq!(
            report.commits[0].retrieval_target_files,
            vec!["src/internal/handler.ts"]
        );
        assert!(report.commits[0]
            .recommended_context_files
            .contains(&"src/internal/handler.ts".to_string()));
        assert!(!report.commits[0]
            .lexical_baseline_files
            .contains(&"src/internal/handler.ts".to_string()));
        assert_eq!(report.memory_reuse_summary.memory_candidate_count, 1);
        assert_eq!(report.memory_reuse_summary.memory_selected_at_10_count, 1);
        assert_eq!(report.memory_reuse_summary.memory_target_hit_at_10_count, 1);
        assert_eq!(
            report.memory_reuse_summary.memory_unique_target_hit_count,
            1
        );
        assert!(report
            .recommended_research_actions
            .iter()
            .any(|action| action.action == "evaluate_memory_reuse_lift"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn compile_context_pack_from_plan_reuses_existing_task_id() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("ctxhelm-home");
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::write(
            repo.join("src/auth/session.ts"),
            "export function requireSession() { return true; }\n",
        )
        .unwrap();
        std::env::set_var("CTXHELM_HOME", &home);

        let plan = prepare_context_plan(&repo, "fix requireSession bug", TaskType::BugFix).unwrap();
        let pack = compile_context_pack_from_plan(
            &repo,
            "fix requireSession bug",
            &plan,
            PackBudget::Brief,
        );
        let codex_pack = compile_context_pack_from_plan_for_agent(
            &repo,
            "fix requireSession bug",
            &plan,
            PackBudget::Brief,
            "codex",
        );

        assert_eq!(pack.task_id, plan.task_id);
        assert_eq!(pack.task_type, plan.task_type);
        assert_eq!(pack.target_agent, "generic");
        assert_eq!(codex_pack.target_agent, "codex");
        assert_eq!(codex_pack.task_hash, task_hash("fix requireSession bug"));
        let markdown = render_pack_markdown(&codex_pack);
        assert!(markdown.contains("src/auth/session.ts"));
        assert!(markdown.contains("- Repo ID: `"));
        assert!(markdown.contains(&format!("- Task hash: `{}`", codex_pack.task_hash)));
        assert!(markdown.contains("- Target agent: `codex`"));
        assert!(!markdown.contains("sourceText"));

        std::env::remove_var("CTXHELM_HOME");
    }

    #[test]
    fn benchmark_suite_runs_multiple_repos_with_source_free_metadata() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("ctxhelm-home");
        let first_repo = temp.path().join("first");
        let second_repo = temp.path().join("second");
        make_benchmark_fixture_repo(&first_repo, "requireSession");
        make_benchmark_fixture_repo(&second_repo, "refreshSession");
        std::env::set_var("CTXHELM_HOME", &home);

        let config = BenchmarkSuiteConfig {
            manifest_version: "ctxhelm-benchmark-corpus-v2.3".to_string(),
            name: "phase-nine-smoke".to_string(),
            corpus_id: Some("phase-nine-smoke-corpus".to_string()),
            privacy_label: Some("source_free_local".to_string()),
            description: Some("source-free benchmark contract smoke".to_string()),
            defaults: BenchmarkDefaults {
                limit: 2,
                ranking_budget: 5,
                mode: TaskType::BugFix,
                target_agent: "codex".to_string(),
                semantic_enabled: false,
                semantic_provider: "local_hash".to_string(),
                semantic_model: None,
                semantic_dimensions: None,
                local_metadata_reranker: false,
                cache_enabled: false,
                force_refresh: false,
                parallelism: 1,
                role_filters: vec![FileRole::Source, FileRole::Test],
                lexical_backend_comparison: false,
            },
            repositories: vec![
                BenchmarkRepoConfig {
                    name: "first".to_string(),
                    path: first_repo.clone(),
                    revision_range_id: Some("first-head-history".to_string()),
                    privacy_label: Some("source_free_local".to_string()),
                    base: None,
                    head: None,
                    limit: None,
                    ranking_budget: None,
                    mode: None,
                    target_agent: None,
                    semantic_enabled: None,
                    semantic_provider: None,
                    semantic_model: None,
                    semantic_dimensions: None,
                    local_metadata_reranker: None,
                    cache_enabled: None,
                    force_refresh: None,
                    parallelism: None,
                    role_filters: Vec::new(),
                    lexical_backend_comparison: None,
                    proof_runtime_ceiling_millis: None,
                    baseline: Some(BenchmarkRepoBaseline {
                        file_recall_at_10: None,
                        lexical_baseline_recall_at_10: None,
                        total_millis: None,
                        gap_families: vec!["no_candidate_signal".to_string()],
                        notes: vec!["source-free fixture baseline".to_string()],
                    }),
                },
                BenchmarkRepoConfig {
                    name: "second".to_string(),
                    path: second_repo.clone(),
                    revision_range_id: Some("second-head-history".to_string()),
                    privacy_label: Some("source_free_local".to_string()),
                    base: None,
                    head: None,
                    limit: Some(1),
                    ranking_budget: Some(3),
                    mode: Some(TaskType::Feature),
                    target_agent: Some("claude-code".to_string()),
                    semantic_enabled: Some(false),
                    semantic_provider: Some("local_hash".to_string()),
                    semantic_model: None,
                    semantic_dimensions: None,
                    local_metadata_reranker: Some(true),
                    cache_enabled: Some(false),
                    force_refresh: Some(false),
                    parallelism: Some(1),
                    role_filters: vec![FileRole::Source],
                    lexical_backend_comparison: None,
                    proof_runtime_ceiling_millis: None,
                    baseline: None,
                },
            ],
        };

        let report = run_benchmark_suite_config(&config, temp.path()).unwrap();
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.suite_name, "phase-nine-smoke");
        assert_eq!(report.manifest_version, "ctxhelm-benchmark-corpus-v2.3");
        assert_eq!(report.corpus_id.as_deref(), Some("phase-nine-smoke-corpus"));
        assert_eq!(report.privacy_label.as_deref(), Some("source_free_local"));
        assert_eq!(report.repository_count, 2);
        assert_eq!(report.evaluated_repository_count, 2);
        assert!(report.evaluated_commit_count >= 2);
        assert_eq!(report.repositories[0].effective_config.ranking_budget, 5);
        assert_eq!(
            report.repositories[0].effective_config.target_agent,
            "codex"
        );
        assert_eq!(
            report.repositories[0]
                .effective_config
                .revision_range_id
                .as_deref(),
            Some("first-head-history")
        );
        assert!(report.repositories[0].baseline.is_some());
        assert!(report.repositories[0].baseline_status.is_some());
        assert_eq!(
            report.repositories[0].effective_config.role_filters,
            vec![FileRole::Source, FileRole::Test]
        );
        assert_eq!(report.repositories[0].effective_config.parallelism, 1);
        assert!(!report.repositories[0].effective_config.cache_enabled);
        assert_eq!(report.repositories[1].effective_config.ranking_budget, 3);
        assert_eq!(
            report.repositories[1].effective_config.mode,
            TaskType::Feature
        );
        assert_eq!(
            report.repositories[1].effective_config.semantic_provider,
            "local_hash"
        );
        assert_eq!(
            report.repositories[1]
                .effective_config
                .semantic_model
                .as_deref(),
            Some("ctxhelm-local-hash-v1")
        );
        assert_eq!(
            report.repositories[1].effective_config.semantic_dimensions,
            Some(64)
        );
        assert_eq!(
            report.repositories[1]
                .effective_config
                .semantic_provider_role,
            "deterministic_scaffold"
        );
        assert!(
            !report.repositories[1]
                .effective_config
                .semantic_quality_backend
        );
        assert_eq!(
            report.repositories[1].effective_config.role_filters,
            vec![FileRole::Source]
        );
        assert!(report.repositories.iter().all(|repo| repo.error.is_none()));
        assert!(report
            .repositories
            .iter()
            .all(|repo| repo.privacy_status.local_only));
        assert!(!report.suite_id.is_empty());
        assert!(!json.contains("return true"));
        assert!(!json.contains("source code"));

        std::env::remove_var("CTXHELM_HOME");
    }

    fn make_benchmark_fixture_repo(repo: &Path, function_name: &str) {
        fs::create_dir_all(repo.join("src/auth")).unwrap();
        fs::create_dir_all(repo.join("tests/auth")).unwrap();
        run_git(repo, &["init"]);
        run_git(repo, &["config", "user.email", "ctxhelm@example.com"]);
        run_git(repo, &["config", "user.name", "ctxhelm"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            format!("export function {function_name}() {{ return true; }}\n"),
        )
        .unwrap();
        fs::write(
            repo.join("tests/auth/session.test.ts"),
            format!(
                "import {{ {function_name} }} from '../../src/auth/session';\ntest('{function_name}', () => expect({function_name}()).toBe(true));\n"
            ),
        )
        .unwrap();
        run_git(repo, &["add", "."]);
        run_git(repo, &["commit", "-m", "add auth session"]);
        fs::write(
            repo.join("src/auth/session.ts"),
            format!("export function {function_name}() {{ return false; }}\n"),
        )
        .unwrap();
        run_git(repo, &["add", "."]);
        run_git(repo, &["commit", "-m", "fix auth session behavior"]);
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

    fn git_stdout(repo: &Path, args: &[&str]) -> String {
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
        String::from_utf8(output.stdout).unwrap()
    }
}
