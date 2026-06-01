#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: e2e-agent-run.sh --target-file PATH [--target-file PATH ...] [--repo PATH] [--task TASK] [--output PATH]

Runs a source-free paired Claude Code agent-run evaluation:
  1. baseline native repository exploration
  2. ctxhelm prepare_task-assisted exploration
  3. ctxhelm prepare_task + get_pack-assisted exploration

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

if [[ "${#target_files[@]}" -eq 0 ]]; then
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
        "readFileCount": 0,
        "irrelevantReadCount": 0,
        "toolCallCount": 0,
        "ctxhelmToolCallCount": 0,
    },
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
read_files = []
discovered_files = []
result_success = False

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
                tool_calls.append({"name": name, "inputKeys": sorted(tool_input.keys())})
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
target_coverage = len(target_hits) / len(targets) if targets else 0.0
irrelevant_reads = sorted(path for path in read_files if path not in targets)
stderr_hash = hashlib.sha256(stderr_file.read_bytes()).hexdigest() if stderr_file.exists() else None
events_hash = hashlib.sha256(events_file.read_bytes()).hexdigest() if events_file.exists() else None
request_hash = hashlib.sha256(request_file.read_bytes()).hexdigest() if request_file.exists() else None

payload = {
    "lane": lane,
    "mode": mode,
    "status": "passed" if client_status == 0 and result_success else "failed",
    "clientExitStatus": client_status,
    "metrics": {
        "targetCoverage": target_coverage,
        "targetHitCount": len(target_hits),
        "targetCount": len(targets),
        "readFileCount": len(read_files),
        "discoveredFileCount": len(discovered_files),
        "irrelevantReadCount": len(irrelevant_reads),
        "toolCallCount": len(tool_calls),
        "ctxhelmToolCallCount": len(ctxhelm_calls),
    },
    "targetHits": target_hits,
    "readFiles": read_files,
    "discoveredFiles": discovered_files,
    "irrelevantReads": irrelevant_reads,
    "toolCalls": tool_calls,
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
        lane.get("metrics", {}).get("targetCoverage", 0.0),
        -lane.get("metrics", {}).get("irrelevantReadCount", 999_999),
        lane.get("metrics", {}).get("ctxhelmToolCallCount", 0),
    ),
)
base_metrics = baseline.get("metrics", {})
best_metrics = best.get("metrics", {})
target_delta = best_metrics.get("targetCoverage", 0.0) - base_metrics.get("targetCoverage", 0.0)
irrelevant_delta = base_metrics.get("irrelevantReadCount", 0) - best_metrics.get("irrelevantReadCount", 0)
ctxhelm_lanes = [lane for lane in lanes if lane.get("mode") in {"plan", "brief"}]
ctxhelm_called = any(lane.get("metrics", {}).get("ctxhelmToolCallCount", 0) > 0 for lane in ctxhelm_lanes)
status = "passed" if any(lane.get("status") == "passed" for lane in lanes) else "skipped"
if status == "passed" and not ctxhelm_called:
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
        "irrelevantReadDelta": irrelevant_delta,
        "ctxhelmToolCallsObserved": ctxhelm_called,
        "outcomeClaim": (
            "ctxhelm_improved"
            if ctxhelm_called and (target_delta > 0 or irrelevant_delta > 0)
            else (
                "ctxhelm_matched"
                if ctxhelm_called and target_delta == 0 and irrelevant_delta == 0
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
