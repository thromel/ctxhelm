#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
release_package_script="$repo_root/scripts/release-package.sh"
check_release_docs_script="$repo_root/scripts/check-release-docs.sh"
smoke_first_pack_script="$repo_root/scripts/smoke-first-pack.sh"
smoke_storage_script="$repo_root/scripts/smoke-storage.sh"
smoke_semantic_script="$repo_root/scripts/smoke-semantic.sh"
smoke_precision_script="$repo_root/scripts/smoke-precision.sh"
smoke_mcp_protocol_script="$repo_root/scripts/smoke-mcp-protocol.sh"
smoke_codex_mcp_script="$repo_root/scripts/smoke-codex-mcp.sh"
smoke_claude_mcp_script="$repo_root/scripts/smoke-claude-mcp.sh"

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

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
if [[ -n "${CTXPACK_BIN:-}" ]]; then
  ctxpack_bin="$(canonical_executable "$CTXPACK_BIN")"
else
  ctxpack_bin="$(extract_binary_from_archive "$archive_path")"
fi
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
"$ctxpack_bin" --version
"$ctxpack_bin" --help >/dev/null

log_step "first-pack smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_first_pack_script"

log_step "storage smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_storage_script"

log_step "semantic smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_semantic_script"

log_step "precision smoke"
CTXPACK_BIN="$ctxpack_bin" bash "$smoke_precision_script"

log_step "wrong-cwd MCP protocol smoke"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$repo_root" \
  CTXPACK_SMOKE_REPO="$repo_root" \
  CTXPACK_SMOKE_TASK="verify release gate MCP protocol proof" \
  CTXPACK_SMOKE_PATH="crates/ctxpack-mcp/src/lib.rs" \
  CTXPACK_SMOKE_QUERY="prepare_task" \
  bash "$smoke_mcp_protocol_script"

log_step "optional benchmark product proof"
if [[ -n "${CTXPACK_BENCHMARK_CONFIG:-}" ]]; then
  proof_report="$work_dir/product-proof.json"
  "$ctxpack_bin" eval proof --config "$CTXPACK_BENCHMARK_CONFIG" --format json >"$proof_report"
  python3 - "$proof_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if not report.get("privacyStatus", {}).get("localOnly"):
    raise SystemExit("product proof privacyStatus.localOnly was not true")
if not report.get("benchmarkReport", {}).get("privacyStatus", {}).get("localOnly"):
    raise SystemExit("embedded benchmark privacyStatus.localOnly was not true")
if not report.get("headlineMetrics"):
    raise SystemExit("product proof headlineMetrics were empty")
PY
else
  echo "benchmark product proof skipped: set CTXPACK_BENCHMARK_CONFIG=/path/to/suite.json"
fi

log_step "optional Codex real-client evidence"
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

log_step "optional Claude real-client evidence"
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

echo "release gate passed: binary=$ctxpack_bin archive=$archive_path"
