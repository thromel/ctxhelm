use crate::eval::{evaluate_historical_commits, HistoricalEvalOptions};
use crate::graph::build_graph_neighborhood_report;
use crate::packs::pack_repo_id;
use crate::planning::prepare_context_plan_with_paths_and_semantic;
use ctxpack_core::{
    Diagnostic, DiagnosticSeverity, PrivacyStatus, RetrievalPolicyExperimentReport,
    RetrievalPolicyExperimentRow, SemanticProviderStatusReport, SemanticUsageSummary, TaskType,
};
use ctxpack_index::{
    normalized_provider, semantic_document_report, semantic_vector_records,
    storage_status_for_repo, task_hash, InventoryError, SemanticDocumentOptions, SemanticOptions,
    SemanticProviderConfig, StoreConfig,
};
use std::path::Path;

pub fn semantic_provider_status_report(
    repo_root: impl AsRef<Path>,
    query: Option<&str>,
    task_type: TaskType,
) -> Result<SemanticProviderStatusReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let provider = normalized_provider(&SemanticProviderConfig::default());
    let document_report =
        semantic_document_report(repo_root, &SemanticDocumentOptions { limit: usize::MAX })?;
    let local_records = semantic_vector_records(
        repo_root,
        &SemanticOptions {
            enabled: true,
            limit: usize::MAX,
            provider: provider.clone(),
        },
    )?;
    let stored_vector_count = storage_status_for_repo(repo_root, &StoreConfig::default())
        .map(|status| status.semantic_vector_records)
        .unwrap_or_default();
    let mut usage = Vec::new();
    if let Some(query) = query {
        let plan =
            prepare_context_plan_with_paths_and_semantic(repo_root, query, task_type, &[], true)?;
        let semantic_candidate_count = plan
            .retrieval_candidates
            .iter()
            .filter(|candidate| {
                candidate.signal_scores.iter().any(|score| {
                    matches!(score.signal, ctxpack_core::RetrievalSignalKind::Semantic)
                }) || candidate.evidence.iter().any(|evidence| {
                    matches!(evidence.signal, ctxpack_core::RetrievalSignalKind::Semantic)
                })
            })
            .count();
        usage.push(SemanticUsageSummary {
            surface: "prepare_task".to_string(),
            semantic_enabled: true,
            semantic_candidate_count,
            remote_embeddings_used: plan.privacy_status.remote_embeddings_used,
        });
    }

    Ok(SemanticProviderStatusReport {
        repo_id: pack_repo_id(repo_root),
        provider_kind: provider.provider,
        model_id: provider.model,
        dimensions: provider.dimensions,
        distance_metric: provider.distance_metric,
        provider_role: provider.provider_role,
        quality_backend: provider.quality_backend,
        local_only: provider.local_only,
        provider_available: provider.available,
        provider_status: if provider.available {
            "available".to_string()
        } else {
            "unavailable".to_string()
        },
        cache_location: "local ctxpack semantic_vectors store".to_string(),
        degraded: !provider.available,
        enabled_by_default: false,
        cloud_embeddings_allowed: false,
        cloud_reranking_allowed: false,
        semantic_document_count: document_report.document_count,
        semantic_facet_count: document_report.facet_count,
        precision_status: document_report.precision_status,
        local_vector_count: local_records.len(),
        stored_vector_count,
        indexing_freshness: "safe_inventory_current_or_refreshed".to_string(),
        usage,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}

pub fn retrieval_policy_experiment_report(
    repo_root: impl AsRef<Path>,
    task: &str,
    task_type: TaskType,
    limit: usize,
    ranking_budget: usize,
) -> Result<RetrievalPolicyExperimentReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let lexical = evaluate_historical_commits(
        repo_root,
        &HistoricalEvalOptions {
            limit,
            ranking_budget,
            task_type: task_type.clone(),
            target_agent: "generic".to_string(),
            base: None,
            head: None,
            semantic_enabled: false,
            cache_enabled: false,
            force_refresh: false,
            parallelism: 1,
        },
    )?;
    let hybrid = evaluate_historical_commits(
        repo_root,
        &HistoricalEvalOptions {
            limit,
            ranking_budget,
            task_type: task_type.clone(),
            target_agent: "generic".to_string(),
            base: None,
            head: None,
            semantic_enabled: true,
            cache_enabled: false,
            force_refresh: false,
            parallelism: 1,
        },
    )?;
    let graph = build_graph_neighborhood_report(repo_root, Some(task), task_type, &[], 40, 80)?;
    let diagnostics = graph.diagnostics.clone();
    let rows = vec![
        RetrievalPolicyExperimentRow {
            policy: "lexical_only".to_string(),
            semantic_enabled: false,
            graph_enabled: false,
            file_recall_at_10: Some(lexical.lexical_baseline_recall_at_10),
            test_recall_at_10: Some(lexical.test_recall_at_10),
            context_precision: None,
            validation_coverage: None,
            graph_node_count: 0,
            graph_edge_count: 0,
            note: "Lexical baseline from historical eval; default ranking unchanged.".to_string(),
        },
        RetrievalPolicyExperimentRow {
            policy: "hybrid_local_semantic".to_string(),
            semantic_enabled: true,
            graph_enabled: false,
            file_recall_at_10: Some(hybrid.file_recall_at_10),
            test_recall_at_10: Some(hybrid.test_recall_at_10),
            context_precision: None,
            validation_coverage: None,
            graph_node_count: 0,
            graph_edge_count: 0,
            note: "Local semantic retrieval enabled explicitly for comparison.".to_string(),
        },
        RetrievalPolicyExperimentRow {
            policy: "graph_neighborhood".to_string(),
            semantic_enabled: false,
            graph_enabled: true,
            file_recall_at_10: None,
            test_recall_at_10: None,
            context_precision: None,
            validation_coverage: None,
            graph_node_count: graph.nodes.len(),
            graph_edge_count: graph.edges.len(),
            note: "Source-free graph report only; not applied to default ranking.".to_string(),
        },
        RetrievalPolicyExperimentRow {
            policy: "disabled_semantic_current_default".to_string(),
            semantic_enabled: false,
            graph_enabled: true,
            file_recall_at_10: Some(lexical.file_recall_at_10),
            test_recall_at_10: Some(lexical.test_recall_at_10),
            context_precision: None,
            validation_coverage: None,
            graph_node_count: graph.nodes.len(),
            graph_edge_count: graph.edges.len(),
            note:
                "Current default remains local lexical/graph/test/history without semantic opt-in."
                    .to_string(),
        },
    ];
    let mut diagnostics = diagnostics;
    diagnostics.push(Diagnostic {
        code: "policy_experiment_default_unchanged".to_string(),
        severity: DiagnosticSeverity::Info,
        message: "Policy experiments are report-only and do not change default ranking."
            .to_string(),
        paths: Vec::new(),
        count: rows.len(),
    });

    Ok(RetrievalPolicyExperimentReport {
        repo_id: pack_repo_id(repo_root),
        task_hash: task_hash(task),
        rows,
        diagnostics,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}
