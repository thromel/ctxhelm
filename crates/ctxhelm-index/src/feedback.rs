use crate::inventory::{canonicalize, ctxhelm_home, repo_id_for_path, InventoryError};
use ctxhelm_core::{
    AgentOutcomeComparisonReport, BudgetOutcome, CandidateFeatureExport, CandidateFeatureLabel,
    Diagnostic, DiagnosticSeverity, FeedbackOutcome, FeedbackSummary, PackBudget,
    PolicyBaselineThreshold, PolicyMetricSummary, PolicyProfileActionReport, PolicyProfileStatus,
    PolicyQualityReport, PolicySafetyFloor, PolicySignalContribution, PolicySignalWeight,
    PolicyTokenRoi, PolicyTrainingSource, RepeatedMissingFileFamily, RetrievalPolicyProfile,
    RetrievalSignalKind, SessionFeedbackEvent, TraceStatus, TraceStatusKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const FEEDBACK_EVENT_SCHEMA_VERSION: u32 = 1;
pub const LEARNED_POLICY_PROFILE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone)]
pub struct LearnedPolicyOptions {
    pub feedback_limit: usize,
    pub min_context_precision: f32,
    pub min_validation_coverage: f32,
    pub min_pass_rate: f32,
    pub min_gold_or_selected_rows: usize,
}

impl Default for LearnedPolicyOptions {
    fn default() -> Self {
        Self {
            feedback_limit: 20,
            min_context_precision: 0.4,
            min_validation_coverage: 0.2,
            min_pass_rate: 0.0,
            min_gold_or_selected_rows: 1,
        }
    }
}

pub fn append_feedback_event(
    repo_root: impl AsRef<Path>,
    event: &SessionFeedbackEvent,
) -> Result<PathBuf, InventoryError> {
    validate_feedback_event(event)?;
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = feedback_path(&repo_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
    let json = serde_json::to_string(event).map_err(InventoryError::Serialize)?;
    writeln!(file, "{json}").map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

pub fn try_append_feedback_event(
    repo_root: impl AsRef<Path>,
    event: &SessionFeedbackEvent,
) -> TraceStatus {
    let path = feedback_path_for_repo_root(repo_root.as_ref());
    match append_feedback_event(repo_root, event) {
        Ok(path) => TraceStatus {
            status: TraceStatusKind::Written,
            path: Some(path.display().to_string()),
            diagnostics: Vec::new(),
        },
        Err(error) => TraceStatus {
            status: TraceStatusKind::WriteFailed,
            path: path.map(|path| path.display().to_string()),
            diagnostics: vec![Diagnostic {
                code: "feedback_write_failed".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!("Feedback event was not recorded: {error}"),
                paths: Vec::new(),
                count: 1,
            }],
        },
    }
}

pub fn list_feedback_events(
    repo_root: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<SessionFeedbackEvent>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = feedback_path(&repo_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(InventoryError::Read {
                path: path.clone(),
                source,
            })
        }
    };

    let mut events = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<SessionFeedbackEvent>(line).map_err(|source| {
                InventoryError::Deserialize {
                    path: path.clone(),
                    source,
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    events.reverse();
    events.truncate(limit.max(1));
    Ok(events)
}

pub fn summarize_feedback_events(
    repo_id: &str,
    events: &[SessionFeedbackEvent],
) -> FeedbackSummary {
    let mut read_files = BTreeSet::new();
    let mut edited_files = BTreeSet::new();
    let mut tested_files = BTreeSet::new();
    let mut corrected_files = BTreeSet::new();
    let mut summary = FeedbackSummary {
        repo_id: repo_id.to_string(),
        event_count: events.len(),
        passed_count: 0,
        failed_count: 0,
        blocked_count: 0,
        unknown_count: 0,
        read_file_count: 0,
        edited_file_count: 0,
        tested_file_count: 0,
        user_corrected_file_count: 0,
        source_text_logged: false,
    };

    for event in events {
        match event.outcome {
            FeedbackOutcome::Passed => summary.passed_count += 1,
            FeedbackOutcome::Failed => summary.failed_count += 1,
            FeedbackOutcome::Blocked => summary.blocked_count += 1,
            FeedbackOutcome::Unknown => summary.unknown_count += 1,
        }
        summary.source_text_logged |= event.source_text_logged;
        read_files.extend(event.read_files.iter().cloned());
        edited_files.extend(event.edited_files.iter().cloned());
        tested_files.extend(event.tested_files.iter().cloned());
        corrected_files.extend(event.user_corrected_files.iter().cloned());
    }

    summary.read_file_count = read_files.len();
    summary.edited_file_count = edited_files.len();
    summary.tested_file_count = tested_files.len();
    summary.user_corrected_file_count = corrected_files.len();
    summary
}

pub fn feedback_path(repo_id: &str) -> PathBuf {
    ctxhelm_home()
        .join("repos")
        .join(repo_id)
        .join("feedback.jsonl")
}

pub fn policy_quality_report(
    repo_root: impl AsRef<Path>,
    limit: usize,
) -> Result<PolicyQualityReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let events = list_feedback_events(&repo_root, limit)?;
    Ok(policy_quality_report_from_events(&repo_id, &events))
}

pub fn outcome_comparison_report(
    repo_root: impl AsRef<Path>,
    limit: usize,
) -> Result<AgentOutcomeComparisonReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let events = list_feedback_events(&repo_root, limit)?;
    Ok(outcome_comparison_report_from_events(&repo_id, &events))
}

pub fn propose_policy_profile(
    repo_root: impl AsRef<Path>,
    limit: usize,
) -> Result<RetrievalPolicyProfile, InventoryError> {
    let report = policy_quality_report(&repo_root, limit)?;
    let profile = policy_profile_from_report(&report, current_unix_seconds());
    write_policy_profile(repo_root, profile)
}

pub fn propose_learned_policy_profile(
    repo_root: impl AsRef<Path>,
    options: &LearnedPolicyOptions,
) -> Result<RetrievalPolicyProfile, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let feedback_report = policy_quality_report(&repo_root, options.feedback_limit)?;
    let outcome_report = outcome_comparison_report(&repo_root, options.feedback_limit)?;
    let feature_exports = list_policy_feature_exports(&repo_id)?;
    let profile = learned_policy_profile_from_evidence(
        &repo_id,
        &feature_exports,
        &feedback_report,
        &outcome_report,
        options,
        current_unix_seconds(),
    );
    write_policy_profile(&repo_root, profile)
}

pub fn list_policy_profiles(
    repo_root: impl AsRef<Path>,
) -> Result<Vec<RetrievalPolicyProfile>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = policy_profiles_path(&repo_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(InventoryError::Read {
                path: path.clone(),
                source,
            })
        }
    };
    serde_json::from_str::<Vec<RetrievalPolicyProfile>>(&content).map_err(|source| {
        InventoryError::Deserialize {
            path: path.clone(),
            source,
        }
    })
}

pub fn apply_policy_profile(
    repo_root: impl AsRef<Path>,
    profile_id: &str,
) -> Result<PolicyProfileActionReport, InventoryError> {
    update_policy_profile_status(repo_root, profile_id, PolicyProfileStatus::Active, "apply")
}

pub fn disable_policy_profile(
    repo_root: impl AsRef<Path>,
    profile_id: &str,
) -> Result<PolicyProfileActionReport, InventoryError> {
    update_policy_profile_status(
        repo_root,
        profile_id,
        PolicyProfileStatus::Disabled,
        "disable",
    )
}

pub fn rollback_policy_profile(
    repo_root: impl AsRef<Path>,
) -> Result<PolicyProfileActionReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let mut profiles = list_policy_profiles(&repo_root)?;
    let Some(active_index) = profiles
        .iter()
        .position(|profile| profile.status == PolicyProfileStatus::Active)
    else {
        return Err(InventoryError::InvalidInput(
            "no active policy profile to roll back".to_string(),
        ));
    };
    let profile_id = profiles[active_index].id.clone();
    profiles[active_index].status = PolicyProfileStatus::RolledBack;
    write_policy_profiles(&repo_id, &profiles)?;
    Ok(PolicyProfileActionReport {
        repo_id,
        profile_id,
        action: "rollback".to_string(),
        active_profile_id: None,
        source_text_logged: false,
    })
}

pub fn policy_profiles_path(repo_id: &str) -> PathBuf {
    ctxhelm_home()
        .join("repos")
        .join(repo_id)
        .join("policy-profiles.json")
}

fn feedback_path_for_repo_root(repo_root: &Path) -> Option<PathBuf> {
    let repo_root = canonicalize(repo_root).ok()?;
    let repo_id = repo_id_for_path(&repo_root);
    Some(feedback_path(&repo_id))
}

fn policy_quality_report_from_events(
    repo_id: &str,
    events: &[SessionFeedbackEvent],
) -> PolicyQualityReport {
    let mut recommended = BTreeSet::new();
    let mut read = BTreeSet::new();
    let mut edited = BTreeSet::new();
    let mut tested = BTreeSet::new();
    let mut corrected = BTreeSet::new();
    let mut missing_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut budget_counts: BTreeMap<String, (Option<PackBudget>, usize, usize)> = BTreeMap::new();
    let mut validation_events = 0;
    let mut source_text_logged = false;

    for event in events {
        let event_recommended = event_recommended_files(event);
        let event_useful = event_useful_files(event);
        let recommended_set: BTreeSet<_> = event_recommended.iter().cloned().collect();
        let useful_set: BTreeSet<_> = event_useful.iter().cloned().collect();
        let useful_hits = useful_set.intersection(&recommended_set).count();

        let key = budget_key(event.budget.as_ref());
        let entry = budget_counts
            .entry(key)
            .or_insert_with(|| (event.budget.clone(), 0, 0));
        entry.1 += 1;
        entry.2 += useful_hits;

        recommended.extend(event_recommended);
        read.extend(event.read_files.iter().cloned());
        edited.extend(event.edited_files.iter().cloned());
        tested.extend(event.tested_files.iter().cloned());
        corrected.extend(event.user_corrected_files.iter().cloned());
        source_text_logged |= event.source_text_logged;
        if !event.tested_files.is_empty() || !event.tested_commands.is_empty() {
            validation_events += 1;
        }
        for path in useful_set.difference(&recommended_set) {
            *missing_counts.entry(path.clone()).or_default() += 1;
        }
    }

    let useful: BTreeSet<String> = read
        .iter()
        .chain(edited.iter())
        .chain(tested.iter())
        .chain(corrected.iter())
        .cloned()
        .collect();
    let useful_hits = recommended.intersection(&useful).count();
    let edit_hits = edited.intersection(&recommended).count();
    let correction_events = events
        .iter()
        .filter(|event| !event.user_corrected_files.is_empty())
        .count();

    PolicyQualityReport {
        repo_id: repo_id.to_string(),
        event_count: events.len(),
        sample_warning: sample_warning(events.len()),
        context_precision: ratio(useful_hits, recommended.len()),
        read_precision: ratio(read.intersection(&recommended).count(), read.len()),
        edit_recall_proxy: ratio(edit_hits, edited.len()),
        validation_coverage: ratio(validation_events, events.len()),
        correction_rate: ratio(correction_events, events.len()),
        token_roi: budget_counts
            .into_values()
            .map(|(budget, event_count, useful_hits)| PolicyTokenRoi {
                budget,
                event_count,
                useful_files_per_event: ratio(useful_hits, event_count),
            })
            .collect(),
        repeated_missing_file_families: missing_counts
            .into_iter()
            .map(|(path, count)| RepeatedMissingFileFamily { path, count })
            .collect(),
        signal_contributions: vec![
            signal_contribution(RetrievalSignalKind::Anchor, events, |event| {
                event.read_files.len()
            }),
            signal_contribution(RetrievalSignalKind::RelatedTest, events, |event| {
                event.tested_files.len() + event.tested_commands.len()
            }),
            signal_contribution(RetrievalSignalKind::Memory, events, |event| {
                usize::from(event.tags.iter().any(|tag| tag.contains("memory")))
            }),
        ],
        source_text_logged,
    }
}

fn outcome_comparison_report_from_events(
    repo_id: &str,
    events: &[SessionFeedbackEvent],
) -> AgentOutcomeComparisonReport {
    let mut by_budget: BTreeMap<String, (Option<PackBudget>, Vec<&SessionFeedbackEvent>)> =
        BTreeMap::new();
    for event in events {
        by_budget
            .entry(budget_key(event.budget.as_ref()))
            .or_insert_with(|| (event.budget.clone(), Vec::new()))
            .1
            .push(event);
    }

    AgentOutcomeComparisonReport {
        repo_id: repo_id.to_string(),
        event_count: events.len(),
        sample_warning: sample_warning(events.len()),
        budgets: by_budget
            .into_values()
            .map(|(budget, events)| budget_outcome(budget, &events))
            .collect(),
        changed_sample_warning: by_budget_sample_warning(events),
        low_information_warning: events.len() < 5,
        source_text_logged: events.iter().any(|event| event.source_text_logged),
    }
}

fn policy_profile_from_report(
    report: &PolicyQualityReport,
    created_at_unix_seconds: u64,
) -> RetrievalPolicyProfile {
    let mut weights = vec![
        PolicySignalWeight {
            signal: RetrievalSignalKind::Anchor,
            weight: 0.25,
            rationale: "Exact active-path and user-focus anchors keep safe minimum priority."
                .to_string(),
        },
        PolicySignalWeight {
            signal: RetrievalSignalKind::Lexical,
            weight: 0.20,
            rationale: "Lexical identifiers remain a strong source-code retrieval baseline."
                .to_string(),
        },
        PolicySignalWeight {
            signal: RetrievalSignalKind::RelatedTest,
            weight: if report.validation_coverage >= 0.5 {
                0.20
            } else {
                0.15
            },
            rationale: "Validation feedback controls how much related-test evidence should influence context.".to_string(),
        },
        PolicySignalWeight {
            signal: RetrievalSignalKind::Memory,
            weight: if report.correction_rate <= 0.2 {
                0.10
            } else {
                0.05
            },
            rationale: "Memory remains bounded and is demoted when corrections are frequent.".to_string(),
        },
    ];
    weights.sort_by(|left, right| format!("{:?}", left.signal).cmp(&format!("{:?}", right.signal)));

    RetrievalPolicyProfile {
        id: format!("policy-{}", created_at_unix_seconds),
        status: PolicyProfileStatus::Candidate,
        profile_schema_version: LEARNED_POLICY_PROFILE_SCHEMA_VERSION,
        created_at_unix_seconds,
        source_report_event_count: report.event_count,
        training_corpus_id: Some(format!("feedback-events-{}", report.event_count)),
        training_sources: vec![PolicyTrainingSource {
            source_kind: "feedback".to_string(),
            source_id: Some(report.repo_id.clone()),
            schema_version: Some(FEEDBACK_EVENT_SCHEMA_VERSION.to_string()),
            row_count: report.event_count,
        }],
        metric_summary: vec![
            PolicyMetricSummary {
                metric: "context_precision".to_string(),
                value: report.context_precision,
                unit: "ratio".to_string(),
            },
            PolicyMetricSummary {
                metric: "validation_coverage".to_string(),
                value: report.validation_coverage,
                unit: "ratio".to_string(),
            },
            PolicyMetricSummary {
                metric: "correction_rate".to_string(),
                value: report.correction_rate,
                unit: "ratio".to_string(),
            },
        ],
        rationale: "Candidate generated from local source-free feedback metrics.".to_string(),
        weights,
        safety_floors: vec![
            PolicySafetyFloor {
                signal: RetrievalSignalKind::Anchor,
                minimum_weight: 0.20,
                reason:
                    "Active paths and explicit user anchors must not be demoted below safety floor."
                        .to_string(),
            },
            PolicySafetyFloor {
                signal: RetrievalSignalKind::Lexical,
                minimum_weight: 0.15,
                reason: "Exact identifiers and paths remain mandatory for code tasks.".to_string(),
            },
            PolicySafetyFloor {
                signal: RetrievalSignalKind::RelatedTest,
                minimum_weight: 0.10,
                reason: "Validation context must stay visible for bug-fix and test tasks."
                    .to_string(),
            },
        ],
        regression_warnings: report
            .sample_warning
            .clone()
            .into_iter()
            .chain(
                (report.context_precision < 0.4).then_some(
                    "Context precision is low; inspect missing-file families before applying."
                        .to_string(),
                ),
            )
            .collect(),
        baseline_thresholds: Vec::new(),
        default_eligible: true,
        rollback_profile_id: None,
        source_text_logged: false,
    }
}

fn learned_policy_profile_from_evidence(
    repo_id: &str,
    feature_exports: &[CandidateFeatureExport],
    feedback_report: &PolicyQualityReport,
    outcome_report: &AgentOutcomeComparisonReport,
    options: &LearnedPolicyOptions,
    created_at_unix_seconds: u64,
) -> RetrievalPolicyProfile {
    let feature_row_count = feature_exports
        .iter()
        .map(|export| export.rows.len())
        .sum::<usize>();
    let selected_or_gold_rows = feature_exports
        .iter()
        .flat_map(|export| export.rows.iter())
        .filter(|row| {
            row.labels.contains(&CandidateFeatureLabel::Gold)
                || row.labels.contains(&CandidateFeatureLabel::Selected)
        })
        .count();
    let pass_rate = if outcome_report.event_count == 0 {
        0.0
    } else {
        outcome_report
            .budgets
            .iter()
            .map(|budget| budget.pass_rate * budget.event_count as f32)
            .sum::<f32>()
            / outcome_report.event_count as f32
    };

    let thresholds = vec![
        PolicyBaselineThreshold {
            metric: "context_precision".to_string(),
            value: feedback_report.context_precision,
            threshold: options.min_context_precision,
            passed: feedback_report.context_precision >= options.min_context_precision,
        },
        PolicyBaselineThreshold {
            metric: "validation_coverage".to_string(),
            value: feedback_report.validation_coverage,
            threshold: options.min_validation_coverage,
            passed: feedback_report.validation_coverage >= options.min_validation_coverage,
        },
        PolicyBaselineThreshold {
            metric: "pass_rate".to_string(),
            value: pass_rate,
            threshold: options.min_pass_rate,
            passed: pass_rate >= options.min_pass_rate,
        },
        PolicyBaselineThreshold {
            metric: "gold_or_selected_rows".to_string(),
            value: selected_or_gold_rows as f32,
            threshold: options.min_gold_or_selected_rows as f32,
            passed: selected_or_gold_rows >= options.min_gold_or_selected_rows,
        },
    ];
    let default_eligible = thresholds.iter().all(|threshold| threshold.passed);
    let mut weights = learned_signal_weights(feature_exports, feedback_report);
    weights.sort_by(|left, right| format!("{:?}", left.signal).cmp(&format!("{:?}", right.signal)));

    RetrievalPolicyProfile {
        id: format!("learned-policy-{}", created_at_unix_seconds),
        status: PolicyProfileStatus::Candidate,
        profile_schema_version: LEARNED_POLICY_PROFILE_SCHEMA_VERSION,
        created_at_unix_seconds,
        source_report_event_count: feedback_report.event_count,
        training_corpus_id: Some(format!(
            "{}-exports-{}-feedback-{}",
            repo_id,
            feature_exports.len(),
            feedback_report.event_count
        )),
        training_sources: learned_training_sources(repo_id, feature_exports, feedback_report),
        metric_summary: vec![
            PolicyMetricSummary {
                metric: "feature_export_rows".to_string(),
                value: feature_row_count as f32,
                unit: "rows".to_string(),
            },
            PolicyMetricSummary {
                metric: "gold_or_selected_rows".to_string(),
                value: selected_or_gold_rows as f32,
                unit: "rows".to_string(),
            },
            PolicyMetricSummary {
                metric: "context_precision".to_string(),
                value: feedback_report.context_precision,
                unit: "ratio".to_string(),
            },
            PolicyMetricSummary {
                metric: "validation_coverage".to_string(),
                value: feedback_report.validation_coverage,
                unit: "ratio".to_string(),
            },
            PolicyMetricSummary {
                metric: "pass_rate".to_string(),
                value: pass_rate,
                unit: "ratio".to_string(),
            },
        ],
        rationale: "Candidate generated from source-free feature exports, historical labels carried by feature rows, and feedback/outcome traces.".to_string(),
        weights,
        safety_floors: learned_safety_floors(),
        regression_warnings: learned_regression_warnings(
            feature_exports,
            feedback_report,
            selected_or_gold_rows,
            default_eligible,
        ),
        baseline_thresholds: thresholds,
        default_eligible,
        rollback_profile_id: None,
        source_text_logged: feature_exports.iter().any(|export| export.source_text_logged)
            || feedback_report.source_text_logged
            || outcome_report.source_text_logged,
    }
}

fn list_policy_feature_exports(
    repo_id: &str,
) -> Result<Vec<CandidateFeatureExport>, InventoryError> {
    let dir = ctxhelm_home()
        .join("repos")
        .join(repo_id)
        .join("feature-exports");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(InventoryError::Read { path: dir, source }),
    };
    let mut exports = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| InventoryError::Read {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|source| InventoryError::Read {
            path: path.clone(),
            source,
        })?;
        exports.push(serde_json::from_str(&content).map_err(|source| {
            InventoryError::Deserialize {
                path: path.clone(),
                source,
            }
        })?);
    }
    exports.sort_by(
        |left: &CandidateFeatureExport, right: &CandidateFeatureExport| {
            right
                .created_at_unix_seconds
                .cmp(&left.created_at_unix_seconds)
                .then_with(|| left.export_id.cmp(&right.export_id))
        },
    );
    Ok(exports)
}

fn learned_signal_weights(
    feature_exports: &[CandidateFeatureExport],
    feedback_report: &PolicyQualityReport,
) -> Vec<PolicySignalWeight> {
    let mut lexical_hit = 0.0;
    let mut semantic_hit = 0.0;
    let mut dependency_hit = 0.0;
    let mut history_hit = 0.0;
    let mut test_hit = 0.0;
    let mut memory_hit = 0.0;
    for row in feature_exports.iter().flat_map(|export| export.rows.iter()) {
        if !(row.labels.contains(&CandidateFeatureLabel::Gold)
            || row.labels.contains(&CandidateFeatureLabel::Selected)
            || row.labels.contains(&CandidateFeatureLabel::Read)
            || row.labels.contains(&CandidateFeatureLabel::Edited))
        {
            continue;
        }
        lexical_hit += row.lexical_score;
        semantic_hit += row.semantic_score;
        dependency_hit += row.graph_score;
        history_hit += row.history_score;
        test_hit += row.test_score;
        memory_hit += row.memory_score;
    }

    let total = (lexical_hit + semantic_hit + dependency_hit + history_hit + test_hit + memory_hit)
        .max(1.0);
    let mut weights = vec![
        learned_weight(
            RetrievalSignalKind::Anchor,
            0.20,
            "Anchors stay at a conservative floor even when offline rows are sparse.",
        ),
        learned_weight(
            RetrievalSignalKind::Lexical,
            (lexical_hit / total).clamp(0.15, 0.35),
            "Learned from selected/gold feature rows while preserving exact-token floor.",
        ),
        learned_weight(
            RetrievalSignalKind::RelatedTest,
            ((test_hit / total) + feedback_report.validation_coverage * 0.15).clamp(0.10, 0.30),
            "Validation coverage from feedback controls related-test weight.",
        ),
        learned_weight(
            RetrievalSignalKind::Dependency,
            (dependency_hit / total).clamp(0.05, 0.25),
            "Graph contribution is bounded by source-free selected/gold rows.",
        ),
        learned_weight(
            RetrievalSignalKind::CoChange,
            (history_hit / total).clamp(0.05, 0.20),
            "History contribution is bounded by source-free selected/gold rows.",
        ),
        learned_weight(
            RetrievalSignalKind::Memory,
            if memory_hit > 0.0 && feedback_report.correction_rate <= 0.2 {
                (memory_hit / total).clamp(0.05, 0.10)
            } else {
                0.05
            },
            "Memory is demoted when feedback correction rate is high.",
        ),
    ];
    if semantic_hit > 0.0 {
        weights.push(learned_weight(
            RetrievalSignalKind::Semantic,
            (semantic_hit / total).clamp(0.05, 0.15),
            "Semantic remains opt-in and bounded by selected/gold feature evidence.",
        ));
    }
    weights
}

fn learned_weight(signal: RetrievalSignalKind, weight: f32, rationale: &str) -> PolicySignalWeight {
    PolicySignalWeight {
        signal,
        weight,
        rationale: rationale.to_string(),
    }
}

fn learned_safety_floors() -> Vec<PolicySafetyFloor> {
    vec![
        PolicySafetyFloor {
            signal: RetrievalSignalKind::Anchor,
            minimum_weight: 0.20,
            reason: "Explicit user/file anchors remain mandatory.".to_string(),
        },
        PolicySafetyFloor {
            signal: RetrievalSignalKind::Lexical,
            minimum_weight: 0.15,
            reason: "Exact identifiers remain mandatory for code tasks.".to_string(),
        },
        PolicySafetyFloor {
            signal: RetrievalSignalKind::RelatedTest,
            minimum_weight: 0.10,
            reason: "Validation context must not be removed by learning.".to_string(),
        },
    ]
}

fn learned_training_sources(
    repo_id: &str,
    feature_exports: &[CandidateFeatureExport],
    feedback_report: &PolicyQualityReport,
) -> Vec<PolicyTrainingSource> {
    let mut sources = feature_exports
        .iter()
        .map(|export| PolicyTrainingSource {
            source_kind: format!("{:?}", export.export_source).to_lowercase(),
            source_id: Some(export.export_id.to_string()),
            schema_version: Some(export.schema_version.to_string()),
            row_count: export.rows.len(),
        })
        .collect::<Vec<_>>();
    sources.push(PolicyTrainingSource {
        source_kind: "feedback".to_string(),
        source_id: Some(repo_id.to_string()),
        schema_version: Some(FEEDBACK_EVENT_SCHEMA_VERSION.to_string()),
        row_count: feedback_report.event_count,
    });
    sources
}

fn learned_regression_warnings(
    feature_exports: &[CandidateFeatureExport],
    feedback_report: &PolicyQualityReport,
    selected_or_gold_rows: usize,
    default_eligible: bool,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if feature_exports.is_empty() {
        warnings.push("No source-free candidate feature exports were available.".to_string());
    }
    if selected_or_gold_rows == 0 {
        warnings.push("No selected or gold feature rows were available.".to_string());
    }
    warnings.extend(feedback_report.sample_warning.clone());
    if !default_eligible {
        warnings.push(
            "Configured thresholds did not pass; profile cannot be applied as active default."
                .to_string(),
        );
    }
    warnings
}

pub fn write_policy_profile(
    repo_root: impl AsRef<Path>,
    profile: RetrievalPolicyProfile,
) -> Result<RetrievalPolicyProfile, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let mut profiles = list_policy_profiles(&repo_root)?;
    profiles.retain(|existing| existing.id != profile.id);
    profiles.push(profile.clone());
    write_policy_profiles(&repo_id, &profiles)?;
    Ok(profile)
}

fn update_policy_profile_status(
    repo_root: impl AsRef<Path>,
    profile_id: &str,
    status: PolicyProfileStatus,
    action: &str,
) -> Result<PolicyProfileActionReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let mut profiles = list_policy_profiles(&repo_root)?;
    let mut found = false;
    for profile in &mut profiles {
        if status == PolicyProfileStatus::Active && profile.status == PolicyProfileStatus::Active {
            profile.status = PolicyProfileStatus::Disabled;
        }
        if profile.id == profile_id {
            if status == PolicyProfileStatus::Active && !profile.default_eligible {
                return Err(InventoryError::InvalidInput(format!(
                    "policy profile {profile_id} is not eligible to become active; inspect baselineThresholds and regressionWarnings"
                )));
            }
            profile.status = status.clone();
            found = true;
        }
    }
    if !found {
        return Err(InventoryError::InvalidInput(format!(
            "policy profile not found: {profile_id}"
        )));
    }
    write_policy_profiles(&repo_id, &profiles)?;
    Ok(PolicyProfileActionReport {
        repo_id,
        profile_id: profile_id.to_string(),
        action: action.to_string(),
        active_profile_id: profiles
            .iter()
            .find(|profile| profile.status == PolicyProfileStatus::Active)
            .map(|profile| profile.id.clone()),
        source_text_logged: false,
    })
}

fn write_policy_profiles(
    repo_id: &str,
    profiles: &[RetrievalPolicyProfile],
) -> Result<(), InventoryError> {
    let path = policy_profiles_path(repo_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(profiles).map_err(InventoryError::Serialize)?;
    fs::write(&path, json).map_err(|source| InventoryError::Write { path, source })
}

fn validate_feedback_event(event: &SessionFeedbackEvent) -> Result<(), InventoryError> {
    if event.source_text_logged {
        return Err(InventoryError::InvalidInput(
            "feedback events must not log source text".to_string(),
        ));
    }
    if event.schema_version != FEEDBACK_EVENT_SCHEMA_VERSION {
        return Err(InventoryError::InvalidInput(format!(
            "unsupported feedback event schema version {}; expected {}",
            event.schema_version, FEEDBACK_EVENT_SCHEMA_VERSION
        )));
    }

    for value in event
        .recommended_files
        .iter()
        .chain(event.recommended_tests.iter())
        .chain(event.read_files.iter())
        .chain(event.edited_files.iter())
        .chain(event.tested_files.iter())
        .chain(event.user_corrected_files.iter())
    {
        validate_path_label(value)?;
    }
    for value in event
        .recommended_commands
        .iter()
        .chain(event.tested_commands.iter())
        .chain(event.tags.iter())
    {
        validate_source_free_label(value)?;
    }
    Ok(())
}

fn validate_path_label(value: &str) -> Result<(), InventoryError> {
    validate_source_free_label(value)?;
    if value.starts_with('/') || value.contains("..") {
        return Err(InventoryError::InvalidInput(format!(
            "feedback path must be a safe repository-relative label: {value}"
        )));
    }
    Ok(())
}

fn validate_source_free_label(value: &str) -> Result<(), InventoryError> {
    if value.trim().is_empty()
        || value.len() > 512
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("```")
    {
        return Err(InventoryError::InvalidInput(
            "feedback labels must be short source-free single-line values".to_string(),
        ));
    }
    Ok(())
}

fn event_recommended_files(event: &SessionFeedbackEvent) -> Vec<String> {
    event
        .recommended_files
        .iter()
        .chain(event.recommended_tests.iter())
        .cloned()
        .collect()
}

fn event_useful_files(event: &SessionFeedbackEvent) -> Vec<String> {
    event
        .read_files
        .iter()
        .chain(event.edited_files.iter())
        .chain(event.tested_files.iter())
        .chain(event.user_corrected_files.iter())
        .cloned()
        .collect()
}

fn signal_contribution(
    signal: RetrievalSignalKind,
    events: &[SessionFeedbackEvent],
    count: impl Fn(&SessionFeedbackEvent) -> usize,
) -> PolicySignalContribution {
    let useful_file_hits = events.iter().map(count).sum();
    PolicySignalContribution {
        signal,
        event_count: events.len(),
        useful_file_hits,
        score: ratio(useful_file_hits, events.len().max(1)),
    }
}

fn budget_outcome(budget: Option<PackBudget>, events: &[&SessionFeedbackEvent]) -> BudgetOutcome {
    let passed = events
        .iter()
        .filter(|event| event.outcome == FeedbackOutcome::Passed)
        .count();
    let blocked = events
        .iter()
        .filter(|event| event.outcome == FeedbackOutcome::Blocked)
        .count();
    let corrected = events
        .iter()
        .filter(|event| !event.user_corrected_files.is_empty())
        .count();
    let validated = events
        .iter()
        .filter(|event| !event.tested_files.is_empty() || !event.tested_commands.is_empty())
        .count();
    let recommended_context_size: usize = events
        .iter()
        .map(|event| event.recommended_files.len() + event.recommended_tests.len())
        .sum();
    let useful_hits: usize = events
        .iter()
        .map(|event| {
            let recommended: BTreeSet<_> = event_recommended_files(event).into_iter().collect();
            let useful: BTreeSet<_> = event_useful_files(event).into_iter().collect();
            recommended.intersection(&useful).count()
        })
        .sum();
    let token_estimate = budget
        .as_ref()
        .map(pack_budget_token_estimate)
        .unwrap_or(1500)
        * events.len().max(1);

    BudgetOutcome {
        budget,
        event_count: events.len(),
        pass_rate: ratio(passed, events.len()),
        blocked_rate: ratio(blocked, events.len()),
        correction_rate: ratio(corrected, events.len()),
        validation_coverage: ratio(validated, events.len()),
        average_recommended_context_size: ratio(recommended_context_size, events.len()),
        useful_target_files_per_1k_tokens: (useful_hits as f32)
            / ((token_estimate as f32) / 1000.0),
    }
}

fn by_budget_sample_warning(events: &[SessionFeedbackEvent]) -> bool {
    let budgets: BTreeSet<_> = events
        .iter()
        .map(|event| budget_key(event.budget.as_ref()))
        .collect();
    budgets.len() > 1 && events.len() < 12
}

fn pack_budget_token_estimate(budget: &PackBudget) -> usize {
    match budget {
        PackBudget::Brief => 4_000,
        PackBudget::Standard => 16_000,
        PackBudget::Deep => 64_000,
    }
}

fn budget_key(budget: Option<&PackBudget>) -> String {
    match budget {
        Some(PackBudget::Brief) => "brief".to_string(),
        Some(PackBudget::Standard) => "standard".to_string(),
        Some(PackBudget::Deep) => "deep".to_string(),
        None => "plan_only".to_string(),
    }
}

fn sample_warning(event_count: usize) -> Option<String> {
    (event_count < 5).then_some(
        "Low feedback sample count; use this report as directional evidence only.".to_string(),
    )
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
