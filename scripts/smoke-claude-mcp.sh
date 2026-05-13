#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxpack_root="$(cd "$script_dir/.." && pwd -P)"
protocol_script="$ctxpack_root/scripts/smoke-mcp-protocol.sh"
repo_input="${CTXPACK_SMOKE_REPO:-$PWD}"
task="${CTXPACK_SMOKE_TASK:-fix requireSession auth bug}"
require_real="${CTXPACK_REQUIRE_REAL_CLIENT:-0}"

repo="$(cd "$repo_input" && pwd -P)"
ctxpack_home="$(mktemp -d)"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$ctxpack_home" "$work_dir"
}
trap cleanup EXIT

echo "ctxpack Claude MCP smoke: running deterministic protocol gate"
CTXPACK_ROOT="$ctxpack_root" \
  CTXPACK_SMOKE_REPO="$repo" \
  CTXPACK_SMOKE_TASK="$task" \
  CTXPACK_HOME="$ctxpack_home" \
  bash "$protocol_script"

if [[ "${CTXPACK_SKIP_REAL_CLIENT:-0}" == "1" ]]; then
  echo "ctxpack Claude MCP smoke skipped: CTXPACK_SKIP_REAL_CLIENT=1 after protocol gate passed"
  exit 0
fi

fail_or_skip() {
  local reason="$1"
  if [[ "$require_real" == "1" ]]; then
    echo "ctxpack Claude MCP smoke failed: $reason" >&2
    exit 1
  fi
  echo "ctxpack Claude MCP smoke skipped: $reason"
  exit 0
}

if ! command -v claude >/dev/null 2>&1; then
  fail_or_skip "claude is not installed"
fi

request_log="$work_dir/ctxpack-mcp-requests.jsonl"
events="$work_dir/claude-stream.jsonl"
stderr_log="$work_dir/claude-stderr.log"
server_wrapper="$work_dir/ctxpack-mcp-server.sh"
mcp_config="$work_dir/claude-mcp.json"
outside_cwd="$work_dir/outside-repo"
mkdir -p "$outside_cwd"

{
  printf '%s\n' '#!/usr/bin/env bash'
  printf '%s\n' 'set -euo pipefail'
  printf 'export CTXPACK_HOME=%q\n' "$ctxpack_home"
  printf 'export CTXPACK_ROOT=%q\n' "$ctxpack_root"
  printf 'export CTXPACK_REAL_CLIENT_REQUEST_LOG=%q\n' "$request_log"
  printf '%s\n' 'tee -a "$CTXPACK_REAL_CLIENT_REQUEST_LOG" | cargo run --quiet --manifest-path "$CTXPACK_ROOT/Cargo.toml" -p ctxpack -- serve-mcp'
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
                "ctxpack": {
                    "command": command,
                    "args": [],
                }
            }
        },
        handle,
    )
PY

prompt=$(cat <<EOF
Use only the ctxpack MCP tools. Call prepare_task first with explicit repo "$repo" and task "$task".
Then call get_pack with the same explicit repo "$repo", the same task, budget "brief", format "json", and recordTrace false.
Do not use shell commands for this smoke. The smoke requires machine-checkable tool-call evidence for prepare_task and get_pack with the repo argument.
EOF
)

set +e
(
  cd "$outside_cwd"
  claude -p \
    --no-session-persistence \
    --strict-mcp-config \
    --mcp-config "$mcp_config" \
    --allowedTools "mcp__ctxpack__prepare_task,mcp__ctxpack__get_pack" \
    --permission-mode bypassPermissions \
    --verbose \
    --output-format stream-json \
    "$prompt"
) >"$events" 2>"$stderr_log"
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
  echo "ctxpack Claude MCP smoke passed: server-side instrumentation recorded prepare_task and get_pack with repo=$repo"
  exit 0
fi

reason="claude did not produce machine-checkable prepare_task/get_pack evidence"
if [[ "$client_status" != "0" ]]; then
  reason="$reason (claude exit $client_status; auth/model/client refusal is optional unless CTXPACK_REQUIRE_REAL_CLIENT=1)"
fi
if [[ -s "$stderr_log" ]]; then
  echo "ctxpack Claude MCP smoke diagnostic stderr:" >&2
  tail -n 40 "$stderr_log" >&2
fi
fail_or_skip "$reason"
