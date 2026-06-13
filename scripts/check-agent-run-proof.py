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


def proof_thresholds(args: argparse.Namespace) -> dict:
    return {
        "requiredStatus": args.require_status,
        "requiredOutcome": args.require_outcome,
        "minTaskCount": args.min_task_count,
        "minComparisonEligible": args.min_comparison_eligible,
        "minComparableCtxhelmLanes": args.min_comparable_ctxhelm_lanes,
        "minCtxhelmTargetReadCoverage": args.min_ctxhelm_target_read_coverage,
        "maxExtraReadDelta": args.max_extra_read_delta,
        "minIrrelevantReadDelta": args.min_irrelevant_read_delta,
        "requireRetryCost": args.require_retry_cost,
        "requireRunnerFingerprint": args.require_runner_fingerprint,
        "strictBoundaries": args.strict,
    }


def report_digest(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def runner_summary(report: dict) -> dict:
    runner = report.get("runner")
    if not isinstance(runner, dict):
        return {"present": False}
    return {
        "present": True,
        "name": runner.get("name"),
        "contractVersion": runner.get("contractVersion"),
        "checkpointValidation": runner.get("checkpointValidation"),
        "scriptSha256": runner.get("scriptSha256"),
    }


def boundary_summary(outcome: dict) -> dict:
    return {field: outcome.get(field) is False for field in STRICT_FALSE_OUTCOME_FIELDS}


def privacy_summary(report: dict) -> dict:
    privacy = require_dict(report.get("privacyStatus"), "privacyStatus")
    return {
        "localOnly": privacy.get("localOnly") is True,
        **{field: privacy.get(field) is False for field in STRICT_FALSE_PRIVACY_FIELDS},
    }


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


def validate_suite(report: dict, args: argparse.Namespace) -> dict:
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
    return {
        "schemaVersion": "ctxhelm-agent-run-proof-check-v1",
        "status": "passed",
        "workflow": "suite",
        "reportFileName": args.report.name,
        "reportSha256": report_digest(args.report),
        "thresholds": proof_thresholds(args),
        "sourceFree": True,
        "privacyStatus": privacy_summary(report),
        "runner": runner_summary(report),
        "metrics": {
            "taskCount": task_count,
            "comparisonEligibleCount": comparison_eligible,
            "comparableCtxhelmLaneCount": aggregate.get("comparableCtxhelmLaneCount"),
            "outcomeClaim": aggregate.get("outcomeClaim"),
            "targetReadCoverageDeltaAvg": aggregate.get("targetReadCoverageDeltaAvg"),
            "targetCoverageDeltaAvg": aggregate.get("targetCoverageDeltaAvg"),
            "readFileDeltaSum": aggregate.get("readFileDeltaSum"),
            "irrelevantReadDeltaSum": aggregate.get("irrelevantReadDeltaSum"),
            "retryCost": aggregate.get("retryCost"),
            "readEfficiency": aggregate.get("readEfficiency"),
        },
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
        "privacyStatus": privacy_summary(report),
        "runner": runner_summary(report),
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
    parser.add_argument("--min-task-count", type=int, default=0)
    parser.add_argument("--min-comparison-eligible", type=int, default=0)
    parser.add_argument("--min-comparable-ctxhelm-lanes", type=int)
    parser.add_argument("--min-ctxhelm-target-read-coverage", type=float)
    parser.add_argument("--max-extra-read-delta", type=int)
    parser.add_argument("--min-irrelevant-read-delta", type=int)
    parser.add_argument("--require-retry-cost", action="store_true")
    parser.add_argument("--require-runner-fingerprint", action="store_true")
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
