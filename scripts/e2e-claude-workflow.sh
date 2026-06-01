#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxpack_root="${CTXPACK_ROOT:-$(cd "$script_dir/.." && pwd -P)}"
claude_smoke_script="$ctxpack_root/scripts/smoke-claude-mcp.sh"
repo_input="${CTXPACK_SMOKE_REPO:-$PWD}"
task="${CTXPACK_SMOKE_TASK:-fix requireSession auth bug}"
anchor_path="${CTXPACK_SMOKE_PATH:-crates/ctxpack-mcp/src/lib.rs}"
query="${CTXPACK_SMOKE_QUERY:-prepare_task}"
report_path="${CTXPACK_CLAUDE_WORKFLOW_REPORT:-}"
require_real="${CTXPACK_REQUIRE_REAL_CLIENT:-0}"
run_real="${CTXPACK_RUN_REAL_CLIENT:-0}"

resolve_ctxpack_bin() {
  if [[ -n "${CTXPACK_BIN:-}" ]]; then
    if [[ ! "$CTXPACK_BIN" = /* ]]; then
      echo "CTXPACK_BIN must be absolute: $CTXPACK_BIN" >&2
      exit 64
    fi
    if [[ ! -x "$CTXPACK_BIN" ]]; then
      echo "CTXPACK_BIN is not executable: $CTXPACK_BIN" >&2
      exit 64
    fi
    printf '%s/%s\n' "$(cd "$(dirname "$CTXPACK_BIN")" && pwd -P)" "$(basename "$CTXPACK_BIN")"
    return
  fi
  cargo build -p ctxpack >/dev/null
  printf '%s/target/debug/ctxpack\n' "$ctxpack_root"
}

write_report() {
  local path="$1"
  local status="$2"
  local smoke_status="$3"
  local repo="$4"
  local ctxpack_version="$5"
  local client_version="$6"
  local smoke_stdout="$7"
  local smoke_stderr="$8"
  local evidence_file="$9"
  local summary_file="${10}"

  python3 - "$path" "$status" "$smoke_status" "$repo" "$task" "$ctxpack_version" "$client_version" "$smoke_stdout" "$smoke_stderr" "$evidence_file" "$summary_file" "$require_real" "$run_real" <<'PY'
import hashlib
import json
import pathlib
import sys

(
    path,
    status,
    smoke_status,
    repo,
    task,
    ctxpack_version,
    client_version,
    smoke_stdout,
    smoke_stderr,
    evidence_file,
    summary_file,
    require_real,
    run_real,
) = sys.argv[1:]

def read_json(path_text):
    if not path_text:
        return None
    candidate = pathlib.Path(path_text)
    if not candidate.is_file():
        return None
    return json.loads(candidate.read_text(encoding="utf-8"))

evidence = read_json(evidence_file) or {}
summary = read_json(summary_file) or {}
observed = summary.get("observedToolCalls") or evidence.get("observedToolCalls") or []
observed_names = [call.get("name") for call in observed]
prepare_task = "prepare_task" in observed_names
get_pack = "get_pack" in observed_names
explicit_repo_calls = int(
    summary.get("explicitRepoToolCallCount")
    or evidence.get("explicitRepoToolCallCount")
    or 0
)
request_line_count = int(
    summary.get("requestLogLineCount")
    or evidence.get("requestLogLineCount")
    or 0
)
request_hash = summary.get("requestLogSha256") or evidence.get("requestLogSha256")
client_version = client_version or evidence.get("clientVersion") or "unavailable"

payload = {
    "schemaVersion": "ctxpack-claude-workflow-eval-v1",
    "workflowKind": "claude-code-mcp-context-workflow",
    "status": status,
    "smokeExitStatus": int(smoke_status),
    "ctxpackVersion": ctxpack_version,
    "client": "claude",
    "clientVersion": client_version,
    "repo": {
        "label": pathlib.Path(repo).name,
        "pathSha256": hashlib.sha256(repo.encode("utf-8")).hexdigest(),
    },
    "task": {
        "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
        "rawTaskStored": False,
    },
    "workflowAssertions": {
        "deterministicProtocol": bool(evidence.get("deterministicProtocol")) or status in {"passed", "skipped"},
        "realClientToolCalls": status == "passed",
        "prepareTaskToolCall": prepare_task,
        "getPackToolCall": get_pack,
        "explicitRepoToolCallCountAtLeastTwo": explicit_repo_calls >= 2,
        "requestSummarySidecar": bool(summary),
        "requestLogHashOnly": bool(request_hash),
        "rawRequestLogStored": False,
        "rawPromptStored": False,
        "sourceTextLogged": False,
        "userProjectCommandsRun": False,
    },
    "requestEvidence": {
        "schemaVersion": summary.get("requestEvidenceSchemaVersion")
        or evidence.get("requestEvidenceSchemaVersion"),
        "requestLogSha256": request_hash,
        "requestLogLineCount": request_line_count,
        "explicitRepoToolCallCount": explicit_repo_calls,
        "observedToolCalls": observed,
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
        "rawMcpTrafficStored": False,
        "rawPromptStored": False,
    },
    "realClientPolicy": {
        "required": require_real == "1",
        "requested": run_real == "1" or require_real == "1",
    },
    "evidenceFiles": {
        "smokeStdoutSha256": hashlib.sha256(pathlib.Path(smoke_stdout).read_bytes()).hexdigest()
        if pathlib.Path(smoke_stdout).is_file()
        else None,
        "smokeStderrSha256": hashlib.sha256(pathlib.Path(smoke_stderr).read_bytes()).hexdigest()
        if pathlib.Path(smoke_stderr).is_file()
        else None,
    },
}

if status == "skipped":
    payload["skipReason"] = (
        "real Claude Code evidence was not requested"
        if run_real != "1" and require_real != "1"
        else "Claude Code did not produce real-client evidence in optional mode"
    )

text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if path:
    target = pathlib.Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")
else:
    print(text, end="")
PY
}

repo="$(cd "$repo_input" && pwd -P)"
ctxpack_bin="$(resolve_ctxpack_bin)"
ctxpack_version="$("$ctxpack_bin" --version)"
client_version=""
if command -v claude >/dev/null 2>&1; then
  client_version="$(claude --version 2>&1 | head -n 1)"
fi

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

evidence_dir="${CTXPACK_CLAUDE_WORKFLOW_EVIDENCE_DIR:-"$work_dir/evidence"}"
mkdir -p "$evidence_dir"
stdout_log="$work_dir/claude-workflow-stdout.log"
stderr_log="$work_dir/claude-workflow-stderr.log"

set +e
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$ctxpack_root" \
  CTXPACK_SMOKE_REPO="$repo" \
  CTXPACK_SMOKE_TASK="$task" \
  CTXPACK_SMOKE_PATH="$anchor_path" \
  CTXPACK_SMOKE_QUERY="$query" \
  CTXPACK_REAL_CLIENT_EVIDENCE_DIR="$evidence_dir" \
  CTXPACK_RUN_REAL_CLIENT="$run_real" \
  CTXPACK_REQUIRE_REAL_CLIENT="$require_real" \
  bash "$claude_smoke_script" >"$stdout_log" 2>"$stderr_log"
smoke_status=$?
set -e

evidence_file="$evidence_dir/claude-mcp-evidence.json"
summary_file="$evidence_dir/claude-mcp-request-summary.json"
if [[ "$smoke_status" -eq 0 && -f "$evidence_file" ]]; then
  workflow_status="passed"
elif [[ "$smoke_status" -eq 0 ]]; then
  workflow_status="skipped"
else
  workflow_status="failed"
fi

write_report "$report_path" "$workflow_status" "$smoke_status" "$repo" "$ctxpack_version" "$client_version" "$stdout_log" "$stderr_log" "$evidence_file" "$summary_file"

if [[ "$workflow_status" == "failed" ]]; then
  cat "$stderr_log" >&2
  exit "$smoke_status"
fi

if [[ "$workflow_status" == "passed" ]]; then
  echo "ctxpack Claude workflow eval passed"
else
  echo "ctxpack Claude workflow eval skipped"
fi
