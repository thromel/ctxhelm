use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProofInspectorReport {
    schema_version: &'static str,
    report_kind: String,
    report_schema_version: String,
    status: String,
    workflow_kind: String,
    client: ProofInspectorClient,
    outcome: ProofInspectorOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    availability: Option<ProofInspectorAvailability>,
    evidence: ProofInspectorEvidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_cost: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read_efficiency: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_proof: Option<ProductProofInspectorSummary>,
    memory_guard: ProofInspectorMemoryGuard,
    boundary: ProofInspectorBoundary,
    privacy_status: ProofInspectorPrivacyStatus,
    recommended_next_action: String,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProofInspectorBundleReport {
    schema_version: &'static str,
    report_count: usize,
    inventory: ProofInspectorBundleInventory,
    boundary: ProofInspectorBundleBoundary,
    maturity_verdict: String,
    recommended_next_action: String,
    reports: Vec<ProofInspectorReport>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorBundleInventory {
    product_proof_count: usize,
    clean_product_proof_count: usize,
    agent_outcome_count: usize,
    clean_agent_outcome_count: usize,
    agent_run_suite_count: usize,
    client_availability_count: usize,
    ready_client_availability_count: usize,
    availability_blocked_count: usize,
    degraded_report_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorBundleBoundary {
    source_free_summary: bool,
    privacy_boundary_failed: bool,
    read_only_boundary_failed: bool,
    client_failures_observed: bool,
    rate_limits_observed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductProofInspectorSummary {
    suite_name: String,
    suite_id: String,
    evaluated_repository_count: Option<u64>,
    evaluated_commit_count: Option<u64>,
    release_gate_decision: String,
    default_promotion_allowed: Option<bool>,
    decision_reason: String,
    corpus_verdict_count: usize,
    context_claim: String,
    agent_evidence_claim: String,
    all_file_claim: String,
    average_context_delta_at_10: Option<f64>,
    average_agent_evidence_delta_at_10: Option<f64>,
    average_file_delta_at_10: Option<f64>,
    max_protected_target_miss_rate_at_10: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorClient {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorOutcome {
    claim: String,
    comparison_eligible: Option<bool>,
    comparison_eligible_count: Option<u64>,
    comparable_ctxhelm_lane_count: Option<u64>,
    target_coverage_delta: Option<f64>,
    target_read_coverage_delta: Option<f64>,
    target_read_coverage: Option<f64>,
    target_read_precision: Option<f64>,
    irrelevant_read_delta: Option<i64>,
    total_irrelevant_read_count: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorAvailability {
    tiny_prompt_available: Option<bool>,
    paired_suite_available: Option<bool>,
    rate_limited: Option<bool>,
    client_failure_observed: Option<bool>,
    comparable_lane_count: Option<u64>,
    availability_blocker: Option<String>,
    client_count: Option<u64>,
    ready_client_count: Option<u64>,
    unavailable_client_count: Option<u64>,
    rate_limited_client_count: Option<u64>,
    stream_disconnected_client_count: Option<u64>,
    real_agent_outcome_currently_runnable: Option<bool>,
    recommended_research_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorEvidence {
    evidence_misses_observed: Option<bool>,
    evidence_only_targets_observed: Option<bool>,
    evidence_only_target_count: Option<u64>,
    evidence_only_targets_after_retry: Option<u64>,
    under_read_targets_observed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorMemoryGuard {
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorBoundary {
    client_failures_observed: Option<bool>,
    rate_limits_observed: Option<bool>,
    forbidden_boundary_events_observed: Option<bool>,
    missing_required_ctxhelm_calls_observed: Option<bool>,
    invalid_required_ctxhelm_calls_observed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProofInspectorPrivacyStatus {
    local_only: Option<bool>,
    source_text_logged: Option<bool>,
    raw_prompt_stored: Option<bool>,
    raw_transcript_stored: Option<bool>,
    raw_mcp_traffic_stored: Option<bool>,
    remote_embeddings_used: Option<bool>,
    remote_reranking_used: Option<bool>,
    source_free_summary: bool,
}

pub(crate) fn build_proof_inspector_report(report: &serde_json::Value) -> ProofInspectorReport {
    let is_client_availability = is_client_availability_report(report);
    let product_proof = product_proof_summary(report);
    let source = proof_summary_source(report);
    let retry_cost = source.get("retryCost").cloned();
    let read_efficiency = source.get("readEfficiency").cloned();
    let privacy = report
        .get("privacyStatus")
        .unwrap_or(&serde_json::Value::Null);
    let comparison_eligible = source
        .get("comparisonEligible")
        .and_then(serde_json::Value::as_bool);
    let comparison_eligible_count = source
        .get("comparisonEligibleCount")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| comparison_eligible.map(|eligible| if eligible { 1 } else { 0 }));
    let evidence_only_after_retry = retry_cost
        .as_ref()
        .and_then(|value| value.get("evidenceOnlyTargetsAfterRetry"))
        .and_then(serde_json::Value::as_u64);
    let total_irrelevant_read_count =
        sum_lane_metric(report, "irrelevantReadCount").or_else(|| {
            read_efficiency.as_ref().and_then(|value| {
                value
                    .get("efficientIrrelevantReadCount")
                    .and_then(serde_json::Value::as_u64)
            })
        });
    let evidence_only_target_count =
        sum_lane_metric(report, "ctxhelmEvidenceOnlyTargetCount").or(evidence_only_after_retry);
    let memory_guard = proof_memory_guard(report);
    let availability = proof_availability(report);
    let boundary = ProofInspectorBoundary {
        client_failures_observed: proof_bool(source, &["clientFailuresObserved"]).or_else(|| {
            if is_client_availability {
                availability
                    .as_ref()
                    .map(|availability| availability.unavailable_client_count.unwrap_or(0) > 0)
            } else {
                None
            }
        }),
        rate_limits_observed: proof_bool(source, &["rateLimitsObserved"]).or_else(|| {
            if is_client_availability {
                availability
                    .as_ref()
                    .map(|availability| availability.rate_limited_client_count.unwrap_or(0) > 0)
            } else {
                None
            }
        }),
        forbidden_boundary_events_observed: proof_bool(
            source,
            &["forbiddenToolCallsObserved", "forbiddenCommandsObserved"],
        ),
        missing_required_ctxhelm_calls_observed: proof_bool(
            source,
            &["missingRequiredCtxhelmCallsObserved"],
        ),
        invalid_required_ctxhelm_calls_observed: proof_bool(
            source,
            &["invalidRequiredCtxhelmCallsObserved"],
        ),
    };
    let outcome_claim = product_proof
        .as_ref()
        .map(|summary| summary.release_gate_decision.clone())
        .or_else(|| {
            if is_client_availability {
                availability.as_ref().map(|availability| {
                    if availability.real_agent_outcome_currently_runnable == Some(true) {
                        "real_agent_outcome_currently_runnable".to_string()
                    } else {
                        "availability_blocked".to_string()
                    }
                })
            } else {
                None
            }
        })
        .or_else(|| proof_string(source, "outcomeClaim"))
        .or_else(|| proof_string(report, "outcomeClaim"))
        .unwrap_or_else(|| "unknown".to_string());
    let outcome = ProofInspectorOutcome {
        claim: outcome_claim,
        comparison_eligible,
        comparison_eligible_count,
        comparable_ctxhelm_lane_count: source
            .get("comparableCtxhelmLaneCount")
            .and_then(serde_json::Value::as_u64),
        target_coverage_delta: proof_f64(
            source,
            &["targetCoverageDeltaAverage", "targetCoverageDelta"],
        ),
        target_read_coverage_delta: proof_f64(
            source,
            &["targetReadCoverageDeltaAverage", "targetReadCoverageDelta"],
        ),
        target_read_coverage: proof_f64_from_value(
            read_efficiency.as_ref(),
            &["efficientTargetReadCoverage"],
        )
        .or_else(|| best_lane_metric(report, "targetReadCoverage")),
        target_read_precision: proof_f64_from_value(
            read_efficiency.as_ref(),
            &["efficientTargetReadPrecision"],
        )
        .or_else(|| best_lane_metric(report, "targetReadPrecision")),
        irrelevant_read_delta: proof_i64(
            source,
            &["irrelevantReadDeltaSum", "irrelevantReadDelta"],
        ),
        total_irrelevant_read_count,
    };
    let evidence = ProofInspectorEvidence {
        evidence_misses_observed: proof_bool(source, &["ctxhelmEvidenceMissesObserved"]),
        evidence_only_targets_observed: proof_bool(source, &["ctxhelmEvidenceOnlyTargetsObserved"]),
        evidence_only_target_count,
        evidence_only_targets_after_retry: evidence_only_after_retry,
        under_read_targets_observed: proof_bool(source, &["ctxhelmUnderReadTargetsObserved"]),
    };
    let privacy_status = ProofInspectorPrivacyStatus {
        local_only: privacy
            .get("localOnly")
            .and_then(serde_json::Value::as_bool),
        source_text_logged: privacy
            .get("sourceTextLogged")
            .and_then(serde_json::Value::as_bool),
        raw_prompt_stored: privacy
            .get("rawPromptStored")
            .and_then(serde_json::Value::as_bool),
        raw_transcript_stored: privacy
            .get("rawTranscriptStored")
            .and_then(serde_json::Value::as_bool),
        raw_mcp_traffic_stored: privacy
            .get("rawMcpTrafficStored")
            .and_then(serde_json::Value::as_bool),
        remote_embeddings_used: privacy
            .get("remoteEmbeddingsUsed")
            .and_then(serde_json::Value::as_bool),
        remote_reranking_used: privacy
            .get("remoteRerankingUsed")
            .and_then(serde_json::Value::as_bool),
        source_free_summary: true,
    };
    let mut notes = Vec::new();
    if report.get("aggregate").is_none() && product_proof.is_none() {
        notes.push("single-report summary; suite aggregate fields were not present".to_string());
    }
    if is_client_availability {
        notes.push(
            "standalone client availability preflight; this is not agent outcome proof".to_string(),
        );
    }
    if memory_guard.status == "not_reported" {
        notes.push("memory guard status was not present in this proof report".to_string());
    }
    if retry_cost.is_none() && product_proof.is_none() && !is_client_availability {
        notes.push("retry-cost fields were not present in this proof report".to_string());
    }
    ProofInspectorReport {
        schema_version: "ctxhelm-proof-inspector-v1",
        report_kind: if product_proof.is_some() {
            "product_proof".to_string()
        } else if is_client_availability {
            "client_availability".to_string()
        } else if report.get("aggregate").is_some() {
            "agent_run_suite".to_string()
        } else {
            "agent_run".to_string()
        },
        report_schema_version: proof_string(report, "schemaVersion")
            .unwrap_or_else(|| "unknown".to_string()),
        status: proof_string(report, "status").unwrap_or_else(|| "unknown".to_string()),
        workflow_kind: proof_string(report, "workflowKind")
            .unwrap_or_else(|| "unknown".to_string()),
        client: ProofInspectorClient {
            name: report
                .get("client")
                .and_then(|value| value.get("name"))
                .and_then(serde_json::Value::as_str)
                .or_else(|| {
                    if is_client_availability {
                        Some("multi-client")
                    } else {
                        None
                    }
                })
                .unwrap_or("unknown")
                .to_string(),
            version: report
                .get("client")
                .and_then(|value| value.get("version"))
                .and_then(serde_json::Value::as_str)
                .or_else(|| {
                    report
                        .get("ctxhelmVersion")
                        .and_then(serde_json::Value::as_str)
                })
                .unwrap_or("unknown")
                .to_string(),
        },
        recommended_next_action: recommended_proof_next_action(
            &outcome,
            availability.as_ref(),
            &evidence,
            &boundary,
            &privacy_status,
            product_proof.as_ref(),
        ),
        outcome,
        availability,
        evidence,
        retry_cost,
        read_efficiency,
        product_proof,
        memory_guard,
        boundary,
        privacy_status,
        notes,
    }
}

pub(crate) fn render_proof_inspector_report(report: &ProofInspectorReport) -> String {
    let mut output = String::from("# ctxhelm Proof Inspector\n\n");
    output.push_str(&format!(
        "- Inspector schema: `{}`\n- Report kind: `{}`\n- Report schema: `{}`\n- Status: `{}`\n- Workflow: `{}`\n- Client: `{}` `{}`\n\n",
        report.schema_version,
        report.report_kind,
        report.report_schema_version,
        report.status,
        report.workflow_kind,
        report.client.name,
        report.client.version
    ));
    output.push_str("## Outcome Claim\n\n");
    output.push_str(&format!(
        "- Claim: `{}`\n- Comparison eligible: `{}`\n- Comparable tasks: `{}`\n- Comparable ctxhelm lanes: `{}`\n- Target coverage delta: `{}`\n- Target-read coverage delta: `{}`\n- Target-read coverage: `{}`\n- Target-read precision: `{}`\n- Irrelevant-read delta: `{}`\n- Total irrelevant reads: `{}`\n\n",
        report.outcome.claim,
        optional_bool(report.outcome.comparison_eligible),
        optional_u64(report.outcome.comparison_eligible_count),
        optional_u64(report.outcome.comparable_ctxhelm_lane_count),
        optional_f64(report.outcome.target_coverage_delta),
        optional_f64(report.outcome.target_read_coverage_delta),
        optional_f64(report.outcome.target_read_coverage),
        optional_f64(report.outcome.target_read_precision),
        optional_i64(report.outcome.irrelevant_read_delta),
        optional_u64(report.outcome.total_irrelevant_read_count),
    ));
    if let Some(availability) = report.availability.as_ref() {
        output.push_str("## Client Availability\n\n");
        output.push_str(&format!(
            "- Tiny prompt available: `{}`\n- Paired suite available: `{}`\n- Availability blocker: `{}`\n- Rate-limited: `{}`\n- Client failure observed: `{}`\n- Comparable lanes: `{}`\n",
            optional_bool(availability.tiny_prompt_available),
            optional_bool(availability.paired_suite_available),
            availability
                .availability_blocker
                .as_deref()
                .unwrap_or("none"),
            optional_bool(availability.rate_limited),
            optional_bool(availability.client_failure_observed),
            optional_u64(availability.comparable_lane_count),
        ));
        if availability.client_count.is_some() {
            output.push_str(&format!(
                "- Clients checked: `{}`\n- Ready clients: `{}`\n- Unavailable clients: `{}`\n- Rate-limited clients: `{}`\n- Stream-disconnected clients: `{}`\n- Real agent outcome currently runnable: `{}`\n",
                optional_u64(availability.client_count),
                optional_u64(availability.ready_client_count),
                optional_u64(availability.unavailable_client_count),
                optional_u64(availability.rate_limited_client_count),
                optional_u64(availability.stream_disconnected_client_count),
                optional_bool(availability.real_agent_outcome_currently_runnable),
            ));
            if !availability.recommended_research_actions.is_empty() {
                output.push_str(&format!(
                    "- Recommended research actions: `{}`\n",
                    availability.recommended_research_actions.join(", ")
                ));
            }
        }
        output.push('\n');
    }
    output.push_str("## Evidence Consumption\n\n");
    output.push_str(&format!(
        "- Evidence misses observed: `{}`\n- Evidence-only targets observed: `{}`\n- Evidence-only target count: `{}`\n- Evidence-only targets after retry: `{}`\n- Under-read targets observed: `{}`\n\n",
        optional_bool(report.evidence.evidence_misses_observed),
        optional_bool(report.evidence.evidence_only_targets_observed),
        optional_u64(report.evidence.evidence_only_target_count),
        optional_u64(report.evidence.evidence_only_targets_after_retry),
        optional_bool(report.evidence.under_read_targets_observed),
    ));
    output.push_str("## Retry Cost\n\n");
    if let Some(retry) = report.retry_cost.as_ref() {
        output.push_str(&render_retry_cost(Some(retry)));
    } else {
        output.push_str("- Retry cost: `not_reported`\n");
    }
    if let Some(product) = report.product_proof.as_ref() {
        output.push_str("\n## Product Proof\n\n");
        output.push_str(&format!(
            "- Suite: `{}`\n- Suite ID: `{}`\n- Evaluated repositories: `{}`\n- Evaluated commits: `{}`\n- Release gate decision: `{}`\n- Default promotion allowed: `{}`\n- Corpus verdicts: `{}`\n- Context claim: `{}`\n- Agent-evidence claim: `{}`\n- All-file claim: `{}`\n- Average context delta@10: `{}`\n- Average agent-evidence delta@10: `{}`\n- Average all-file delta@10: `{}`\n- Max protected target miss-rate@10: `{}`\n- Decision reason: {}\n",
            product.suite_name,
            product.suite_id,
            optional_u64(product.evaluated_repository_count),
            optional_u64(product.evaluated_commit_count),
            product.release_gate_decision,
            optional_bool(product.default_promotion_allowed),
            product.corpus_verdict_count,
            product.context_claim,
            product.agent_evidence_claim,
            product.all_file_claim,
            optional_f64(product.average_context_delta_at_10),
            optional_f64(product.average_agent_evidence_delta_at_10),
            optional_f64(product.average_file_delta_at_10),
            optional_f64(product.max_protected_target_miss_rate_at_10),
            product.decision_reason,
        ));
    }
    output.push_str("\n## Memory Guard\n\n");
    output.push_str(&format!(
        "- Status: `{}`\n- Detail: {}\n\n",
        report.memory_guard.status, report.memory_guard.detail
    ));
    output.push_str("## Boundaries\n\n");
    output.push_str(&format!(
        "- Client failures observed: `{}`\n- Rate limits observed: `{}`\n- Forbidden boundary events observed: `{}`\n- Missing required ctxhelm calls observed: `{}`\n- Invalid required ctxhelm calls observed: `{}`\n\n",
        optional_bool(report.boundary.client_failures_observed),
        optional_bool(report.boundary.rate_limits_observed),
        optional_bool(report.boundary.forbidden_boundary_events_observed),
        optional_bool(report.boundary.missing_required_ctxhelm_calls_observed),
        optional_bool(report.boundary.invalid_required_ctxhelm_calls_observed),
    ));
    output.push_str("## Source-Free Privacy\n\n");
    output.push_str(&format!(
        "- Local only: `{}`\n- Source text logged: `{}`\n- Raw prompt stored: `{}`\n- Raw transcript stored: `{}`\n- Raw MCP traffic stored: `{}`\n- Remote embeddings used: `{}`\n- Remote reranking used: `{}`\n- Summary source-free: `{}`\n\n",
        optional_bool(report.privacy_status.local_only),
        optional_bool(report.privacy_status.source_text_logged),
        optional_bool(report.privacy_status.raw_prompt_stored),
        optional_bool(report.privacy_status.raw_transcript_stored),
        optional_bool(report.privacy_status.raw_mcp_traffic_stored),
        optional_bool(report.privacy_status.remote_embeddings_used),
        optional_bool(report.privacy_status.remote_reranking_used),
        report.privacy_status.source_free_summary,
    ));
    output.push_str("## Recommended Next Action\n\n");
    output.push_str(&format!("- {}\n", report.recommended_next_action));
    if !report.notes.is_empty() {
        output.push_str("\n## Notes\n\n");
        for note in &report.notes {
            output.push_str(&format!("- {note}\n"));
        }
    }
    output
}

pub(crate) fn build_proof_inspector_bundle(
    reports: Vec<ProofInspectorReport>,
) -> ProofInspectorBundleReport {
    let product_proof_count = reports
        .iter()
        .filter(|report| report.report_kind == "product_proof")
        .count();
    let clean_product_proof_count = reports
        .iter()
        .filter(|report| proof_report_has_clean_product_proof(report))
        .count();
    let agent_outcome_count = reports
        .iter()
        .filter(|report| {
            report.report_kind == "agent_run" || report.report_kind == "agent_run_suite"
        })
        .count();
    let clean_agent_outcome_count = reports
        .iter()
        .filter(|report| proof_report_has_clean_agent_outcome(report))
        .count();
    let agent_run_suite_count = reports
        .iter()
        .filter(|report| report.report_kind == "agent_run_suite")
        .count();
    let client_availability_count = reports
        .iter()
        .filter(|report| report.report_kind == "client_availability")
        .count();
    let ready_client_availability_count = reports
        .iter()
        .filter(|report| {
            report.report_kind == "client_availability"
                && report
                    .availability
                    .as_ref()
                    .and_then(|availability| availability.real_agent_outcome_currently_runnable)
                    == Some(true)
        })
        .count();
    let availability_blocked_count = reports
        .iter()
        .filter(|report| {
            (report.report_kind != "client_availability"
                && (report.boundary.client_failures_observed == Some(true)
                    || report.boundary.rate_limits_observed == Some(true)))
                || report
                    .availability
                    .as_ref()
                    .and_then(|availability| availability.paired_suite_available)
                    == Some(false)
                || report.outcome.claim == "insufficient_comparable_lanes"
                || report.outcome.claim == "availability_blocked"
        })
        .count();
    let degraded_report_count = reports
        .iter()
        .filter(|report| report.status == "degraded" || report.status == "failed")
        .count();
    let privacy_boundary_failed = reports
        .iter()
        .any(proof_report_has_privacy_boundary_failure);
    let read_only_boundary_failed = reports.iter().any(|report| {
        report.boundary.forbidden_boundary_events_observed == Some(true)
            || report.boundary.missing_required_ctxhelm_calls_observed == Some(true)
            || report.boundary.invalid_required_ctxhelm_calls_observed == Some(true)
    });
    let client_failures_observed = reports
        .iter()
        .any(|report| report.boundary.client_failures_observed == Some(true));
    let rate_limits_observed = reports
        .iter()
        .any(|report| report.boundary.rate_limits_observed == Some(true));
    let inventory = ProofInspectorBundleInventory {
        product_proof_count,
        clean_product_proof_count,
        agent_outcome_count,
        clean_agent_outcome_count,
        agent_run_suite_count,
        client_availability_count,
        ready_client_availability_count,
        availability_blocked_count,
        degraded_report_count,
    };
    let boundary = ProofInspectorBundleBoundary {
        source_free_summary: !privacy_boundary_failed,
        privacy_boundary_failed,
        read_only_boundary_failed,
        client_failures_observed,
        rate_limits_observed,
    };
    let (maturity_verdict, recommended_next_action) =
        proof_bundle_verdict_and_action(&inventory, &boundary);
    let mut notes = Vec::new();
    if product_proof_count == 0 {
        notes.push("bundle did not include a product-proof report".to_string());
    }
    if agent_outcome_count == 0 {
        notes.push("bundle did not include an agent-run outcome report".to_string());
    }
    if client_availability_count > 0 && ready_client_availability_count > 0 {
        notes.push(
            "bundle included ready client availability preflight; run comparable outcome suites next"
                .to_string(),
        );
    }
    if availability_blocked_count > 0 {
        notes.push("at least one report was availability-blocked or not comparable".to_string());
    }
    ProofInspectorBundleReport {
        schema_version: "ctxhelm-proof-inspector-bundle-v1",
        report_count: reports.len(),
        inventory,
        boundary,
        maturity_verdict,
        recommended_next_action,
        reports,
        notes,
    }
}

pub(crate) fn render_proof_inspector_bundle(bundle: &ProofInspectorBundleReport) -> String {
    let mut output = String::from("# ctxhelm Proof Bundle Inspector\n\n");
    output.push_str(&format!(
        "- Inspector schema: `{}`\n- Reports: `{}`\n- Maturity verdict: `{}`\n\n",
        bundle.schema_version, bundle.report_count, bundle.maturity_verdict
    ));
    output.push_str("## Evidence Inventory\n\n");
    output.push_str(&format!(
        "- Product-proof reports: `{}`\n- Clean product proofs: `{}`\n- Agent outcome reports: `{}`\n- Clean agent outcomes: `{}`\n- Agent-run suite reports: `{}`\n- Client availability reports: `{}`\n- Ready client availability reports: `{}`\n- Availability-blocked reports: `{}`\n- Degraded reports: `{}`\n\n",
        bundle.inventory.product_proof_count,
        bundle.inventory.clean_product_proof_count,
        bundle.inventory.agent_outcome_count,
        bundle.inventory.clean_agent_outcome_count,
        bundle.inventory.agent_run_suite_count,
        bundle.inventory.client_availability_count,
        bundle.inventory.ready_client_availability_count,
        bundle.inventory.availability_blocked_count,
        bundle.inventory.degraded_report_count,
    ));
    output.push_str("## Boundaries\n\n");
    output.push_str(&format!(
        "- Summary source-free: `{}`\n- Privacy boundary failed: `{}`\n- Read-only boundary failed: `{}`\n- Client failures observed: `{}`\n- Rate limits observed: `{}`\n\n",
        bundle.boundary.source_free_summary,
        bundle.boundary.privacy_boundary_failed,
        bundle.boundary.read_only_boundary_failed,
        bundle.boundary.client_failures_observed,
        bundle.boundary.rate_limits_observed,
    ));
    output.push_str("## Report Summaries\n\n");
    for (index, report) in bundle.reports.iter().enumerate() {
        output.push_str(&format!(
            "- Report `{}` kind `{}` status `{}` claim `{}` next `{}`\n",
            index + 1,
            report.report_kind,
            report.status,
            report.outcome.claim,
            report.recommended_next_action
        ));
    }
    output.push_str("\n## Recommended Next Action\n\n");
    output.push_str(&format!("- {}\n", bundle.recommended_next_action));
    if !bundle.notes.is_empty() {
        output.push_str("\n## Notes\n\n");
        for note in &bundle.notes {
            output.push_str(&format!("- {note}\n"));
        }
    }
    output
}

fn proof_report_has_clean_product_proof(report: &ProofInspectorReport) -> bool {
    let Some(product) = report.product_proof.as_ref() else {
        return false;
    };
    !proof_report_has_privacy_boundary_failure(report)
        && report.boundary.forbidden_boundary_events_observed != Some(true)
        && product.release_gate_decision == "promote"
        && product.default_promotion_allowed == Some(true)
        && product.max_protected_target_miss_rate_at_10.unwrap_or(1.0) <= f64::EPSILON
}

pub(crate) fn proof_inspector_report_ready(report: &ProofInspectorReport) -> bool {
    proof_report_has_clean_product_proof(report) || proof_report_has_clean_agent_outcome(report)
}

pub(crate) fn proof_inspector_bundle_ready(bundle: &ProofInspectorBundleReport) -> bool {
    bundle.maturity_verdict == "release_and_agent_outcome_evidence_ready"
}

fn proof_report_has_clean_agent_outcome(report: &ProofInspectorReport) -> bool {
    (report.report_kind == "agent_run" || report.report_kind == "agent_run_suite")
        && !proof_report_has_privacy_boundary_failure(report)
        && report
            .availability
            .as_ref()
            .and_then(|availability| availability.paired_suite_available)
            != Some(false)
        && report.boundary.forbidden_boundary_events_observed != Some(true)
        && report.boundary.client_failures_observed != Some(true)
        && report.boundary.rate_limits_observed != Some(true)
        && report.boundary.missing_required_ctxhelm_calls_observed != Some(true)
        && report.boundary.invalid_required_ctxhelm_calls_observed != Some(true)
        && report.outcome.claim == "ctxhelm_improved"
        && report.evidence.evidence_misses_observed != Some(true)
        && report
            .evidence
            .evidence_only_targets_after_retry
            .unwrap_or(0)
            == 0
        && report.evidence.under_read_targets_observed != Some(true)
}

fn proof_report_has_privacy_boundary_failure(report: &ProofInspectorReport) -> bool {
    report.privacy_status.source_text_logged == Some(true)
        || report.privacy_status.raw_prompt_stored == Some(true)
        || report.privacy_status.raw_transcript_stored == Some(true)
        || report.privacy_status.raw_mcp_traffic_stored == Some(true)
}

fn proof_bundle_verdict_and_action(
    inventory: &ProofInspectorBundleInventory,
    boundary: &ProofInspectorBundleBoundary,
) -> (String, String) {
    if boundary.privacy_boundary_failed {
        return (
            "privacy_boundary_failed".to_string(),
            "Reject the bundle for public claims until every report is source-free.".to_string(),
        );
    }
    if boundary.read_only_boundary_failed {
        return (
            "read_only_boundary_failed".to_string(),
            "Fix forbidden or invalid agent-boundary events before using this bundle as proof."
                .to_string(),
        );
    }
    if inventory.clean_product_proof_count > 0 && inventory.clean_agent_outcome_count > 0 {
        return (
            "release_and_agent_outcome_evidence_ready".to_string(),
            "Use this as the adoption-facing proof bundle, then repeat agent outcome suites across more clients before broader productivity claims.".to_string(),
        );
    }
    if inventory.clean_product_proof_count > 0 {
        return (
            "product_proof_ready_agent_outcome_needed".to_string(),
            "Pair the clean product proof with comparable real-agent outcome evidence before making agent-productivity claims.".to_string(),
        );
    }
    if inventory.clean_agent_outcome_count > 0 {
        return (
            "agent_outcome_ready_product_proof_needed".to_string(),
            "Pair the clean agent outcome proof with a current product-proof release gate before making release-readiness claims.".to_string(),
        );
    }
    if inventory.availability_blocked_count > 0 {
        return (
            "availability_blocked".to_string(),
            "Rerun availability-blocked clients before treating this proof bundle as comparable evidence.".to_string(),
        );
    }
    (
        "insufficient_proof".to_string(),
        "Add at least one clean product-proof report and one clean agent outcome report."
            .to_string(),
    )
}

fn proof_summary_source(report: &serde_json::Value) -> &serde_json::Value {
    report
        .get("aggregate")
        .or_else(|| report.get("comparison"))
        .unwrap_or(report)
}

fn is_client_availability_report(report: &serde_json::Value) -> bool {
    report
        .get("schemaVersion")
        .and_then(serde_json::Value::as_str)
        == Some("ctxhelm-agent-client-availability-v1")
        || report
            .get("workflowKind")
            .and_then(serde_json::Value::as_str)
            == Some("agent-client-availability")
}

fn product_proof_summary(report: &serde_json::Value) -> Option<ProductProofInspectorSummary> {
    let release_gate = report.get("releaseGate")?;
    let lexical = release_gate
        .get("lexicalComparison")
        .unwrap_or(&serde_json::Value::Null);
    let verdicts = release_gate
        .get("corpusVerdicts")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let max_protected_target_miss_rate_at_10 = verdicts
        .iter()
        .filter_map(|verdict| {
            verdict
                .get("protectedEvidenceTargetMissRateAt10")
                .or_else(|| verdict.get("protectedEvidenceMissRateAt10"))
                .and_then(serde_json::Value::as_f64)
        })
        .reduce(f64::max);
    Some(ProductProofInspectorSummary {
        suite_name: proof_string(report, "suiteName").unwrap_or_else(|| "unknown".to_string()),
        suite_id: proof_string(report, "suiteId").unwrap_or_else(|| "unknown".to_string()),
        evaluated_repository_count: report
            .get("evaluatedRepositoryCount")
            .and_then(serde_json::Value::as_u64),
        evaluated_commit_count: report
            .get("evaluatedCommitCount")
            .and_then(serde_json::Value::as_u64),
        release_gate_decision: proof_string(release_gate, "decision")
            .unwrap_or_else(|| "unknown".to_string()),
        default_promotion_allowed: release_gate
            .get("defaultPromotionAllowed")
            .and_then(serde_json::Value::as_bool),
        decision_reason: proof_string(release_gate, "decisionReason")
            .unwrap_or_else(|| "not reported".to_string()),
        corpus_verdict_count: verdicts.len(),
        context_claim: proof_string(lexical, "contextClaim")
            .unwrap_or_else(|| "unknown".to_string()),
        agent_evidence_claim: proof_string(lexical, "agentEvidenceClaim")
            .unwrap_or_else(|| "unknown".to_string()),
        all_file_claim: proof_string(lexical, "allFileClaim")
            .unwrap_or_else(|| "unknown".to_string()),
        average_context_delta_at_10: lexical
            .get("averageContextDeltaAt10")
            .and_then(serde_json::Value::as_f64),
        average_agent_evidence_delta_at_10: lexical
            .get("averageAgentEvidenceDeltaAt10")
            .and_then(serde_json::Value::as_f64),
        average_file_delta_at_10: lexical
            .get("averageFileDeltaAt10")
            .and_then(serde_json::Value::as_f64),
        max_protected_target_miss_rate_at_10,
    })
}

fn proof_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn proof_bool(value: &serde_json::Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_bool))
}

fn proof_f64(value: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_f64))
}

fn proof_f64_from_value(value: Option<&serde_json::Value>, keys: &[&str]) -> Option<f64> {
    value.and_then(|value| proof_f64(value, keys))
}

fn proof_i64(value: &serde_json::Value, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_i64))
}

fn sum_lane_metric(report: &serde_json::Value, metric: &str) -> Option<u64> {
    let lane_summaries = report
        .get("aggregate")
        .and_then(|value| value.get("laneSummaries"))
        .and_then(serde_json::Value::as_array);
    if let Some(lanes) = lane_summaries {
        return Some(
            lanes
                .iter()
                .filter_map(|lane| lane.get(metric).and_then(serde_json::Value::as_u64))
                .sum(),
        );
    }
    let lanes = report.get("lanes").and_then(serde_json::Value::as_array)?;
    Some(
        lanes
            .iter()
            .filter_map(|lane| {
                lane.get("metrics")
                    .and_then(|metrics| metrics.get(metric))
                    .and_then(serde_json::Value::as_u64)
            })
            .sum(),
    )
}

fn best_lane_metric(report: &serde_json::Value, metric: &str) -> Option<f64> {
    let source = proof_summary_source(report);
    let efficient_lane = source
        .get("readEfficiency")
        .and_then(|value| value.get("efficientCtxhelmLane"))
        .and_then(serde_json::Value::as_str);
    let best_lane = source
        .get("bestLane")
        .and_then(serde_json::Value::as_str)
        .or(efficient_lane);
    if let Some(lane_name) = best_lane {
        if let Some(lanes) = report
            .get("aggregate")
            .and_then(|value| value.get("laneSummaries"))
            .and_then(serde_json::Value::as_array)
        {
            if let Some(lane) = lanes.iter().find(|lane| {
                lane.get("lane").and_then(serde_json::Value::as_str) == Some(lane_name)
            }) {
                return lane
                    .get(metric)
                    .or_else(|| {
                        if metric == "targetReadCoverage" {
                            lane.get("averageTargetReadCoverage")
                        } else {
                            None
                        }
                    })
                    .and_then(serde_json::Value::as_f64);
            }
        }
        if let Some(lanes) = report.get("lanes").and_then(serde_json::Value::as_array) {
            if let Some(lane) = lanes.iter().find(|lane| {
                lane.get("lane").and_then(serde_json::Value::as_str) == Some(lane_name)
            }) {
                return lane
                    .get("metrics")
                    .and_then(|metrics| metrics.get(metric))
                    .and_then(serde_json::Value::as_f64);
            }
        }
    }
    None
}

fn proof_memory_guard(report: &serde_json::Value) -> ProofInspectorMemoryGuard {
    if let Some(status) = report
        .get("memoryGuard")
        .and_then(|value| value.get("status"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            report
                .get("aggregate")
                .and_then(|value| value.get("memoryGuardStatus"))
                .and_then(serde_json::Value::as_str)
        })
    {
        return ProofInspectorMemoryGuard {
            status: status.to_string(),
            detail: "memory guard status was reported by the proof artifact".to_string(),
        };
    }
    ProofInspectorMemoryGuard {
        status: "not_reported".to_string(),
        detail:
            "this proof report did not include a dedicated memory guard field; inspect memory-specific proof before making memory claims"
                .to_string(),
    }
}

fn proof_availability(report: &serde_json::Value) -> Option<ProofInspectorAvailability> {
    if is_client_availability_report(report) {
        let summary = report.get("summary")?;
        let real_agent_outcome_currently_runnable = summary
            .get("realAgentOutcomeCurrentlyRunnable")
            .and_then(serde_json::Value::as_bool);
        let rate_limited_client_count = summary
            .get("rateLimitedClientCount")
            .and_then(serde_json::Value::as_u64);
        let unavailable_client_count = summary
            .get("unavailableClientCount")
            .and_then(serde_json::Value::as_u64);
        let stream_disconnected_client_count = summary
            .get("streamDisconnectedClientCount")
            .and_then(serde_json::Value::as_u64);
        let availability_blocker = if real_agent_outcome_currently_runnable == Some(true) {
            None
        } else if rate_limited_client_count.unwrap_or(0) > 0 {
            Some("rate_limit".to_string())
        } else if stream_disconnected_client_count.unwrap_or(0) > 0 {
            Some("stream_disconnected".to_string())
        } else if unavailable_client_count.unwrap_or(0) > 0 {
            Some("client_unavailable".to_string())
        } else {
            Some("no_ready_clients".to_string())
        };
        return Some(ProofInspectorAvailability {
            tiny_prompt_available: None,
            paired_suite_available: None,
            rate_limited: rate_limited_client_count.map(|count| count > 0),
            client_failure_observed: unavailable_client_count.map(|count| count > 0),
            comparable_lane_count: None,
            availability_blocker,
            client_count: summary
                .get("clientCount")
                .and_then(serde_json::Value::as_u64),
            ready_client_count: summary
                .get("readyClientCount")
                .and_then(serde_json::Value::as_u64),
            unavailable_client_count,
            rate_limited_client_count,
            stream_disconnected_client_count,
            real_agent_outcome_currently_runnable,
            recommended_research_actions: string_array(summary, "recommendedResearchActions"),
        });
    }

    let availability = report.get("clientAvailability")?;
    Some(ProofInspectorAvailability {
        tiny_prompt_available: availability
            .get("tinyPromptAvailable")
            .and_then(serde_json::Value::as_bool),
        paired_suite_available: availability
            .get("pairedSuiteAvailable")
            .and_then(serde_json::Value::as_bool),
        rate_limited: availability
            .get("rateLimited")
            .and_then(serde_json::Value::as_bool),
        client_failure_observed: availability
            .get("clientFailureObserved")
            .and_then(serde_json::Value::as_bool),
        comparable_lane_count: availability
            .get("comparableLaneCount")
            .and_then(serde_json::Value::as_u64),
        availability_blocker: availability
            .get("availabilityBlocker")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        client_count: None,
        ready_client_count: None,
        unavailable_client_count: None,
        rate_limited_client_count: None,
        stream_disconnected_client_count: None,
        real_agent_outcome_currently_runnable: None,
        recommended_research_actions: Vec::new(),
    })
}

fn string_array(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn recommended_proof_next_action(
    outcome: &ProofInspectorOutcome,
    availability: Option<&ProofInspectorAvailability>,
    evidence: &ProofInspectorEvidence,
    boundary: &ProofInspectorBoundary,
    privacy: &ProofInspectorPrivacyStatus,
    product_proof: Option<&ProductProofInspectorSummary>,
) -> String {
    if privacy.source_text_logged == Some(true)
        || privacy.raw_prompt_stored == Some(true)
        || privacy.raw_transcript_stored == Some(true)
        || privacy.raw_mcp_traffic_stored == Some(true)
    {
        return "Reject this proof for public claims until the source-free privacy boundary is restored.".to_string();
    }
    if boundary.forbidden_boundary_events_observed == Some(true) {
        return "Do not promote this run; investigate read-only boundary violations first."
            .to_string();
    }
    if availability.and_then(|availability| availability.real_agent_outcome_currently_runnable)
        == Some(true)
    {
        return "Run comparable paired real-agent outcome suites for the ready clients; do not treat availability as outcome proof.".to_string();
    }
    if boundary.client_failures_observed == Some(true)
        || boundary.rate_limits_observed == Some(true)
        || availability.and_then(|availability| availability.paired_suite_available) == Some(false)
    {
        return "Treat this as availability-blocked and rerun after the client is stable."
            .to_string();
    }
    if let Some(product) = product_proof {
        if product.release_gate_decision == "promote"
            && product.default_promotion_allowed == Some(true)
            && product.max_protected_target_miss_rate_at_10.unwrap_or(1.0) <= f64::EPSILON
        {
            return "Use as source-free product-proof evidence, then pair it with real-agent outcome proof before making agent-productivity claims.".to_string();
        }
        return "Do not promote this product proof until the release-gate decision, promotion allowance, and protected-target miss-rate are clean.".to_string();
    }
    if outcome.claim == "availability_blocked" {
        return "Treat this as availability-blocked and rerun after at least one real client is ready.".to_string();
    }
    if outcome.comparison_eligible_count == Some(0)
        || outcome.comparable_ctxhelm_lane_count == Some(0)
        || outcome.claim == "insufficient_comparable_lanes"
    {
        return "Collect a comparable baseline plus at least one ctxhelm-assisted lane before making an outcome claim.".to_string();
    }
    if evidence.evidence_misses_observed == Some(true) {
        return "Fix retrieval or task construction before promoting this proof.".to_string();
    }
    if evidence.evidence_only_targets_after_retry.unwrap_or(0) > 0
        || evidence.under_read_targets_observed == Some(true)
    {
        return "Tighten agent consumption guidance or retry enforcement before claiming reliable consumption.".to_string();
    }
    if outcome.claim == "ctxhelm_improved" {
        return "Use as source-free outcome evidence, then repeat the suite to check stability and efficiency.".to_string();
    }
    "Inspect the full agent-run report and choose the next R&D action from its recommendations."
        .to_string()
}

fn optional_bool(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn optional_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn render_retry_cost(value: Option<&serde_json::Value>) -> String {
    let Some(retry) = value else {
        return String::new();
    };
    format!(
        "- Retry cost: triggered `{}` selected `{}` avg reads before `{}` after `{}` avg irrelevant before `{}` after `{}` target-read coverage before `{}` after `{}` evidence-only targets before `{}` after `{}`\n",
        retry
            .get("retryTriggeredLanes")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("retrySelectedLanes")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("avgReadFilesBeforeRetry")
            .and_then(serde_json::Value::as_f64)
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("avgReadFilesAfterRetry")
            .and_then(serde_json::Value::as_f64)
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("avgIrrelevantReadsBeforeRetry")
            .and_then(serde_json::Value::as_f64)
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("avgIrrelevantReadsAfterRetry")
            .and_then(serde_json::Value::as_f64)
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("targetReadCoverageBeforeRetry")
            .and_then(serde_json::Value::as_f64)
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("targetReadCoverageAfterRetry")
            .and_then(serde_json::Value::as_f64)
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("evidenceOnlyTargetsBeforeRetry")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        retry
            .get("evidenceOnlyTargetsAfterRetry")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
    )
}
