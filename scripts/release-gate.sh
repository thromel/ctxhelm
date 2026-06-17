#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
release_package_script="$repo_root/scripts/release-package.sh"
verify_release_archive_script="$repo_root/scripts/verify-release-archive.sh"
check_release_docs_script="$repo_root/scripts/check-release-docs.sh"
check_agent_run_proof_script="$repo_root/scripts/check-agent-run-proof.py"
smoke_first_pack_script="$repo_root/scripts/smoke-first-pack.sh"
smoke_storage_script="$repo_root/scripts/smoke-storage.sh"
smoke_memory_script="$repo_root/scripts/smoke-memory.sh"
smoke_memory_reuse_script="$repo_root/scripts/smoke-memory-reuse.sh"
smoke_memory_history_lift_script="$repo_root/scripts/smoke-memory-history-lift.sh"
smoke_memory_parent_snapshot_lift_script="$repo_root/scripts/smoke-memory-parent-snapshot-lift.sh"
smoke_memory_benchmark_lift_script="$repo_root/scripts/smoke-memory-benchmark-lift.sh"
smoke_feedback_script="$repo_root/scripts/smoke-feedback.sh"
smoke_governor_script="$repo_root/scripts/smoke-governor.sh"
smoke_workspace_script="$repo_root/scripts/smoke-workspace.sh"
smoke_shared_artifacts_script="$repo_root/scripts/smoke-shared-artifacts.sh"
smoke_inspector_script="$repo_root/scripts/smoke-inspector.sh"
smoke_retrieval_health_script="$repo_root/scripts/smoke-retrieval-health.sh"
smoke_graph_script="$repo_root/scripts/smoke-graph.sh"
smoke_policy_embedding_script="$repo_root/scripts/smoke-policy-embedding.sh"
smoke_agent_preview_script="$repo_root/scripts/smoke-agent-preview.sh"
smoke_agent_native_fallback_script="$repo_root/scripts/smoke-agent-native-fallback.sh"
smoke_demo_artifacts_script="$repo_root/scripts/smoke-demo-artifacts.sh"
smoke_distribution_metadata_script="$repo_root/scripts/smoke-distribution-metadata.sh"
smoke_release_governance_script="$repo_root/scripts/smoke-release-governance.sh"
smoke_semantic_script="$repo_root/scripts/smoke-semantic.sh"
smoke_precision_script="$repo_root/scripts/smoke-precision.sh"
smoke_v23_eval_script="$repo_root/scripts/smoke-v23-eval.sh"
smoke_v24_gate_script="$repo_root/scripts/smoke-v24-gate.sh"
smoke_mcp_protocol_script="$repo_root/scripts/smoke-mcp-protocol.sh"
smoke_codex_mcp_script="$repo_root/scripts/smoke-codex-mcp.sh"
smoke_claude_mcp_script="$repo_root/scripts/smoke-claude-mcp.sh"
claude_workflow_eval_script="$repo_root/scripts/e2e-claude-workflow.sh"
smoke_cursor_mcp_script="$repo_root/scripts/smoke-cursor-mcp.sh"
smoke_opencode_mcp_script="$repo_root/scripts/smoke-opencode-mcp.sh"
smoke_cursor_real_client_script="$repo_root/scripts/smoke-cursor-real-client.sh"
smoke_opencode_real_client_script="$repo_root/scripts/smoke-opencode-real-client.sh"
clean_fixture_config_default="$repo_root/.planning/e2e/2026-06-03-phase183-clean-fixture-refresh-config.json"

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT
proof_dir="${CTXHELM_PROOF_DIR:-"$work_dir/proof-bundle"}"
proof_summary_path="$proof_dir/release-proof-summary.json"

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    sha256sum "$1" | awk '{ print $1 }'
  fi
}

log_step() {
  printf '\n==> %s\n' "$1"
}

check_worktree_clean() {
  local label="$1"
  if [[ "${CTXHELM_SKIP_WORKTREE_CLEAN_CHECK:-0}" == "1" ]]; then
    return 0
  fi
  if [[ -n "$(git -C "$repo_root" status --porcelain)" ]]; then
    echo "release gate failed: worktree changed during $label; commit intentional generated artifacts or fix the smoke" >&2
    git -C "$repo_root" status --short >&2
    exit 1
  fi
}

clean_fixture_ready() {
  local config_path="$1"
  python3 - "$config_path" <<'PY'
import json
import pathlib
import sys

config_path = pathlib.Path(sys.argv[1])
if not config_path.is_file():
    raise SystemExit(f"missing clean fixture config: {config_path}")
config = json.loads(config_path.read_text())
config_dir = config_path.parent
missing = []
for repo in config.get("repositories", []):
    raw_path = pathlib.Path(repo.get("path", ""))
    path = raw_path if raw_path.is_absolute() else config_dir / raw_path
    if not path.is_dir() or not (path / ".git").exists():
        missing.append(f"{repo.get('name', '<unnamed>')}={path}")
if missing:
    raise SystemExit("missing clean proof fixtures; run scripts/prepare-proof-fixtures.sh: " + ", ".join(missing))
stale = []
for repo in config.get("repositories", []):
    raw_path = pathlib.Path(repo.get("path", ""))
    path = raw_path if raw_path.is_absolute() else config_dir / raw_path
    expected = repo.get("head")
    if not expected:
        continue
    import subprocess
    available = subprocess.run(
        ["git", "-C", str(path), "cat-file", "-e", f"{expected}^{{commit}}"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    actual = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    actual_head = actual.stdout.strip() if actual.returncode == 0 else "<unavailable>"
    if available.returncode != 0:
        stale.append(f"{repo.get('name', '<unnamed>')} requested revision unavailable: {expected}")
    elif actual_head != expected:
        stale.append(f"{repo.get('name', '<unnamed>')} expected {expected} got {actual_head}")
if stale:
    raise SystemExit("stale clean proof fixtures; run scripts/prepare-proof-fixtures.sh: " + ", ".join(stale))
PY
}

canonical_executable() {
  local candidate="$1"
  if [[ -z "$candidate" ]]; then
    return 1
  fi
  if [[ ! "$candidate" = /* ]]; then
    echo "CTXHELM_BIN must be an absolute path: $candidate" >&2
    exit 64
  fi
  if [[ ! -x "$candidate" ]]; then
    echo "CTXHELM_BIN is not executable: $candidate" >&2
    exit 64
  fi
  printf '%s/%s\n' "$(cd "$(dirname "$candidate")" && pwd -P)" "$(basename "$candidate")"
}

latest_archive() {
  local dist_dir="$1"
  python3 - "$dist_dir" <<'PY'
import pathlib
import sys

dist = pathlib.Path(sys.argv[1])
archives = sorted(
    dist.glob("ctxhelm-v*.tar.gz"),
    key=lambda path: path.stat().st_mtime,
    reverse=True,
)
if not archives:
    raise SystemExit(f"no ctxhelm release archive found in {dist}")
print(archives[0])
PY
}

extract_binary_from_archive() {
  local archive="$1"
  local extract_dir="$work_dir/extracted"
  mkdir -p "$extract_dir"
  tar -xzf "$archive" -C "$extract_dir"
  local extracted
  extracted="$(find "$extract_dir" -type f -name ctxhelm -perm -111 | head -n 1)"
  if [[ -z "$extracted" ]]; then
    echo "release archive did not contain an executable ctxhelm binary: $archive" >&2
    exit 65
  fi
  canonical_executable "$extracted"
}

cd "$repo_root"

log_step "workspace tests"
cargo test --workspace

log_step "release docs consistency"
bash "$check_release_docs_script"

dist_dir="${CTXHELM_DIST_DIR:-"$work_dir/dist"}"
log_step "release package and artifact audit"
CTXHELM_DIST_DIR="$dist_dir" bash "$release_package_script"

archive_path="$(latest_archive "$dist_dir")"
manifest_path="${archive_path%.tar.gz}.manifest.json"
audit_report_path="${archive_path%.tar.gz}.audit.json"
checksums_path="$dist_dir/sha256sums.txt"
log_step "clean extraction archive verification"
bash "$verify_release_archive_script" \
  --archive "$archive_path" \
  --manifest "$manifest_path" \
  --checksums "$checksums_path"

if [[ -n "${CTXHELM_BIN:-}" ]]; then
  ctxhelm_bin="$(canonical_executable "$CTXHELM_BIN")"
  binary_source="selected"
else
  ctxhelm_bin="$(extract_binary_from_archive "$archive_path")"
  binary_source="archive"
fi
archive_sha256="$(sha256_file "$archive_path")"
real_client_required="${CTXHELM_REQUIRE_REAL_CLIENT:-0}"
real_client_skip="${CTXHELM_SKIP_REAL_CLIENT:-}"
if [[ -z "$real_client_skip" ]]; then
  if [[ "$real_client_required" == "1" ]]; then
    real_client_skip="0"
  elif [[ "${CTXHELM_REQUIRE_CURSOR_REAL_CLIENT:-0}" == "1" || "${CTXHELM_RUN_CURSOR_REAL_CLIENT:-0}" == "1" || "${CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT:-0}" == "1" || "${CTXHELM_RUN_OPENCODE_REAL_CLIENT:-0}" == "1" ]]; then
    real_client_skip="0"
  else
    real_client_skip="1"
  fi
fi
clean_fixture_config="${CTXHELM_CLEAN_FIXTURE_CONFIG:-"$clean_fixture_config_default"}"
clean_fixture_required="${CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF:-0}"
clean_fixture_skip="${CTXHELM_SKIP_CLEAN_FIXTURE_PROOF:-0}"

log_step "selected binary identity"
ctxhelm_version="$("$ctxhelm_bin" --version)"
printf '%s\n' "$ctxhelm_version"
"$ctxhelm_bin" --help >/dev/null
binary_sha256="$(sha256_file "$ctxhelm_bin")"

log_step "first-pack smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_first_pack_script"

log_step "storage smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_storage_script"

log_step "memory smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_memory_script"

log_step "memory reuse smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_memory_reuse_script"

log_step "memory history lift smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_memory_history_lift_script"

log_step "memory parent-snapshot lift smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_memory_parent_snapshot_lift_script"

log_step "memory benchmark lift smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_memory_benchmark_lift_script"

log_step "feedback smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_feedback_script"

log_step "context governor smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_governor_script"

log_step "workspace smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_workspace_script"

log_step "shared artifacts smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_shared_artifacts_script"

log_step "inspector smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_inspector_script"

log_step "retrieval health smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_retrieval_health_script"

log_step "graph smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_graph_script"

log_step "policy and embedding smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_policy_embedding_script"

log_step "agent preview smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_agent_preview_script"

log_step "agent-native fallback smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_agent_native_fallback_script"

log_step "public demo artifacts smoke"
bash "$smoke_demo_artifacts_script"

log_step "distribution metadata smoke"
CTXHELM_DIST_DIR="$dist_dir" \
  CTXHELM_DISTRIBUTION_METADATA_OUT="$work_dir/distribution-metadata-smoke.json" \
  bash "$smoke_distribution_metadata_script"

log_step "post-smoke worktree cleanliness"
check_worktree_clean "distribution metadata smoke"

log_step "release governance smoke"
bash "$smoke_release_governance_script"

log_step "semantic smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_semantic_script"

log_step "precision smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_precision_script"

log_step "v2.3 eval smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_v23_eval_script"

log_step "v2.4 semantic/precision gate smoke"
CTXHELM_BIN="$ctxhelm_bin" bash "$smoke_v24_gate_script"

mkdir -p "$proof_dir"
real_client_evidence_dir="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-"$proof_dir/real-client-evidence"}"
mkdir -p "$real_client_evidence_dir"

log_step "wrong-cwd MCP protocol smoke"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_SMOKE_REPO="$repo_root" \
  CTXHELM_SMOKE_TASK="verify release gate MCP protocol proof" \
  CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" \
  CTXHELM_SMOKE_QUERY="prepare_task" \
  bash "$smoke_mcp_protocol_script"

log_step "Cursor setup and MCP protocol evidence"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_REAL_CLIENT_EVIDENCE_DIR="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_cursor_mcp_script"

log_step "OpenCode setup and MCP protocol evidence"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_REAL_CLIENT_EVIDENCE_DIR="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_opencode_mcp_script"

log_step "optional Cursor real-client evidence"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_SMOKE_REPO="$repo_root" \
  CTXHELM_SMOKE_TASK="verify release gate Cursor real-client MCP proof" \
  CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" \
  CTXHELM_SMOKE_QUERY="prepare_task" \
  CTXHELM_SKIP_REAL_CLIENT="$real_client_skip" \
  CTXHELM_REQUIRE_CURSOR_REAL_CLIENT="${CTXHELM_REQUIRE_CURSOR_REAL_CLIENT:-0}" \
  CTXHELM_REAL_CLIENT_EVIDENCE_DIR="$real_client_evidence_dir" \
  bash "$smoke_cursor_real_client_script"
cursor_real_client_status="$(python3 - "$real_client_evidence_dir/cursor-real-client-evidence.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("missing_evidence")
    raise SystemExit
payload = json.loads(path.read_text(encoding="utf-8"))
status = payload.get("status", "unknown")
if status == "passed":
    print("passed")
else:
    reason = payload.get("skipReason") or payload.get("clientFailureKind") or "unknown"
    print(f"{status}:{reason}")
PY
)"

log_step "optional OpenCode real-client evidence"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_SMOKE_REPO="$repo_root" \
  CTXHELM_SMOKE_TASK="verify release gate OpenCode real-client MCP proof" \
  CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" \
  CTXHELM_SMOKE_QUERY="prepare_task" \
  CTXHELM_SKIP_REAL_CLIENT="$real_client_skip" \
  CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT="${CTXHELM_REQUIRE_OPENCODE_REAL_CLIENT:-0}" \
  CTXHELM_REAL_CLIENT_EVIDENCE_DIR="$real_client_evidence_dir" \
  bash "$smoke_opencode_real_client_script"
opencode_real_client_status="$(python3 - "$real_client_evidence_dir/opencode-real-client-evidence.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("missing_evidence")
    raise SystemExit
payload = json.loads(path.read_text(encoding="utf-8"))
status = payload.get("status", "unknown")
if status == "passed":
    print("passed")
else:
    reason = payload.get("skipReason") or payload.get("clientFailureKind") or "unknown"
    print(f"{status}:{reason}")
PY
)"

log_step "optional benchmark product proof"
product_proof_bundle_report=""
if [[ -n "${CTXHELM_BENCHMARK_CONFIG:-}" ]]; then
  proof_report="$proof_dir/product-proof.json"
  "$ctxhelm_bin" eval proof --config "$CTXHELM_BENCHMARK_CONFIG" --format json >"$proof_report"
  python3 "$repo_root/scripts/check-product-proof.py" "$proof_report"
  benchmark_status="passed"
  product_proof_bundle_report="$proof_report"
else
  echo "benchmark product proof skipped: set CTXHELM_BENCHMARK_CONFIG=/path/to/suite.json"
  benchmark_status="skipped"
fi

log_step "clean cold fixture product proof"
clean_fixture_status="skipped"
if [[ "$clean_fixture_skip" == "1" ]]; then
  echo "clean cold fixture proof skipped: CTXHELM_SKIP_CLEAN_FIXTURE_PROOF=1"
  clean_fixture_status="skipped_explicit"
elif clean_fixture_error="$(clean_fixture_ready "$clean_fixture_config" 2>&1)"; then
  clean_fixture_report="$proof_dir/phase183-clean-fixture-product-proof.json"
  "$ctxhelm_bin" eval proof --config "$clean_fixture_config" --format json >"$clean_fixture_report"
  python3 "$repo_root/scripts/check-product-proof.py" "$clean_fixture_report"
  clean_fixture_status="passed"
  product_proof_bundle_report="$clean_fixture_report"
else
  if [[ "$clean_fixture_required" == "1" ]]; then
    echo "$clean_fixture_error" >&2
    exit 66
  fi
  echo "$clean_fixture_error"
  echo "clean cold fixture proof skipped: set CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1 to require it"
  clean_fixture_status="skipped_missing_fixtures"
fi

log_step "optional agent-run outcome proof"
agent_run_proof_status="skipped"
agent_run_proof_check_report=""
agent_run_proof_required="${CTXHELM_REQUIRE_AGENT_RUN_PROOF:-0}"
agent_run_expected_client_name="${CTXHELM_AGENT_RUN_EXPECTED_CLIENT_NAME:-codex}"
agent_run_expected_client_version="${CTXHELM_AGENT_RUN_EXPECTED_CLIENT_VERSION:-codex-cli 0.137.0}"
if [[ -n "${CTXHELM_AGENT_RUN_PROOF_REPORT:-}" ]]; then
  agent_run_proof_check_report="$proof_dir/agent-run-outcome-proof.json"
  python3 "$check_agent_run_proof_script" "$CTXHELM_AGENT_RUN_PROOF_REPORT" \
    --workflow suite \
    --require-outcome "${CTXHELM_AGENT_RUN_REQUIRE_OUTCOME:-ctxhelm_improved}" \
    --expected-ctxhelm-version "$ctxhelm_version" \
    --expected-client-name "$agent_run_expected_client_name" \
    --expected-client-version "$agent_run_expected_client_version" \
    --min-task-count "${CTXHELM_AGENT_RUN_MIN_TASK_COUNT:-4}" \
    --min-comparison-eligible "${CTXHELM_AGENT_RUN_MIN_COMPARISON_ELIGIBLE:-4}" \
    --min-comparable-ctxhelm-lanes "${CTXHELM_AGENT_RUN_MIN_COMPARABLE_CTXHELM_LANES:-16}" \
    --min-ctxhelm-target-read-coverage "${CTXHELM_AGENT_RUN_MIN_TARGET_READ_COVERAGE:-1.0}" \
    --max-extra-read-delta "${CTXHELM_AGENT_RUN_MAX_EXTRA_READ_DELTA:-2}" \
    --min-irrelevant-read-delta "${CTXHELM_AGENT_RUN_MIN_IRRELEVANT_READ_DELTA:-0}" \
    --require-retry-cost \
    --require-runner-fingerprint \
    --current-runner-script "$repo_root/scripts/e2e-agent-run-codex.sh" \
    --current-suite "$repo_root/.planning/e2e/2026-06-06-phase251-codex-rd-suite.json" \
    --format json \
    --output "$agent_run_proof_check_report"
  agent_run_proof_status="passed"
elif [[ "$agent_run_proof_required" == "1" ]]; then
  echo "agent-run proof required but CTXHELM_AGENT_RUN_PROOF_REPORT was not set" >&2
  exit 68
else
  echo "agent-run outcome proof skipped: set CTXHELM_AGENT_RUN_PROOF_REPORT=/path/to/report.json"
fi

log_step "optional proof-inspector readiness bundle"
proof_inspector_readiness_status="skipped"
proof_inspector_readiness_report=""
proof_inspector_readiness_required="${CTXHELM_REQUIRE_PROOF_INSPECTOR_READY:-0}"
if [[ -n "$product_proof_bundle_report" && "$agent_run_proof_status" == "passed" ]]; then
  proof_inspector_readiness_report="$proof_dir/proof-inspector-readiness-bundle.json"
  "$ctxhelm_bin" inspector proof \
    --repo "$repo_root" \
    --report "$product_proof_bundle_report" \
    --report "$CTXHELM_AGENT_RUN_PROOF_REPORT" \
    --format json \
    --require-ready \
    --output "$proof_inspector_readiness_report"
  proof_inspector_readiness_status="passed"
elif [[ "$proof_inspector_readiness_required" == "1" ]]; then
  echo "proof-inspector readiness required but clean product proof and agent-run proof were not both available" >&2
  exit 69
else
  echo "proof-inspector readiness bundle skipped: provide both clean product proof and agent-run proof, or set CTXHELM_REQUIRE_PROOF_INSPECTOR_READY=1 to require it"
fi

log_step "optional Codex real-client evidence"
codex_status="skipped"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_SMOKE_REPO="$repo_root" \
  CTXHELM_SMOKE_TASK="verify release gate Codex MCP proof" \
  CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" \
  CTXHELM_SMOKE_QUERY="prepare_task" \
  CTXHELM_SKIP_REAL_CLIENT="$real_client_skip" \
  CTXHELM_REQUIRE_REAL_CLIENT="$real_client_required" \
  CTXHELM_REAL_CLIENT_EVIDENCE_DIR="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_codex_mcp_script"
if [[ "$real_client_skip" != "1" && ( "$real_client_required" == "1" || "${CTXHELM_RUN_REAL_CLIENT:-0}" == "1" ) ]]; then
  codex_status="passed"
fi

log_step "optional Claude real-client evidence"
claude_status="skipped"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$repo_root" \
  CTXHELM_SMOKE_REPO="$repo_root" \
  CTXHELM_SMOKE_TASK="verify release gate Claude MCP proof" \
  CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" \
  CTXHELM_SMOKE_QUERY="prepare_task" \
  CTXHELM_SKIP_REAL_CLIENT="$real_client_skip" \
  CTXHELM_REQUIRE_REAL_CLIENT="$real_client_required" \
  CTXHELM_REAL_CLIENT_EVIDENCE_DIR="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_claude_mcp_script"
if [[ "$real_client_skip" != "1" && ( "$real_client_required" == "1" || "${CTXHELM_RUN_REAL_CLIENT:-0}" == "1" ) ]]; then
  claude_status="passed"
fi

log_step "optional Claude workflow eval"
claude_workflow_status="skipped"
claude_workflow_required="${CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL:-0}"
if [[ "${CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL:-0}" == "1" || "$claude_workflow_required" == "1" ]]; then
  claude_workflow_report="$proof_dir/claude-workflow-eval.json"
  CTXHELM_BIN="$ctxhelm_bin" \
    CTXHELM_ROOT="$repo_root" \
    CTXHELM_SMOKE_REPO="$repo_root" \
    CTXHELM_SMOKE_TASK="verify Claude Code can use ctxhelm prepare_task and get_pack as a context workflow" \
    CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" \
    CTXHELM_SMOKE_QUERY="prepare_task" \
    CTXHELM_RUN_REAL_CLIENT="1" \
    CTXHELM_REQUIRE_REAL_CLIENT="$claude_workflow_required" \
    CTXHELM_CLAUDE_WORKFLOW_REPORT="$claude_workflow_report" \
    bash "$claude_workflow_eval_script"
  claude_workflow_status="$(python3 - "$claude_workflow_report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    print(json.load(handle).get("status", "unknown"))
PY
)"
  if [[ "$claude_workflow_required" == "1" && "$claude_workflow_status" != "passed" ]]; then
    echo "required Claude workflow eval did not pass: $claude_workflow_status" >&2
    exit 67
  fi
else
  echo "Claude workflow eval skipped: set CTXHELM_RUN_CLAUDE_WORKFLOW_EVAL=1 or CTXHELM_REQUIRE_CLAUDE_WORKFLOW_EVAL=1"
fi

log_step "release proof bundle"
agent_run_proof_check_report_name=""
if [[ -n "$agent_run_proof_check_report" ]]; then
  agent_run_proof_check_report_name="$(basename "$agent_run_proof_check_report")"
fi
proof_inspector_readiness_report_name=""
if [[ -n "$proof_inspector_readiness_report" ]]; then
  proof_inspector_readiness_report_name="$(basename "$proof_inspector_readiness_report")"
fi
python3 - "$proof_summary_path" "$ctxhelm_version" "$(basename "$ctxhelm_bin")" "$binary_source" "$binary_sha256" "$(basename "$archive_path")" "$archive_sha256" "$(basename "$manifest_path")" "$(basename "$audit_report_path")" "$benchmark_status" "$clean_fixture_status" "$clean_fixture_required" "$agent_run_proof_status" "$agent_run_proof_required" "$agent_run_proof_check_report_name" "$proof_inspector_readiness_status" "$proof_inspector_readiness_required" "$proof_inspector_readiness_report_name" "$codex_status" "$claude_status" "$claude_workflow_status" "$claude_workflow_required" "$real_client_required" "$cursor_real_client_status" "$opencode_real_client_status" <<'PY'
import json
import sys

(
    proof_summary_path,
    ctxhelm_version,
    binary_name,
    binary_source,
    binary_sha256,
    archive_name,
    archive_sha256,
    manifest_name,
    audit_report_name,
    benchmark_status,
    clean_fixture_status,
    clean_fixture_required,
    agent_run_proof_status,
    agent_run_proof_required,
    agent_run_proof_check_report_name,
    proof_inspector_readiness_status,
    proof_inspector_readiness_required,
    proof_inspector_readiness_report_name,
    codex_status,
    claude_status,
    claude_workflow_status,
    claude_workflow_required,
    real_client_required,
    cursor_real_client_status,
    opencode_real_client_status,
) = sys.argv[1:]

required_checks = [
    "cargo test --workspace",
    "scripts/check-release-docs.sh",
    "scripts/release-package.sh",
    "scripts/verify-release-archive.sh",
    "ctxhelm --version",
    "ctxhelm --help",
    "scripts/smoke-first-pack.sh",
    "scripts/smoke-storage.sh",
    "scripts/smoke-memory.sh",
    "scripts/smoke-memory-reuse.sh",
    "scripts/smoke-memory-history-lift.sh",
    "scripts/smoke-memory-parent-snapshot-lift.sh",
    "scripts/smoke-memory-benchmark-lift.sh",
    "scripts/smoke-feedback.sh",
    "scripts/smoke-governor.sh",
    "scripts/smoke-workspace.sh",
    "scripts/smoke-shared-artifacts.sh",
    "scripts/smoke-inspector.sh",
    "scripts/smoke-retrieval-health.sh",
    "scripts/smoke-graph.sh",
    "scripts/smoke-policy-embedding.sh",
    "scripts/smoke-agent-preview.sh",
    "scripts/smoke-agent-native-fallback.sh",
    "scripts/smoke-demo-artifacts.sh",
    "scripts/smoke-distribution-metadata.sh",
    "scripts/smoke-release-governance.sh",
    "scripts/smoke-semantic.sh",
    "scripts/smoke-precision.sh",
    "scripts/smoke-v23-eval.sh",
    "scripts/smoke-v24-gate.sh",
    "scripts/smoke-mcp-protocol.sh",
    "scripts/smoke-cursor-mcp.sh",
    "scripts/smoke-opencode-mcp.sh",
]
if clean_fixture_required == "1":
    required_checks.append("clean cold fixture product proof")
if agent_run_proof_required == "1":
    required_checks.append("agent-run outcome proof")
if proof_inspector_readiness_required == "1":
    required_checks.append("proof-inspector readiness bundle")
payload = {
    "schemaVersion": 1,
    "status": "passed",
    "ctxhelmVersion": ctxhelm_version,
    "binaryIdentity": {
        "fileName": binary_name,
        "source": binary_source,
        "sha256": binary_sha256,
    },
    "releaseArchive": {
        "name": archive_name,
        "sha256": archive_sha256,
        "manifest": manifest_name,
        "auditReport": audit_report_name,
    },
    "requiredChecks": [{"name": check, "status": "passed"} for check in required_checks],
    "optionalProofs": {
        "benchmarkProductProof": benchmark_status,
        "cleanColdFixtureProductProof": clean_fixture_status,
        "cleanColdFixtureRequired": clean_fixture_required == "1",
        "agentRunOutcomeProof": agent_run_proof_status,
        "agentRunOutcomeProofRequired": agent_run_proof_required == "1",
        "agentRunOutcomeProofReport": agent_run_proof_check_report_name,
        "proofInspectorReadinessBundle": proof_inspector_readiness_status,
        "proofInspectorReadinessRequired": proof_inspector_readiness_required == "1",
        "proofInspectorReadinessReport": proof_inspector_readiness_report_name,
        "resourceBackedGapSummaryContract": (
            "checked"
            if benchmark_status == "passed" or clean_fixture_status == "passed"
            else "skipped"
        ),
        "codexRealClientProof": codex_status,
        "claudeRealClientProof": claude_status,
        "claudeWorkflowEval": claude_workflow_status,
        "claudeWorkflowEvalRequired": claude_workflow_required == "1",
        "realClientRequired": real_client_required == "1",
        "cursorRealClientProof": cursor_real_client_status,
        "opencodeRealClientProof": opencode_real_client_status,
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "unsupportedActions": [
        "publishing",
        "tag creation",
        "global agent config mutation",
        "user project test execution",
        "cloud upload",
    ],
}
with open(proof_summary_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
echo "wrote release proof summary: $proof_summary_path"

log_step "final worktree cleanliness"
check_worktree_clean "release gate"

echo "release gate passed: binary=$ctxhelm_bin archive=$archive_path proof=$proof_summary_path"
