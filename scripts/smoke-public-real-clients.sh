#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: smoke-public-real-clients.sh [--repo OWNER/REPO] [--tag TAG] [--target-label TARGET] [--expected-version VERSION] [--smoke-repo PATH] [--output PATH]

Downloads ctxpack release assets from the public GitHub release URL, verifies
the archive, and runs the Codex CLI plus Claude Code MCP smoke wrappers against
the extracted release binary. Real-client checks remain optional unless
CTXPACK_REQUIRE_REAL_CLIENT=1 is set; skipped clients write source-free evidence
when an output path is provided.

This script does not install globally, mutate agent configuration, publish,
upload, create tags, or run user project tests.
EOF
}

repo="thromel/ctxpack"
tag="v1.1.6"
target_label="$(rustc -vV 2>/dev/null | awk '/^host:/ { print $2 }')"
expected_version="ctxpack 1.1.6"
smoke_repo="$PWD"
output_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --tag)
      tag="${2:-}"
      shift 2
      ;;
    --target-label)
      target_label="${2:-}"
      shift 2
      ;;
    --expected-version)
      expected_version="${2:-}"
      shift 2
      ;;
    --smoke-repo)
      smoke_repo="${2:-}"
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

if [[ -z "$repo" || -z "$tag" || -z "$target_label" || -z "$expected_version" || -z "$smoke_repo" ]]; then
  usage
  exit 64
fi
if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 69
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
verify_release_archive_script="$repo_root/scripts/verify-release-archive.sh"
codex_smoke_script="$repo_root/scripts/smoke-codex-mcp.sh"
claude_smoke_script="$repo_root/scripts/smoke-claude-mcp.sh"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

download_dir="$work_dir/download"
extract_dir="$work_dir/extract"
evidence_dir="$work_dir/evidence"
mkdir -p "$download_dir" "$extract_dir" "$evidence_dir"

if [[ -n "$output_path" ]]; then
  output_abs="$(python3 - "$output_path" <<'PY'
import pathlib
import sys
print(pathlib.Path(sys.argv[1]).resolve())
PY
)"
  mkdir -p "$(dirname "$output_abs")"
  evidence_dir="${output_abs%.json}-evidence"
  rm -rf "$evidence_dir"
  mkdir -p "$evidence_dir"
else
  output_abs=""
fi

version="${tag#v}"
prefix="ctxpack-v${version}-${target_label}"
archive_name="${prefix}.tar.gz"
manifest_name="${prefix}.manifest.json"
audit_name="${prefix}.audit.json"
archive_checksum_name="${archive_name}.sha256"
checksums_name="sha256sums.txt"
base_url="https://github.com/${repo}/releases/download/${tag}"

download_asset() {
  local name="$1"
  curl -fsSL "$base_url/$name" -o "$download_dir/$name"
}

download_asset "$archive_name"
download_asset "$manifest_name"
download_asset "$audit_name"
download_asset "$archive_checksum_name"
download_asset "$checksums_name"

(
  cd "$download_dir"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "$checksums_name" >/dev/null
    shasum -a 256 -c "$archive_checksum_name" >/dev/null
  else
    sha256sum -c "$checksums_name" >/dev/null
    sha256sum -c "$archive_checksum_name" >/dev/null
  fi
)

bash "$verify_release_archive_script" \
  --archive "$download_dir/$archive_name" \
  --manifest "$download_dir/$manifest_name" \
  --checksums "$download_dir/$checksums_name" >/dev/null

tar -xzf "$download_dir/$archive_name" -C "$extract_dir"
ctxpack_bin="$extract_dir/$prefix/ctxpack"
if [[ ! -x "$ctxpack_bin" ]]; then
  echo "extracted ctxpack binary missing or not executable: $ctxpack_bin" >&2
  exit 65
fi

version_output="$("$ctxpack_bin" --version)"
if [[ "$version_output" != "$expected_version" ]]; then
  echo "version mismatch: $version_output != $expected_version" >&2
  exit 65
fi

run_smoke() {
  local client="$1"
  local script="$2"
  local stdout_log="$work_dir/${client}.stdout"
  local stderr_log="$work_dir/${client}.stderr"
  set +e
  local require_resource_scope="${CTXPACK_REQUIRE_RESOURCE_SCOPE:-1}"
  if [[ "$expected_version" == "ctxpack 1.1.0" ]]; then
    require_resource_scope="${CTXPACK_PUBLIC_SMOKE_REQUIRE_RESOURCE_SCOPE:-0}"
  fi
  CTXPACK_BIN="$ctxpack_bin" \
    CTXPACK_SMOKE_REPO="$smoke_repo" \
    CTXPACK_RUN_REAL_CLIENT="${CTXPACK_RUN_REAL_CLIENT:-1}" \
    CTXPACK_REQUIRE_REAL_CLIENT="${CTXPACK_REQUIRE_REAL_CLIENT:-0}" \
    CTXPACK_SKIP_REAL_CLIENT="${CTXPACK_SKIP_REAL_CLIENT:-0}" \
    CTXPACK_REQUIRE_RESOURCE_SCOPE="$require_resource_scope" \
    CTXPACK_REAL_CLIENT_EVIDENCE_DIR="$evidence_dir" \
    bash "$script" >"$stdout_log" 2>"$stderr_log"
  local status=$?
  set -e
  printf '%s\n' "$status" >"$work_dir/${client}.status"
  if [[ "$status" != "0" && "${CTXPACK_REQUIRE_REAL_CLIENT:-0}" == "1" ]]; then
    echo "public ${client} real-client smoke failed; see wrapper diagnostics above" >&2
  fi
}

run_smoke "codex" "$codex_smoke_script"
run_smoke "claude" "$claude_smoke_script"

archive_sha="$(python3 - "$download_dir/$manifest_name" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload["archive"]["sha256"])
PY
)"
binary_sha="$(python3 - "$download_dir/$manifest_name" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload["binary"]["sha256"])
PY
)"

summary_json="$(python3 - "$repo" "$tag" "$target_label" "$base_url" "$version_output" "$archive_name" "$archive_sha" "$binary_sha" "$smoke_repo" "$evidence_dir" "$work_dir/codex.status" "$work_dir/claude.status" <<'PY'
import json
import pathlib
import sys

(
    repo,
    tag,
    target_label,
    base_url,
    version_output,
    archive_name,
    archive_sha,
    binary_sha,
    smoke_repo,
    evidence_dir,
    codex_status_path,
    claude_status_path,
) = sys.argv[1:]

def read_status(path):
    return int(pathlib.Path(path).read_text().strip())

def read_evidence(client):
    path = pathlib.Path(evidence_dir) / f"{client}-mcp-evidence.json"
    if not path.exists():
        return {
            "client": client,
            "status": "missing_evidence",
            "evidenceFile": path.name,
        }
    payload = json.loads(path.read_text())
    payload["evidenceFile"] = path.name
    return payload

payload = {
    "schemaVersion": 1,
    "repo": repo,
    "tag": tag,
    "targetLabel": target_label,
    "releaseUrl": f"https://github.com/{repo}/releases/tag/{tag}",
    "downloadBaseUrl": base_url,
    "archiveName": archive_name,
    "archiveSha256": archive_sha,
    "binarySha256": binary_sha,
    "version": version_output,
    "smokeRepo": str(pathlib.Path(smoke_repo).resolve()),
    "checks": {
        "downloadedPublicAssets": True,
        "checksumsVerified": True,
        "archiveVerified": True,
        "versionPassed": True,
    },
    "clients": {
        "codex": read_evidence("codex") | {"wrapperExitStatus": read_status(codex_status_path)},
        "claude": read_evidence("claude") | {"wrapperExitStatus": read_status(claude_status_path)},
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "unsupportedActions": [
        "global install",
        "global agent config mutation",
        "publishing",
        "tag creation",
        "user project test execution",
    ],
}
print(json.dumps(payload, sort_keys=True))
PY
)"

if [[ -n "$output_abs" ]]; then
  printf '%s\n' "$summary_json" >"$output_abs"
fi

python3 - "$summary_json" <<'PY'
import json
import sys
payload = json.loads(sys.argv[1])
clients = payload["clients"]
for client, evidence in clients.items():
    if evidence.get("status") == "missing_evidence":
        raise SystemExit(f"{client} did not write real-client evidence")
    if evidence.get("wrapperExitStatus") != 0:
        raise SystemExit(f"{client} wrapper exited non-zero")
print(f"public real-client smoke checked: {payload['tag']} {payload['targetLabel']}")
PY
