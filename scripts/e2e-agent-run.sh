#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: e2e-agent-run.sh --target-file PATH [--target-file PATH ...] [--repo PATH] [--task TASK] [--output PATH]
       e2e-agent-run.sh --suite SUITE.json [--repo PATH] [--output PATH]

Runs a source-free paired Claude Code agent-run evaluation:
  1. baseline native repository exploration
  2. ctxhelm prepare_task-assisted exploration
  3. ctxhelm prepare_task + get_pack-assisted exploration

With --suite, the script runs the same paired evaluation for each task in a
source-free benchmark suite and writes aggregate native-vs-ctxhelm metrics.
Suite files may be either an array of task objects or an object with a "tasks"
array. Each task needs "task" or "prompt" plus "targetFiles" or "target_files".

Real Claude Code execution is optional. Set CTXHELM_RUN_REAL_CLIENT=1 to run the
client. Without it, the script writes a skipped source-free report that preserves
the contract and does not pretend outcome proof exists.

The script does not edit source files, run user project tests, mutate global
agent configuration, publish releases, upload data, or store raw prompts/source
text/transcripts in the report.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
repo_input="${CTXHELM_AGENT_RUN_REPO:-$PWD}"
task="${CTXHELM_AGENT_RUN_TASK:-}"
output_path="${CTXHELM_AGENT_RUN_REPORT:-}"
run_real="${CTXHELM_RUN_REAL_CLIENT:-0}"
require_real="${CTXHELM_REQUIRE_REAL_CLIENT:-0}"
client_timeout_seconds="${CTXHELM_AGENT_RUN_TIMEOUT_SECONDS:-90}"
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
if command -v claude >/dev/null 2>&1; then
  client_version="$(claude --version 2>&1 | head -n 1)"
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
    CTXHELM_BIN="$ctxhelm_bin" "$script_dir/e2e-agent-run.sh" "${task_args[@]}" >/dev/null
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
    "targetCoverageSum": 0.0,
    "targetReadCoverageSum": 0.0,
    "readFileCount": 0,
    "irrelevantReadCount": 0,
    "targetReadCount": 0,
    "targetDiscoveredOnlyCount": 0,
    "missedTargetCount": 0,
    "toolCallCount": 0,
    "ctxhelmToolCallCount": 0,
    "forbiddenToolCallCount": 0,
    "readRoleCounts": defaultdict(int),
    "missedTargetRoleCounts": defaultdict(int),
})
comparison = {
    "ctxhelmToolCallsObserved": False,
    "forbiddenToolCallsObserved": False,
    "targetCoverageDeltaSum": 0.0,
    "targetReadCoverageDeltaSum": 0.0,
    "irrelevantReadDeltaSum": 0,
    "ctxhelmUnderReadTargetsObserved": False,
}
privacy = {
    "localOnly": True,
    "remoteEmbeddingsUsed": False,
    "remoteRerankingUsed": False,
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
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
    for key in ["remoteEmbeddingsUsed", "remoteRerankingUsed", "sourceTextLogged", "rawPromptStored", "rawTranscriptStored", "rawMcpTrafficStored"]:
        privacy[key] = privacy[key] or bool(task_privacy.get(key, False))
    privacy["localOnly"] = privacy["localOnly"] and bool(task_privacy.get("localOnly", True))
    task_comparison = report.get("comparison", {})
    comparison["ctxhelmToolCallsObserved"] = (
        comparison["ctxhelmToolCallsObserved"]
        or bool(task_comparison.get("ctxhelmToolCallsObserved", False))
    )
    comparison["forbiddenToolCallsObserved"] = (
        comparison["forbiddenToolCallsObserved"]
        or bool(task_comparison.get("forbiddenToolCallsObserved", False))
    )
    comparison["targetCoverageDeltaSum"] += float(task_comparison.get("targetCoverageDelta", 0.0) or 0.0)
    comparison["targetReadCoverageDeltaSum"] += float(task_comparison.get("targetReadCoverageDelta", 0.0) or 0.0)
    comparison["irrelevantReadDeltaSum"] += int(task_comparison.get("irrelevantReadDelta", 0) or 0)
    comparison["ctxhelmUnderReadTargetsObserved"] = (
        comparison["ctxhelmUnderReadTargetsObserved"]
        or bool(task_comparison.get("ctxhelmUnderReadTargetsObserved", False))
    )
    for lane in report.get("lanes", []):
        lane_id = lane.get("lane", "unknown")
        metrics = lane.get("metrics", {})
        bucket = lane_totals[lane_id]
        bucket["taskCount"] += 1
        bucket["passedCount"] += 1 if lane.get("status") == "passed" else 0
        bucket["targetCoverageSum"] += float(metrics.get("targetCoverage", 0.0) or 0.0)
        bucket["targetReadCoverageSum"] += float(metrics.get("targetReadCoverage", 0.0) or 0.0)
        bucket["readFileCount"] += int(metrics.get("readFileCount", 0) or 0)
        bucket["irrelevantReadCount"] += int(metrics.get("irrelevantReadCount", 0) or 0)
        bucket["targetReadCount"] += int(metrics.get("targetReadCount", 0) or 0)
        bucket["targetDiscoveredOnlyCount"] += int(metrics.get("targetDiscoveredOnlyCount", 0) or 0)
        bucket["missedTargetCount"] += int(metrics.get("missedTargetCount", 0) or 0)
        bucket["toolCallCount"] += int(metrics.get("toolCallCount", 0) or 0)
        bucket["ctxhelmToolCallCount"] += int(metrics.get("ctxhelmToolCallCount", 0) or 0)
        bucket["forbiddenToolCallCount"] += int(metrics.get("forbiddenToolCallCount", 0) or 0)
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
        "averageTargetCoverage": bucket["targetCoverageSum"] / task_count if task_count else 0.0,
        "averageTargetReadCoverage": bucket["targetReadCoverageSum"] / task_count if task_count else 0.0,
        "readFileCount": bucket["readFileCount"],
        "irrelevantReadCount": bucket["irrelevantReadCount"],
        "targetReadCount": bucket["targetReadCount"],
        "targetDiscoveredOnlyCount": bucket["targetDiscoveredOnlyCount"],
        "missedTargetCount": bucket["missedTargetCount"],
        "toolCallCount": bucket["toolCallCount"],
        "ctxhelmToolCallCount": bucket["ctxhelmToolCallCount"],
        "forbiddenToolCallCount": bucket["forbiddenToolCallCount"],
        "readRoleCounts": dict(sorted(bucket["readRoleCounts"].items())),
        "missedTargetRoleCounts": dict(sorted(bucket["missedTargetRoleCounts"].items())),
    })

task_count = len(tasks)
target_delta_avg = comparison["targetCoverageDeltaSum"] / task_count if task_count else 0.0
target_read_delta_avg = comparison["targetReadCoverageDeltaSum"] / task_count if task_count else 0.0
irrelevant_delta_sum = comparison["irrelevantReadDeltaSum"]
if comparison["ctxhelmToolCallsObserved"] and (target_delta_avg > 0 or target_read_delta_avg > 0 or irrelevant_delta_sum > 0):
    outcome_claim = "ctxhelm_improved"
elif comparison["ctxhelmToolCallsObserved"] and target_delta_avg == 0 and target_read_delta_avg == 0 and irrelevant_delta_sum == 0:
    outcome_claim = "ctxhelm_matched"
else:
    outcome_claim = "no_measured_lift"

payload = {
    "schemaVersion": "ctxhelm-agent-run-eval-v1",
    "status": (
        "degraded"
        if comparison["forbiddenToolCallsObserved"]
        else ("passed" if any(task.get("status") == "passed" for task in tasks) else "skipped")
    ),
    "workflowKind": "paired-agent-context-suite",
    "client": {"name": "claude", "version": client_version},
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
        "ctxhelmToolCallsObserved": comparison["ctxhelmToolCallsObserved"],
        "forbiddenToolCallsObserved": comparison["forbiddenToolCallsObserved"],
        "ctxhelmUnderReadTargetsObserved": comparison["ctxhelmUnderReadTargetsObserved"],
        "outcomeClaim": outcome_claim,
    },
    "privacyStatus": privacy,
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
  echo "agent-run suite eval wrote ${output_path:-stdout}"
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
  local out="$2"
  local reason="$3"
  python3 - "$lane" "$out" "$reason" <<'PY'
import json
import pathlib
import sys

lane, out, reason = sys.argv[1:]
payload = {
    "lane": lane,
    "status": "skipped",
    "skipReason": reason,
    "metrics": {
        "targetCoverage": 0.0,
        "targetReadCoverage": 0.0,
        "readFileCount": 0,
        "irrelevantReadCount": 0,
        "targetReadCount": 0,
        "targetDiscoveredOnlyCount": 0,
        "missedTargetCount": 0,
        "toolCallCount": 0,
        "ctxhelmToolCallCount": 0,
        "forbiddenToolCallCount": 0,
    },
    "targetHits": [],
    "targetReads": [],
    "discoveredOnlyTargets": [],
    "missedTargets": [],
    "readRoleCounts": {},
    "missedTargetRoleCounts": {},
    "sourceTextLogged": False,
    "rawTranscriptStored": False,
}
pathlib.Path(out).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_mcp_config() {
  local lane_dir="$1"
  local request_log="$2"
  local wrapper="$lane_dir/ctxhelm-mcp-server.sh"
  local config="$lane_dir/claude-mcp.json"
  {
    printf '%s\n' '#!/usr/bin/env bash'
    printf '%s\n' 'set -euo pipefail'
    printf 'export CTXHELM_REAL_CLIENT_REQUEST_LOG=%q\n' "$request_log"
    printf 'ctxhelm_bin=%q\n' "$ctxhelm_bin"
    printf '%s\n' 'tee -a "$CTXHELM_REAL_CLIENT_REQUEST_LOG" | "$ctxhelm_bin" serve-mcp'
  } >"$wrapper"
  chmod +x "$wrapper"
  python3 - "$config" "$wrapper" <<'PY'
import json
import pathlib
import sys

path, command = sys.argv[1:]
pathlib.Path(path).write_text(
    json.dumps({"mcpServers": {"ctxhelm": {"command": command, "args": []}}}),
    encoding="utf-8",
)
PY
  printf '%s\n' "$config"
}

run_lane() {
  local lane="$1"
  local mode="$2"
  local lane_dir="$work_dir/$lane"
  mkdir -p "$lane_dir"
  local events="$lane_dir/events.jsonl"
  local stderr_log="$lane_dir/stderr.log"
  local request_log="$lane_dir/ctxhelm-requests.jsonl"
  local lane_json="$lane_dir/lane.json"

  if [[ "$run_real" != "1" && "$require_real" != "1" ]]; then
    write_skip_lane "$lane" "$lane_json" "real Claude Code execution not requested; set CTXHELM_RUN_REAL_CLIENT=1"
    printf '%s\n' "$lane_json"
    return
  fi
  if ! command -v claude >/dev/null 2>&1; then
    if [[ "$require_real" == "1" ]]; then
      echo "claude is required for agent-run proof" >&2
      exit 69
    fi
    write_skip_lane "$lane" "$lane_json" "claude is not installed"
    printf '%s\n' "$lane_json"
    return
  fi

  local allowed="Read,Glob,Grep,LS"
  local mcp_config=""
  local prompt
  if [[ "$mode" == "baseline" ]]; then
    prompt=$(cat <<EOF
Do not edit files, do not run shell commands, and do not write files.
Use native Read, Glob, Grep, or LS tools only.
Task: $task
Identify and inspect the files most relevant to this task. Prefer reading likely implementation and validation files before answering. Return a short JSON object with keyFiles.
EOF
)
  else
    mcp_config="$(write_mcp_config "$lane_dir" "$request_log")"
    allowed="Read,Glob,Grep,LS,mcp__ctxhelm__prepare_task"
    if [[ "$mode" == "brief" ]]; then
      allowed="$allowed,mcp__ctxhelm__get_pack"
    fi
    if [[ "$mode" == "plan" ]]; then
      prompt=$(cat <<EOF
Do not edit files, do not run shell commands, and do not write files.
First call ctxhelm prepare_task with explicit repo "$repo" and task "$task".
Then use native Read, Glob, Grep, or LS tools to inspect the most relevant implementation and validation files before answering.
Return a short JSON object with keyFiles.
EOF
)
    else
      prompt=$(cat <<EOF
Do not edit files, do not run shell commands, and do not write files.
First call ctxhelm prepare_task with explicit repo "$repo" and task "$task".
Then call ctxhelm get_pack with explicit repo "$repo", the same task, budget "brief", format "json", and recordTrace false.
Then use native Read, Glob, Grep, or LS tools to inspect the most relevant implementation and validation files before answering.
Return a short JSON object with keyFiles.
EOF
)
    fi
  fi

  local claude_args=(
    -p
    --no-session-persistence
  )
  if [[ -n "$mcp_config" ]]; then
    claude_args+=(
      --strict-mcp-config
      --mcp-config
      "$mcp_config"
    )
  fi
  claude_args+=(
    --allowedTools
    "$allowed"
    --permission-mode
    bypassPermissions
    --verbose
    --output-format
    stream-json
  )

  set +e
  (
    cd "$repo"
    claude "${claude_args[@]}" "$prompt"
  ) >"$events" 2>"$stderr_log" &
  local pid=$!
  local client_status=0
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

  python3 - "$lane" "$mode" "$client_status" "$repo" "$target_json" "$events" "$stderr_log" "$request_log" "$lane_json" <<'PY'
import hashlib
import json
import pathlib
import sys

lane, mode, status_text, repo_text, target_path, events_path, stderr_path, request_log_path, output_path = sys.argv[1:]
repo = pathlib.Path(repo_text).resolve()
targets = set(json.loads(pathlib.Path(target_path).read_text(encoding="utf-8")))
events_file = pathlib.Path(events_path)
request_file = pathlib.Path(request_log_path)
stderr_file = pathlib.Path(stderr_path)
client_status = int(status_text)

tool_calls = []
forbidden_tool_names = {"Bash", "Edit", "MultiEdit", "NotebookEdit", "Write"}
forbidden_tool_calls = []
read_files = []
discovered_files = []
result_success = False

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
    if value.startswith(("src/", "crates/", "app/", "lib/", "packages/", "scripts/")) or value.endswith((".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".java", ".kt", ".cs", ".rb", ".php")):
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
    value = str(raw)
    path = pathlib.Path(value)
    if path.is_absolute():
        try:
            return path.resolve().relative_to(repo).as_posix()
        except Exception:
            return path.name
    return path.as_posix()

if events_file.exists():
    for line in events_file.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("type") == "result" and payload.get("subtype") == "success":
            result_success = True
        message = payload.get("message") or {}
        if not isinstance(message, dict):
            message = {}
        content = message.get("content") or []
        if not isinstance(content, list):
            content = []
        for item in content:
            if not isinstance(item, dict):
                continue
            if item.get("type") == "tool_use":
                name = item.get("name")
                tool_input = item.get("input") or {}
                if not isinstance(tool_input, dict):
                    tool_input = {}
                input_keys = sorted(tool_input.keys())
                tool_calls.append({"name": name, "inputKeys": input_keys})
                if name in forbidden_tool_names:
                    forbidden_tool_calls.append({"name": name, "inputKeys": input_keys})
                if name == "Read":
                    label = rel_label(tool_input.get("file_path"))
                    if label and label not in read_files:
                        read_files.append(label)
                if name == "Grep":
                    label = rel_label(tool_input.get("path"))
                    if label and label not in discovered_files:
                        discovered_files.append(label)
            elif item.get("type") == "tool_result":
                pass
        result = payload.get("tool_use_result") or {}
        if isinstance(result, dict):
            filenames = result.get("filenames") or []
            if not isinstance(filenames, list):
                filenames = []
            for filename in filenames:
                label = rel_label(filename)
                if label and label not in discovered_files:
                    discovered_files.append(label)
            file_info = result.get("file") or {}
            if isinstance(file_info, dict):
                label = rel_label(file_info.get("filePath"))
                if label and label not in read_files:
                    read_files.append(label)

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
        })

evidence_files = set(read_files) | set(discovered_files)
target_hits = sorted(target for target in targets if target in evidence_files)
target_reads = sorted(target for target in targets if target in set(read_files))
discovered_only_targets = sorted(
    target for target in target_hits if target not in set(read_files)
)
missed_targets = sorted(target for target in targets if target not in evidence_files)
target_coverage = len(target_hits) / len(targets) if targets else 0.0
target_read_coverage = len(target_reads) / len(targets) if targets else 0.0
irrelevant_reads = sorted(path for path in read_files if path not in targets)
stderr_hash = hashlib.sha256(stderr_file.read_bytes()).hexdigest() if stderr_file.exists() else None
events_hash = hashlib.sha256(events_file.read_bytes()).hexdigest() if events_file.exists() else None
request_hash = hashlib.sha256(request_file.read_bytes()).hexdigest() if request_file.exists() else None

payload = {
    "lane": lane,
    "mode": mode,
    "status": "passed" if client_status == 0 and result_success and not forbidden_tool_calls else "failed",
    "clientExitStatus": client_status,
    "metrics": {
        "targetCoverage": target_coverage,
        "targetReadCoverage": target_read_coverage,
        "targetHitCount": len(target_hits),
        "targetCount": len(targets),
        "targetReadCount": len(target_reads),
        "targetDiscoveredOnlyCount": len(discovered_only_targets),
        "missedTargetCount": len(missed_targets),
        "readFileCount": len(read_files),
        "discoveredFileCount": len(discovered_files),
        "irrelevantReadCount": len(irrelevant_reads),
        "toolCallCount": len(tool_calls),
        "ctxhelmToolCallCount": len(ctxhelm_calls),
        "forbiddenToolCallCount": len(forbidden_tool_calls),
    },
    "targetHits": target_hits,
    "targetReads": target_reads,
    "discoveredOnlyTargets": discovered_only_targets,
    "missedTargets": missed_targets,
    "readRoleCounts": role_counts(read_files),
    "missedTargetRoleCounts": role_counts(missed_targets),
    "readFiles": read_files,
    "discoveredFiles": discovered_files,
    "irrelevantReads": irrelevant_reads,
    "toolCalls": tool_calls,
    "forbiddenToolCalls": forbidden_tool_calls,
    "ctxhelmToolCalls": ctxhelm_calls,
    "evidenceHashes": {
        "streamJsonSha256": events_hash,
        "stderrSha256": stderr_hash,
        "ctxhelmRequestLogSha256": request_hash,
    },
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
}
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  printf '%s\n' "$lane_json"
}

baseline_json="$(run_lane baseline baseline)"
plan_json="$(run_lane ctxhelm-plan plan)"
brief_json="$(run_lane ctxhelm-brief brief)"

python3 - "$repo" "$task" "$ctxhelm_version" "$client_version" "$target_json" "$baseline_json" "$plan_json" "$brief_json" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys

repo, task, ctxhelm_version, client_version, target_path, *rest = sys.argv[1:]
lane_paths = rest[:3]
output_path = rest[3]
targets = json.loads(pathlib.Path(target_path).read_text(encoding="utf-8"))
lanes = [json.loads(pathlib.Path(path).read_text(encoding="utf-8")) for path in lane_paths]

baseline = lanes[0]
best = max(
    lanes,
    key=lambda lane: (
        1 if lane.get("status") == "passed" else 0,
        lane.get("metrics", {}).get("targetCoverage", 0.0),
        lane.get("metrics", {}).get("targetReadCoverage", 0.0),
        -lane.get("metrics", {}).get("forbiddenToolCallCount", 0),
        -lane.get("metrics", {}).get("irrelevantReadCount", 999_999),
        lane.get("metrics", {}).get("ctxhelmToolCallCount", 0),
    ),
)
base_metrics = baseline.get("metrics", {})
best_metrics = best.get("metrics", {})
target_delta = best_metrics.get("targetCoverage", 0.0) - base_metrics.get("targetCoverage", 0.0)
target_read_delta = best_metrics.get("targetReadCoverage", 0.0) - base_metrics.get("targetReadCoverage", 0.0)
irrelevant_delta = base_metrics.get("irrelevantReadCount", 0) - best_metrics.get("irrelevantReadCount", 0)
ctxhelm_lanes = [lane for lane in lanes if lane.get("mode") in {"plan", "brief"}]
ctxhelm_called = any(lane.get("metrics", {}).get("ctxhelmToolCallCount", 0) > 0 for lane in ctxhelm_lanes)
forbidden_called = any(lane.get("metrics", {}).get("forbiddenToolCallCount", 0) > 0 for lane in lanes)
ctxhelm_under_read = any(
    lane.get("metrics", {}).get("targetReadCoverage", 0.0)
    < base_metrics.get("targetReadCoverage", 0.0)
    for lane in ctxhelm_lanes
)
status = "passed" if any(lane.get("status") == "passed" for lane in lanes) else "skipped"
if status == "passed" and not ctxhelm_called:
    status = "degraded"
if status == "passed" and forbidden_called:
    status = "degraded"

payload = {
    "schemaVersion": "ctxhelm-agent-run-eval-v1",
    "status": status,
    "workflowKind": "paired-agent-context-run",
    "client": {"name": "claude", "version": client_version},
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
        "targetCoverageDelta": target_delta,
        "targetReadCoverageDelta": target_read_delta,
        "irrelevantReadDelta": irrelevant_delta,
        "ctxhelmToolCallsObserved": ctxhelm_called,
        "forbiddenToolCallsObserved": forbidden_called,
        "ctxhelmUnderReadTargetsObserved": ctxhelm_under_read,
        "outcomeClaim": (
            "ctxhelm_improved"
            if ctxhelm_called and (target_delta > 0 or target_read_delta > 0 or irrelevant_delta > 0)
            else (
                "ctxhelm_matched"
                if ctxhelm_called and target_delta == 0 and target_read_delta == 0 and irrelevant_delta == 0
                else "no_measured_lift"
            )
        ),
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
        "rawPromptStored": False,
        "rawTranscriptStored": False,
        "rawMcpTrafficStored": False,
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

echo "agent-run eval wrote ${output_path:-stdout}"
