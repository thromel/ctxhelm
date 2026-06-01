#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxhelm_root="${CTXHELM_ROOT:-$(cd "$script_dir/.." && pwd -P)}"
protocol_script="$ctxhelm_root/scripts/smoke-mcp-protocol.sh"
repo_input="${CTXHELM_SMOKE_REPO:-$PWD}"
task="${CTXHELM_SMOKE_TASK:-fix requireSession auth bug}"
anchor_path="${CTXHELM_SMOKE_PATH:-crates/ctxhelm-mcp/src/lib.rs}"
query="${CTXHELM_SMOKE_QUERY:-prepare_task}"
semantic="${CTXHELM_SMOKE_SEMANTIC:-0}"
semantic_provider="${CTXHELM_SMOKE_SEMANTIC_PROVIDER:-}"
semantic_model="${CTXHELM_SMOKE_SEMANTIC_MODEL:-}"
semantic_dimensions="${CTXHELM_SMOKE_SEMANTIC_DIMENSIONS:-}"
client_timeout_seconds="${CTXHELM_REAL_CLIENT_TIMEOUT_SECONDS:-180}"
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
      "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/claude-mcp-evidence.json" \
      "$reason" \
      "${request_log:-}"
  fi
  if [[ "$require_real" == "1" ]]; then
    echo "ctxhelm Claude MCP smoke failed: $reason" >&2
    exit 1
  fi
  echo "ctxhelm Claude MCP smoke skipped: $reason"
  exit 0
}

write_skip_evidence() {
  local evidence_file="$1"
  local reason="$2"
  local request_log_path="${3:-}"
  python3 - "$evidence_file" "$repo" "${client_version:-unavailable}" "$ctxhelm_version" "$require_real" "$semantic" "$semantic_provider" "$semantic_model" "$semantic_dimensions" "$reason" "$request_log_path" <<'PY'
import hashlib
import json
import pathlib
import sys

path, repo, client_version, ctxhelm_version, required, semantic, provider, model, dimensions, reason, request_log_path = sys.argv[1:]

def semantic_matches(arguments):
    if semantic != "1":
        return None
    if arguments.get("semantic") is not True:
        return False
    if provider and arguments.get("semanticProvider") != provider:
        return False
    if model and arguments.get("semanticModel") != model:
        return False
    if dimensions:
        try:
            actual_dimensions = int(arguments.get("semanticDimensions"))
        except (TypeError, ValueError):
            return False
        if actual_dimensions != int(dimensions):
            return False
    return True

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
        matched = semantic_matches(arguments)
        if matched is not None:
            entry["semanticMatched"] = matched
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
    "client": "claude",
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
if semantic == "1":
    evidence["semantic"] = True
    if provider:
        evidence["semanticProvider"] = provider
    if model:
        evidence["semanticModel"] = model
    if dimensions:
        evidence["semanticDimensions"] = int(dimensions)
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
  python3 - "$evidence_file" "$repo" "$client_version" "$ctxhelm_version" "$require_real" "$semantic" "$semantic_provider" "$semantic_model" "$semantic_dimensions" "$request_log_path" <<'PY'
import hashlib
import json
import pathlib
import sys

path, repo, client_version, ctxhelm_version, required, semantic, provider, model, dimensions, request_log_path = sys.argv[1:]

def semantic_matches(arguments):
    if semantic != "1":
        return None
    if arguments.get("semantic") is not True:
        return False
    if provider and arguments.get("semanticProvider") != provider:
        return False
    if model and arguments.get("semanticModel") != model:
        return False
    if dimensions:
        try:
            actual_dimensions = int(arguments.get("semanticDimensions"))
        except (TypeError, ValueError):
            return False
        if actual_dimensions != int(dimensions):
            return False
    return True

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
        matched = semantic_matches(arguments)
        if matched is not None:
            entry["semanticMatched"] = matched
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
    "client": "claude",
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
if semantic == "1":
    evidence["semantic"] = True
    if provider:
        evidence["semanticProvider"] = provider
    if model:
        evidence["semanticModel"] = model
    if dimensions:
        evidence["semanticDimensions"] = int(dimensions)
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
    print("ctxhelm Claude MCP smoke evidence: " + payload)
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

echo "ctxhelm Claude MCP smoke: running deterministic protocol gate"
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
    write_skip_evidence "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/claude-mcp-evidence.json" "$reason" ""
  fi
  echo "ctxhelm Claude MCP smoke skipped: $reason"
  exit 0
fi

if [[ "$require_real" != "1" && "$run_real" != "1" ]]; then
  reason="set CTXHELM_RUN_REAL_CLIENT=1 or CTXHELM_REQUIRE_REAL_CLIENT=1 after protocol gate passed"
  if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
    write_skip_evidence "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/claude-mcp-evidence.json" "$reason" ""
  fi
  echo "ctxhelm Claude MCP smoke skipped: $reason"
  exit 0
fi

if ! command -v claude >/dev/null 2>&1; then
  fail_or_skip "claude is not installed"
fi

client_version="$(claude --version 2>&1 | head -n 1)"
request_log="$work_dir/ctxhelm-mcp-requests.jsonl"
events="$work_dir/claude-stream.jsonl"
stderr_log="$work_dir/claude-stderr.log"
server_wrapper="$work_dir/ctxhelm-mcp-server.sh"
mcp_config="$work_dir/claude-mcp.json"
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

python3 - "$mcp_config" "$server_wrapper" <<'PY'
import json
import sys

config_path, command = sys.argv[1:]
with open(config_path, "w", encoding="utf-8") as handle:
    json.dump(
        {
            "mcpServers": {
                "ctxhelm": {
                    "command": command,
                    "args": [],
                }
            }
        },
        handle,
    )
PY

semantic_instruction=""
if [[ "$semantic" == "1" ]]; then
  semantic_instruction='Also pass semantic true'
  if [[ -n "$semantic_provider" ]]; then
    semantic_instruction="$semantic_instruction, semanticProvider \"$semantic_provider\""
  fi
  if [[ -n "$semantic_model" ]]; then
    semantic_instruction="$semantic_instruction, semanticModel \"$semantic_model\""
  fi
  if [[ -n "$semantic_dimensions" ]]; then
    semantic_instruction="$semantic_instruction, semanticDimensions $semantic_dimensions"
  fi
  semantic_instruction="$semantic_instruction in both tool calls."
fi

prompt=$(cat <<EOF
Use only the ctxhelm MCP tools. Call prepare_task first with explicit repo "$repo" and task "$task".
Then call get_pack with the same explicit repo "$repo", the same task, budget "brief", format "json", and recordTrace false.
$semantic_instruction
Do not use shell commands for this smoke. The smoke requires machine-checkable tool-call evidence for prepare_task and get_pack with the repo argument.
EOF
)

check_request_evidence() {
  python3 - "$request_log" "$repo" "$semantic" "$semantic_provider" "$semantic_model" "$semantic_dimensions" <<'PY'
import json
import sys

log_path, expected_repo, semantic, expected_provider, expected_model, expected_dimensions = sys.argv[1:]
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
    if name not in seen or arguments.get("repo") != expected_repo:
        continue
    if semantic == "1":
        if arguments.get("semantic") is not True:
            continue
        if expected_provider and arguments.get("semanticProvider") != expected_provider:
            continue
        if expected_model and arguments.get("semanticModel") != expected_model:
            continue
        if expected_dimensions:
            try:
                actual_dimensions = int(arguments.get("semanticDimensions"))
            except (TypeError, ValueError):
                continue
            if actual_dimensions != int(expected_dimensions):
                continue
    seen[name] = True

missing = [name for name, found in seen.items() if not found]
if missing:
    requirement = "explicit-repo"
    if semantic == "1":
        requirement += " semantic-provider"
    raise SystemExit(f"missing {requirement} tool calls: " + ", ".join(missing))
PY
}

set +e
(
  cd "$outside_cwd"
  claude -p \
    --no-session-persistence \
    --strict-mcp-config \
    --mcp-config "$mcp_config" \
    --allowedTools "mcp__ctxhelm__prepare_task,mcp__ctxhelm__get_pack" \
    --permission-mode bypassPermissions \
    --verbose \
    --output-format stream-json \
    "$prompt"
) >"$events" 2>"$stderr_log" &
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
    evidence_path="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/claude-mcp-evidence.json"
  fi
  write_evidence "$evidence_path" "$request_log"
  echo "ctxhelm Claude MCP smoke passed: server-side instrumentation recorded prepare_task and get_pack with repo=$repo"
  exit 0
fi

reason="claude did not produce machine-checkable prepare_task/get_pack evidence"
if [[ "$client_status" != "0" ]]; then
  reason="$reason (claude exit $client_status; auth/model/client refusal is optional unless CTXHELM_REQUIRE_REAL_CLIENT=1)"
fi
if [[ -s "$stderr_log" ]]; then
  echo "ctxhelm Claude MCP smoke diagnostic stderr:" >&2
  tail -n 40 "$stderr_log" >&2
fi
fail_or_skip "$reason"
