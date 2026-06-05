#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: e2e-agent-client-availability.sh [--repo PATH] [--task TASK] [--output PATH]

Checks whether installed real-agent clients can currently produce source-free
ctxhelm evidence:
  - Claude Code: lightweight no-tool preflight for client/model availability.
  - Codex CLI: existing MCP smoke requiring machine-checkable prepare_task and
    get_pack calls with explicit repo arguments.

The report is source-free. It stores hashes/counts/status only, not raw prompts,
source text, stderr, transcripts, or MCP traffic.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
repo_input="${CTXHELM_AGENT_AVAILABILITY_REPO:-$PWD}"
task="${CTXHELM_AGENT_AVAILABILITY_TASK:-Improve paired agent-run lane matrix}"
output_path="${CTXHELM_AGENT_AVAILABILITY_REPORT:-}"
client_timeout_seconds="${CTXHELM_AGENT_AVAILABILITY_TIMEOUT_SECONDS:-30}"

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

claude_report="$work_dir/claude-availability.json"
codex_report="$work_dir/codex-availability.json"

run_claude_preflight() {
  if ! command -v claude >/dev/null 2>&1; then
    python3 - "$claude_report" "$ctxhelm_version" <<'PY'
import json
import pathlib
import sys

path, ctxhelm_version = sys.argv[1:]
payload = {
    "client": "claude",
    "clientVersion": "unavailable",
    "ctxhelmVersion": ctxhelm_version,
    "status": "unavailable",
    "realClientReady": False,
    "clientExitStatus": None,
    "clientFailureKind": "missing_client",
    "clientApiErrorStatus": None,
    "rateLimitObserved": False,
    "deterministicProtocol": None,
    "explicitRepoToolCallCount": None,
    "observedToolCalls": [],
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
}
pathlib.Path(path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
    return
  fi

  local version
  version="$(claude --version 2>&1 | head -n 1)"
  local preflight_dir="$work_dir/claude"
  mkdir -p "$preflight_dir"
  local events="$preflight_dir/events.jsonl"
  local stderr_log="$preflight_dir/stderr.log"
  local prompt='Return a short JSON object {"ok":true}. Do not use tools.'

  set +e
  (
    cd "$repo"
    claude -p \
      --no-session-persistence \
      --allowedTools Read \
      --permission-mode bypassPermissions \
      --verbose \
      --output-format stream-json \
      "$prompt"
  ) >"$events" 2>"$stderr_log" &
  local pid=$!
  local exit_status=0
  local deadline=$((SECONDS + client_timeout_seconds))
  while kill -0 "$pid" 2>/dev/null; do
    if (( SECONDS >= deadline )); then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
      exit_status=124
      break
    fi
    sleep 2
  done
  if [[ "$exit_status" == "0" ]]; then
    wait "$pid"
    exit_status=$?
  fi
  set -e

  python3 - "$claude_report" "$version" "$ctxhelm_version" "$events" "$stderr_log" "$exit_status" <<'PY'
import hashlib
import json
import pathlib
import sys

path, version, ctxhelm_version, events_path, stderr_path, exit_status_text = sys.argv[1:]
events_file = pathlib.Path(events_path)
stderr_file = pathlib.Path(stderr_path)
exit_status = int(exit_status_text)
failure_kind = None
api_status = None
rate_limited = False
success = False

if events_file.exists():
    for line in events_file.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("type") == "result" and payload.get("subtype") == "success":
            success = True
        if payload.get("type") == "rate_limit_event":
            rate_limited = True
            failure_kind = "rate_limited"
        if payload.get("type") == "result" and payload.get("is_error"):
            value = payload.get("api_error_status")
            if isinstance(value, int):
                api_status = value
            if value == 429:
                rate_limited = True
                failure_kind = "rate_limited"
            elif failure_kind is None:
                failure_kind = "api_error" if value else "client_error"

if exit_status == 124:
    failure_kind = failure_kind or "timeout"
elif exit_status != 0 and failure_kind is None:
    failure_kind = "client_exit_nonzero"

ready = exit_status == 0 and success and failure_kind is None
payload = {
    "client": "claude",
    "clientVersion": version,
    "ctxhelmVersion": ctxhelm_version,
    "status": "available" if ready else "unavailable",
    "realClientReady": ready,
    "clientExitStatus": exit_status,
    "clientFailureKind": failure_kind,
    "clientApiErrorStatus": api_status,
    "rateLimitObserved": rate_limited,
    "deterministicProtocol": None,
    "explicitRepoToolCallCount": None,
    "observedToolCalls": [],
    "evidenceHashes": {
        "streamJsonSha256": hashlib.sha256(events_file.read_bytes()).hexdigest() if events_file.exists() else None,
        "stderrSha256": hashlib.sha256(stderr_file.read_bytes()).hexdigest() if stderr_file.exists() else None,
    },
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
}
pathlib.Path(path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
}

run_codex_smoke() {
  local evidence_dir="$work_dir/codex-evidence"
  mkdir -p "$evidence_dir"
  local stdout_log="$work_dir/codex-stdout.log"
  local stderr_log="$work_dir/codex-stderr.log"
  set +e
  CTXHELM_BIN="$ctxhelm_bin" \
    CTXHELM_RUN_REAL_CLIENT=1 \
    CTXHELM_REAL_CLIENT_EVIDENCE_DIR="$evidence_dir" \
    CTXHELM_SMOKE_REPO="$repo" \
    CTXHELM_SMOKE_TASK="$task" \
    CTXHELM_SMOKE_PATH="scripts/e2e-agent-run.sh" \
    "$script_dir/smoke-codex-mcp.sh" >"$stdout_log" 2>"$stderr_log"
  local smoke_status=$?
  set -e

  local evidence_file="$evidence_dir/codex-mcp-evidence.json"
  python3 - "$codex_report" "$ctxhelm_version" "$evidence_file" "$smoke_status" "$stdout_log" "$stderr_log" <<'PY'
import hashlib
import json
import pathlib
import sys

path, ctxhelm_version, evidence_path, smoke_status_text, stdout_path, stderr_path = sys.argv[1:]
evidence_file = pathlib.Path(evidence_path)
stdout_file = pathlib.Path(stdout_path)
stderr_file = pathlib.Path(stderr_path)
smoke_status = int(smoke_status_text)

if evidence_file.exists():
    evidence = json.loads(evidence_file.read_text(encoding="utf-8"))
else:
    evidence = {}

observed = evidence.get("observedToolCalls") or []
explicit_count = int(evidence.get("explicitRepoToolCallCount") or 0)
has_prepare = bool(evidence.get("prepareTask"))
has_pack = bool(evidence.get("getPack"))
ready = smoke_status == 0 and evidence.get("status") == "passed" and has_prepare and has_pack
failure_kind = evidence.get("clientFailureKind")
if not ready and not failure_kind:
    failure_kind = "tool_call_missing" if evidence else "unknown"

payload = {
    "client": "codex",
    "clientVersion": evidence.get("clientVersion") or "unavailable",
    "ctxhelmVersion": ctxhelm_version,
    "status": "available" if ready else "unavailable",
    "realClientReady": ready,
    "clientExitStatus": evidence.get("clientExitStatus"),
    "clientFailureKind": failure_kind,
    "clientApiErrorStatus": None,
    "rateLimitObserved": failure_kind == "rate_limited",
    "deterministicProtocol": bool(evidence.get("deterministicProtocol", False)),
    "deterministicContextAreaResourceRead": bool(evidence.get("deterministicContextAreaResourceRead", False)),
    "prepareTask": has_prepare,
    "getPack": has_pack,
    "explicitRepoToolCallCount": explicit_count,
    "methodCounts": evidence.get("methodCounts", {}),
    "observedToolCalls": observed,
    "evidenceHashes": {
        "evidenceSha256": hashlib.sha256(evidence_file.read_bytes()).hexdigest() if evidence_file.exists() else None,
        "stdoutSha256": hashlib.sha256(stdout_file.read_bytes()).hexdigest() if stdout_file.exists() else None,
        "stderrSha256": hashlib.sha256(stderr_file.read_bytes()).hexdigest() if stderr_file.exists() else None,
    },
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
}
pathlib.Path(path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
}

run_claude_preflight
run_codex_smoke

python3 - "$repo" "$task" "$ctxhelm_version" "$claude_report" "$codex_report" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys

repo, task, ctxhelm_version, claude_path, codex_path, output_path = sys.argv[1:]
clients = [
    json.loads(pathlib.Path(claude_path).read_text(encoding="utf-8")),
    json.loads(pathlib.Path(codex_path).read_text(encoding="utf-8")),
]
ready = [client for client in clients if client.get("realClientReady")]
rate_limited = [client for client in clients if client.get("rateLimitObserved")]
stream_disconnected = [
    client for client in clients if client.get("clientFailureKind") == "stream_disconnected"
]

recommended = []
if ready:
    recommended.append("run_real_agent_outcome_matrix")
else:
    if rate_limited:
        recommended.append("retry_real_client_when_available")
    if stream_disconnected:
        recommended.append("upgrade_or_reconfigure_codex_cli")
    if not recommended:
        recommended.append("inspect_real_client_configuration")

payload = {
    "schemaVersion": "ctxhelm-agent-client-availability-v1",
    "status": "passed" if ready else "degraded",
    "workflowKind": "agent-client-availability",
    "ctxhelmVersion": ctxhelm_version,
    "repo": {
        "label": pathlib.Path(repo).name,
        "pathSha256": hashlib.sha256(repo.encode("utf-8")).hexdigest(),
    },
    "task": {
        "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
        "rawTaskStored": False,
    },
    "summary": {
        "clientCount": len(clients),
        "readyClientCount": len(ready),
        "unavailableClientCount": len(clients) - len(ready),
        "rateLimitedClientCount": len(rate_limited),
        "streamDisconnectedClientCount": len(stream_disconnected),
        "realAgentOutcomeCurrentlyRunnable": bool(ready),
        "recommendedResearchActions": recommended,
    },
    "clients": clients,
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

echo "agent client availability wrote ${output_path:-stdout}"
