use crate::eval::{evaluate_historical_commits, HistoricalEvalOptions, SemanticQueryMode};
use crate::graph::build_graph_neighborhood_report;
use crate::packs::pack_repo_id;
use ctxhelm_core::{
    Diagnostic, DiagnosticSeverity, PrecisionStatus, PrecisionStatusReport, PrivacyStatus,
    ProviderCapability, ProviderDataClass, ProviderDecision, ProviderDecisionStatus,
    ProviderPolicy, ProviderPolicyReport, RetrievalPolicyExperimentReport,
    RetrievalPolicyExperimentRow, SemanticProviderStatusReport, SemanticUsageSummary, TaskType,
};
use ctxhelm_index::{
    normalized_provider, semantic_document_report, semantic_search_report, storage_status_for_repo,
    task_hash, team_policy_report, InventoryError, SemanticDocumentOptions, SemanticOptions,
    SemanticProviderConfig, StoreConfig,
};
use std::fs;
use std::path::Path;

pub const PROVIDER_POLICY_SCHEMA_VERSION: u32 = 1;

pub fn provider_policy_report(
    repo_root: impl AsRef<Path>,
) -> Result<ProviderPolicyReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let path = repo_root.join(".ctxhelm").join("provider-policy.json");
    let (policy_path, mut policy, mut diagnostics) = if path.exists() {
        let json = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
            path: path.clone(),
            source,
        })?;
        let mut loaded = serde_json::from_str::<ProviderPolicy>(&json).map_err(|source| {
            InventoryError::Deserialize {
                path: path.clone(),
                source,
            }
        })?;
        loaded.schema_version = PROVIDER_POLICY_SCHEMA_VERSION;
        (Some(path.display().to_string()), loaded, Vec::new())
    } else {
        (
            None,
            ProviderPolicy::default(),
            vec![Diagnostic {
                code: "provider_policy_absent_safe_defaults".to_string(),
                severity: DiagnosticSeverity::Info,
                message: "Provider policy file was not found; using local source-free defaults."
                    .to_string(),
                paths: Vec::new(),
                count: 0,
            }],
        )
    };

    if let Ok(team_report) = team_policy_report(repo_root) {
        policy.allow_cloud_embeddings &=
            team_report.policy.allow_cloud_embeddings && policy.allow_source_transfer;
        policy.allow_cloud_reranking &=
            team_report.policy.allow_cloud_reranking && policy.allow_source_transfer;
        policy.allow_source_transfer &=
            team_report.policy.allow_source_snippets_in_shared_artifacts;
        diagnostics.extend(
            team_report
                .diagnostics
                .into_iter()
                .map(|diagnostic| Diagnostic {
                    code: format!("provider_policy_{}", diagnostic.code),
                    severity: diagnostic.severity,
                    message: diagnostic.message,
                    paths: diagnostic.paths,
                    count: 1,
                }),
        );
    }

    let decisions = provider_decisions(&policy);
    if decisions.iter().any(|decision| {
        matches!(decision.status, ProviderDecisionStatus::Denied)
            && matches!(
                decision.capability,
                ProviderCapability::SemanticEmbedding | ProviderCapability::Reranking
            )
    }) {
        diagnostics.push(Diagnostic {
            code: "provider_policy_remote_denied".to_string(),
            severity: DiagnosticSeverity::Info,
            message:
                "Remote embedding or reranking providers are blocked by local source-free policy."
                    .to_string(),
            paths: policy_path.clone().into_iter().collect(),
            count: 1,
        });
    }

    Ok(ProviderPolicyReport {
        policy_path,
        policy,
        decisions,
        diagnostics,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}

pub(crate) fn semantic_provider_decision(
    report: &ProviderPolicyReport,
    provider: &SemanticProviderConfig,
    requested: bool,
) -> ProviderDecision {
    if !requested {
        return ProviderDecision {
            capability: ProviderCapability::SemanticEmbedding,
            provider: provider.provider.clone(),
            status: ProviderDecisionStatus::Skipped,
            data_classes: vec![
                ProviderDataClass::Metadata,
                ProviderDataClass::SemanticVector,
            ],
            remote_allowed: false,
            source_text_allowed: false,
            reason: "Semantic retrieval was not requested for this operation.".to_string(),
        };
    }
    if !provider.available {
        return ProviderDecision {
            capability: ProviderCapability::SemanticEmbedding,
            provider: provider.provider.clone(),
            status: ProviderDecisionStatus::Unavailable,
            data_classes: vec![
                ProviderDataClass::Metadata,
                ProviderDataClass::SemanticVector,
            ],
            remote_allowed: false,
            source_text_allowed: false,
            reason: "Requested semantic provider is not available in this build.".to_string(),
        };
    }
    if provider.local_only {
        let status = if report.policy.allow_local_providers {
            ProviderDecisionStatus::Allowed
        } else {
            ProviderDecisionStatus::Denied
        };
        return ProviderDecision {
            capability: ProviderCapability::SemanticEmbedding,
            provider: provider.provider.clone(),
            status,
            data_classes: vec![
                ProviderDataClass::Metadata,
                ProviderDataClass::SemanticVector,
            ],
            remote_allowed: false,
            source_text_allowed: false,
            reason: if report.policy.allow_local_providers {
                "Local source-free semantic provider is allowed.".to_string()
            } else {
                "Local semantic providers are disabled by provider policy.".to_string()
            },
        };
    }

    ProviderDecision {
        capability: ProviderCapability::SemanticEmbedding,
        provider: provider.provider.clone(),
        status: if report.policy.allow_cloud_embeddings && report.policy.allow_source_transfer {
            ProviderDecisionStatus::Allowed
        } else {
            ProviderDecisionStatus::Denied
        },
        data_classes: vec![ProviderDataClass::Metadata, ProviderDataClass::SourceText],
        remote_allowed: report.policy.allow_cloud_embeddings,
        source_text_allowed: report.policy.allow_source_transfer,
        reason: "Cloud semantic providers require explicit cloud and source-transfer policy."
            .to_string(),
    }
}

pub(crate) fn query_family_routed_reranker_enabled_for_family(family: &str) -> bool {
    matches!(family, "commit_clue")
}

pub(crate) fn reranker_decision(report: &ProviderPolicyReport) -> ProviderDecision {
    if report.policy.enable_local_metadata_reranker
        || report.policy.enable_local_fixture_reranker
        || report.policy.enable_query_family_routed_reranker
    {
        let provider = if report.policy.enable_local_metadata_reranker
            || report.policy.enable_local_fixture_reranker
        {
            "local_metadata"
        } else {
            "local_metadata_routed"
        };
        let reason = if report.policy.enable_local_metadata_reranker
            || report.policy.enable_local_fixture_reranker
        {
            "Deterministic local metadata reranker is policy-enabled."
        } else {
            "Query-family routed local metadata reranker is policy-enabled."
        };
        return ProviderDecision {
            capability: ProviderCapability::Reranking,
            provider: provider.to_string(),
            status: if report.policy.allow_local_providers {
                ProviderDecisionStatus::Allowed
            } else {
                ProviderDecisionStatus::Denied
            },
            data_classes: vec![ProviderDataClass::Metadata],
            remote_allowed: false,
            source_text_allowed: false,
            reason: reason.to_string(),
        };
    }
    ProviderDecision {
        capability: ProviderCapability::Reranking,
        provider: "local_metadata".to_string(),
        status: ProviderDecisionStatus::Disabled,
        data_classes: vec![ProviderDataClass::Metadata],
        remote_allowed: false,
        source_text_allowed: false,
        reason: "Reranker is disabled by default.".to_string(),
    }
}

fn provider_decisions(policy: &ProviderPolicy) -> Vec<ProviderDecision> {
    let local_provider = if policy.allow_local_providers {
        ProviderDecisionStatus::Allowed
    } else {
        ProviderDecisionStatus::Denied
    };
    vec![
        ProviderDecision {
            capability: ProviderCapability::SemanticEmbedding,
            provider: "local_hash".to_string(),
            status: local_provider.clone(),
            data_classes: vec![
                ProviderDataClass::Metadata,
                ProviderDataClass::SemanticVector,
            ],
            remote_allowed: false,
            source_text_allowed: false,
            reason: "Default deterministic local semantic metadata provider.".to_string(),
        },
        ProviderDecision {
            capability: ProviderCapability::SemanticEmbedding,
            provider: "cloud_embedding".to_string(),
            status: if policy.allow_cloud_embeddings && policy.allow_source_transfer {
                ProviderDecisionStatus::Allowed
            } else {
                ProviderDecisionStatus::Denied
            },
            data_classes: vec![ProviderDataClass::Metadata, ProviderDataClass::SourceText],
            remote_allowed: policy.allow_cloud_embeddings,
            source_text_allowed: policy.allow_source_transfer,
            reason: "Cloud embeddings require both cloud and source-transfer policy.".to_string(),
        },
        ProviderDecision {
            capability: ProviderCapability::PrecisionGraph,
            provider: "local_precision_overlay".to_string(),
            status: local_provider,
            data_classes: vec![ProviderDataClass::Metadata],
            remote_allowed: false,
            source_text_allowed: false,
            reason: "Local precision overlays are metadata-only.".to_string(),
        },
        reranker_decision(&ProviderPolicyReport {
            policy_path: None,
            policy: policy.clone(),
            decisions: Vec::new(),
            diagnostics: Vec::new(),
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        }),
    ]
}

pub fn semantic_provider_status_report(
    repo_root: impl AsRef<Path>,
    query: Option<&str>,
    task_type: TaskType,
) -> Result<SemanticProviderStatusReport, InventoryError> {
    semantic_provider_status_report_with_provider(
        repo_root,
        query,
        task_type,
        SemanticProviderConfig::default(),
    )
}

pub fn semantic_provider_status_report_with_provider(
    repo_root: impl AsRef<Path>,
    query: Option<&str>,
    _task_type: TaskType,
    semantic_provider: SemanticProviderConfig,
) -> Result<SemanticProviderStatusReport, InventoryError> {
    let repo_root = repo_root.as_ref();
    let provider = normalized_provider(&semantic_provider);
    let mut provider_policy = provider_policy_report(repo_root)?;
    let selected_provider_decision = semantic_provider_decision(&provider_policy, &provider, true);
    provider_policy.decisions.push(selected_provider_decision);
    let query_semantic_report = if let Some(query) = query {
        Some(semantic_search_report(
            repo_root,
            query,
            &SemanticOptions {
                enabled: true,
                limit: 10,
                provider: provider.clone(),
            },
        )?)
    } else {
        None
    };
    let document_report = if query_semantic_report.is_none() {
        Some(semantic_document_report(
            repo_root,
            &SemanticDocumentOptions {
                limit: 500,
                query: None,
                include_symbols: false,
                include_dependencies: false,
                include_related_tests: false,
            },
        )?)
    } else {
        None
    };
    let local_vector_count = 0;
    let stored_vector_count = storage_status_for_repo(repo_root, &StoreConfig::default())
        .map(|status| status.semantic_vector_records)
        .unwrap_or_default();
    let mut usage = Vec::new();
    if let Some(semantic_report) = &query_semantic_report {
        usage.push(SemanticUsageSummary {
            surface: "semantic_search".to_string(),
            semantic_enabled: true,
            semantic_candidate_count: semantic_report.results.len(),
            remote_embeddings_used: semantic_report.privacy_status.remote_embeddings_used,
        });
    }
    let semantic_document_count = document_report
        .as_ref()
        .map(|report| report.document_count)
        .or_else(|| {
            query_semantic_report
                .as_ref()
                .map(|report| report.results.len())
        })
        .unwrap_or_default();
    let semantic_facet_count = document_report
        .as_ref()
        .map(|report| report.facet_count)
        .or_else(|| {
            query_semantic_report.as_ref().map(|report| {
                report
                    .results
                    .iter()
                    .map(|result| result.matched_facets.len())
                    .sum()
            })
        })
        .unwrap_or_default();
    let precision_status = document_report
        .map(|report| report.precision_status)
        .or_else(|| {
            query_semantic_report
                .as_ref()
                .map(|report| PrecisionStatusReport {
                    status: report
                        .results
                        .iter()
                        .find_map(|result| result.precision_status.clone())
                        .unwrap_or(PrecisionStatus::Unavailable),
                    provider: None,
                    overlay_path: None,
                    edge_count: 0,
                    rejected_edge_count: 0,
                    stale: false,
                    degraded: false,
                    diagnostics: Vec::new(),
                })
        })
        .unwrap_or(PrecisionStatusReport {
            status: PrecisionStatus::Unavailable,
            provider: None,
            overlay_path: None,
            edge_count: 0,
            rejected_edge_count: 0,
            stale: false,
            degraded: false,
            diagnostics: Vec::new(),
        });

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
        cache_location: "local ctxhelm semantic_vectors store; local_fastembed model cache defaults to repo .ctxhelm/cache/fastembed or CTXHELM_HOME/cache/fastembed (override with CTXHELM_FASTEMBED_CACHE_DIR)".to_string(),
        degraded: !provider.available,
        enabled_by_default: false,
        cloud_embeddings_allowed: false,
        cloud_reranking_allowed: false,
        semantic_document_count,
        semantic_facet_count,
        precision_status,
        local_vector_count,
        stored_vector_count,
        indexing_freshness: "safe_inventory_current_or_refreshed".to_string(),
        usage,
        provider_policy,
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
    let provider_policy = provider_policy_report(repo_root)?;
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
            semantic_provider: SemanticProviderConfig::default(),
            local_metadata_reranker: false,
            query_family_routed_reranker: false,
            semantic_corroborated_reranker: false,
            semantic_query_mode: SemanticQueryMode::Plain,
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
            semantic_provider: SemanticProviderConfig::default(),
            local_metadata_reranker: false,
            query_family_routed_reranker: false,
            semantic_corroborated_reranker: false,
            semantic_query_mode: SemanticQueryMode::Plain,
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
        provider_policy,
        source_text_logged: false,
        privacy_status: PrivacyStatus::local_only(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn provider_policy_uses_safe_defaults_when_config_is_absent() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();

        let report = provider_policy_report(repo).unwrap();

        assert!(report.policy.allow_local_providers);
        assert!(!report.policy.allow_cloud_embeddings);
        assert!(!report.policy.allow_cloud_reranking);
        assert!(!report.policy.allow_source_transfer);
        assert!(!report.policy.enable_local_metadata_reranker);
        assert!(!report.policy.enable_query_family_routed_reranker);
        assert!(!report.policy.enable_local_fixture_reranker);
        assert!(report.privacy_status.local_only);
        assert!(report
            .decisions
            .iter()
            .any(|decision| decision.status == ProviderDecisionStatus::Denied
                && decision.provider == "cloud_embedding"));
    }

    #[test]
    fn provider_policy_team_policy_denies_cloud_even_if_provider_file_allows_it() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir_all(repo.join(".ctxhelm")).unwrap();
        fs::write(
            repo.join(".ctxhelm/provider-policy.json"),
            r#"{
              "schemaVersion": 1,
              "name": "unsafe-request",
              "allowLocalProviders": true,
              "allowCloudEmbeddings": true,
              "allowCloudReranking": true,
              "allowSourceTransfer": true,
              "enableLocalFixtureReranker": true,
              "sourceTextLogged": false
            }"#,
        )
        .unwrap();

        let report = provider_policy_report(repo).unwrap();

        assert!(!report.policy.allow_cloud_embeddings);
        assert!(!report.policy.allow_cloud_reranking);
        assert!(!report.policy.allow_source_transfer);
        assert!(report.policy.enable_local_fixture_reranker);
        assert!(report
            .decisions
            .iter()
            .any(|decision| decision.provider == "cloud_embedding"
                && decision.status == ProviderDecisionStatus::Denied));
    }

    #[test]
    fn reranker_decision_is_disabled_by_default_and_local_when_enabled() {
        let default_report = ProviderPolicyReport {
            policy_path: None,
            policy: ProviderPolicy::default(),
            decisions: Vec::new(),
            diagnostics: Vec::new(),
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        };
        let disabled = reranker_decision(&default_report);
        assert_eq!(disabled.status, ProviderDecisionStatus::Disabled);
        assert!(!disabled.remote_allowed);
        assert!(!disabled.source_text_allowed);

        let mut enabled_report = default_report;
        enabled_report.policy.enable_local_metadata_reranker = true;
        let enabled = reranker_decision(&enabled_report);
        assert_eq!(enabled.status, ProviderDecisionStatus::Allowed);
        assert_eq!(enabled.provider, "local_metadata");
        assert!(!enabled.remote_allowed);
        assert!(!enabled.source_text_allowed);

        let mut routed_report = ProviderPolicyReport {
            policy_path: None,
            policy: ProviderPolicy::default(),
            decisions: Vec::new(),
            diagnostics: Vec::new(),
            source_text_logged: false,
            privacy_status: PrivacyStatus::local_only(),
        };
        routed_report.policy.enable_query_family_routed_reranker = true;
        let routed = reranker_decision(&routed_report);
        assert_eq!(routed.status, ProviderDecisionStatus::Allowed);
        assert_eq!(routed.provider, "local_metadata_routed");
        assert!(!routed.remote_allowed);
        assert!(!routed.source_text_allowed);
        assert!(query_family_routed_reranker_enabled_for_family(
            "commit_clue"
        ));
        assert!(!query_family_routed_reranker_enabled_for_family(
            "symbol_identifier"
        ));
    }

    #[test]
    fn semantic_status_does_not_materialize_non_default_provider_vectors() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/session.rs"), "pub fn require_session() {}\n").unwrap();

        let report = semantic_provider_status_report_with_provider(
            repo,
            None,
            TaskType::Explain,
            SemanticProviderConfig {
                provider: "local_fastembed".to_string(),
                ..SemanticProviderConfig::default()
            },
        )
        .unwrap();

        assert_eq!(report.provider_kind, "local_fastembed");
        assert_eq!(report.local_vector_count, 0);
        assert!(!report.privacy_status.remote_embeddings_used);
    }

    #[test]
    fn semantic_status_query_uses_bounded_semantic_search_sample() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/type_script_visitor.rs"),
            "pub fn type_script_visitor() {}\n",
        )
        .unwrap();

        let report = semantic_provider_status_report_with_provider(
            repo,
            Some("type script visitor"),
            TaskType::Explain,
            SemanticProviderConfig::default(),
        )
        .unwrap();

        assert_eq!(report.local_vector_count, 0);
        assert_eq!(report.semantic_document_count, 1);
        assert_eq!(report.usage.len(), 1);
        assert_eq!(report.usage[0].surface, "semantic_search");
        assert_eq!(report.usage[0].semantic_candidate_count, 1);
        assert!(!report.usage[0].remote_embeddings_used);
    }
}
