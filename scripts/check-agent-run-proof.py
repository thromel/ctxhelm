#!/usr/bin/env python3
"""Validate source-free paired agent-run proof JSON for release-gate use."""

import argparse
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


def validate_privacy(report: dict, label: str) -> None:
    privacy = require_dict(report.get("privacyStatus"), f"{label}.privacyStatus")
    if privacy.get("localOnly") is not True:
        fail(f"{label}.privacyStatus.localOnly was not true")
    for field in STRICT_FALSE_PRIVACY_FIELDS:
        if privacy.get(field) is not False:
            fail(f"{label}.privacyStatus.{field} was not false")


def validate_runner(report: dict, label: str, require_runner: bool) -> None:
    runner = report.get("runner")
    if runner is None:
        if require_runner:
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
    validate_privacy(report, label)
    validate_runner(report, label, args.require_runner_fingerprint)


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


def validate_suite(report: dict, args: argparse.Namespace) -> str:
    if report.get("workflowKind") != "paired-agent-context-suite":
        fail("workflowKind was not paired-agent-context-suite")
    validate_common_report(report, args, "report")
    suite = require_dict(report.get("suite"), "suite")
    if suite.get("rawTasksStored") is not False:
        fail("suite.rawTasksStored was not false")
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
    for index, task in enumerate(require_list(report.get("tasks"), "tasks")):
        task = require_dict(task, f"tasks[{index}]")
        if task.get("status") != "passed":
            fail(f"tasks[{index}].status was not passed")
        if task.get("targetFiles") is None:
            fail(f"tasks[{index}].targetFiles was missing")
        if task.get("taskSha256") is None:
            fail(f"tasks[{index}].taskSha256 was missing")
        validate_privacy(task, f"tasks[{index}]")
    return (
        "agent-run proof passed: "
        f"workflow=suite tasks={task_count} comparable={comparison_eligible} "
        f"ctxhelm_lanes={aggregate.get('comparableCtxhelmLaneCount')} "
        f"outcome={aggregate.get('outcomeClaim')}"
    )


def validate_run(report: dict, args: argparse.Namespace) -> str:
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
    return (
        "agent-run proof passed: "
        f"workflow=run comparable=1 ctxhelm_lanes={ctxhelm_lane_count} "
        f"outcome={comparison.get('outcomeClaim')}"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path, help="Source-free agent-run JSON report")
    parser.add_argument("--workflow", choices=("auto", "suite", "run"), default="auto")
    parser.add_argument("--require-status", default="passed")
    parser.add_argument("--require-outcome")
    parser.add_argument("--min-task-count", type=int, default=0)
    parser.add_argument("--min-comparison-eligible", type=int, default=0)
    parser.add_argument("--min-comparable-ctxhelm-lanes", type=int)
    parser.add_argument("--min-ctxhelm-target-read-coverage", type=float)
    parser.add_argument("--max-extra-read-delta", type=int)
    parser.add_argument("--min-irrelevant-read-delta", type=int)
    parser.add_argument("--require-retry-cost", action="store_true")
    parser.add_argument("--require-runner-fingerprint", action="store_true")
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
    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON: {error}")
    report = require_dict(report, "report")
    workflow = report.get("workflowKind")
    if args.workflow == "suite" or (args.workflow == "auto" and workflow == "paired-agent-context-suite"):
        print(validate_suite(report, args))
    elif args.workflow == "run" or (args.workflow == "auto" and workflow == "paired-agent-context-run"):
        print(validate_run(report, args))
    else:
        fail(f"unsupported workflowKind: {workflow}")


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(1)
