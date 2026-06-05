#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: e2e-agent-run-codex.sh --target-file PATH [--target-file PATH ...] [--repo PATH] [--task TASK] [--output PATH]
       e2e-agent-run-codex.sh --suite SUITE.json [--repo PATH] [--output PATH]

Runs a source-free paired Codex CLI agent-run evaluation:
  1. baseline native read-only shell exploration
  2. ctxhelm prepare_task-assisted exploration
  3. ctxhelm prepare_task + brief get_pack-assisted exploration
  4. ctxhelm prepare_task + standard get_pack-assisted exploration
  5. ctxhelm prepare_task + standard get_pack with memory-consumption guidance

Real Codex execution is optional. Set CTXHELM_RUN_REAL_CLIENT=1 to run the
client. Without it, the script writes a skipped source-free report that preserves
the contract and does not pretend outcome proof exists.

With --suite, the script runs the same paired Codex evaluation for each task in
a source-free suite and writes aggregate native-vs-ctxhelm metrics. Suite files
may be either an array of task objects or an object with a "tasks" array. Each
task needs "task" or "prompt" plus "targetFiles" or "target_files".

The script does not edit source files, run user project tests, mutate global
agent configuration, publish releases, upload data, or store raw prompts/source
text/transcripts/MCP traffic/command output in the report.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
repo_input="${CTXHELM_AGENT_RUN_REPO:-$PWD}"
task="${CTXHELM_AGENT_RUN_TASK:-}"
output_path="${CTXHELM_AGENT_RUN_REPORT:-}"
run_real="${CTXHELM_RUN_REAL_CLIENT:-0}"
require_real="${CTXHELM_REQUIRE_REAL_CLIENT:-0}"
client_timeout_seconds="${CTXHELM_AGENT_RUN_TIMEOUT_SECONDS:-120}"
suite_path=""
target_files=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo_input="${2:-}"
      shift 2
      ;;
    --task)
      task="${2:-}"
      shift 2
      ;;
    --target-file)
      target_files+=("${2:-}")
      shift 2
      ;;
    --output)
      output_path="${2:-}"
      shift 2
      ;;
    --suite)
      suite_path="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 64
      ;;
  esac
done

if [[ -n "$suite_path" && "${#target_files[@]}" -gt 0 ]]; then
  echo "--suite cannot be combined with --target-file" >&2
  exit 64
fi
if [[ -z "$suite_path" && "${#target_files[@]}" -eq 0 ]]; then
  usage
  exit 64
fi
if [[ -z "$task" ]]; then
  task="Identify the files relevant to the requested change and explain which files should be inspected first."
fi
if [[ ! -d "$repo_input" ]]; then
  echo "repo not found: $repo_input" >&2
  exit 66
fi

repo="$(cd "$repo_input" && pwd -P)"

resolve_ctxhelm_bin() {
  if [[ -n "${CTXHELM_BIN:-}" ]]; then
    if [[ ! "$CTXHELM_BIN" = /* || ! -x "$CTXHELM_BIN" ]]; then
      echo "CTXHELM_BIN must be an absolute executable path: $CTXHELM_BIN" >&2
      exit 64
    fi
    printf '%s\n' "$CTXHELM_BIN"
    return
  fi
  cargo build -p ctxhelm >/dev/null
  printf '%s/target/debug/ctxhelm\n' "$repo_root"
}

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

ctxhelm_bin="$(resolve_ctxhelm_bin)"
ctxhelm_version="$("$ctxhelm_bin" --version)"
client_version="unavailable"
if command -v codex >/dev/null 2>&1; then
  client_version="$(codex --version 2>&1 | head -n 1)"
fi

if [[ -n "$suite_path" ]]; then
  if [[ ! -f "$suite_path" ]]; then
    echo "suite not found: $suite_path" >&2
    exit 66
  fi
  suite_tasks_jsonl="$work_dir/suite-tasks.jsonl"
  suite_reports_jsonl="$work_dir/suite-reports.jsonl"
  python3 - "$suite_path" "$suite_tasks_jsonl" <<'PY'
import json
import pathlib
import sys

suite_path = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2])
payload = json.loads(suite_path.read_text(encoding="utf-8"))
tasks = payload.get("tasks") if isinstance(payload, dict) else payload
if not isinstance(tasks, list) or not tasks:
    raise SystemExit("suite must contain a non-empty tasks array")

rows = []
for index, task in enumerate(tasks, start=1):
    if not isinstance(task, dict):
        raise SystemExit(f"suite task {index} must be an object")
    task_text = task.get("task") or task.get("prompt")
    targets = task.get("targetFiles") or task.get("target_files")
    if not isinstance(task_text, str) or not task_text.strip():
        raise SystemExit(f"suite task {index} must include task or prompt")
    if not isinstance(targets, list) or not targets:
        raise SystemExit(f"suite task {index} must include targetFiles")
    target_strings = []
    for target in targets:
        if not isinstance(target, str) or not target.strip():
            raise SystemExit(f"suite task {index} has an invalid target file")
        target_strings.append(target)
    task_id = task.get("id") or task.get("name") or f"task-{index}"
    rows.append({
        "id": str(task_id),
        "task": task_text,
        "targetFiles": target_strings,
    })

out.write_text(
    "".join(json.dumps(row, sort_keys=True) + "\n" for row in rows),
    encoding="utf-8",
)
PY

  task_index=0
  while IFS= read -r suite_task; do
    task_index=$((task_index + 1))
    task_id="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["id"])' "$suite_task")"
    task_text="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["task"])' "$suite_task")"
    suite_targets=()
    while IFS= read -r suite_target; do
      suite_targets+=("$suite_target")
    done < <(python3 -c 'import json,sys; print("\n".join(json.loads(sys.argv[1])["targetFiles"]))' "$suite_task")
    task_report="$work_dir/suite-task-${task_index}.json"
    task_args=(--repo "$repo" --task "$task_text" --output "$task_report")
    for target in "${suite_targets[@]}"; do
      task_args+=(--target-file "$target")
    done
    CTXHELM_BIN="$ctxhelm_bin" "$script_dir/e2e-agent-run-codex.sh" "${task_args[@]}" >/dev/null
    python3 - "$task_id" "$task_report" "$suite_reports_jsonl" <<'PY'
import json
import pathlib
import sys

task_id, report_path, manifest_path = sys.argv[1:]
entry = {"taskId": task_id, "reportPath": report_path}
with pathlib.Path(manifest_path).open("a", encoding="utf-8") as handle:
    handle.write(json.dumps(entry, sort_keys=True) + "\n")
PY
  done <"$suite_tasks_jsonl"

  python3 - "$suite_path" "$suite_reports_jsonl" "$repo" "$ctxhelm_version" "$client_version" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys
from collections import defaultdict

suite_path, manifest_path, repo, ctxhelm_version, client_version, output_path = sys.argv[1:]
entries = [
    json.loads(line)
    for line in pathlib.Path(manifest_path).read_text(encoding="utf-8").splitlines()
    if line.strip()
]
tasks = []
lane_totals = defaultdict(lambda: {
    "taskCount": 0,
    "passedCount": 0,
    "evaluationEligibleCount": 0,
    "targetCoverageSum": 0.0,
    "targetReadCoverageSum": 0.0,
    "readFileCount": 0,
    "irrelevantReadCount": 0,
    "targetReadCount": 0,
    "targetDiscoveredOnlyCount": 0,
    "missedTargetCount": 0,
    "commandExecutionCount": 0,
    "ctxhelmToolCallCount": 0,
    "forbiddenCommandCount": 0,
    "requiredCtxhelmCallCount": 0,
    "observedRequiredCtxhelmCallCount": 0,
    "missingRequiredCtxhelmCallCount": 0,
    "invalidRequiredCtxhelmCallCount": 0,
    "clientFailureCount": 0,
    "rateLimitCount": 0,
    "ctxhelmEvidenceFileCount": 0,
    "ctxhelmEvidenceTargetHitCount": 0,
    "ctxhelmEvidenceOnlyTargetCount": 0,
    "ctxhelmEvidenceMissedTargetCount": 0,
    "readRoleCounts": defaultdict(int),
    "missedTargetRoleCounts": defaultdict(int),
})
comparison = {
    "ctxhelmToolCallsObserved": False,
    "forbiddenCommandsObserved": False,
    "missingRequiredCtxhelmCallsObserved": False,
    "invalidRequiredCtxhelmCallsObserved": False,
    "clientFailuresObserved": False,
    "rateLimitsObserved": False,
    "ctxhelmEvidenceMissesObserved": False,
    "ctxhelmEvidenceOnlyTargetsObserved": False,
    "ctxhelmUnderReadTargetsObserved": False,
    "comparisonEligibleCount": 0,
    "comparableCtxhelmLaneCount": 0,
    "targetCoverageDeltaSum": 0.0,
    "targetReadCoverageDeltaSum": 0.0,
    "irrelevantReadDeltaSum": 0,
    "commandExecutionDeltaSum": 0,
    "readFileDeltaSum": 0,
}
privacy = {
    "localOnly": True,
    "remoteEmbeddingsUsed": False,
    "remoteRerankingUsed": False,
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
    "rawCommandOutputStored": False,
}

for entry in entries:
    report = json.loads(pathlib.Path(entry["reportPath"]).read_text(encoding="utf-8"))
    tasks.append({
        "taskId": entry["taskId"],
        "status": report.get("status", "unknown"),
        "taskSha256": report.get("task", {}).get("taskSha256"),
        "targetFiles": report.get("targetFiles", []),
        "comparison": report.get("comparison", {}),
        "lanes": report.get("lanes", []),
        "privacyStatus": report.get("privacyStatus", {}),
    })
    task_privacy = report.get("privacyStatus", {})
    task_has_executed_lane = any(
        lane.get("status") == "passed" for lane in report.get("lanes", [])
    )
    for key in [
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
        "sourceTextLogged",
        "rawPromptStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "rawCommandOutputStored",
    ]:
        privacy[key] = privacy[key] or bool(task_privacy.get(key, False))
    privacy["localOnly"] = privacy["localOnly"] and bool(task_privacy.get("localOnly", True))
    task_comparison = report.get("comparison", {})
    for key in [
        "ctxhelmToolCallsObserved",
        "clientFailuresObserved",
        "rateLimitsObserved",
        "ctxhelmEvidenceMissesObserved",
        "ctxhelmEvidenceOnlyTargetsObserved",
        "ctxhelmUnderReadTargetsObserved",
    ]:
        comparison[key] = comparison[key] or bool(task_comparison.get(key, False))
    if task_has_executed_lane:
        comparison["missingRequiredCtxhelmCallsObserved"] = (
            comparison["missingRequiredCtxhelmCallsObserved"]
            or bool(task_comparison.get("missingRequiredCtxhelmCallsObserved", False))
        )
        comparison["invalidRequiredCtxhelmCallsObserved"] = (
            comparison["invalidRequiredCtxhelmCallsObserved"]
            or bool(task_comparison.get("invalidRequiredCtxhelmCallsObserved", False))
        )
    comparison["forbiddenCommandsObserved"] = (
        comparison["forbiddenCommandsObserved"]
        or bool(task_comparison.get("forbiddenCommandsObserved", False))
    )
    comparison["comparisonEligibleCount"] += 1 if task_comparison.get("comparisonEligible", False) else 0
    comparison["comparableCtxhelmLaneCount"] += int(task_comparison.get("comparableCtxhelmLaneCount", 0) or 0)
    comparison["targetCoverageDeltaSum"] += float(task_comparison.get("targetCoverageDelta", 0.0) or 0.0)
    comparison["targetReadCoverageDeltaSum"] += float(task_comparison.get("targetReadCoverageDelta", 0.0) or 0.0)
    comparison["irrelevantReadDeltaSum"] += int(task_comparison.get("irrelevantReadDelta", 0) or 0)
    comparison["commandExecutionDeltaSum"] += int(task_comparison.get("commandExecutionDelta", 0) or 0)
    comparison["readFileDeltaSum"] += int(task_comparison.get("readFileDelta", 0) or 0)
    for lane in report.get("lanes", []):
        lane_id = lane.get("lane", "unknown")
        metrics = lane.get("metrics", {})
        bucket = lane_totals[lane_id]
        bucket["taskCount"] += 1
        bucket["passedCount"] += 1 if lane.get("status") == "passed" else 0
        bucket["evaluationEligibleCount"] += 1 if lane.get("evaluationEligible", False) else 0
        bucket["targetCoverageSum"] += float(metrics.get("targetCoverage", 0.0) or 0.0)
        bucket["targetReadCoverageSum"] += float(metrics.get("targetReadCoverage", 0.0) or 0.0)
        for key in [
            "readFileCount",
            "irrelevantReadCount",
            "targetReadCount",
            "targetDiscoveredOnlyCount",
            "missedTargetCount",
            "commandExecutionCount",
            "ctxhelmToolCallCount",
            "forbiddenCommandCount",
            "requiredCtxhelmCallCount",
            "observedRequiredCtxhelmCallCount",
            "missingRequiredCtxhelmCallCount",
            "invalidRequiredCtxhelmCallCount",
            "ctxhelmEvidenceFileCount",
            "ctxhelmEvidenceTargetHitCount",
            "ctxhelmEvidenceOnlyTargetCount",
            "ctxhelmEvidenceMissedTargetCount",
        ]:
            bucket[key] += int(metrics.get(key, 0) or 0)
        bucket["clientFailureCount"] += 1 if lane.get("clientFailureKind") else 0
        bucket["rateLimitCount"] += 1 if lane.get("rateLimitObserved", False) else 0
        for role, count in (lane.get("readRoleCounts") or {}).items():
            bucket["readRoleCounts"][role] += int(count or 0)
        for role, count in (lane.get("missedTargetRoleCounts") or {}).items():
            bucket["missedTargetRoleCounts"][role] += int(count or 0)

lane_summaries = []
for lane_id, bucket in sorted(lane_totals.items()):
    task_count = bucket["taskCount"]
    lane_summaries.append({
        "lane": lane_id,
        "taskCount": task_count,
        "passedCount": bucket["passedCount"],
        "evaluationEligibleCount": bucket["evaluationEligibleCount"],
        "averageTargetCoverage": bucket["targetCoverageSum"] / task_count if task_count else 0.0,
        "averageTargetReadCoverage": bucket["targetReadCoverageSum"] / task_count if task_count else 0.0,
        "readFileCount": bucket["readFileCount"],
        "irrelevantReadCount": bucket["irrelevantReadCount"],
        "targetReadCount": bucket["targetReadCount"],
        "targetDiscoveredOnlyCount": bucket["targetDiscoveredOnlyCount"],
        "missedTargetCount": bucket["missedTargetCount"],
        "commandExecutionCount": bucket["commandExecutionCount"],
        "toolCallCount": bucket["commandExecutionCount"],
        "ctxhelmToolCallCount": bucket["ctxhelmToolCallCount"],
        "forbiddenCommandCount": bucket["forbiddenCommandCount"],
        "requiredCtxhelmCallCount": bucket["requiredCtxhelmCallCount"],
        "observedRequiredCtxhelmCallCount": bucket["observedRequiredCtxhelmCallCount"],
        "missingRequiredCtxhelmCallCount": bucket["missingRequiredCtxhelmCallCount"],
        "invalidRequiredCtxhelmCallCount": bucket["invalidRequiredCtxhelmCallCount"],
        "clientFailureCount": bucket["clientFailureCount"],
        "rateLimitCount": bucket["rateLimitCount"],
        "ctxhelmEvidenceFileCount": bucket["ctxhelmEvidenceFileCount"],
        "ctxhelmEvidenceTargetHitCount": bucket["ctxhelmEvidenceTargetHitCount"],
        "ctxhelmEvidenceOnlyTargetCount": bucket["ctxhelmEvidenceOnlyTargetCount"],
        "ctxhelmEvidenceMissedTargetCount": bucket["ctxhelmEvidenceMissedTargetCount"],
        "readRoleCounts": dict(sorted(bucket["readRoleCounts"].items())),
        "missedTargetRoleCounts": dict(sorted(bucket["missedTargetRoleCounts"].items())),
    })

task_count = len(tasks)
target_delta_avg = comparison["targetCoverageDeltaSum"] / task_count if task_count else 0.0
target_read_delta_avg = comparison["targetReadCoverageDeltaSum"] / task_count if task_count else 0.0
irrelevant_delta_sum = comparison["irrelevantReadDeltaSum"]
comparison_eligible_count = comparison["comparisonEligibleCount"]
if task_count and comparison_eligible_count == 0:
    outcome_claim = "insufficient_comparable_lanes"
elif comparison["ctxhelmToolCallsObserved"] and (target_delta_avg > 0 or target_read_delta_avg > 0 or irrelevant_delta_sum > 0):
    outcome_claim = "ctxhelm_improved"
elif comparison["ctxhelmToolCallsObserved"] and target_delta_avg == 0 and target_read_delta_avg == 0 and irrelevant_delta_sum == 0:
    outcome_claim = "ctxhelm_matched"
else:
    outcome_claim = "no_measured_lift"

def recommended_research_actions():
    actions = []

    def add(action, priority, reason):
        actions.append({"action": action, "priority": priority, "reason": reason})

    if comparison["clientFailuresObserved"] or comparison["rateLimitsObserved"]:
        add("retry_real_client_when_available", 1, "Client availability prevented comparable outcome proof.")
    elif not comparison["ctxhelmToolCallsObserved"] and not comparison_eligible_count:
        add("collect_real_client_evidence", 1, "No comparable real-client ctxhelm call evidence was collected.")
    if (
        (comparison["missingRequiredCtxhelmCallsObserved"] or comparison["invalidRequiredCtxhelmCallsObserved"])
        and not comparison["clientFailuresObserved"]
        and comparison["ctxhelmToolCallsObserved"]
    ):
        add("harden_required_ctxhelm_call_guidance", 1, "A ctxhelm-assisted lane did not make all required source-free ctxhelm calls.")
    if comparison["ctxhelmEvidenceMissesObserved"]:
        add("fix_retrieval_or_query_construction", 1, "ctxhelm evidence did not surface at least one expected target.")
    if comparison["ctxhelmEvidenceOnlyTargetsObserved"] and not comparison["clientFailuresObserved"]:
        add("improve_agent_consumption_guidance", 2, "ctxhelm surfaced expected targets that Codex did not consume with read-only commands.")
    if comparison["ctxhelmUnderReadTargetsObserved"] and not comparison["clientFailuresObserved"]:
        add("inspect_pack_ordering_and_native_read_instruction", 2, "A ctxhelm-assisted lane under-read targets relative to the native baseline.")
    if comparison_eligible_count and outcome_claim == "no_measured_lift":
        add("analyze_native_baseline_gap", 2, "Comparable lanes produced no measured ctxhelm lift.")
    if not actions and comparison_eligible_count and outcome_claim in {"ctxhelm_improved", "ctxhelm_matched"}:
        add("preserve_current_agent_contract", 3, "Comparable lanes produced stable source-free outcome evidence.")
    return actions

payload = {
    "schemaVersion": "ctxhelm-agent-run-eval-v1",
    "status": (
        "degraded"
        if (
            comparison["forbiddenCommandsObserved"]
            or comparison["missingRequiredCtxhelmCallsObserved"]
            or comparison["invalidRequiredCtxhelmCallsObserved"]
            or comparison["clientFailuresObserved"]
            or (task_count and comparison_eligible_count == 0)
        )
        else ("passed" if any(task.get("status") == "passed" for task in tasks) else "skipped")
    ),
    "workflowKind": "paired-agent-context-suite",
    "client": {"name": "codex", "version": client_version},
    "ctxhelmVersion": ctxhelm_version,
    "repo": {
        "label": pathlib.Path(repo).name,
        "pathSha256": hashlib.sha256(repo.encode("utf-8")).hexdigest(),
    },
    "suite": {
        "suiteSha256": hashlib.sha256(pathlib.Path(suite_path).read_bytes()).hexdigest(),
        "rawTasksStored": False,
        "taskCount": task_count,
    },
    "tasks": tasks,
    "aggregate": {
        "taskCount": task_count,
        "laneSummaries": lane_summaries,
        "targetCoverageDeltaAverage": target_delta_avg,
        "targetReadCoverageDeltaAverage": target_read_delta_avg,
        "irrelevantReadDeltaSum": irrelevant_delta_sum,
        "commandExecutionDeltaSum": comparison["commandExecutionDeltaSum"],
        "readFileDeltaSum": comparison["readFileDeltaSum"],
        "comparisonEligibleCount": comparison_eligible_count,
        "comparableCtxhelmLaneCount": comparison["comparableCtxhelmLaneCount"],
        "ctxhelmToolCallsObserved": comparison["ctxhelmToolCallsObserved"],
        "forbiddenCommandsObserved": comparison["forbiddenCommandsObserved"],
        "missingRequiredCtxhelmCallsObserved": comparison["missingRequiredCtxhelmCallsObserved"],
        "invalidRequiredCtxhelmCallsObserved": comparison["invalidRequiredCtxhelmCallsObserved"],
        "clientFailuresObserved": comparison["clientFailuresObserved"],
        "rateLimitsObserved": comparison["rateLimitsObserved"],
        "ctxhelmEvidenceMissesObserved": comparison["ctxhelmEvidenceMissesObserved"],
        "ctxhelmEvidenceOnlyTargetsObserved": comparison["ctxhelmEvidenceOnlyTargetsObserved"],
        "ctxhelmUnderReadTargetsObserved": comparison["ctxhelmUnderReadTargetsObserved"],
        "outcomeClaim": outcome_claim,
        "recommendedResearchActions": recommended_research_actions(),
    },
    "privacyStatus": privacy,
    "unsupportedActions": [
        "source edits",
        "user project tests",
        "global agent config mutation",
        "cloud upload",
        "raw prompt storage",
        "raw transcript storage",
        "raw MCP traffic storage",
        "raw command output storage",
    ],
}
text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output_path:
    path = pathlib.Path(output_path).resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
else:
    print(text, end="")
PY
  echo "codex agent-run suite eval wrote ${output_path:-stdout}"
  exit 0
fi

target_json="$work_dir/targets.json"
python3 - "$repo" "$target_json" "${target_files[@]}" <<'PY'
import json
import pathlib
import sys

repo = pathlib.Path(sys.argv[1]).resolve()
out = pathlib.Path(sys.argv[2])
targets = []
for raw in sys.argv[3:]:
    path = pathlib.Path(raw)
    if path.is_absolute():
        try:
            label = path.resolve().relative_to(repo).as_posix()
        except ValueError:
            label = path.name
    else:
        label = path.as_posix()
    if label not in targets:
        targets.append(label)
out.write_text(json.dumps(targets), encoding="utf-8")
PY

write_skip_lane() {
  local lane="$1"
  local mode="$2"
  local out="$3"
  local reason="$4"
  python3 - "$lane" "$mode" "$out" "$reason" <<'PY'
import json
import pathlib
import sys

lane, mode, out, reason = sys.argv[1:]
required = {
    "baseline": [],
    "plan": [{"name": "prepare_task", "requiresRepo": True, "requiresTask": True}],
    "brief": [
        {"name": "prepare_task", "requiresRepo": True, "requiresTask": True},
        {"name": "get_pack", "requiresRepo": True, "requiresTask": True, "budget": "brief", "format": "json", "recordTrace": False},
    ],
    "standard": [
        {"name": "prepare_task", "requiresRepo": True, "requiresTask": True},
        {"name": "get_pack", "requiresRepo": True, "requiresTask": True, "budget": "standard", "format": "json", "recordTrace": False},
    ],
    "memory": [
        {"name": "prepare_task", "requiresRepo": True, "requiresTask": True},
        {"name": "get_pack", "requiresRepo": True, "requiresTask": True, "budget": "standard", "format": "json", "recordTrace": False},
    ],
}.get(mode, [])
calls = [spec["name"] for spec in required]
payload = {
    "lane": lane,
    "mode": mode,
    "status": "skipped",
    "evaluationStatus": "skipped",
    "evaluationEligible": False,
    "skipReason": reason,
    "metrics": {
        "targetCoverage": 0.0,
        "targetReadCoverage": 0.0,
        "readFileCount": 0,
        "discoveredFileCount": 0,
        "irrelevantReadCount": 0,
        "targetReadCount": 0,
        "targetDiscoveredOnlyCount": 0,
        "missedTargetCount": 0,
        "commandExecutionCount": 0,
        "ctxhelmToolCallCount": 0,
        "forbiddenCommandCount": 0,
        "requiredCtxhelmCallCount": len(calls),
        "observedRequiredCtxhelmCallCount": 0,
        "missingRequiredCtxhelmCallCount": len(calls),
        "invalidRequiredCtxhelmCallCount": 0,
        "ctxhelmEvidenceFileCount": 0,
        "ctxhelmEvidenceTargetHitCount": 0,
        "ctxhelmEvidenceOnlyTargetCount": 0,
        "ctxhelmEvidenceMissedTargetCount": 0,
    },
    "requiredCtxhelmCallSpecs": required,
    "requiredCtxhelmCalls": calls,
    "observedRequiredCtxhelmCalls": [],
    "missingRequiredCtxhelmCalls": calls,
    "invalidRequiredCtxhelmCalls": [],
    "ctxhelmCallCompliance": "not_required" if not calls else "missing",
    "clientFailureKind": None,
    "clientApiErrorStatus": None,
    "rateLimitObserved": False,
    "targetHits": [],
    "targetReads": [],
    "discoveredOnlyTargets": [],
    "missedTargets": [],
    "ctxhelmEvidenceFiles": [],
    "ctxhelmEvidenceTargetHits": [],
    "ctxhelmEvidenceOnlyTargets": [],
    "ctxhelmEvidenceMissedTargets": [],
    "readRoleCounts": {},
    "missedTargetRoleCounts": {},
    "commandExecutions": [],
    "forbiddenCommands": [],
    "sourceTextLogged": False,
    "rawTranscriptStored": False,
    "rawPromptStored": False,
    "rawCommandOutputStored": False,
}
pathlib.Path(out).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_mcp_wrapper() {
  local lane_dir="$1"
  local request_log="$2"
  local wrapper="$lane_dir/ctxhelm-mcp-server.sh"
  {
    printf '%s\n' '#!/usr/bin/env bash'
    printf '%s\n' 'set -euo pipefail'
    printf 'export CTXHELM_REAL_CLIENT_REQUEST_LOG=%q\n' "$request_log"
    if [[ -n "${CTXHELM_HOME:-}" ]]; then
      printf 'export CTXHELM_HOME=%q\n' "$CTXHELM_HOME"
    fi
    printf 'ctxhelm_bin=%q\n' "$ctxhelm_bin"
    printf '%s\n' 'tee -a "$CTXHELM_REAL_CLIENT_REQUEST_LOG" | "$ctxhelm_bin" serve-mcp'
  } >"$wrapper"
  chmod +x "$wrapper"
  printf '%s\n' "$wrapper"
}

run_lane() {
  local lane="$1"
  local mode="$2"
  local lane_dir="$work_dir/$lane"
  mkdir -p "$lane_dir"
  local events="$lane_dir/events.jsonl"
  local stderr_log="$lane_dir/stderr.log"
  local last_message="$lane_dir/last-message.txt"
  local request_log="$lane_dir/ctxhelm-requests.jsonl"
  local ctxhelm_evidence="$lane_dir/ctxhelm-evidence.json"
  local lane_json="$lane_dir/lane.json"

  if [[ "$run_real" != "1" && "$require_real" != "1" ]]; then
    write_skip_lane "$lane" "$mode" "$lane_json" "real Codex execution not requested; set CTXHELM_RUN_REAL_CLIENT=1"
    printf '%s\n' "$lane_json"
    return
  fi
  if ! command -v codex >/dev/null 2>&1; then
    if [[ "$require_real" == "1" ]]; then
      echo "codex is required for agent-run proof" >&2
      exit 69
    fi
    write_skip_lane "$lane" "$mode" "$lane_json" "codex is not installed"
    printf '%s\n' "$lane_json"
    return
  fi

  local prompt
  if [[ "$mode" == "baseline" ]]; then
    prompt=$(cat <<EOF
Do not edit files, do not write files, do not use shell redirection, do not run tests, and do not mutate git or global config. Do not run awk, bootstrap, setup, install, hook, or superpowers commands.
Use only read-only shell commands such as pwd, ls, find, rg, grep, sed, cat, nl, head, tail, and wc.
Use at most 8 shell commands total. After reading up to 5 relevant files, stop exploring and answer.
Task: $task
Identify and inspect the files most relevant to this task. Prefer reading likely implementation and validation files before answering. Return a short JSON object with keyFiles.
EOF
)
  elif [[ "$mode" == "plan" ]]; then
    prompt=$(cat <<EOF
Do not edit files, do not write files, do not use shell redirection, do not run tests, and do not mutate git or global config. Do not run awk, bootstrap, setup, install, hook, or superpowers commands.
First call ctxhelm prepare_task with explicit repo "$repo" and task "$task".
Use at most 8 shell commands total after ctxhelm calls.
Identify the returned targetFiles paths. Consume the first up to 5 targetFiles first with read commands such as sed, cat, nl, head, or tail before broader exploration. rg, grep, find, ls, and wc may locate or count evidence, but they do not count as consuming a target file.
If selectedMemory appears, also read up to 3 selectedMemory sourceLinks or evidence paths with read-only shell commands before broader exploration.
Treat docs, config, schema, script, and selected-memory paths in that initial set as first-class targets, not optional background. Stop after those reads if they answer the task.
Return a short JSON object with keyFiles.
EOF
)
  elif [[ "$mode" == "brief" ]]; then
    prompt=$(cat <<EOF
Do not edit files, do not write files, do not use shell redirection, do not run tests, and do not mutate git or global config. Do not run awk, bootstrap, setup, install, hook, or superpowers commands.
First call ctxhelm prepare_task with explicit repo "$repo" and task "$task".
Then call ctxhelm get_pack with explicit repo "$repo", the same task, budget "brief", format "json", and recordTrace false.
Use at most 8 shell commands total after ctxhelm calls.
Identify targetFiles from prepare_task and high-confidence target paths from get_pack. Consume the first up to 5 target files first with read commands such as sed, cat, nl, head, or tail before broader exploration. rg, grep, find, ls, and wc may locate or count evidence, but they do not count as consuming a target file.
If selectedMemory appears, also read up to 3 selectedMemory sourceLinks or evidence paths with read-only shell commands before broader exploration.
Treat docs, config, schema, script, and selected-memory paths in that initial set as first-class targets, not optional background. Stop after those reads if they answer the task.
Return a short JSON object with keyFiles.
EOF
)
  elif [[ "$mode" == "standard" ]]; then
    prompt=$(cat <<EOF
Do not edit files, do not write files, do not use shell redirection, do not run tests, and do not mutate git or global config. Do not run awk, bootstrap, setup, install, hook, or superpowers commands.
First call ctxhelm prepare_task with explicit repo "$repo" and task "$task".
Then call ctxhelm get_pack with explicit repo "$repo", the same task, budget "standard", format "json", and recordTrace false.
Use at most 8 shell commands total after ctxhelm calls.
Identify targetFiles from prepare_task and high-confidence target paths from get_pack. Consume the first up to 5 target files first with read commands such as sed, cat, nl, head, or tail before broader exploration. rg, grep, find, ls, and wc may locate or count evidence, but they do not count as consuming a target file.
If selectedMemory appears, also read up to 3 selectedMemory sourceLinks or evidence paths with read-only shell commands before broader exploration.
Treat docs, config, schema, script, and selected-memory paths in that initial set as first-class targets, not optional background. Stop after those reads if they answer the task.
Return a short JSON object with keyFiles.
EOF
)
  else
    prompt=$(cat <<EOF
Do not edit files, do not write files, do not use shell redirection, do not run tests, and do not mutate git or global config. Do not run awk, bootstrap, setup, install, hook, or superpowers commands.
First call ctxhelm prepare_task with explicit repo "$repo" and task "$task".
Then call ctxhelm get_pack with explicit repo "$repo", the same task, budget "standard", format "json", and recordTrace false.
This is a memory-efficiency probe. Use at most 4 shell commands total after ctxhelm calls.
First inspect selectedMemory, sourceLinks, experience-card evidence, targetFiles, and high-confidence get_pack paths. Choose the smallest current-file set that memory points to, preferring paths that appear in both memory evidence and targetFiles or high-confidence pack targets.
Consume at most 2 memory-backed current files first with read commands such as sed, cat, nl, head, or tail. rg, grep, find, ls, and wc may locate or count evidence, but they do not count as consuming a target file.
If those memory-backed reads answer which files matter, stop immediately and return the JSON. Do not keep exploring just to fill the command budget.
Only read additional files when the memory-backed path is missing, clearly stale, or needs one immediate caller/callee/test/config neighbor to interpret it.
If ctxhelm returns memory or experience-card evidence, use it only as guidance for which current files to inspect; still consume current files using read-only shell commands before answering.
Return a short JSON object with keyFiles.
EOF
)
  fi

  local codex_args=(exec)
  local codex_env=(env -u CODEX_THREAD_ID -u CODEX_INTERNAL_ORIGINATOR_OVERRIDE -u CODEX_SHELL)
  local codex_exec_help
  codex_exec_help="$(codex exec --help 2>&1 || true)"
  if [[ "$codex_exec_help" == *"--ephemeral"* ]]; then
    codex_args+=(--ephemeral)
  fi
  if [[ "$codex_exec_help" == *"--ignore-user-config"* ]]; then
    codex_args+=(--ignore-user-config)
  else
    local codex_home="$lane_dir/codex-home"
    mkdir -p "$codex_home"
    codex_env+=(CODEX_HOME="$codex_home")
  fi
  if [[ "$codex_exec_help" == *"--ignore-rules"* ]]; then
    codex_args+=(--ignore-rules)
  fi
  codex_args+=(
    --skip-git-repo-check
    --cd "$repo"
    --dangerously-bypass-approvals-and-sandbox
    --json
    --output-last-message "$last_message"
  )
  if [[ "$mode" != "baseline" ]]; then
    local server_wrapper
    server_wrapper="$(write_mcp_wrapper "$lane_dir" "$request_log")"
    codex_args+=(
      -c "mcp_servers.ctxhelm.command=\"$server_wrapper\""
      -c "mcp_servers.ctxhelm.args=[]"
      -c "mcp_servers.ctxhelm.cwd=\"$repo\""
      -c "mcp_servers.ctxhelm.startup_timeout_sec=30"
      -c "mcp_servers.ctxhelm.tool_timeout_sec=120"
    )
  fi
  codex_args+=("$prompt")

  local client_status=0
  set +e
  (
    cd "$repo"
    "${codex_env[@]}" codex "${codex_args[@]}"
  ) >"$events" 2>"$stderr_log" &
  local pid=$!
  local deadline=$((SECONDS + client_timeout_seconds))
  while kill -0 "$pid" 2>/dev/null; do
    if (( SECONDS >= deadline )); then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
      client_status=124
      break
    fi
    sleep 2
  done
  if [[ "$client_status" == "0" ]]; then
    wait "$pid"
    client_status=$?
  fi
  set -e

  if [[ "$mode" == "plan" ]]; then
    "$ctxhelm_bin" prepare-task --repo "$repo" --no-trace "$task" >"$ctxhelm_evidence" 2>/dev/null || true
  elif [[ "$mode" == "brief" ]]; then
    "$ctxhelm_bin" get-pack --repo "$repo" --budget brief --format json --no-trace "$task" >"$ctxhelm_evidence" 2>/dev/null || true
  elif [[ "$mode" == "standard" || "$mode" == "memory" ]]; then
    "$ctxhelm_bin" get-pack --repo "$repo" --budget standard --format json --no-trace "$task" >"$ctxhelm_evidence" 2>/dev/null || true
  else
    : >"$ctxhelm_evidence"
  fi

  python3 - "$lane" "$mode" "$client_status" "$repo" "$target_json" "$events" "$stderr_log" "$request_log" "$ctxhelm_evidence" "$lane_json" <<'PY'
import hashlib
import json
import pathlib
import re
import shlex
import sys

lane, mode, status_text, repo_text, target_path, events_path, stderr_path, request_log_path, ctxhelm_evidence_path, output_path = sys.argv[1:]
repo = pathlib.Path(repo_text).resolve()
targets = set(json.loads(pathlib.Path(target_path).read_text(encoding="utf-8")))
events_file = pathlib.Path(events_path)
request_file = pathlib.Path(request_log_path)
ctxhelm_evidence_file = pathlib.Path(ctxhelm_evidence_path)
stderr_file = pathlib.Path(stderr_path)
client_status = int(status_text)

READ_VERBS = {"cat", "sed", "nl", "head", "tail", "less", "more", "wc"}
DISCOVERY_VERBS = {"rg", "grep", "ls", "find", "pwd"}
FORBIDDEN_PATTERNS = [
    r"\bapply_patch\b",
    r"\bgit\s+(commit|push|reset|checkout|clean|merge|rebase|tag)\b",
    r"\bsuperpowers-codex\b",
    r"\bbootstrap\b",
    r"\b(setup|install)\s+",
    r"\b(rm|mv|cp|touch|mkdir|rmdir)\b",
    r"\b(cargo\s+test|npm\s+test|pnpm\s+test|pytest|go\s+test|mvn\s+test|gradle\s+test|make\s+test)\b",
    r">\s*[^&]",
    r"\btee\s+",
    r"\bchmod\s+",
    r"\bpython3?\b.*\b(write_text|open\(.+['\"]w|Path\(.+\)\.write)",
]

def sha(data):
    return hashlib.sha256(data).hexdigest()

def role_for_path(path):
    value = str(path).lower()
    name = pathlib.PurePosixPath(str(path)).name.lower()
    if (
        "/test/" in f"/{value}/"
        or value.startswith("tests/")
        or value.endswith("_test.py")
        or value.endswith(".test.ts")
        or value.endswith(".test.tsx")
        or value.endswith(".spec.ts")
        or value.endswith(".spec.tsx")
        or value.endswith("test.rs")
    ):
        return "test"
    if value.startswith("docs/") or name in {"readme.md", "readme.rst"} or value.endswith((".md", ".rst", ".adoc")):
        return "docs"
    if (
        value.startswith(".github/")
        or name in {"cargo.toml", "package.json", "pyproject.toml", "go.mod", "pom.xml", "build.gradle", "settings.gradle", "makefile"}
        or value.endswith((".yml", ".yaml", ".json", ".toml", ".lock"))
    ):
        return "config"
    if value.startswith(("src/", "crates/", "app/", "lib/", "packages/", "scripts/")) or value.endswith((".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".java", ".kt", ".cs", ".rb", ".php", ".sh")):
        return "source"
    return "other"

def role_counts(paths):
    counts = {}
    for path in paths:
        role = role_for_path(path)
        counts[role] = counts.get(role, 0) + 1
    return dict(sorted(counts.items()))

def rel_label(raw):
    if not raw:
        return None
    value = str(raw).strip().strip("'\"")
    if not value:
        return None
    path = pathlib.Path(value)
    if path.is_absolute():
        try:
            return path.resolve().relative_to(repo).as_posix()
        except Exception:
            return None
    return path.as_posix()

def looks_like_repo_path(label):
    if not isinstance(label, str) or not label or any(char.isspace() for char in label):
        return False
    if label.startswith(("ctxhelm://", "repo://", "http://", "https://", "-")):
        return False
    if "/" not in label and label not in targets:
        return False
    suffixes = (
        ".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".java", ".kt", ".cs",
        ".rb", ".php", ".md", ".rst", ".adoc", ".toml", ".json", ".yml", ".yaml",
        ".lock", ".sh", ".sql", ".graphql", ".proto", ".xml", ".gradle",
    )
    return label.lower().endswith(suffixes) or label in targets

def add_path(paths, raw):
    label = rel_label(raw)
    if label and looks_like_repo_path(label) and label not in paths:
        paths.append(label)

def shell_payload(command):
    try:
        parts = shlex.split(command)
    except ValueError:
        parts = command.split()
    if len(parts) >= 3 and pathlib.PurePosixPath(parts[0]).name in {"zsh", "bash", "sh"} and parts[1] in {"-lc", "-c"}:
        return parts[2]
    return command

def command_verb(command):
    payload = shell_payload(command)
    try:
        parts = shlex.split(payload)
    except ValueError:
        parts = payload.split()
    if not parts:
        return ""
    return pathlib.PurePosixPath(parts[0]).name

def command_paths(command):
    paths = []
    payload = shell_payload(command)
    for token in re.findall(r"(?<![A-Za-z0-9_.-])([A-Za-z0-9_./@+-]+/[A-Za-z0-9_./@+-]+(?:\.[A-Za-z0-9_+-]+)?)", payload):
        add_path(paths, token)
    try:
        parts = shlex.split(payload)
    except ValueError:
        parts = payload.split()
    for token in parts:
        add_path(paths, token)
    return paths

def collect_path_evidence(value):
    paths = []
    def visit(item, key=None):
        if isinstance(item, dict):
            for child_key, child_value in item.items():
                visit(child_value, child_key)
            return
        if isinstance(item, list):
            for child in item:
                visit(child, key)
            return
        if not isinstance(item, str):
            return
        normalized_key = str(key or "")
        if normalized_key == "content":
            for match in re.findall(r"`([^`]+)`", item):
                add_path(paths, match)
            return
        if normalized_key in {"path", "file", "source", "resourcePath", "targetFile"}:
            add_path(paths, item)
            return
        if normalized_key in {"paths", "files", "targetFiles", "nextReadPaths", "examplePaths", "sourceLinks"}:
            add_path(paths, item)
    visit(value)
    return paths

def forbidden_reasons(command):
    reasons = []
    payload = shell_payload(command)
    for pattern in FORBIDDEN_PATTERNS:
        if re.search(pattern, payload):
            reasons.append(pattern)
    return reasons

command_executions = []
forbidden_commands = []
read_files = []
discovered_files = []
turn_completed = False
client_failure_kind = None
client_api_error_status = None
rate_limit_observed = False

if events_file.exists():
    for line in events_file.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("type") == "turn.completed":
            turn_completed = True
        item = payload.get("item") or {}
        if isinstance(item, dict) and item.get("type") == "error":
            client_failure_kind = client_failure_kind or "client_error"
        if not (isinstance(item, dict) and item.get("type") == "command_execution" and payload.get("type") == "item.completed"):
            continue
        command = item.get("command") or ""
        verb = command_verb(command)
        paths = command_paths(command)
        reasons = forbidden_reasons(command)
        entry = {
            "commandSha256": sha(command.encode("utf-8")),
            "verb": verb,
            "exitCode": item.get("exit_code"),
            "status": item.get("status"),
            "pathCount": len(paths),
            "paths": paths,
            "forbidden": bool(reasons),
        }
        command_executions.append(entry)
        if reasons:
            forbidden_commands.append({
                "commandSha256": entry["commandSha256"],
                "verb": verb,
                "reasons": reasons,
                "pathCount": len(paths),
            })
        if verb in READ_VERBS:
            for path in paths:
                if path not in read_files:
                    read_files.append(path)
        if verb in READ_VERBS or verb in DISCOVERY_VERBS:
            for path in paths:
                if path not in discovered_files:
                    discovered_files.append(path)

if client_status == 124:
    client_failure_kind = client_failure_kind or "timeout"
elif client_status != 0 or not turn_completed:
    stderr_text = stderr_file.read_text(encoding="utf-8", errors="replace") if stderr_file.exists() else ""
    stderr_lower = stderr_text.lower()
    if "rate limit" in stderr_lower or "429" in stderr_lower:
        rate_limit_observed = True
        client_failure_kind = "rate_limited"
        client_api_error_status = 429 if "429" in stderr_lower else None
    elif "stream disconnected" in stderr_lower:
        client_failure_kind = client_failure_kind or "stream_disconnected"
    elif any(token in stderr_lower for token in ["api key", "unauthorized", "authentication", "not logged in", "login required"]):
        client_failure_kind = client_failure_kind or "auth_or_model_refusal"
    elif client_failure_kind is None:
        client_failure_kind = "client_exit_nonzero"

ctxhelm_calls = []
if request_file.exists():
    for line in request_file.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("method") != "tools/call":
            continue
        params = payload.get("params") or {}
        arguments = params.get("arguments") or {}
        ctxhelm_calls.append({
            "name": params.get("name"),
            "hasRepo": arguments.get("repo") == repo_text,
            "hasTask": "task" in arguments,
            "budget": arguments.get("budget"),
            "format": arguments.get("format"),
            "recordTrace": arguments.get("recordTrace"),
        })

ctxhelm_evidence_files = []
if ctxhelm_evidence_file.exists() and ctxhelm_evidence_file.stat().st_size > 0:
    try:
        ctxhelm_evidence_payload = json.loads(ctxhelm_evidence_file.read_text(encoding="utf-8", errors="replace"))
        ctxhelm_evidence_files = collect_path_evidence(ctxhelm_evidence_payload)
    except json.JSONDecodeError:
        ctxhelm_evidence_files = []

evidence_files = set(read_files) | set(discovered_files)
target_hits = sorted(target for target in targets if target in evidence_files)
target_reads = sorted(target for target in targets if target in set(read_files))
discovered_only_targets = sorted(target for target in target_hits if target not in set(read_files))
missed_targets = sorted(target for target in targets if target not in evidence_files)
ctxhelm_evidence_set = set(ctxhelm_evidence_files)
ctxhelm_evidence_target_hits = sorted(target for target in targets if target in ctxhelm_evidence_set)
ctxhelm_evidence_only_targets = sorted(target for target in ctxhelm_evidence_target_hits if target not in evidence_files)
ctxhelm_evidence_missed_targets = sorted(target for target in targets if target not in ctxhelm_evidence_set)
irrelevant_reads = sorted(path for path in read_files if path not in targets)

required_call_specs_by_mode = {
    "baseline": [],
    "plan": [{"name": "prepare_task", "requiresRepo": True, "requiresTask": True}],
    "brief": [
        {"name": "prepare_task", "requiresRepo": True, "requiresTask": True},
        {"name": "get_pack", "requiresRepo": True, "requiresTask": True, "budget": "brief", "format": "json", "recordTrace": False},
    ],
    "standard": [
        {"name": "prepare_task", "requiresRepo": True, "requiresTask": True},
        {"name": "get_pack", "requiresRepo": True, "requiresTask": True, "budget": "standard", "format": "json", "recordTrace": False},
    ],
    "memory": [
        {"name": "prepare_task", "requiresRepo": True, "requiresTask": True},
        {"name": "get_pack", "requiresRepo": True, "requiresTask": True, "budget": "standard", "format": "json", "recordTrace": False},
    ],
}
required_call_specs = required_call_specs_by_mode.get(mode, [])
required_calls = [spec["name"] for spec in required_call_specs]

def invalid_reasons(call, spec):
    reasons = []
    if spec.get("requiresRepo") and not call.get("hasRepo", False):
        reasons.append("repo")
    if spec.get("requiresTask") and not call.get("hasTask", False):
        reasons.append("task")
    if "budget" in spec and call.get("budget") != spec["budget"]:
        reasons.append("budget")
    if "format" in spec and call.get("format") != spec["format"]:
        reasons.append("format")
    if "recordTrace" in spec and call.get("recordTrace") != spec["recordTrace"]:
        reasons.append("recordTrace")
    return reasons

observed_required_calls = []
invalid_required_calls = []
for spec in required_call_specs:
    attempts = [call for call in ctxhelm_calls if call.get("name") == spec["name"]]
    valid = any(not invalid_reasons(call, spec) for call in attempts)
    if valid:
        observed_required_calls.append(spec["name"])
    elif attempts:
        reason_set = sorted({reason for call in attempts for reason in invalid_reasons(call, spec)})
        invalid_required_calls.append({"name": spec["name"], "reasons": reason_set, "attemptCount": len(attempts)})

missing_required_calls = [name for name in required_calls if name not in observed_required_calls]
ctxhelm_call_compliance = (
    "not_required"
    if not required_calls
    else ("satisfied" if not missing_required_calls else ("invalid" if invalid_required_calls else "missing"))
)
if client_failure_kind:
    missing_required_calls = []
    invalid_required_calls = []
    ctxhelm_call_compliance = "client_unavailable" if required_calls else "not_required"

lane_status = "passed" if client_status == 0 and turn_completed and not forbidden_commands else "failed"
evaluation_eligible = lane_status == "passed" and not client_failure_kind and ctxhelm_call_compliance != "missing"
if evaluation_eligible:
    evaluation_status = "eligible"
elif lane_status != "passed" or client_failure_kind:
    evaluation_status = "failed"
else:
    evaluation_status = "not_comparable"
if not evaluation_eligible:
    ctxhelm_evidence_only_targets = []

payload = {
    "lane": lane,
    "mode": mode,
    "status": lane_status,
    "evaluationStatus": evaluation_status,
    "evaluationEligible": evaluation_eligible,
    "clientExitStatus": client_status,
    "clientFailureKind": client_failure_kind,
    "clientApiErrorStatus": client_api_error_status,
    "rateLimitObserved": rate_limit_observed,
    "metrics": {
        "targetCoverage": len(target_hits) / len(targets) if targets else 0.0,
        "targetReadCoverage": len(target_reads) / len(targets) if targets else 0.0,
        "targetHitCount": len(target_hits),
        "targetCount": len(targets),
        "targetReadCount": len(target_reads),
        "targetDiscoveredOnlyCount": len(discovered_only_targets),
        "missedTargetCount": len(missed_targets),
        "readFileCount": len(read_files),
        "discoveredFileCount": len(discovered_files),
        "irrelevantReadCount": len(irrelevant_reads),
        "commandExecutionCount": len(command_executions),
        "ctxhelmToolCallCount": len(ctxhelm_calls),
        "forbiddenCommandCount": len(forbidden_commands),
        "requiredCtxhelmCallCount": len(required_calls),
        "observedRequiredCtxhelmCallCount": len(observed_required_calls),
        "missingRequiredCtxhelmCallCount": len(missing_required_calls),
        "invalidRequiredCtxhelmCallCount": len(invalid_required_calls),
        "ctxhelmEvidenceFileCount": len(ctxhelm_evidence_files),
        "ctxhelmEvidenceTargetHitCount": len(ctxhelm_evidence_target_hits),
        "ctxhelmEvidenceOnlyTargetCount": len(ctxhelm_evidence_only_targets),
        "ctxhelmEvidenceMissedTargetCount": len(ctxhelm_evidence_missed_targets),
    },
    "requiredCtxhelmCallSpecs": required_call_specs,
    "requiredCtxhelmCalls": required_calls,
    "observedRequiredCtxhelmCalls": observed_required_calls,
    "missingRequiredCtxhelmCalls": missing_required_calls,
    "invalidRequiredCtxhelmCalls": invalid_required_calls,
    "ctxhelmCallCompliance": ctxhelm_call_compliance,
    "targetHits": target_hits,
    "targetReads": target_reads,
    "discoveredOnlyTargets": discovered_only_targets,
    "missedTargets": missed_targets,
    "ctxhelmEvidenceFiles": ctxhelm_evidence_files,
    "ctxhelmEvidenceTargetHits": ctxhelm_evidence_target_hits,
    "ctxhelmEvidenceOnlyTargets": ctxhelm_evidence_only_targets,
    "ctxhelmEvidenceMissedTargets": ctxhelm_evidence_missed_targets,
    "readRoleCounts": role_counts(read_files),
    "missedTargetRoleCounts": role_counts(missed_targets),
    "readFiles": read_files,
    "discoveredFiles": discovered_files,
    "irrelevantReads": irrelevant_reads,
    "commandExecutions": command_executions,
    "forbiddenCommands": forbidden_commands,
    "ctxhelmToolCalls": ctxhelm_calls,
    "evidenceHashes": {
        "streamJsonSha256": sha(events_file.read_bytes()) if events_file.exists() else None,
        "stderrSha256": sha(stderr_file.read_bytes()) if stderr_file.exists() else None,
        "ctxhelmRequestLogSha256": sha(request_file.read_bytes()) if request_file.exists() else None,
    },
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
    "rawCommandOutputStored": False,
}
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  printf '%s\n' "$lane_json"
}

baseline_json="$(run_lane baseline baseline)"
plan_json="$(run_lane ctxhelm-plan plan)"
brief_json="$(run_lane ctxhelm-brief brief)"
standard_json="$(run_lane ctxhelm-standard standard)"
memory_json="$(run_lane ctxhelm-memory memory)"

python3 - "$repo" "$task" "$ctxhelm_version" "$client_version" "$target_json" "$baseline_json" "$plan_json" "$brief_json" "$standard_json" "$memory_json" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys

repo, task, ctxhelm_version, client_version, target_path, *rest = sys.argv[1:]
lane_paths = rest[:5]
output_path = rest[5]
targets = json.loads(pathlib.Path(target_path).read_text(encoding="utf-8"))
lanes = [json.loads(pathlib.Path(path).read_text(encoding="utf-8")) for path in lane_paths]

baseline = lanes[0]
ctxhelm_lanes = [lane for lane in lanes if lane.get("mode") in {"plan", "brief", "standard", "memory"}]
baseline_eligible = bool(baseline.get("evaluationEligible", baseline.get("status") == "passed"))
eligible_ctxhelm_lanes = [lane for lane in ctxhelm_lanes if bool(lane.get("evaluationEligible", lane.get("status") == "passed"))]
comparison_eligible = baseline_eligible and bool(eligible_ctxhelm_lanes)
eligible_lanes = ([baseline] if baseline_eligible else []) + eligible_ctxhelm_lanes
best_candidates = eligible_lanes if eligible_lanes else lanes
best = max(
    best_candidates,
    key=lambda lane: (
        1 if lane.get("status") == "passed" else 0,
        lane.get("metrics", {}).get("targetCoverage", 0.0),
        lane.get("metrics", {}).get("targetReadCoverage", 0.0),
        -lane.get("metrics", {}).get("forbiddenCommandCount", 0),
        -lane.get("metrics", {}).get("irrelevantReadCount", 999_999),
        lane.get("metrics", {}).get("ctxhelmToolCallCount", 0),
    ),
)
base_metrics = baseline.get("metrics", {})
best_metrics = best.get("metrics", {})
target_delta = best_metrics.get("targetCoverage", 0.0) - base_metrics.get("targetCoverage", 0.0)
target_read_delta = best_metrics.get("targetReadCoverage", 0.0) - base_metrics.get("targetReadCoverage", 0.0)
irrelevant_delta = base_metrics.get("irrelevantReadCount", 0) - best_metrics.get("irrelevantReadCount", 0)
command_execution_delta = base_metrics.get("commandExecutionCount", 0) - best_metrics.get("commandExecutionCount", 0)
read_file_delta = base_metrics.get("readFileCount", 0) - best_metrics.get("readFileCount", 0)
ctxhelm_called = any(lane.get("metrics", {}).get("ctxhelmToolCallCount", 0) > 0 for lane in ctxhelm_lanes)
forbidden_called = any(lane.get("metrics", {}).get("forbiddenCommandCount", 0) > 0 for lane in lanes)
client_failures_observed = any(bool(lane.get("clientFailureKind")) for lane in lanes)
rate_limits_observed = any(bool(lane.get("rateLimitObserved", False)) for lane in lanes)
missing_required_calls = {lane.get("lane"): lane.get("missingRequiredCtxhelmCalls", []) for lane in lanes if lane.get("missingRequiredCtxhelmCalls")}
missing_required_observed = bool(missing_required_calls)
invalid_required_calls = {lane.get("lane"): lane.get("invalidRequiredCtxhelmCalls", []) for lane in lanes if lane.get("invalidRequiredCtxhelmCalls")}
invalid_required_observed = bool(invalid_required_calls)
ctxhelm_evidence_misses = {lane.get("lane"): lane.get("ctxhelmEvidenceMissedTargets", []) for lane in ctxhelm_lanes if lane.get("ctxhelmEvidenceMissedTargets")}
ctxhelm_evidence_misses_observed = bool(ctxhelm_evidence_misses)
ctxhelm_evidence_only_targets = {
    lane.get("lane"): lane.get("ctxhelmEvidenceOnlyTargets", [])
    for lane in ctxhelm_lanes
    if lane.get("evaluationEligible", False) and lane.get("ctxhelmEvidenceOnlyTargets")
}
ctxhelm_evidence_only_observed = bool(ctxhelm_evidence_only_targets)
ctxhelm_under_read = any(
    lane.get("evaluationEligible", False)
    and lane.get("metrics", {}).get("targetReadCoverage", 0.0) < base_metrics.get("targetReadCoverage", 0.0)
    for lane in ctxhelm_lanes
)
status = "passed" if any(lane.get("status") == "passed" for lane in lanes) else "skipped"
if status == "passed" and (not ctxhelm_called or not comparison_eligible or missing_required_observed or invalid_required_observed or client_failures_observed or forbidden_called):
    status = "degraded"
if not comparison_eligible:
    outcome_claim = "insufficient_comparable_lanes"
elif ctxhelm_called and (target_delta > 0 or target_read_delta > 0 or irrelevant_delta > 0 or command_execution_delta > 0):
    outcome_claim = "ctxhelm_improved"
elif ctxhelm_called and target_delta == 0 and target_read_delta == 0 and irrelevant_delta == 0:
    outcome_claim = "ctxhelm_matched"
else:
    outcome_claim = "no_measured_lift"

def recommended_research_actions():
    actions = []
    def add(action, priority, reason):
        actions.append({"action": action, "priority": priority, "reason": reason})
    if client_failures_observed or rate_limits_observed:
        add("retry_real_client_when_available", 1, "Client availability prevented comparable outcome proof.")
    elif not ctxhelm_called and not comparison_eligible:
        add("collect_real_client_evidence", 1, "No comparable real-client ctxhelm call evidence was collected.")
    if (missing_required_observed or invalid_required_observed) and not client_failures_observed and ctxhelm_called:
        add("harden_required_ctxhelm_call_guidance", 1, "A ctxhelm-assisted lane did not make all required source-free ctxhelm calls.")
    if forbidden_called:
        add("harden_codex_read_only_prompt", 1, "A Codex lane used a command outside the read-only evaluation boundary.")
    if ctxhelm_evidence_misses_observed:
        add("fix_retrieval_or_query_construction", 1, "ctxhelm evidence did not surface at least one expected target.")
    if ctxhelm_evidence_only_observed and not client_failures_observed:
        add("improve_agent_consumption_guidance", 2, "ctxhelm surfaced expected targets that Codex did not consume with read-only commands.")
    if ctxhelm_under_read and not client_failures_observed:
        add("inspect_pack_ordering_and_native_read_instruction", 2, "A ctxhelm-assisted lane under-read targets relative to the native baseline.")
    if comparison_eligible and outcome_claim == "no_measured_lift":
        add("analyze_native_baseline_gap", 2, "Comparable lanes produced no measured ctxhelm lift.")
    if not actions and comparison_eligible and outcome_claim in {"ctxhelm_improved", "ctxhelm_matched"}:
        add("preserve_current_agent_contract", 3, "Comparable lanes produced stable source-free outcome evidence.")
    return actions

payload = {
    "schemaVersion": "ctxhelm-agent-run-eval-v1",
    "status": status,
    "workflowKind": "paired-agent-context-run",
    "client": {"name": "codex", "version": client_version},
    "ctxhelmVersion": ctxhelm_version,
    "repo": {
        "label": pathlib.Path(repo).name,
        "pathSha256": hashlib.sha256(repo.encode("utf-8")).hexdigest(),
    },
    "task": {
        "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
        "rawTaskStored": False,
    },
    "targetFiles": targets,
    "lanes": lanes,
    "comparison": {
        "baselineLane": baseline.get("lane"),
        "bestLane": best.get("lane"),
        "comparisonEligible": comparison_eligible,
        "baselineEligible": baseline_eligible,
        "comparableCtxhelmLaneCount": len(eligible_ctxhelm_lanes),
        "targetCoverageDelta": target_delta,
        "targetReadCoverageDelta": target_read_delta,
        "irrelevantReadDelta": irrelevant_delta,
        "commandExecutionDelta": command_execution_delta,
        "readFileDelta": read_file_delta,
        "ctxhelmToolCallsObserved": ctxhelm_called,
        "forbiddenCommandsObserved": forbidden_called,
        "missingRequiredCtxhelmCallsObserved": missing_required_observed,
        "missingRequiredCtxhelmCalls": missing_required_calls,
        "invalidRequiredCtxhelmCallsObserved": invalid_required_observed,
        "invalidRequiredCtxhelmCalls": invalid_required_calls,
        "clientFailuresObserved": client_failures_observed,
        "rateLimitsObserved": rate_limits_observed,
        "ctxhelmEvidenceMissesObserved": ctxhelm_evidence_misses_observed,
        "ctxhelmEvidenceMisses": ctxhelm_evidence_misses,
        "ctxhelmEvidenceOnlyTargetsObserved": ctxhelm_evidence_only_observed,
        "ctxhelmEvidenceOnlyTargets": ctxhelm_evidence_only_targets,
        "ctxhelmUnderReadTargetsObserved": ctxhelm_under_read,
        "outcomeClaim": outcome_claim,
        "recommendedResearchActions": recommended_research_actions(),
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
        "rawPromptStored": False,
        "rawTranscriptStored": False,
        "rawMcpTrafficStored": False,
        "rawCommandOutputStored": False,
    },
    "unsupportedActions": [
        "source edits",
        "user project tests",
        "global agent config mutation",
        "cloud upload",
    ],
}
text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output_path:
    path = pathlib.Path(output_path).resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
else:
    print(text, end="")
PY

echo "codex agent-run eval wrote ${output_path:-stdout}"
