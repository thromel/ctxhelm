#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxhelm_root="${CTXHELM_ROOT:-$(cd "$script_dir/.." && pwd -P)}"
protocol_script="$ctxhelm_root/scripts/smoke-mcp-protocol.sh"
repo_input="${CTXHELM_SMOKE_REPO:-$PWD}"
task="${CTXHELM_SMOKE_TASK:-fix requireSession auth bug}"
anchor_path="${CTXHELM_SMOKE_PATH:-crates/ctxhelm-mcp/src/lib.rs}"
query="${CTXHELM_SMOKE_QUERY:-prepare_task}"
require_real="${CTXHELM_REQUIRE_REAL_CLIENT:-0}"
run_real="${CTXHELM_RUN_REAL_CLIENT:-0}"

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

fail_or_skip() {
  local reason="$1"
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" && -n "${repo:-}" && -n "${ctxhelm_version:-}" ]]; then
    write_skip_evidence \
      "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/codex-mcp-evidence.json" \
      "$reason" \
      "${request_log:-}"
  fi
  if [[ "$require_real" == "1" ]]; then
    echo "ctxhelm Codex MCP smoke failed: $reason" >&2
    exit 1
  fi
  echo "ctxhelm Codex MCP smoke skipped: $reason"
  exit 0
}

write_skip_evidence() {
  local evidence_file="$1"
  local reason="$2"
  local request_log_path="${3:-}"
  python3 - "$evidence_file" "$repo" "${client_version:-unavailable}" "$ctxhelm_version" "$require_real" "$reason" "$request_log_path" <<'PY'
import hashlib
import json
import pathlib
import sys

path, repo, client_version, ctxhelm_version, required, reason, request_log_path = sys.argv[1:]

def request_log_summary(log_path, expected_repo):
    raw = b""
    if log_path:
        candidate = pathlib.Path(log_path)
        if candidate.exists():
            raw = candidate.read_bytes()
    lines = raw.decode("utf-8", errors="replace").splitlines()
    observed = []
    explicit_repo_tool_call_count = 0
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
        "requestEvidenceSchemaVersion": "ctxhelm-real-client-evidence-v2",
        "serverSideRequestLog": bool(raw),
        "requestLogSha256": hashlib.sha256(raw).hexdigest(),
        "requestLogLineCount": len(lines),
        "explicitRepoToolCallCount": explicit_repo_tool_call_count,
        "observedToolCalls": observed,
    }

summary = request_log_summary(request_log_path, repo)
evidence = {
    "client": "codex",
    "clientVersion": client_version,
    "ctxhelmVersion": ctxhelm_version,
    "repo": repo,
    "status": "skipped",
    "skipReason": reason,
    "deterministicProtocol": True,
    "deterministicContextAreaResourceRead": True,
    "prepareTask": False,
    "getPack": False,
    "required": required == "1",
}
evidence.update(summary)
target = pathlib.Path(path)
target.parent.mkdir(parents=True, exist_ok=True)
summary_target = target.with_name(target.name.replace("-evidence", "-request-summary"))
if summary_target == target:
    summary_target = target.with_name(target.stem + "-request-summary" + target.suffix)
summary_target.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")
evidence["requestSummaryFile"] = summary_target.name
target.write_text(json.dumps(evidence, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_evidence() {
  local evidence_file="$1"
  local request_log_path="$2"
  python3 - "$evidence_file" "$repo" "$client_version" "$ctxhelm_version" "$require_real" "$request_log_path" <<'PY'
import hashlib
import json
import pathlib
import sys

path, repo, client_version, ctxhelm_version, required, request_log_path = sys.argv[1:]

def request_log_summary(log_path, expected_repo):
    raw = b""
    if log_path:
        candidate = pathlib.Path(log_path)
        if candidate.exists():
            raw = candidate.read_bytes()
    lines = raw.decode("utf-8", errors="replace").splitlines()
    observed = []
    explicit_repo_tool_call_count = 0
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
        "requestEvidenceSchemaVersion": "ctxhelm-real-client-evidence-v2",
        "serverSideRequestLog": True,
        "requestLogSha256": hashlib.sha256(raw).hexdigest(),
        "requestLogLineCount": len(lines),
        "explicitRepoToolCallCount": explicit_repo_tool_call_count,
        "observedToolCalls": observed,
    }

summary = request_log_summary(request_log_path, repo)
evidence = {
    "client": "codex",
    "clientVersion": client_version,
    "ctxhelmVersion": ctxhelm_version,
    "repo": repo,
    "status": "passed",
    "deterministicProtocol": True,
    "deterministicContextAreaResourceRead": True,
    "prepareTask": True,
    "getPack": True,
    "required": required == "1",
}
evidence.update(summary)
payload = json.dumps(evidence, sort_keys=True)
if path:
    target = pathlib.Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    summary_target = target.with_name(target.name.replace("-evidence", "-request-summary"))
    if summary_target == target:
        summary_target = target.with_name(target.stem + "-request-summary" + target.suffix)
    summary_target.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")
    evidence["requestSummaryFile"] = summary_target.name
    payload = json.dumps(evidence, sort_keys=True)
    target.write_text(payload + "\n", encoding="utf-8")
else:
    print("ctxhelm Codex MCP smoke evidence: " + payload)
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

echo "ctxhelm Codex MCP smoke: running deterministic protocol gate"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$ctxhelm_root" \
  CTXHELM_SMOKE_REPO="$repo" \
  CTXHELM_SMOKE_TASK="$task" \
  CTXHELM_SMOKE_PATH="$anchor_path" \
  CTXHELM_SMOKE_QUERY="$query" \
  CTXHELM_HOME="$ctxhelm_home" \
  bash "$protocol_script"

if [[ "${CTXHELM_SKIP_REAL_CLIENT:-0}" == "1" ]]; then
  reason="CTXHELM_SKIP_REAL_CLIENT=1 after protocol gate passed"
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
    write_skip_evidence "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/codex-mcp-evidence.json" "$reason" ""
  fi
  echo "ctxhelm Codex MCP smoke skipped: $reason"
  exit 0
fi

if [[ "$require_real" != "1" && "$run_real" != "1" ]]; then
  reason="set CTXHELM_RUN_REAL_CLIENT=1 or CTXHELM_REQUIRE_REAL_CLIENT=1 after protocol gate passed"
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
    write_skip_evidence "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/codex-mcp-evidence.json" "$reason" ""
  fi
  echo "ctxhelm Codex MCP smoke skipped: $reason"
  exit 0
fi

if ! command -v codex >/dev/null 2>&1; then
  fail_or_skip "codex is not installed"
fi

client_version="$(codex --version 2>&1 | head -n 1)"
request_log="$work_dir/ctxhelm-mcp-requests.jsonl"
events="$work_dir/codex-events.jsonl"
last_message="$work_dir/codex-last-message.txt"
stderr_log="$work_dir/codex-stderr.log"
server_wrapper="$work_dir/ctxhelm-mcp-server.sh"
outside_cwd="$work_dir/outside-repo"
mkdir -p "$outside_cwd"

{
  printf '%s\n' '#!/usr/bin/env bash'
  printf '%s\n' 'set -euo pipefail'
  printf 'export CTXHELM_HOME=%q\n' "$ctxhelm_home"
  printf 'export CTXHELM_REAL_CLIENT_REQUEST_LOG=%q\n' "$request_log"
  printf 'ctxhelm_bin=%q\n' "$ctxhelm_bin"
  printf '%s\n' 'tee -a "$CTXHELM_REAL_CLIENT_REQUEST_LOG" | "$ctxhelm_bin" serve-mcp'
} >"$server_wrapper"
chmod +x "$server_wrapper"

prompt=$(cat <<EOF
Use the ctxhelm MCP server. Call prepare_task first with explicit repo "$repo" and task "$task".
Then call get_pack with the same explicit repo "$repo", the same task, budget "brief", format "json", and recordTrace false.
Do not use shell commands for this smoke. The smoke requires machine-checkable tool-call evidence for prepare_task and get_pack with the repo argument.
EOF
)

codex_exec_help="$(codex exec --help 2>&1 || true)"
codex_exec_args=(exec)
codex_env=(env)
if [[ "$codex_exec_help" == *"--ephemeral"* ]]; then
  codex_exec_args+=(--ephemeral)
fi
if [[ "$codex_exec_help" == *"--ignore-user-config"* ]]; then
  codex_exec_args+=(--ignore-user-config)
else
  codex_compat_home="$work_dir/codex-home"
  mkdir -p "$codex_compat_home"
  codex_env+=(CODEX_HOME="$codex_compat_home")
fi
codex_exec_args+=(
  --skip-git-repo-check
  --cd "$outside_cwd"
  --dangerously-bypass-approvals-and-sandbox
  --json
  --output-last-message "$last_message"
  -c "mcp_servers.ctxhelm.command=\"$server_wrapper\""
  -c "mcp_servers.ctxhelm.args=[]"
  -c "mcp_servers.ctxhelm.cwd=\"$outside_cwd\""
  -c "mcp_servers.ctxhelm.startup_timeout_sec=30"
  -c "mcp_servers.ctxhelm.tool_timeout_sec=120"
  "$prompt"
)

set +e
"${codex_env[@]}" codex "${codex_exec_args[@]}" >"$events" 2>"$stderr_log"
client_status=$?
set -e

if python3 - "$request_log" "$repo" <<'PY'
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
then
  evidence_path=""
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
    evidence_path="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/codex-mcp-evidence.json"
  fi
  write_evidence "$evidence_path" "$request_log"
  echo "ctxhelm Codex MCP smoke passed: server-side instrumentation recorded prepare_task and get_pack with repo=$repo"
  exit 0
fi

reason="codex did not produce machine-checkable prepare_task/get_pack evidence"
if [[ "$client_status" != "0" ]]; then
  reason="$reason (codex exit $client_status; auth/model/client refusal is optional unless CTXHELM_REQUIRE_REAL_CLIENT=1)"
fi
if [[ -s "$stderr_log" ]]; then
  echo "ctxhelm Codex MCP smoke diagnostic stderr:" >&2
  tail -n 40 "$stderr_log" >&2
fi
fail_or_skip "$reason"
