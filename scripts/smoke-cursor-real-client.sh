#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxhelm_root="${CTXHELM_ROOT:-$(cd "$script_dir/.." && pwd -P)}"
protocol_script="$ctxhelm_root/scripts/smoke-mcp-protocol.sh"
repo_input="${CTXHELM_SMOKE_REPO:-$PWD}"
task="${CTXHELM_SMOKE_TASK:-fix requireSession auth bug}"
anchor_path="${CTXHELM_SMOKE_PATH:-crates/ctxhelm-mcp/src/lib.rs}"
query="${CTXHELM_SMOKE_QUERY:-prepare_task}"
client_timeout_seconds="${CTXHELM_REAL_CLIENT_TIMEOUT_SECONDS:-180}"
require_real="${CTXHELM_REQUIRE_CURSOR_REAL_CLIENT:-${CTXHELM_REQUIRE_REAL_CLIENT:-0}}"
run_real="${CTXHELM_RUN_CURSOR_REAL_CLIENT:-${CTXHELM_RUN_REAL_CLIENT:-0}}"

resolve_ctxhelm_bin() {
  if [[ -n "${CTXHELM_BIN:-}" ]]; then
    if [[ ! "$CTXHELM_BIN" = /* ]]; then
      echo "CTXHELM_BIN must be absolute: $CTXHELM_BIN" >&2
      exit 64
    fi
    if [[ ! -x "$CTXHELM_BIN" ]]; then
      echo "CTXHELM_BIN is not executable: $CTXHELM_BIN" >&2
      exit 64
    fi
    printf '%s/%s\n' "$(cd "$(dirname "$CTXHELM_BIN")" && pwd -P)" "$(basename "$CTXHELM_BIN")"
    return
  fi
  cargo build -p ctxhelm >/dev/null
  printf '%s/target/debug/ctxhelm\n' "$ctxhelm_root"
}

write_evidence_json() {
  local evidence_file="$1"
  local status="$2"
  local reason="$3"
  local request_log_path="${4:-}"
  local client_exit_status="${5:-}"
  python3 - "$evidence_file" "$repo" "${client_version:-unavailable}" "$ctxhelm_version" "$require_real" "$status" "$reason" "$request_log_path" "$client_exit_status" <<'PY'
import hashlib
import json
import pathlib
import sys

path, repo, client_version, ctxhelm_version, required, status, reason, request_log_path, client_exit_status = sys.argv[1:]

def parse_exit_status(value):
    if not value:
        return None
    try:
        return int(value)
    except ValueError:
        return None

def request_log_summary(log_path, expected_repo):
    raw = b""
    if log_path:
        candidate = pathlib.Path(log_path)
        if candidate.exists():
            raw = candidate.read_bytes()
    lines = raw.decode("utf-8", errors="replace").splitlines()
    method_counts = {}
    observed = []
    explicit_repo_tool_call_count = 0
    for line in lines:
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        method = payload.get("method")
        if method:
            method_counts[method] = method_counts.get(method, 0) + 1
        if method != "tools/call":
            continue
        params = payload.get("params") or {}
        name = params.get("name")
        if name not in {"prepare_task", "get_pack"}:
            continue
        arguments = params.get("arguments") or {}
        repo_matched = arguments.get("repo") == expected_repo
        if repo_matched:
            explicit_repo_tool_call_count += 1
        entry = {
            "name": name,
            "repoMatched": repo_matched,
            "hasTask": bool(arguments.get("task")),
        }
        if name == "get_pack":
            entry["budget"] = arguments.get("budget")
            entry["format"] = arguments.get("format")
            entry["recordTraceFalse"] = arguments.get("recordTrace") is False
        observed.append(entry)
    return {
        "requestEvidenceSchemaVersion": "ctxhelm-cursor-real-client-evidence-v1",
        "serverSideRequestLog": bool(raw),
        "requestLogSha256": hashlib.sha256(raw).hexdigest(),
        "requestLogLineCount": len(lines),
        "methodCounts": method_counts,
        "initializeRequested": method_counts.get("initialize", 0) > 0,
        "toolsListRequested": method_counts.get("tools/list", 0) > 0,
        "explicitRepoToolCallCount": explicit_repo_tool_call_count,
        "observedToolCalls": observed,
    }

summary = request_log_summary(request_log_path, repo)
prepare = any(call.get("name") == "prepare_task" and call.get("repoMatched") for call in summary["observedToolCalls"])
pack = any(call.get("name") == "get_pack" and call.get("repoMatched") for call in summary["observedToolCalls"])
evidence = {
    "client": "cursor",
    "clientVersion": client_version,
    "ctxhelmVersion": ctxhelm_version,
    "repo": repo,
    "status": status,
    "skipReason": reason if status == "skipped" else None,
    "deterministicProtocol": True,
    "deterministicContextAreaResourceRead": True,
    "realClientToolCalls": prepare and pack,
    "prepareTask": prepare,
    "getPack": pack,
    "required": required == "1",
    "clientExitStatus": parse_exit_status(client_exit_status),
    "proofBoundary": "Cursor Agent CLI real-client proof is source-free and passes only when server-side instrumentation records prepare_task and get_pack with the explicit repo.",
}
evidence.update(summary)
if evidence["skipReason"] is None:
    evidence.pop("skipReason")
payload = json.dumps(evidence, sort_keys=True)
if path:
    target = pathlib.Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    summary_target = target.with_name(target.name.replace("-evidence", "-request-summary"))
    if summary_target == target:
        summary_target = target.with_name(target.stem + "-request-summary" + target.suffix)
    summary_target.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")
    evidence["requestSummaryFile"] = summary_target.name
    target.write_text(json.dumps(evidence, sort_keys=True) + "\n", encoding="utf-8")
else:
    print("ctxhelm Cursor real-client evidence: " + payload)
PY
}

fail_or_skip() {
  local reason="$1"
  local request_log_path="${2:-}"
  local client_status="${3:-}"
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
    write_evidence_json "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/cursor-real-client-evidence.json" "skipped" "$reason" "$request_log_path" "$client_status"
  fi
  if [[ "$require_real" == "1" ]]; then
    echo "ctxhelm Cursor real-client smoke failed: $reason" >&2
    exit 1
  fi
  echo "ctxhelm Cursor real-client smoke skipped: $reason"
  exit 0
}

check_request_evidence() {
  python3 - "$request_log" "$repo" <<'PY'
import json
import sys

log_path, expected_repo = sys.argv[1:]
seen = {"prepare_task": False, "get_pack": False}
try:
    lines = open(log_path, encoding="utf-8").read().splitlines()
except FileNotFoundError:
    lines = []
for line in lines:
    if not line.strip():
        continue
    try:
        payload = json.loads(line)
    except json.JSONDecodeError:
        continue
    if payload.get("method") != "tools/call":
        continue
    params = payload.get("params") or {}
    name = params.get("name")
    arguments = params.get("arguments") or {}
    if name in seen and arguments.get("repo") == expected_repo:
        seen[name] = True
missing = [name for name, found in seen.items() if not found]
if missing:
    raise SystemExit("missing explicit-repo tool calls: " + ", ".join(missing))
PY
}

repo="$(cd "$repo_input" && pwd -P)"
ctxhelm_bin="$(resolve_ctxhelm_bin)"
ctxhelm_home="$(mktemp -d)"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$ctxhelm_home" "$work_dir"
}
trap cleanup EXIT

ctxhelm_version="$("$ctxhelm_bin" --version)"

echo "ctxhelm Cursor real-client smoke: running deterministic protocol gate"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$ctxhelm_root" \
  CTXHELM_SMOKE_REPO="$repo" \
  CTXHELM_SMOKE_TASK="$task" \
  CTXHELM_SMOKE_PATH="$anchor_path" \
  CTXHELM_SMOKE_QUERY="$query" \
  CTXHELM_HOME="$ctxhelm_home" \
  bash "$protocol_script"

if [[ "${CTXHELM_SKIP_REAL_CLIENT:-0}" == "1" ]]; then
  fail_or_skip "CTXHELM_SKIP_REAL_CLIENT=1 after protocol gate passed"
fi
if [[ "$require_real" != "1" && "$run_real" != "1" ]]; then
  fail_or_skip "set CTXHELM_RUN_CURSOR_REAL_CLIENT=1 or CTXHELM_REQUIRE_CURSOR_REAL_CLIENT=1 after protocol gate passed"
fi
if ! command -v cursor >/dev/null 2>&1; then
  fail_or_skip "cursor is not installed"
fi

client_version="$(cursor --version 2>&1 | head -n 1)"
if cursor agent status 2>&1 | grep -qi "not logged in"; then
  fail_or_skip "cursor agent is not logged in"
fi

request_log="$work_dir/ctxhelm-mcp-requests.jsonl"
events="$work_dir/cursor-events.jsonl"
stderr_log="$work_dir/cursor-stderr.log"
server_wrapper="$work_dir/ctxhelm-mcp-server.sh"
cursor_workspace="$work_dir/cursor-workspace"
mkdir -p "$cursor_workspace/.cursor"

{
  printf '%s\n' '#!/usr/bin/env bash'
  printf '%s\n' 'set -euo pipefail'
  printf 'export CTXHELM_HOME=%q\n' "$ctxhelm_home"
  printf 'export CTXHELM_REAL_CLIENT_REQUEST_LOG=%q\n' "$request_log"
  printf 'ctxhelm_bin=%q\n' "$ctxhelm_bin"
  printf '%s\n' 'tee -a "$CTXHELM_REAL_CLIENT_REQUEST_LOG" | "$ctxhelm_bin" serve-mcp'
} >"$server_wrapper"
chmod +x "$server_wrapper"

python3 - "$cursor_workspace/.cursor/mcp.json" "$server_wrapper" <<'PY'
import json
import sys

path, command = sys.argv[1:]
with open(path, "w", encoding="utf-8") as handle:
    json.dump({"mcpServers": {"ctxhelm": {"command": command, "args": []}}}, handle)
PY

prompt=$(cat <<EOF
Use the ctxhelm MCP server. Call prepare_task first with explicit repo "$repo" and task "$task".
Then call get_pack with the same explicit repo "$repo", the same task, budget "brief", format "json", and recordTrace false.
Do not use shell commands. This smoke requires machine-checkable MCP tool calls.
EOF
)

set +e
cursor agent --print \
  --output-format stream-json \
  --mode plan \
  --approve-mcps \
  --trust \
  --workspace "$cursor_workspace" \
  "$prompt" >"$events" 2>"$stderr_log" &
client_pid=$!
client_status=0
evidence_found=0
deadline=$((SECONDS + client_timeout_seconds))
while kill -0 "$client_pid" 2>/dev/null; do
  if check_request_evidence >/dev/null 2>&1; then
    evidence_found=1
    kill "$client_pid" >/dev/null 2>&1 || true
    wait "$client_pid" >/dev/null 2>&1 || true
    client_status=0
    break
  fi
  if (( SECONDS >= deadline )); then
    kill "$client_pid" >/dev/null 2>&1 || true
    wait "$client_pid" >/dev/null 2>&1 || true
    client_status=124
    break
  fi
  sleep 2
done
if [[ "$evidence_found" != "1" && "$client_status" == "0" ]]; then
  wait "$client_pid"
  client_status=$?
  if check_request_evidence >/dev/null 2>&1; then
    evidence_found=1
  fi
fi
set -e

if [[ "$evidence_found" == "1" ]]; then
  evidence_path=""
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
    evidence_path="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/cursor-real-client-evidence.json"
  fi
  write_evidence_json "$evidence_path" "passed" "" "$request_log" "$client_status"
  echo "ctxhelm Cursor real-client smoke passed: server-side instrumentation recorded prepare_task and get_pack with repo=$repo"
  exit 0
fi

if [[ -s "$stderr_log" ]]; then
  echo "ctxhelm Cursor real-client smoke diagnostic stderr:" >&2
  tail -n 40 "$stderr_log" >&2
fi
fail_or_skip "cursor did not produce machine-checkable prepare_task/get_pack evidence" "$request_log" "$client_status"
