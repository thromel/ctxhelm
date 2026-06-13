#!/usr/bin/env python3
"""Validate source-free paired agent-run proof JSON for release-gate use."""

import argparse
import hashlib
import json
import pathlib
import sys
from typing import Any


STRICT_FALSE_PRIVACY_FIELDS = (
    "sourceTextLogged",
    "rawPromptStored",
    "rawTranscriptStored",
    "rawMcpTrafficStored",
    "rawCommandOutputStored",
    "remoteEmbeddingsUsed",
    "remoteRerankingUsed",
)

STRICT_FALSE_OUTCOME_FIELDS = (
    "clientFailuresObserved",
    "rateLimitsObserved",
    "forbiddenCommandsObserved",
    "ctxhelmEvidenceMissesObserved",
    "ctxhelmEvidenceOnlyTargetsObserved",
    "ctxhelmUnderReadTargetsObserved",
    "missingRequiredCtxhelmCallsObserved",
    "invalidRequiredCtxhelmCallsObserved",
)

LANE_SUMMARY_COUNT_FIELDS = (
    "readFileCount",
    "targetReadCount",
    "irrelevantReadCount",
    "commandExecutionCount",
    "ctxhelmToolCallCount",
    "forbiddenCommandCount",
    "requiredCtxhelmCallCount",
    "observedRequiredCtxhelmCallCount",
    "missingRequiredCtxhelmCallCount",
    "invalidRequiredCtxhelmCallCount",
    "ctxhelmEvidenceFileCount",
    "ctxhelmEvidenceTargetHitCount",
    "ctxhelmEvidenceMissedTargetCount",
    "ctxhelmEvidenceOnlyTargetCount",
    "missedTargetCount",
    "targetDiscoveredOnlyCount",
)

RETRY_COST_COUNT_FIELDS = (
    "retryTriggeredLanes",
    "retrySelectedLanes",
    "evidenceOnlyTargetsBeforeRetry",
    "evidenceOnlyTargetsAfterRetry",
)

RETRY_COST_AVERAGE_FIELDS = (
    "avgReadFilesBeforeRetry",
    "avgReadFilesAfterRetry",
    "avgIrrelevantReadsBeforeRetry",
    "avgIrrelevantReadsAfterRetry",
    "targetReadCoverageBeforeRetry",
    "targetReadCoverageAfterRetry",
)

AGGREGATE_COMPARISON_SUM_FIELDS = {
    "readFileDeltaSum": "readFileDelta",
    "irrelevantReadDeltaSum": "irrelevantReadDelta",
    "commandExecutionDeltaSum": "commandExecutionDelta",
}

AGGREGATE_COMPARISON_AVERAGE_FIELDS = {
    "targetReadCoverageDeltaAverage": "targetReadCoverageDelta",
    "targetCoverageDeltaAverage": "targetCoverageDelta",
}

FLOAT_TOLERANCE = 1e-9


def fail(message: str) -> None:
    raise SystemExit(message)


def require_dict(value: Any, label: str) -> dict:
    if not isinstance(value, dict):
        fail(f"{label} was not an object")
    return value


def require_list(value: Any, label: str) -> list:
    if not isinstance(value, list):
        fail(f"{label} was not a list")
    return value


def require_number(value: Any, label: str) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        fail(f"{label} was not numeric")


def require_int(value: Any, label: str) -> int:
    if isinstance(value, bool):
        fail(f"{label} was not an integer")
    try:
        return int(value)
    except (TypeError, ValueError):
        fail(f"{label} was not an integer")


def numbers_match(actual: Any, expected: float) -> bool:
    try:
        actual_number = float(actual)
    except (TypeError, ValueError):
        return False
    return abs(actual_number - expected) <= FLOAT_TOLERANCE


def safe_ratio(numerator: float, denominator: float) -> float:
    if denominator == 0:
        return 0.0
    return numerator / denominator


def validate_privacy(report: dict, label: str) -> None:
    privacy = require_dict(report.get("privacyStatus"), f"{label}.privacyStatus")
    if privacy.get("localOnly") is not True:
        fail(f"{label}.privacyStatus.localOnly was not true")
    for field in STRICT_FALSE_PRIVACY_FIELDS:
        if privacy.get(field) is not False:
            fail(f"{label}.privacyStatus.{field} was not false")


def validate_report_identity(report: dict, args: argparse.Namespace, label: str) -> None:
    if args.expected_ctxhelm_version is not None:
        actual = report.get("ctxhelmVersion")
        if actual != args.expected_ctxhelm_version:
            fail(
                f"{label}.ctxhelmVersion was {actual}, "
                f"expected {args.expected_ctxhelm_version}"
            )
    if args.expected_client_name is None and args.expected_client_version is None:
        return
    client = require_dict(report.get("client"), f"{label}.client")
    if args.expected_client_name is not None:
        actual = client.get("name")
        if actual != args.expected_client_name:
            fail(
                f"{label}.client.name was {actual}, "
                f"expected {args.expected_client_name}"
            )
    if args.expected_client_version is not None:
        actual = client.get("version")
        if actual != args.expected_client_version:
            fail(
                f"{label}.client.version was {actual}, "
                f"expected {args.expected_client_version}"
            )


def validate_runner(report: dict, args: argparse.Namespace, label: str) -> None:
    runner = report.get("runner")
    if runner is None:
        if args.require_runner_fingerprint:
            fail(f"{label}.runner was missing")
        return
    runner = require_dict(runner, f"{label}.runner")
    if runner.get("name") != "e2e-agent-run-codex":
        fail(f"{label}.runner.name was not e2e-agent-run-codex")
    if runner.get("contractVersion") != "ctxhelm-agent-run-codex-runner-v1":
        fail(f"{label}.runner.contractVersion was not ctxhelm-agent-run-codex-runner-v1")
    if runner.get("checkpointValidation") != "runner_fingerprint_v1":
        fail(f"{label}.runner.checkpointValidation was not runner_fingerprint_v1")
    script_sha256 = runner.get("scriptSha256")
    if not isinstance(script_sha256, str) or len(script_sha256) != 64:
        fail(f"{label}.runner.scriptSha256 was not a SHA-256 hex string")
    try:
        int(script_sha256, 16)
    except ValueError:
        fail(f"{label}.runner.scriptSha256 was not hex")
    if args.current_runner_script is not None:
        current_sha256 = report_digest(args.current_runner_script)
        if script_sha256 != current_sha256:
            fail(
                f"{label}.runner.scriptSha256 did not match current runner script "
                f"{args.current_runner_script.name}"
            )


def validate_suite_fingerprint(suite: dict, args: argparse.Namespace) -> None:
    suite_sha256 = suite.get("suiteSha256")
    if args.current_suite is None:
        return
    if not isinstance(suite_sha256, str) or len(suite_sha256) != 64:
        fail("suite.suiteSha256 was not a SHA-256 hex string")
    try:
        int(suite_sha256, 16)
    except ValueError:
        fail("suite.suiteSha256 was not hex")
    current_sha256 = report_digest(args.current_suite)
    if suite_sha256 != current_sha256:
        fail(f"suite.suiteSha256 did not match current suite {args.current_suite.name}")


def current_suite_task_specs(args: argparse.Namespace) -> list | None:
    if args.current_suite is None:
        return None
    try:
        payload = json.loads(args.current_suite.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid current suite JSON: {error}")
    tasks = payload.get("tasks") if isinstance(payload, dict) else payload
    if not isinstance(tasks, list) or not tasks:
        fail("current suite did not contain a non-empty tasks array")
    specs = []
    for index, task in enumerate(tasks):
        if not isinstance(task, dict):
            fail(f"current suite tasks[{index}] was not an object")
        task_text = task.get("task") or task.get("prompt")
        if not isinstance(task_text, str) or not task_text.strip():
            fail(f"current suite tasks[{index}] had no task or prompt")
        targets = task.get("targetFiles") or task.get("target_files")
        if not isinstance(targets, list) or not targets:
            fail(f"current suite tasks[{index}] had no targetFiles")
        target_strings = []
        for target_index, target in enumerate(targets):
            if not isinstance(target, str) or not target.strip():
                fail(f"current suite tasks[{index}].targetFiles[{target_index}] was invalid")
            target_strings.append(target)
        specs.append(
            {
                "taskSha256": hashlib.sha256(task_text.encode("utf-8")).hexdigest(),
                "targetFiles": target_strings,
            }
        )
    return specs


def validate_tasks_against_current_suite(
    tasks: list,
    args: argparse.Namespace,
) -> list | None:
    specs = current_suite_task_specs(args)
    if specs is None:
        return None
    if len(tasks) != len(specs):
        fail(f"tasks length {len(tasks)} did not match current suite task count {len(specs)}")
    for index, (task, spec) in enumerate(zip(tasks, specs)):
        if not isinstance(task, dict):
            fail(f"tasks[{index}] was not an object")
        if task.get("taskSha256") != spec["taskSha256"]:
            fail(f"tasks[{index}].taskSha256 did not match current suite task")
        target_files = task.get("targetFiles")
        if not isinstance(target_files, list):
            fail(f"tasks[{index}].targetFiles was not a list")
        if target_files != spec["targetFiles"]:
            fail(f"tasks[{index}].targetFiles did not match current suite task")
    return specs


def validate_required_calls(summary: dict, label: str) -> None:
    for field in ("missingRequiredCtxhelmCallCount", "invalidRequiredCtxhelmCallCount"):
        if field in summary and int(summary.get(field, 0)) != 0:
            fail(f"{label}.{field} was not zero")


def validate_lane_summaries(
    summaries: list,
    min_ctxhelm_target_read_coverage: float | None,
) -> None:
    ctxhelm_lanes = [
        summary
        for summary in summaries
        if isinstance(summary, dict) and str(summary.get("lane", "")).startswith("ctxhelm-")
    ]
    if not ctxhelm_lanes:
        fail("laneSummaries had no ctxhelm lanes")
    for lane in ctxhelm_lanes:
        lane_name = lane.get("lane", "<unknown>")
        for field in ("clientFailureCount", "rateLimitCount", "forbiddenCommandCount"):
            if int(lane.get(field, 0)) != 0:
                fail(f"laneSummaries[{lane_name}].{field} was not zero")
        validate_required_calls(lane, f"laneSummaries[{lane_name}]")
        for field in (
            "ctxhelmEvidenceMissedTargetCount",
            "ctxhelmEvidenceOnlyTargetCount",
            "missedTargetCount",
            "targetDiscoveredOnlyCount",
        ):
            if field in lane and int(lane.get(field, 0)) != 0:
                fail(f"laneSummaries[{lane_name}].{field} was not zero")
        if min_ctxhelm_target_read_coverage is not None:
            coverage = require_number(
                lane.get("averageTargetReadCoverage"),
                f"laneSummaries[{lane_name}].averageTargetReadCoverage",
            )
            if coverage < min_ctxhelm_target_read_coverage:
                fail(
                    f"laneSummaries[{lane_name}].averageTargetReadCoverage "
                    f"{coverage} < {min_ctxhelm_target_read_coverage}"
                )


def validate_empty_or_missing_list(value: Any, label: str) -> None:
    if value is None:
        return
    items = require_list(value, label)
    if items:
        fail(f"{label} was not empty")


def validate_task_comparison(comparison: dict, args: argparse.Namespace, label: str) -> None:
    if comparison.get("comparisonEligible") is not True:
        fail(f"{label}.comparisonEligible was not true")
    if args.strict:
        for field in STRICT_FALSE_OUTCOME_FIELDS:
            if field in comparison and comparison.get(field) is not False:
                fail(f"{label}.{field} was not false")
    if args.require_retry_cost:
        validate_retry_cost(require_dict(comparison.get("retryCost"), f"{label}.retryCost"), label)


def validate_task_lane(lane: dict, args: argparse.Namespace, label: str) -> None:
    lane_name = str(lane.get("lane", ""))
    if not lane_name:
        fail(f"{label}.lane was missing")
    if lane.get("status") != "passed":
        fail(f"{label}.status was not passed")
    if int(lane.get("clientExitStatus", -1)) != 0:
        fail(f"{label}.clientExitStatus was not zero")
    metrics = require_dict(lane.get("metrics"), f"{label}.metrics")
    for field in ("forbiddenCommandCount", "missingRequiredCtxhelmCallCount", "invalidRequiredCtxhelmCallCount"):
        if int(metrics.get(field, 0)) != 0:
            fail(f"{label}.metrics.{field} was not zero")
    validate_empty_or_missing_list(lane.get("forbiddenCommands"), f"{label}.forbiddenCommands")
    if lane.get("clientFailure") is not None:
        fail(f"{label}.clientFailure was not null")
    if lane.get("rateLimit") is not None:
        fail(f"{label}.rateLimit was not null")
    validate_empty_or_missing_list(lane.get("ctxhelmUnderReadTargets"), f"{label}.ctxhelmUnderReadTargets")

    if not lane_name.startswith("ctxhelm-"):
        return

    if args.min_ctxhelm_target_read_coverage is not None:
        coverage = require_number(
            metrics.get("targetReadCoverage"),
            f"{label}.metrics.targetReadCoverage",
        )
        if coverage < args.min_ctxhelm_target_read_coverage:
            fail(
                f"{label}.metrics.targetReadCoverage "
                f"{coverage} < {args.min_ctxhelm_target_read_coverage}"
            )
    for field in (
        "ctxhelmEvidenceMissedTargetCount",
        "ctxhelmEvidenceOnlyTargetCount",
        "missedTargetCount",
        "targetDiscoveredOnlyCount",
    ):
        if int(metrics.get(field, 0)) != 0:
            fail(f"{label}.metrics.{field} was not zero")
    required_calls = require_list(lane.get("requiredCtxhelmCalls"), f"{label}.requiredCtxhelmCalls")
    if not required_calls:
        fail(f"{label}.requiredCtxhelmCalls was empty")
    required_count = int(metrics.get("requiredCtxhelmCallCount", 0))
    observed_count = int(metrics.get("observedRequiredCtxhelmCallCount", 0))
    tool_count = int(metrics.get("ctxhelmToolCallCount", 0))
    if required_count != len(required_calls):
        fail(f"{label}.metrics.requiredCtxhelmCallCount did not match requiredCtxhelmCalls")
    if observed_count < required_count:
        fail(f"{label}.metrics.observedRequiredCtxhelmCallCount was below required count")
    if tool_count < required_count:
        fail(f"{label}.metrics.ctxhelmToolCallCount was below required count")
    validate_empty_or_missing_list(lane.get("ctxhelmEvidenceMisses"), f"{label}.ctxhelmEvidenceMisses")
    validate_empty_or_missing_list(
        lane.get("ctxhelmEvidenceOnlyTargets"),
        f"{label}.ctxhelmEvidenceOnlyTargets",
    )


def validate_task_lanes(task: dict, args: argparse.Namespace, label: str) -> None:
    lanes = require_list(task.get("lanes"), f"{label}.lanes")
    if not lanes:
        fail(f"{label}.lanes was empty")
    ctxhelm_lane_count = 0
    for index, lane in enumerate(lanes):
        lane = require_dict(lane, f"{label}.lanes[{index}]")
        if str(lane.get("lane", "")).startswith("ctxhelm-"):
            ctxhelm_lane_count += 1
        validate_task_lane(lane, args, f"{label}.lanes[{index}]")
    if ctxhelm_lane_count == 0:
        fail(f"{label}.lanes had no ctxhelm lanes")


def validate_retry_cost(retry_cost: dict, label: str) -> None:
    for field in (
        "retryTriggeredLanes",
        "retrySelectedLanes",
        "avgReadFilesBeforeRetry",
        "avgReadFilesAfterRetry",
        "avgIrrelevantReadsBeforeRetry",
        "avgIrrelevantReadsAfterRetry",
        "targetReadCoverageBeforeRetry",
        "targetReadCoverageAfterRetry",
        "evidenceOnlyTargetsBeforeRetry",
        "evidenceOnlyTargetsAfterRetry",
    ):
        if field not in retry_cost:
            fail(f"{label}.retryCost.{field} was missing")
    if int(retry_cost["retryTriggeredLanes"]) < int(retry_cost["retrySelectedLanes"]):
        fail(f"{label}.retryCost selected more lanes than it triggered")
    if int(retry_cost["evidenceOnlyTargetsAfterRetry"]) != 0:
        fail(f"{label}.retryCost evidence-only targets after retry was not zero")


def validate_common_report(report: dict, args: argparse.Namespace, label: str) -> None:
    if report.get("schemaVersion") != "ctxhelm-agent-run-eval-v1":
        fail(f"{label}.schemaVersion was not ctxhelm-agent-run-eval-v1")
    if args.require_status and report.get("status") != args.require_status:
        fail(f"{label}.status was {report.get('status')}, expected {args.require_status}")
    validate_report_identity(report, args, label)
    validate_privacy(report, label)
    validate_runner(report, args, label)


def validate_common_outcome_fields(outcome: dict, args: argparse.Namespace, label: str) -> None:
    if args.require_outcome and outcome.get("outcomeClaim") != args.require_outcome:
        fail(
            f"{label}.outcomeClaim was {outcome.get('outcomeClaim')}, "
            f"expected {args.require_outcome}"
        )
    if args.strict:
        for field in STRICT_FALSE_OUTCOME_FIELDS:
            if field in outcome and outcome.get(field) is not False:
                fail(f"{label}.{field} was not false")
    if args.min_comparable_ctxhelm_lanes is not None:
        comparable = int(outcome.get("comparableCtxhelmLaneCount", 0))
        if comparable < args.min_comparable_ctxhelm_lanes:
            fail(
                f"{label}.comparableCtxhelmLaneCount {comparable} "
                f"< {args.min_comparable_ctxhelm_lanes}"
            )
    if args.max_extra_read_delta is not None and "readFileDeltaSum" in outcome:
        read_delta = int(outcome.get("readFileDeltaSum", 0))
        if read_delta < -args.max_extra_read_delta:
            fail(
                f"{label}.readFileDeltaSum {read_delta} allows more than "
                f"{args.max_extra_read_delta} extra reads"
            )
    if args.max_extra_read_delta is not None and "readFileDelta" in outcome:
        read_delta = int(outcome.get("readFileDelta", 0))
        if read_delta < -args.max_extra_read_delta:
            fail(
                f"{label}.readFileDelta {read_delta} allows more than "
                f"{args.max_extra_read_delta} extra reads"
            )
    if args.min_irrelevant_read_delta is not None and "irrelevantReadDeltaSum" in outcome:
        irrelevant_delta = int(outcome.get("irrelevantReadDeltaSum", 0))
        if irrelevant_delta < args.min_irrelevant_read_delta:
            fail(
                f"{label}.irrelevantReadDeltaSum {irrelevant_delta} "
                f"< {args.min_irrelevant_read_delta}"
            )
    if args.min_irrelevant_read_delta is not None and "irrelevantReadDelta" in outcome:
        irrelevant_delta = int(outcome.get("irrelevantReadDelta", 0))
        if irrelevant_delta < args.min_irrelevant_read_delta:
            fail(
                f"{label}.irrelevantReadDelta {irrelevant_delta} "
                f"< {args.min_irrelevant_read_delta}"
            )
    retry_cost = outcome.get("retryCost")
    if args.require_retry_cost:
        validate_retry_cost(require_dict(retry_cost, f"{label}.retryCost"), label)


def proof_thresholds(args: argparse.Namespace) -> dict:
    return {
        "requiredStatus": args.require_status,
        "requiredOutcome": args.require_outcome,
        "expectedCtxhelmVersion": args.expected_ctxhelm_version,
        "expectedClientName": args.expected_client_name,
        "expectedClientVersion": args.expected_client_version,
        "minTaskCount": args.min_task_count,
        "minComparisonEligible": args.min_comparison_eligible,
        "minComparableCtxhelmLanes": args.min_comparable_ctxhelm_lanes,
        "minCtxhelmTargetReadCoverage": args.min_ctxhelm_target_read_coverage,
        "maxExtraReadDelta": args.max_extra_read_delta,
        "minIrrelevantReadDelta": args.min_irrelevant_read_delta,
        "requireRetryCost": args.require_retry_cost,
        "requireRunnerFingerprint": args.require_runner_fingerprint,
        "currentRunnerScriptName": (
            args.current_runner_script.name if args.current_runner_script is not None else None
        ),
        "requireCurrentRunnerScript": args.current_runner_script is not None,
        "currentSuiteName": args.current_suite.name if args.current_suite is not None else None,
        "requireCurrentSuite": args.current_suite is not None,
        "strictBoundaries": args.strict,
    }


def report_digest(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def runner_summary(report: dict, args: argparse.Namespace) -> dict:
    runner = report.get("runner")
    if not isinstance(runner, dict):
        return {"present": False}
    summary = {
        "present": True,
        "name": runner.get("name"),
        "contractVersion": runner.get("contractVersion"),
        "checkpointValidation": runner.get("checkpointValidation"),
        "scriptSha256": runner.get("scriptSha256"),
    }
    if args.current_runner_script is not None:
        current_sha256 = report_digest(args.current_runner_script)
        summary["currentRunnerScriptName"] = args.current_runner_script.name
        summary["currentRunnerScriptSha256"] = current_sha256
        summary["matchesCurrentRunnerScript"] = runner.get("scriptSha256") == current_sha256
    return summary


def suite_summary(suite: dict, args: argparse.Namespace) -> dict:
    summary = {
        "taskCount": suite.get("taskCount"),
        "rawTasksStored": suite.get("rawTasksStored"),
        "suiteSha256": suite.get("suiteSha256"),
        "checkpointEnabled": suite.get("checkpointEnabled"),
        "reusedTaskCount": suite.get("reusedTaskCount"),
    }
    if args.current_suite is not None:
        current_sha256 = report_digest(args.current_suite)
        summary["currentSuiteName"] = args.current_suite.name
        summary["currentSuiteSha256"] = current_sha256
        summary["matchesCurrentSuite"] = suite.get("suiteSha256") == current_sha256
    return summary


def boundary_summary(outcome: dict) -> dict:
    return {field: outcome.get(field) is False for field in STRICT_FALSE_OUTCOME_FIELDS}


def boundary_status(outcome: dict) -> dict:
    return {field: outcome.get(field) for field in STRICT_FALSE_OUTCOME_FIELDS}


def privacy_summary(report: dict) -> dict:
    privacy = require_dict(report.get("privacyStatus"), "privacyStatus")
    return {
        "localOnly": privacy.get("localOnly"),
        **{field: privacy.get(field) for field in STRICT_FALSE_PRIVACY_FIELDS},
    }


def privacy_checks(report: dict) -> dict:
    privacy = require_dict(report.get("privacyStatus"), "privacyStatus")
    strict_false_fields = {
        field: privacy.get(field) is False for field in STRICT_FALSE_PRIVACY_FIELDS
    }
    return {
        "localOnly": privacy.get("localOnly") is True,
        "allSourceFreeFieldsFalse": all(strict_false_fields.values()),
        "strictFalseFields": strict_false_fields,
    }


def identity_summary(report: dict, args: argparse.Namespace) -> dict:
    client = report.get("client") if isinstance(report.get("client"), dict) else {}
    summary = {
        "ctxhelmVersion": report.get("ctxhelmVersion"),
        "clientName": client.get("name"),
        "clientVersion": client.get("version"),
    }
    if args.expected_ctxhelm_version is not None:
        summary["expectedCtxhelmVersion"] = args.expected_ctxhelm_version
        summary["matchesExpectedCtxhelmVersion"] = (
            report.get("ctxhelmVersion") == args.expected_ctxhelm_version
        )
    if args.expected_client_name is not None:
        summary["expectedClientName"] = args.expected_client_name
        summary["matchesExpectedClientName"] = client.get("name") == args.expected_client_name
    if args.expected_client_version is not None:
        summary["expectedClientVersion"] = args.expected_client_version
        summary["matchesExpectedClientVersion"] = (
            client.get("version") == args.expected_client_version
        )
    return summary


def lane_quality_summary(summaries: list) -> list:
    quality = []
    for summary in summaries:
        if not isinstance(summary, dict) or not str(summary.get("lane", "")).startswith("ctxhelm-"):
            continue
        quality.append(
            {
                "lane": summary.get("lane"),
                "averageTargetReadCoverage": summary.get("averageTargetReadCoverage"),
                "readFileCount": summary.get("readFileCount"),
                "irrelevantReadCount": summary.get("irrelevantReadCount"),
                "targetReadPrecision": summary.get("targetReadPrecision"),
                "irrelevantReadRate": summary.get("irrelevantReadRate"),
                "forbiddenCommandCount": summary.get("forbiddenCommandCount"),
                "clientFailureCount": summary.get("clientFailureCount"),
                "rateLimitCount": summary.get("rateLimitCount"),
                "missingRequiredCtxhelmCallCount": summary.get("missingRequiredCtxhelmCallCount"),
                "invalidRequiredCtxhelmCallCount": summary.get("invalidRequiredCtxhelmCallCount"),
                "ctxhelmEvidenceMissedTargetCount": summary.get("ctxhelmEvidenceMissedTargetCount"),
                "ctxhelmEvidenceOnlyTargetCount": summary.get("ctxhelmEvidenceOnlyTargetCount"),
            }
        )
    return quality


def task_lane_summary(tasks: list) -> dict:
    task_lane_count = 0
    ctxhelm_task_lane_count = 0
    for task in tasks:
        if not isinstance(task, dict):
            continue
        for lane in task.get("lanes", []):
            if not isinstance(lane, dict):
                continue
            task_lane_count += 1
            if str(lane.get("lane", "")).startswith("ctxhelm-"):
                ctxhelm_task_lane_count += 1
    return {
        "strictTaskLaneChecks": True,
        "taskLaneCount": task_lane_count,
        "ctxhelmTaskLaneCount": ctxhelm_task_lane_count,
    }


def suite_task_check_summary(tasks: list, current_suite_specs: list | None) -> dict:
    summary = {
        "reportTaskCount": len(tasks),
        "strictCurrentSuiteTaskChecks": current_suite_specs is not None,
    }
    if current_suite_specs is not None:
        summary["currentSuiteTaskCount"] = len(current_suite_specs)
        summary["matchesCurrentSuiteTasks"] = True
    return summary


def derived_suite_status(tasks: list) -> tuple[str, int, bool]:
    comparison_eligible_count = 0
    boundary_observed = False
    for task_index, task in enumerate(tasks):
        task = require_dict(task, f"tasks[{task_index}]")
        comparison = require_dict(task.get("comparison"), f"tasks[{task_index}].comparison")
        if comparison.get("comparisonEligible") is True:
            comparison_eligible_count += 1
        for field in STRICT_FALSE_OUTCOME_FIELDS:
            if comparison.get(field) is True:
                boundary_observed = True

    task_count = len(tasks)
    if boundary_observed or (task_count and comparison_eligible_count == 0):
        return "degraded", comparison_eligible_count, boundary_observed
    if any(isinstance(task, dict) and task.get("status") == "passed" for task in tasks):
        return "passed", comparison_eligible_count, boundary_observed
    return "skipped", comparison_eligible_count, boundary_observed


def validate_suite_consistency(report: dict, suite: dict, tasks: list) -> dict:
    suite_task_count = require_int(suite.get("taskCount"), "suite.taskCount")
    derived_task_count = len(tasks)
    if suite_task_count != derived_task_count:
        fail(
            "suite.taskCount did not match derived tasks: "
            f"{suite_task_count} != {derived_task_count}"
        )

    expected_status, comparison_eligible_count, boundary_observed = derived_suite_status(tasks)
    actual_status = report.get("status")
    if actual_status != expected_status:
        fail(
            "report.status did not match derived suite status: "
            f"{actual_status} != {expected_status}"
        )

    return {
        "strictSuiteStatusChecks": True,
        "suiteTaskCount": suite_task_count,
        "derivedTaskCount": derived_task_count,
        "derivedComparisonEligibleCount": comparison_eligible_count,
        "derivedBoundaryObserved": boundary_observed,
        "derivedStatus": expected_status,
        "matchesDerivedTaskCount": True,
        "matchesDerivedStatus": True,
    }


def derived_aggregate_consistency(aggregate: dict, summaries: list, tasks: list) -> dict:
    comparison_eligible_count = 0
    comparable_ctxhelm_lane_count = 0
    derived_boundary_fields = {field: False for field in STRICT_FALSE_OUTCOME_FIELDS}
    task_lane_names: set[str] = set()

    for task_index, task in enumerate(tasks):
        task = require_dict(task, f"tasks[{task_index}]")
        comparison = require_dict(task.get("comparison"), f"tasks[{task_index}].comparison")
        if comparison.get("comparisonEligible") is True:
            comparison_eligible_count += 1
        comparable_ctxhelm_lane_count += require_int(
            comparison.get("comparableCtxhelmLaneCount", 0),
            f"tasks[{task_index}].comparison.comparableCtxhelmLaneCount",
        )
        for field in STRICT_FALSE_OUTCOME_FIELDS:
            if comparison.get(field) is True:
                derived_boundary_fields[field] = True
        lanes = require_list(task.get("lanes"), f"tasks[{task_index}].lanes")
        for lane_index, lane in enumerate(lanes):
            lane = require_dict(lane, f"tasks[{task_index}].lanes[{lane_index}]")
            lane_name = lane.get("lane")
            if not isinstance(lane_name, str) or not lane_name:
                fail(f"tasks[{task_index}].lanes[{lane_index}].lane was missing")
            task_lane_names.add(lane_name)

    summary_lane_names: set[str] = set()
    for index, summary in enumerate(summaries):
        summary = require_dict(summary, f"aggregate.laneSummaries[{index}]")
        lane_name = summary.get("lane")
        if not isinstance(lane_name, str) or not lane_name:
            fail(f"aggregate.laneSummaries[{index}].lane was missing")
        summary_lane_names.add(lane_name)

    return {
        "taskCount": len(tasks),
        "comparisonEligibleCount": comparison_eligible_count,
        "comparableCtxhelmLaneCount": comparable_ctxhelm_lane_count,
        "boundaryFields": derived_boundary_fields,
        "taskLaneNames": sorted(task_lane_names),
        "summaryLaneNames": sorted(summary_lane_names),
    }


def sum_role_counts(lanes: list, field: str) -> dict:
    counts: dict[str, int] = {}
    for lane in lanes:
        role_counts = lane.get(field)
        if role_counts is None:
            continue
        role_counts = require_dict(role_counts, f"lane.{field}")
        for role, count in role_counts.items():
            if not isinstance(role, str) or not role:
                fail(f"lane.{field} contained an invalid role")
            counts[role] = counts.get(role, 0) + require_int(
                count,
                f"lane.{field}.{role}",
            )
    return dict(sorted(counts.items()))


def derived_lane_summary(lane_name: str, lanes: list) -> dict:
    count_sums = {field: 0 for field in LANE_SUMMARY_COUNT_FIELDS}
    target_read_coverage_sum = 0.0
    target_coverage_sum = 0.0
    passed_count = 0
    evaluation_eligible_count = 0
    client_failure_count = 0
    rate_limit_count = 0

    for lane_index, lane in enumerate(lanes):
        metrics = require_dict(
            lane.get("metrics"),
            f"derivedLane[{lane_name}][{lane_index}].metrics",
        )
        for field in LANE_SUMMARY_COUNT_FIELDS:
            count_sums[field] += require_int(
                metrics.get(field, 0),
                f"derivedLane[{lane_name}][{lane_index}].metrics.{field}",
            )
        target_read_coverage_sum += require_number(
            metrics.get("targetReadCoverage"),
            f"derivedLane[{lane_name}][{lane_index}].metrics.targetReadCoverage",
        )
        target_coverage_sum += require_number(
            metrics.get("targetCoverage"),
            f"derivedLane[{lane_name}][{lane_index}].metrics.targetCoverage",
        )
        if lane.get("status") == "passed":
            passed_count += 1
        if lane.get("evaluationEligible") is True:
            evaluation_eligible_count += 1
        if int(lane.get("clientExitStatus", 0)) != 0 or lane.get("clientFailureKind") is not None:
            client_failure_count += 1
        if lane.get("clientApiErrorStatus") is not None:
            client_failure_count += 1
        if lane.get("rateLimitObserved") is True:
            rate_limit_count += 1

    task_count = len(lanes)
    read_file_count = count_sums["readFileCount"]
    target_read_count = count_sums["targetReadCount"]
    derived = {
        **count_sums,
        "toolCallCount": count_sums["commandExecutionCount"],
        "taskCount": task_count,
        "passedCount": passed_count,
        "evaluationEligibleCount": evaluation_eligible_count,
        "clientFailureCount": client_failure_count,
        "rateLimitCount": rate_limit_count,
        "averageTargetReadCoverage": safe_ratio(target_read_coverage_sum, task_count),
        "averageTargetCoverage": safe_ratio(target_coverage_sum, task_count),
        "targetReadPrecision": safe_ratio(target_read_count, read_file_count),
        "irrelevantReadRate": safe_ratio(count_sums["irrelevantReadCount"], read_file_count),
        "readsPerTargetRead": safe_ratio(read_file_count, target_read_count),
        "readRoleCounts": sum_role_counts(lanes, "readRoleCounts"),
        "missedTargetRoleCounts": sum_role_counts(lanes, "missedTargetRoleCounts"),
    }
    return derived


def validate_lane_summary_consistency(summaries: list, tasks: list) -> dict:
    lanes_by_name: dict[str, list] = {}
    for task_index, task in enumerate(tasks):
        task = require_dict(task, f"tasks[{task_index}]")
        lanes = require_list(task.get("lanes"), f"tasks[{task_index}].lanes")
        for lane_index, lane in enumerate(lanes):
            lane = require_dict(lane, f"tasks[{task_index}].lanes[{lane_index}]")
            lane_name = str(lane.get("lane", ""))
            lanes_by_name.setdefault(lane_name, []).append(lane)

    numeric_fields = (
        *LANE_SUMMARY_COUNT_FIELDS,
        "toolCallCount",
        "taskCount",
        "passedCount",
        "evaluationEligibleCount",
        "clientFailureCount",
        "rateLimitCount",
    )
    float_fields = (
        "averageTargetReadCoverage",
        "averageTargetCoverage",
        "targetReadPrecision",
        "irrelevantReadRate",
        "readsPerTargetRead",
    )
    map_fields = ("readRoleCounts", "missedTargetRoleCounts")

    checked_lane_count = 0
    checked_metric_count = 0
    for summary_index, summary in enumerate(summaries):
        summary = require_dict(summary, f"aggregate.laneSummaries[{summary_index}]")
        lane_name = str(summary.get("lane", ""))
        lanes = lanes_by_name.get(lane_name)
        if not lanes:
            fail(f"aggregate.laneSummaries[{summary_index}].lane had no derived task lanes")
        derived = derived_lane_summary(lane_name, lanes)
        checked_lane_count += 1
        for field in numeric_fields:
            if field not in summary:
                fail(f"aggregate.laneSummaries[{lane_name}].{field} was missing")
            actual = require_int(
                summary.get(field),
                f"aggregate.laneSummaries[{lane_name}].{field}",
            )
            if actual != derived[field]:
                fail(
                    f"aggregate.laneSummaries[{lane_name}].{field} did not match derived task lanes: "
                    f"{actual} != {derived[field]}"
                )
            checked_metric_count += 1
        for field in float_fields:
            if field not in summary:
                fail(f"aggregate.laneSummaries[{lane_name}].{field} was missing")
            if not numbers_match(summary.get(field), derived[field]):
                fail(
                    f"aggregate.laneSummaries[{lane_name}].{field} did not match derived task lanes: "
                    f"{summary.get(field)} != {derived[field]}"
                )
            checked_metric_count += 1
        for field in map_fields:
            actual = require_dict(
                summary.get(field),
                f"aggregate.laneSummaries[{lane_name}].{field}",
            )
            normalized_actual = {
                str(role): require_int(
                    count,
                    f"aggregate.laneSummaries[{lane_name}].{field}.{role}",
                )
                for role, count in actual.items()
            }
            normalized_actual = dict(sorted(normalized_actual.items()))
            if normalized_actual != derived[field]:
                fail(
                    f"aggregate.laneSummaries[{lane_name}].{field} did not match derived task lanes: "
                    f"{normalized_actual} != {derived[field]}"
                )
            checked_metric_count += 1

    return {
        "strictLaneSummaryMetricChecks": True,
        "checkedLaneSummaryCount": checked_lane_count,
        "checkedLaneSummaryMetricCount": checked_metric_count,
    }


def derived_retry_cost(tasks: list) -> dict:
    count_sums = {field: 0 for field in RETRY_COST_COUNT_FIELDS}
    weighted_sums = {field: 0.0 for field in RETRY_COST_AVERAGE_FIELDS}
    triggered_lanes = 0

    for task_index, task in enumerate(tasks):
        comparison = require_dict(
            require_dict(task, f"tasks[{task_index}]").get("comparison"),
            f"tasks[{task_index}].comparison",
        )
        retry_cost = require_dict(
            comparison.get("retryCost"),
            f"tasks[{task_index}].comparison.retryCost",
        )
        task_triggered = require_int(
            retry_cost.get("retryTriggeredLanes", 0),
            f"tasks[{task_index}].comparison.retryCost.retryTriggeredLanes",
        )
        triggered_lanes += task_triggered
        for field in RETRY_COST_COUNT_FIELDS:
            count_sums[field] += require_int(
                retry_cost.get(field, 0),
                f"tasks[{task_index}].comparison.retryCost.{field}",
            )
        for field in RETRY_COST_AVERAGE_FIELDS:
            weighted_sums[field] += (
                require_number(
                    retry_cost.get(field, 0.0),
                    f"tasks[{task_index}].comparison.retryCost.{field}",
                )
                * task_triggered
            )

    return {
        **count_sums,
        **{
            field: safe_ratio(weighted_sums[field], triggered_lanes)
            for field in RETRY_COST_AVERAGE_FIELDS
        },
    }


def validate_retry_cost_consistency(aggregate: dict, tasks: list) -> dict:
    retry_cost = require_dict(aggregate.get("retryCost"), "aggregate.retryCost")
    derived = derived_retry_cost(tasks)
    checked_metric_count = 0
    for field in RETRY_COST_COUNT_FIELDS:
        actual = require_int(retry_cost.get(field), f"aggregate.retryCost.{field}")
        if actual != derived[field]:
            fail(
                f"aggregate.retryCost.{field} did not match derived task retry costs: "
                f"{actual} != {derived[field]}"
            )
        checked_metric_count += 1
    for field in RETRY_COST_AVERAGE_FIELDS:
        if not numbers_match(retry_cost.get(field), derived[field]):
            fail(
                f"aggregate.retryCost.{field} did not match derived task retry costs: "
                f"{retry_cost.get(field)} != {derived[field]}"
            )
        checked_metric_count += 1
    return {
        "strictRetryCostConsistencyChecks": True,
        "checkedRetryCostMetricCount": checked_metric_count,
    }


def lane_summaries_by_name(summaries: list) -> dict:
    by_name = {}
    for index, summary in enumerate(summaries):
        summary = require_dict(summary, f"aggregate.laneSummaries[{index}]")
        lane_name = summary.get("lane")
        if not isinstance(lane_name, str) or not lane_name:
            fail(f"aggregate.laneSummaries[{index}].lane was missing")
        by_name[lane_name] = summary
    return by_name


def derived_read_efficiency(read_efficiency: dict, summaries: list) -> dict:
    summaries_by_name = lane_summaries_by_name(summaries)
    baseline_lane = read_efficiency.get("baselineLane")
    efficient_lane = read_efficiency.get("efficientCtxhelmLane")
    if baseline_lane not in summaries_by_name:
        fail(f"aggregate.readEfficiency.baselineLane had no lane summary: {baseline_lane}")
    if efficient_lane not in summaries_by_name:
        fail(
            f"aggregate.readEfficiency.efficientCtxhelmLane had no lane summary: "
            f"{efficient_lane}"
        )
    baseline = summaries_by_name[baseline_lane]
    efficient = summaries_by_name[efficient_lane]
    baseline_read_count = require_int(
        baseline.get("readFileCount"),
        f"aggregate.laneSummaries[{baseline_lane}].readFileCount",
    )
    efficient_read_count = require_int(
        efficient.get("readFileCount"),
        f"aggregate.laneSummaries[{efficient_lane}].readFileCount",
    )
    baseline_irrelevant_count = require_int(
        baseline.get("irrelevantReadCount"),
        f"aggregate.laneSummaries[{baseline_lane}].irrelevantReadCount",
    )
    efficient_irrelevant_count = require_int(
        efficient.get("irrelevantReadCount"),
        f"aggregate.laneSummaries[{efficient_lane}].irrelevantReadCount",
    )
    baseline_target_read_count = require_int(
        baseline.get("targetReadCount"),
        f"aggregate.laneSummaries[{baseline_lane}].targetReadCount",
    )
    efficient_target_read_count = require_int(
        efficient.get("targetReadCount"),
        f"aggregate.laneSummaries[{efficient_lane}].targetReadCount",
    )
    recovered_target_reads = efficient_target_read_count - baseline_target_read_count
    extra_reads = efficient_read_count - baseline_read_count
    extra_irrelevant_reads = efficient_irrelevant_count - baseline_irrelevant_count
    baseline_target_read_coverage = require_number(
        baseline.get("averageTargetReadCoverage"),
        f"aggregate.laneSummaries[{baseline_lane}].averageTargetReadCoverage",
    )
    efficient_target_read_coverage = require_number(
        efficient.get("averageTargetReadCoverage"),
        f"aggregate.laneSummaries[{efficient_lane}].averageTargetReadCoverage",
    )
    baseline_precision = require_number(
        baseline.get("targetReadPrecision"),
        f"aggregate.laneSummaries[{baseline_lane}].targetReadPrecision",
    )
    efficient_precision = require_number(
        efficient.get("targetReadPrecision"),
        f"aggregate.laneSummaries[{efficient_lane}].targetReadPrecision",
    )
    baseline_irrelevant_rate = require_number(
        baseline.get("irrelevantReadRate"),
        f"aggregate.laneSummaries[{baseline_lane}].irrelevantReadRate",
    )
    efficient_irrelevant_rate = require_number(
        efficient.get("irrelevantReadRate"),
        f"aggregate.laneSummaries[{efficient_lane}].irrelevantReadRate",
    )
    return {
        "baselineReadFileCount": baseline_read_count,
        "baselineIrrelevantReadCount": baseline_irrelevant_count,
        "baselineTargetReadCoverage": baseline_target_read_coverage,
        "baselineTargetReadPrecision": baseline_precision,
        "baselineIrrelevantReadRate": baseline_irrelevant_rate,
        "efficientReadFileCount": efficient_read_count,
        "efficientIrrelevantReadCount": efficient_irrelevant_count,
        "efficientTargetReadCoverage": efficient_target_read_coverage,
        "efficientTargetReadPrecision": efficient_precision,
        "efficientIrrelevantReadRate": efficient_irrelevant_rate,
        "recoveredTargetReadCount": recovered_target_reads,
        "extraReadFileCount": extra_reads,
        "extraIrrelevantReadCount": extra_irrelevant_reads,
        "targetReadCoverageDelta": efficient_target_read_coverage - baseline_target_read_coverage,
        "targetReadPrecisionDelta": efficient_precision - baseline_precision,
        "irrelevantReadRateDelta": efficient_irrelevant_rate - baseline_irrelevant_rate,
        "extraReadsPerRecoveredTarget": safe_ratio(extra_reads, recovered_target_reads),
        "extraIrrelevantReadsPerRecoveredTarget": safe_ratio(
            extra_irrelevant_reads,
            recovered_target_reads,
        ),
    }


def validate_read_efficiency_consistency(aggregate: dict, summaries: list) -> dict:
    read_efficiency = require_dict(aggregate.get("readEfficiency"), "aggregate.readEfficiency")
    if read_efficiency.get("analysisAvailable") is not True:
        fail("aggregate.readEfficiency.analysisAvailable was not true")
    derived = derived_read_efficiency(read_efficiency, summaries)
    int_fields = (
        "baselineReadFileCount",
        "baselineIrrelevantReadCount",
        "efficientReadFileCount",
        "efficientIrrelevantReadCount",
        "recoveredTargetReadCount",
        "extraReadFileCount",
        "extraIrrelevantReadCount",
    )
    float_fields = (
        "baselineTargetReadCoverage",
        "baselineTargetReadPrecision",
        "baselineIrrelevantReadRate",
        "efficientTargetReadCoverage",
        "efficientTargetReadPrecision",
        "efficientIrrelevantReadRate",
        "targetReadCoverageDelta",
        "targetReadPrecisionDelta",
        "irrelevantReadRateDelta",
        "extraReadsPerRecoveredTarget",
        "extraIrrelevantReadsPerRecoveredTarget",
    )
    checked_metric_count = 0
    for field in int_fields:
        actual = require_int(read_efficiency.get(field), f"aggregate.readEfficiency.{field}")
        if actual != derived[field]:
            fail(
                f"aggregate.readEfficiency.{field} did not match derived lane summaries: "
                f"{actual} != {derived[field]}"
            )
        checked_metric_count += 1
    for field in float_fields:
        if not numbers_match(read_efficiency.get(field), derived[field]):
            fail(
                f"aggregate.readEfficiency.{field} did not match derived lane summaries: "
                f"{read_efficiency.get(field)} != {derived[field]}"
            )
        checked_metric_count += 1
    return {
        "strictReadEfficiencyConsistencyChecks": True,
        "checkedReadEfficiencyMetricCount": checked_metric_count,
    }


def derived_comparison_aggregates(tasks: list) -> dict:
    sum_fields = {field: 0 for field in AGGREGATE_COMPARISON_SUM_FIELDS}
    average_sums = {field: 0.0 for field in AGGREGATE_COMPARISON_AVERAGE_FIELDS}
    ctxhelm_tool_calls_observed = False

    for task_index, task in enumerate(tasks):
        comparison = require_dict(
            require_dict(task, f"tasks[{task_index}]").get("comparison"),
            f"tasks[{task_index}].comparison",
        )
        for aggregate_field, task_field in AGGREGATE_COMPARISON_SUM_FIELDS.items():
            sum_fields[aggregate_field] += require_int(
                comparison.get(task_field, 0),
                f"tasks[{task_index}].comparison.{task_field}",
            )
        for aggregate_field, task_field in AGGREGATE_COMPARISON_AVERAGE_FIELDS.items():
            average_sums[aggregate_field] += require_number(
                comparison.get(task_field, 0.0),
                f"tasks[{task_index}].comparison.{task_field}",
            )
        if comparison.get("ctxhelmToolCallsObserved") is True:
            ctxhelm_tool_calls_observed = True

    return {
        **sum_fields,
        **{
            field: safe_ratio(total, len(tasks))
            for field, total in average_sums.items()
        },
        "ctxhelmToolCallsObserved": ctxhelm_tool_calls_observed,
    }


def validate_comparison_aggregate_consistency(aggregate: dict, tasks: list) -> dict:
    derived = derived_comparison_aggregates(tasks)
    checked_metric_count = 0
    for field in AGGREGATE_COMPARISON_SUM_FIELDS:
        actual = require_int(aggregate.get(field), f"aggregate.{field}")
        if actual != derived[field]:
            fail(
                f"aggregate.{field} did not match derived task comparisons: "
                f"{actual} != {derived[field]}"
            )
        checked_metric_count += 1
    for field in AGGREGATE_COMPARISON_AVERAGE_FIELDS:
        if not numbers_match(aggregate.get(field), derived[field]):
            fail(
                f"aggregate.{field} did not match derived task comparisons: "
                f"{aggregate.get(field)} != {derived[field]}"
            )
        checked_metric_count += 1
    if aggregate.get("ctxhelmToolCallsObserved") is not derived["ctxhelmToolCallsObserved"]:
        fail(
            "aggregate.ctxhelmToolCallsObserved did not match derived task comparisons: "
            f"{aggregate.get('ctxhelmToolCallsObserved')} != "
            f"{derived['ctxhelmToolCallsObserved']}"
        )
    checked_metric_count += 1
    return {
        "strictComparisonAggregateChecks": True,
        "checkedComparisonAggregateMetricCount": checked_metric_count,
    }


def derived_outcome_claim(
    task_count: int,
    comparison_eligible_count: int,
    comparison_aggregates: dict,
) -> str:
    target_read_delta_avg = comparison_aggregates["targetReadCoverageDeltaAverage"]
    target_delta_avg = comparison_aggregates["targetCoverageDeltaAverage"]
    irrelevant_delta_sum = comparison_aggregates["irrelevantReadDeltaSum"]
    ctxhelm_tool_calls_observed = comparison_aggregates["ctxhelmToolCallsObserved"]
    if task_count and comparison_eligible_count == 0:
        return "insufficient_comparable_lanes"
    if ctxhelm_tool_calls_observed and (
        target_delta_avg > 0
        or target_read_delta_avg > 0
        or irrelevant_delta_sum > 0
    ):
        return "ctxhelm_improved"
    if (
        ctxhelm_tool_calls_observed
        and target_delta_avg == 0
        and target_read_delta_avg == 0
        and irrelevant_delta_sum == 0
    ):
        return "ctxhelm_matched"
    return "no_measured_lift"


def derived_research_actions(
    comparison_eligible_count: int,
    outcome_claim: str,
    comparison_aggregates: dict,
    boundary_fields: dict,
) -> list:
    actions = []

    def add(action: str, priority: int, reason: str) -> None:
        actions.append({"action": action, "priority": priority, "reason": reason})

    ctxhelm_tool_calls_observed = comparison_aggregates["ctxhelmToolCallsObserved"]
    if boundary_fields["clientFailuresObserved"] or boundary_fields["rateLimitsObserved"]:
        add(
            "retry_real_client_when_available",
            1,
            "Client availability prevented comparable outcome proof.",
        )
    elif not ctxhelm_tool_calls_observed and not comparison_eligible_count:
        add(
            "collect_real_client_evidence",
            1,
            "No comparable real-client ctxhelm call evidence was collected.",
        )
    if (
        boundary_fields["missingRequiredCtxhelmCallsObserved"]
        or boundary_fields["invalidRequiredCtxhelmCallsObserved"]
    ) and not boundary_fields["clientFailuresObserved"] and ctxhelm_tool_calls_observed:
        add(
            "harden_required_ctxhelm_call_guidance",
            1,
            "A ctxhelm-assisted lane did not make all required source-free ctxhelm calls.",
        )
    if boundary_fields["ctxhelmEvidenceMissesObserved"]:
        add(
            "fix_retrieval_or_query_construction",
            1,
            "ctxhelm evidence did not surface at least one expected target.",
        )
    if (
        boundary_fields["ctxhelmEvidenceOnlyTargetsObserved"]
        and not boundary_fields["clientFailuresObserved"]
    ):
        add(
            "improve_agent_consumption_guidance",
            2,
            "ctxhelm surfaced expected targets that Codex did not consume with read-only commands.",
        )
    if (
        boundary_fields["ctxhelmUnderReadTargetsObserved"]
        and not boundary_fields["clientFailuresObserved"]
    ):
        add(
            "inspect_pack_ordering_and_native_read_instruction",
            2,
            "A ctxhelm-assisted lane under-read targets relative to the native baseline.",
        )
    if comparison_eligible_count and outcome_claim == "no_measured_lift":
        add(
            "analyze_native_baseline_gap",
            2,
            "Comparable lanes produced no measured ctxhelm lift.",
        )
    if (
        comparison_eligible_count
        and outcome_claim == "ctxhelm_improved"
        and not boundary_fields["clientFailuresObserved"]
        and (
            comparison_aggregates["irrelevantReadDeltaSum"] < 0
            or comparison_aggregates["readFileDeltaSum"] < 0
        )
    ):
        add(
            "optimize_agent_read_efficiency",
            2,
            "ctxhelm improved target consumption but required more reads or more irrelevant reads than the native baseline.",
        )
    if (
        not actions
        and comparison_eligible_count
        and outcome_claim in {"ctxhelm_improved", "ctxhelm_matched"}
    ):
        add(
            "preserve_current_agent_contract",
            3,
            "Comparable lanes produced stable source-free outcome evidence.",
        )
    return actions


def validate_outcome_and_action_consistency(
    aggregate: dict,
    derived: dict,
    comparison_aggregates: dict,
) -> dict:
    expected_outcome = derived_outcome_claim(
        derived["taskCount"],
        derived["comparisonEligibleCount"],
        comparison_aggregates,
    )
    if aggregate.get("outcomeClaim") != expected_outcome:
        fail(
            "aggregate.outcomeClaim did not match derived task comparisons: "
            f"{aggregate.get('outcomeClaim')} != {expected_outcome}"
        )
    expected_actions = derived_research_actions(
        derived["comparisonEligibleCount"],
        expected_outcome,
        comparison_aggregates,
        derived["boundaryFields"],
    )
    actual_actions = require_list(
        aggregate.get("recommendedResearchActions"),
        "aggregate.recommendedResearchActions",
    )
    if actual_actions != expected_actions:
        fail(
            "aggregate.recommendedResearchActions did not match derived outcome routing: "
            f"{actual_actions} != {expected_actions}"
        )
    return {
        "strictOutcomeRoutingChecks": True,
        "derivedOutcomeClaim": expected_outcome,
        "checkedRecommendedResearchActionCount": len(expected_actions),
    }


def validate_aggregate_consistency(aggregate: dict, summaries: list, tasks: list) -> dict:
    derived = derived_aggregate_consistency(aggregate, summaries, tasks)
    for field in ("taskCount", "comparisonEligibleCount", "comparableCtxhelmLaneCount"):
        actual = require_int(aggregate.get(field), f"aggregate.{field}")
        if actual != derived[field]:
            fail(
                f"aggregate.{field} did not match derived task comparisons: "
                f"{actual} != {derived[field]}"
            )

    for field in STRICT_FALSE_OUTCOME_FIELDS:
        if field not in aggregate:
            fail(f"aggregate.{field} was missing")
        actual = aggregate.get(field)
        expected = derived["boundaryFields"][field]
        if actual is not expected:
            fail(
                f"aggregate.{field} did not match derived task comparisons: "
                f"{actual} != {expected}"
            )

    if derived["summaryLaneNames"] != derived["taskLaneNames"]:
        fail(
            "aggregate.laneSummaries lane names did not match derived task lanes: "
            f"{derived['summaryLaneNames']} != {derived['taskLaneNames']}"
        )

    lane_summary_metrics = validate_lane_summary_consistency(summaries, tasks)
    retry_cost_metrics = validate_retry_cost_consistency(aggregate, tasks)
    read_efficiency_metrics = validate_read_efficiency_consistency(aggregate, summaries)
    comparison_aggregates = derived_comparison_aggregates(tasks)
    comparison_aggregate_metrics = validate_comparison_aggregate_consistency(aggregate, tasks)
    outcome_routing_metrics = validate_outcome_and_action_consistency(
        aggregate,
        derived,
        comparison_aggregates,
    )
    return {
        "strictAggregateConsistencyChecks": True,
        "derivedTaskCount": derived["taskCount"],
        "derivedComparisonEligibleCount": derived["comparisonEligibleCount"],
        "derivedComparableCtxhelmLaneCount": derived["comparableCtxhelmLaneCount"],
        "derivedLaneNameCount": len(derived["taskLaneNames"]),
        "laneSummaryCount": len(derived["summaryLaneNames"]),
        **lane_summary_metrics,
        **retry_cost_metrics,
        **read_efficiency_metrics,
        **comparison_aggregate_metrics,
        **outcome_routing_metrics,
        "matchesDerivedAggregates": True,
    }


def validate_suite(report: dict, args: argparse.Namespace) -> dict:
    if report.get("workflowKind") != "paired-agent-context-suite":
        fail("workflowKind was not paired-agent-context-suite")
    validate_common_report(report, args, "report")
    suite = require_dict(report.get("suite"), "suite")
    if suite.get("rawTasksStored") is not False:
        fail("suite.rawTasksStored was not false")
    validate_suite_fingerprint(suite, args)
    task_count = int(suite.get("taskCount", report.get("aggregate", {}).get("taskCount", 0)))
    if task_count < args.min_task_count:
        fail(f"suite taskCount {task_count} < {args.min_task_count}")
    aggregate = require_dict(report.get("aggregate"), "aggregate")
    validate_common_outcome_fields(aggregate, args, "aggregate")
    comparison_eligible = int(aggregate.get("comparisonEligibleCount", 0))
    if comparison_eligible < args.min_comparison_eligible:
        fail(
            f"aggregate.comparisonEligibleCount {comparison_eligible} "
            f"< {args.min_comparison_eligible}"
        )
    summaries = require_list(aggregate.get("laneSummaries"), "aggregate.laneSummaries")
    validate_lane_summaries(
        summaries,
        args.min_ctxhelm_target_read_coverage,
    )
    tasks = require_list(report.get("tasks"), "tasks")
    current_suite_specs = validate_tasks_against_current_suite(tasks, args)
    for index, task in enumerate(tasks):
        task = require_dict(task, f"tasks[{index}]")
        if task.get("status") != "passed":
            fail(f"tasks[{index}].status was not passed")
        if task.get("targetFiles") is None:
            fail(f"tasks[{index}].targetFiles was missing")
        if task.get("taskSha256") is None:
            fail(f"tasks[{index}].taskSha256 was missing")
        validate_privacy(task, f"tasks[{index}]")
        validate_task_comparison(
            require_dict(task.get("comparison"), f"tasks[{index}].comparison"),
            args,
            f"tasks[{index}].comparison",
        )
        validate_task_lanes(task, args, f"tasks[{index}]")
    aggregate_consistency = validate_aggregate_consistency(aggregate, summaries, tasks)
    suite_consistency = validate_suite_consistency(report, suite, tasks)
    return {
        "schemaVersion": "ctxhelm-agent-run-proof-check-v1",
        "status": "passed",
        "workflow": "suite",
        "reportFileName": args.report.name,
        "reportSha256": report_digest(args.report),
        "thresholds": proof_thresholds(args),
        "sourceFree": True,
        "identity": identity_summary(report, args),
        "privacyStatus": privacy_summary(report),
        "privacyChecks": privacy_checks(report),
        "runner": runner_summary(report, args),
        "suite": suite_summary(suite, args),
        "suiteTaskChecks": suite_task_check_summary(tasks, current_suite_specs),
        "suiteConsistency": suite_consistency,
        "taskLaneChecks": task_lane_summary(tasks),
        "aggregateConsistency": aggregate_consistency,
        "metrics": {
            "taskCount": task_count,
            "comparisonEligibleCount": comparison_eligible,
            "comparableCtxhelmLaneCount": aggregate.get("comparableCtxhelmLaneCount"),
            "outcomeClaim": aggregate.get("outcomeClaim"),
            "targetReadCoverageDeltaAverage": aggregate.get("targetReadCoverageDeltaAverage"),
            "targetCoverageDeltaAverage": aggregate.get("targetCoverageDeltaAverage"),
            "readFileDeltaSum": aggregate.get("readFileDeltaSum"),
            "irrelevantReadDeltaSum": aggregate.get("irrelevantReadDeltaSum"),
            "commandExecutionDeltaSum": aggregate.get("commandExecutionDeltaSum"),
            "ctxhelmToolCallsObserved": aggregate.get("ctxhelmToolCallsObserved"),
            "retryCost": aggregate.get("retryCost"),
            "readEfficiency": aggregate.get("readEfficiency"),
        },
        "boundaryStatus": boundary_status(aggregate),
        "boundaryChecks": boundary_summary(aggregate),
        "laneSummaries": lane_quality_summary(summaries),
    }


def validate_run(report: dict, args: argparse.Namespace) -> dict:
    if report.get("workflowKind") != "paired-agent-context-run":
        fail("workflowKind was not paired-agent-context-run")
    validate_common_report(report, args, "report")
    comparison = require_dict(report.get("comparison"), "comparison")
    validate_common_outcome_fields(comparison, args, "comparison")
    if comparison.get("comparisonEligible") is not True:
        fail("comparison.comparisonEligible was not true")
    if args.min_comparison_eligible > 1:
        fail("single-run report cannot satisfy min comparison eligible > 1")
    lanes = require_list(report.get("lanes"), "lanes")
    ctxhelm_lane_count = len(
        [
            lane
            for lane in lanes
            if isinstance(lane, dict) and str(lane.get("lane", "")).startswith("ctxhelm-")
        ]
    )
    if ctxhelm_lane_count < (args.min_comparable_ctxhelm_lanes or 0):
        fail(
            f"lanes had too few ctxhelm lanes: "
            f"{ctxhelm_lane_count} < {args.min_comparable_ctxhelm_lanes}"
        )
    if args.min_ctxhelm_target_read_coverage is not None:
        for lane in lanes:
            if not isinstance(lane, dict) or not str(lane.get("lane", "")).startswith("ctxhelm-"):
                continue
            metrics = require_dict(lane.get("metrics"), f"lanes[{lane.get('lane')}].metrics")
            coverage = require_number(
                metrics.get("targetReadCoverage"),
                f"lanes[{lane.get('lane')}].metrics.targetReadCoverage",
            )
            if coverage < args.min_ctxhelm_target_read_coverage:
                fail(
                    f"lanes[{lane.get('lane')}].metrics.targetReadCoverage "
                    f"{coverage} < {args.min_ctxhelm_target_read_coverage}"
                )
    return {
        "schemaVersion": "ctxhelm-agent-run-proof-check-v1",
        "status": "passed",
        "workflow": "run",
        "reportFileName": args.report.name,
        "reportSha256": report_digest(args.report),
        "thresholds": proof_thresholds(args),
        "sourceFree": True,
        "identity": identity_summary(report, args),
        "privacyStatus": privacy_summary(report),
        "privacyChecks": privacy_checks(report),
        "runner": runner_summary(report, args),
        "metrics": {
            "comparisonEligibleCount": 1,
            "comparableCtxhelmLaneCount": comparison.get("comparableCtxhelmLaneCount"),
            "outcomeClaim": comparison.get("outcomeClaim"),
            "targetReadCoverageDelta": comparison.get("targetReadCoverageDelta"),
            "targetCoverageDelta": comparison.get("targetCoverageDelta"),
            "readFileDelta": comparison.get("readFileDelta"),
            "irrelevantReadDelta": comparison.get("irrelevantReadDelta"),
            "retryCost": comparison.get("retryCost"),
            "readEfficiency": comparison.get("readEfficiency"),
        },
        "boundaryStatus": boundary_status(comparison),
        "boundaryChecks": boundary_summary(comparison),
        "laneSummaries": [],
    }


def render_summary(summary: dict, args: argparse.Namespace) -> str:
    if args.format == "json":
        return json.dumps(summary, indent=2, sort_keys=True) + "\n"
    metrics = summary["metrics"]
    task_text = (
        f"tasks={metrics.get('taskCount')} "
        if summary["workflow"] == "suite"
        else ""
    )
    return (
        "agent-run proof passed: "
        f"workflow={summary['workflow']} {task_text}"
        f"comparable={metrics.get('comparisonEligibleCount')} "
        f"ctxhelm_lanes={metrics.get('comparableCtxhelmLaneCount')} "
        f"outcome={metrics.get('outcomeClaim')}"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path, help="Source-free agent-run JSON report")
    parser.add_argument("--workflow", choices=("auto", "suite", "run"), default="auto")
    parser.add_argument("--require-status", default="passed")
    parser.add_argument("--require-outcome")
    parser.add_argument("--expected-ctxhelm-version")
    parser.add_argument("--expected-client-name")
    parser.add_argument("--expected-client-version")
    parser.add_argument("--min-task-count", type=int, default=0)
    parser.add_argument("--min-comparison-eligible", type=int, default=0)
    parser.add_argument("--min-comparable-ctxhelm-lanes", type=int)
    parser.add_argument("--min-ctxhelm-target-read-coverage", type=float)
    parser.add_argument("--max-extra-read-delta", type=int)
    parser.add_argument("--min-irrelevant-read-delta", type=int)
    parser.add_argument("--require-retry-cost", action="store_true")
    parser.add_argument("--require-runner-fingerprint", action="store_true")
    parser.add_argument(
        "--current-runner-script",
        type=pathlib.Path,
        help="Require runner.scriptSha256 to match this current local runner script.",
    )
    parser.add_argument(
        "--current-suite",
        type=pathlib.Path,
        help="Require suite.suiteSha256 to match this current local suite file.",
    )
    parser.add_argument("--format", choices=("text", "json"), default="text")
    parser.add_argument("--output", type=pathlib.Path, help="Write rendered proof check output here")
    parser.add_argument(
        "--allow-degraded-boundaries",
        action="store_false",
        dest="strict",
        help="Allow client failures, rate limits, forbidden commands, or evidence gaps.",
    )
    parser.set_defaults(strict=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if not args.report.is_file():
        fail(f"missing agent-run report: {args.report}")
    if args.current_runner_script is not None and not args.current_runner_script.is_file():
        fail(f"missing current runner script: {args.current_runner_script}")
    if args.current_suite is not None and not args.current_suite.is_file():
        fail(f"missing current suite: {args.current_suite}")
    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON: {error}")
    report = require_dict(report, "report")
    workflow = report.get("workflowKind")
    if args.workflow == "suite" or (args.workflow == "auto" and workflow == "paired-agent-context-suite"):
        summary = validate_suite(report, args)
    elif args.workflow == "run" or (args.workflow == "auto" and workflow == "paired-agent-context-run"):
        summary = validate_run(report, args)
    else:
        fail(f"unsupported workflowKind: {workflow}")
    rendered = render_summary(summary, args)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(1)
