#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
release_package_script="$repo_root/scripts/release-package.sh"
verify_release_archive_script="$repo_root/scripts/verify-release-archive.sh"
check_release_docs_script="$repo_root/scripts/check-release-docs.sh"
smoke_first_pack_script="$repo_root/scripts/smoke-first-pack.sh"
smoke_storage_script="$repo_root/scripts/smoke-storage.sh"
smoke_memory_script="$repo_root/scripts/smoke-memory.sh"
smoke_feedback_script="$repo_root/scripts/smoke-feedback.sh"
smoke_workspace_script="$repo_root/scripts/smoke-workspace.sh"
smoke_shared_artifacts_script="$repo_root/scripts/smoke-shared-artifacts.sh"
smoke_inspector_script="$repo_root/scripts/smoke-inspector.sh"
smoke_retrieval_health_script="$repo_root/scripts/smoke-retrieval-health.sh"
smoke_graph_script="$repo_root/scripts/smoke-graph.sh"
smoke_policy_embedding_script="$repo_root/scripts/smoke-policy-embedding.sh"
smoke_agent_preview_script="$repo_root/scripts/smoke-agent-preview.sh"
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
smoke_cursor_mcp_script="$repo_root/scripts/smoke-cursor-mcp.sh"
smoke_opencode_mcp_script="$repo_root/scripts/smoke-opencode-mcp.sh"

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT
proof_dir="${CTXPACK_PROOF_DIR:-"$work_dir/proof-bundle"}"
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

canonical_executable() {
  local candidate="$1"
  if [[ -z "$candidate" ]]; then
    return 1
  fi
  if [[ ! "$candidate" = /* ]]; then
    echo "CTXPACK_BIN must be an absolute path: $candidate" >&2
    exit 64
  fi
  if [[ ! -x "$candidate" ]]; then
    echo "CTXPACK_BIN is not executable: $candidate" >&2
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
    dist.glob("ctxpack-v*.tar.gz"),
    key=lambda path: path.stat().st_mtime,
    reverse=True,
)
if not archives:
    raise SystemExit(f"no ctxpack release archive found in {dist}")
print(archives[0])
PY
}

extract_binary_from_archive() {
  local archive="$1"
  local extract_dir="$work_dir/extracted"
  mkdir -p "$extract_dir"
  tar -xzf "$archive" -C "$extract_dir"
  local extracted
  extracted="$(find "$extract_dir" -type f -name ctxpack -perm -111 | head -n 1)"
  if [[ -z "$extracted" ]]; then
    echo "release archive did not contain an executable ctxpack binary: $archive" >&2
    exit 65
  fi
  canonical_executable "$extracted"
}

cd "$repo_root"

log_step "workspace tests"
cargo test --workspace

log_step "release docs consistency"
bash "$check_release_docs_script"

dist_dir="${CTXPACK_DIST_DIR:-"$work_dir/dist"}"
log_step "release package and artifact audit"
CTXPACK_DIST_DIR="$dist_dir" bash "$release_package_script"

archive_path="$(latest_archive "$dist_dir")"
manifest_path="${archive_path%.tar.gz}.manifest.json"
audit_report_path="${archive_path%.tar.gz}.audit.json"
checksums_path="$dist_dir/sha256sums.txt"
log_step "clean extraction archive verification"
bash "$verify_release_archive_script" \
  --archive "$archive_path" \
  --manifest "$manifest_path" \
  --checksums "$checksums_path"

if [[ -n "${CTXPACK_BIN:-}" ]]; then
  ctxpack_bin="$(canonical_executable "$CTXPACK_BIN")"
  binary_source="selected"
else
  ctxpack_bin="$(extract_binary_from_archive "$archive_path")"
  binary_source="archive"
fi
archive_sha256="$(sha256_file "$archive_path")"
real_client_required="${CTXPACK_REQUIRE_REAL_CLIENT:-0}"
real_client_skip="${CTXPACK_SKIP_REAL_CLIENT:-}"
if [[ -z "$real_client_skip" ]]; then
  if [[ "$real_client_required" == "1" ]]; then
    real_client_skip="0"
  else
    real_client_skip="1"
  fi
fi

log_step "selected binary identity"
ctxpack_version="$("$ctxpack_bin" --version)"
printf '%s\n' "$ctxpack_version"
"$ctxpack_bin" --help >/dev/null
binary_sha256="$(sha256_file "$ctxpack_bin")"

log_step "first-pack smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_first_pack_script"

log_step "storage smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_storage_script"

log_step "memory smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_memory_script"

log_step "feedback smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_feedback_script"

log_step "workspace smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_workspace_script"

log_step "shared artifacts smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_shared_artifacts_script"

log_step "inspector smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_inspector_script"

log_step "retrieval health smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_retrieval_health_script"

log_step "graph smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_graph_script"

log_step "policy and embedding smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_policy_embedding_script"

log_step "agent preview smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_agent_preview_script"

log_step "public demo artifacts smoke"
bash "$smoke_demo_artifacts_script"

log_step "distribution metadata smoke"
bash "$smoke_distribution_metadata_script"

log_step "release governance smoke"
bash "$smoke_release_governance_script"

log_step "semantic smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_semantic_script"

log_step "precision smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_precision_script"

log_step "v2.3 eval smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_v23_eval_script"

log_step "v2.4 semantic/precision gate smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_v24_gate_script"

log_step "wrong-cwd MCP protocol smoke"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$repo_root" \
  CTXPACK_SMOKE_REPO="$repo_root" \
  CTXPACK_SMOKE_TASK="verify release gate MCP protocol proof" \
  CTXPACK_SMOKE_PATH="crates/ctxpack-mcp/src/lib.rs" \
  CTXPACK_SMOKE_QUERY="prepare_task" \
  bash "$smoke_mcp_protocol_script"

log_step "Cursor setup and MCP protocol evidence"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$repo_root" \
  CTXPACK_REAL_CLIENT_EVIDENCE_DIR="${CTXPACK_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_cursor_mcp_script"

log_step "OpenCode setup and MCP protocol evidence"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$repo_root" \
  CTXPACK_REAL_CLIENT_EVIDENCE_DIR="${CTXPACK_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_opencode_mcp_script"

log_step "optional benchmark product proof"
if [[ -n "${CTXPACK_BENCHMARK_CONFIG:-}" ]]; then
  proof_report="$work_dir/product-proof.json"
  "$ctxpack_bin" eval proof --config "$CTXPACK_BENCHMARK_CONFIG" --format json >"$proof_report"
  python3 "$repo_root/scripts/check-product-proof.py" "$proof_report"
  benchmark_status="passed"
else
  echo "benchmark product proof skipped: set CTXPACK_BENCHMARK_CONFIG=/path/to/suite.json"
  benchmark_status="skipped"
fi

log_step "optional Codex real-client evidence"
codex_status="skipped"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$repo_root" \
  CTXPACK_SMOKE_REPO="$repo_root" \
  CTXPACK_SMOKE_TASK="verify release gate Codex MCP proof" \
  CTXPACK_SMOKE_PATH="crates/ctxpack-mcp/src/lib.rs" \
  CTXPACK_SMOKE_QUERY="prepare_task" \
  CTXPACK_SKIP_REAL_CLIENT="$real_client_skip" \
  CTXPACK_REQUIRE_REAL_CLIENT="$real_client_required" \
  CTXPACK_REAL_CLIENT_EVIDENCE_DIR="${CTXPACK_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_codex_mcp_script"
if [[ "$real_client_skip" != "1" && ( "$real_client_required" == "1" || "${CTXPACK_RUN_REAL_CLIENT:-0}" == "1" ) ]]; then
  codex_status="passed"
fi

log_step "optional Claude real-client evidence"
claude_status="skipped"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$repo_root" \
  CTXPACK_SMOKE_REPO="$repo_root" \
  CTXPACK_SMOKE_TASK="verify release gate Claude MCP proof" \
  CTXPACK_SMOKE_PATH="crates/ctxpack-mcp/src/lib.rs" \
  CTXPACK_SMOKE_QUERY="prepare_task" \
  CTXPACK_SKIP_REAL_CLIENT="$real_client_skip" \
  CTXPACK_REQUIRE_REAL_CLIENT="$real_client_required" \
  CTXPACK_REAL_CLIENT_EVIDENCE_DIR="${CTXPACK_REAL_CLIENT_EVIDENCE_DIR:-}" \
  bash "$smoke_claude_mcp_script"
if [[ "$real_client_skip" != "1" && ( "$real_client_required" == "1" || "${CTXPACK_RUN_REAL_CLIENT:-0}" == "1" ) ]]; then
  claude_status="passed"
fi

log_step "release proof bundle"
mkdir -p "$proof_dir"
python3 - "$proof_summary_path" "$ctxpack_version" "$(basename "$ctxpack_bin")" "$binary_source" "$binary_sha256" "$(basename "$archive_path")" "$archive_sha256" "$(basename "$manifest_path")" "$(basename "$audit_report_path")" "$benchmark_status" "$codex_status" "$claude_status" "$real_client_required" <<'PY'
import json
import sys

(
    proof_summary_path,
    ctxpack_version,
    binary_name,
    binary_source,
    binary_sha256,
    archive_name,
    archive_sha256,
    manifest_name,
    audit_report_name,
    benchmark_status,
    codex_status,
    claude_status,
    real_client_required,
) = sys.argv[1:]

required_checks = [
    "cargo test --workspace",
    "scripts/check-release-docs.sh",
    "scripts/release-package.sh",
    "scripts/verify-release-archive.sh",
    "ctxpack --version",
    "ctxpack --help",
    "scripts/smoke-first-pack.sh",
    "scripts/smoke-storage.sh",
    "scripts/smoke-memory.sh",
    "scripts/smoke-feedback.sh",
    "scripts/smoke-workspace.sh",
    "scripts/smoke-shared-artifacts.sh",
    "scripts/smoke-inspector.sh",
    "scripts/smoke-retrieval-health.sh",
    "scripts/smoke-graph.sh",
    "scripts/smoke-policy-embedding.sh",
    "scripts/smoke-agent-preview.sh",
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
payload = {
    "schemaVersion": 1,
    "status": "passed",
    "ctxpackVersion": ctxpack_version,
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
        "resourceBackedGapSummaryContract": (
            "checked" if benchmark_status == "passed" else "skipped"
        ),
        "codexRealClientProof": codex_status,
        "claudeRealClientProof": claude_status,
        "realClientRequired": real_client_required == "1",
        "cursorRealClientProof": "not_claimed",
        "opencodeRealClientProof": "not_claimed",
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

echo "release gate passed: binary=$ctxpack_bin archive=$archive_path proof=$proof_summary_path"
