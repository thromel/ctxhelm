use crate::inventory::{canonicalize, ctxpack_home, repo_id_for_path, InventoryError};
use ctxpack_core::{
    AgentOutcomeComparisonReport, BudgetOutcome, Diagnostic, DiagnosticSeverity, FeedbackOutcome,
    FeedbackSummary, PackBudget, PolicyProfileActionReport, PolicyProfileStatus,
    PolicyQualityReport, PolicySafetyFloor, PolicySignalContribution, PolicySignalWeight,
    PolicyTokenRoi, RepeatedMissingFileFamily, RetrievalPolicyProfile, RetrievalSignalKind,
    SessionFeedbackEvent, TraceStatus, TraceStatusKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const FEEDBACK_EVENT_SCHEMA_VERSION: u32 = 1;

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
    ctxpack_home()
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
    ctxpack_home()
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
        created_at_unix_seconds,
        source_report_event_count: report.event_count,
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
        rollback_profile_id: None,
        source_text_logged: false,
    }
}

fn write_policy_profile(
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
