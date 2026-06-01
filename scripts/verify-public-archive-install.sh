#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: verify-public-archive-install.sh [--repo OWNER/REPO] [--tag TAG] [--target-label TARGET] [--expected-version VERSION] [--output PATH]

Downloads ctxpack release assets from the public GitHub release URL, verifies
checksums, installs the binary into a temporary bin directory, and runs
version/help/doctor plus the first-pack smoke against the installed binary.

The script does not install globally, mutate agent configuration, publish,
upload, create tags, or run user project tests.
EOF
}

repo="thromel/ctxpack"
tag="v1.1.4"
target_label="$(rustc -vV 2>/dev/null | awk '/^host:/ { print $2 }')"
expected_version="ctxpack 1.1.4"
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

if [[ -z "$repo" || -z "$tag" || -z "$target_label" || -z "$expected_version" ]]; then
  usage
  exit 64
fi
if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 69
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
verify_release_archive_script="$repo_root/scripts/verify-release-archive.sh"
smoke_first_pack_script="$repo_root/scripts/smoke-first-pack.sh"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

download_dir="$work_dir/download"
extract_dir="$work_dir/extract"
install_dir="$work_dir/install"
mkdir -p "$download_dir" "$extract_dir" "$install_dir/bin"

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
extracted_bin="$extract_dir/$prefix/ctxpack"
if [[ ! -x "$extracted_bin" ]]; then
  echo "extracted ctxpack binary missing or not executable: $extracted_bin" >&2
  exit 65
fi
install -m 0755 "$extracted_bin" "$install_dir/bin/ctxpack"
installed_bin="$install_dir/bin/ctxpack"

version_output="$("$installed_bin" --version)"
if [[ "$version_output" != "$expected_version" ]]; then
  echo "version mismatch: $version_output != $expected_version" >&2
  exit 65
fi
"$installed_bin" --help >/dev/null
doctor_json="$work_dir/doctor.json"
"$installed_bin" doctor \
  --binary "$installed_bin" \
  --release-manifest "$download_dir/$manifest_name" \
  --format json >"$doctor_json"

python3 - "$doctor_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("passed") is not True:
    raise SystemExit("doctor did not pass")
if payload.get("mutatesGlobalAgentConfig") is not False:
    raise SystemExit("doctor unexpectedly mutates global agent config")
privacy = payload.get("privacyStatus", {})
if privacy.get("localOnly") is not True or privacy.get("sourceTextLogged") is not False:
    raise SystemExit("doctor privacy status is not source-free local-only")
PY

CTXPACK_BIN="$installed_bin" bash "$smoke_first_pack_script" >/dev/null

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

if [[ -n "$output_path" ]]; then
  mkdir -p "$(dirname "$output_path")"
  python3 - "$output_path" "$repo" "$tag" "$target_label" "$base_url" "$version_output" "$archive_name" "$archive_sha" "$binary_sha" <<'PY'
import json
import pathlib
import sys

(
    output_path,
    repo,
    tag,
    target_label,
    base_url,
    version_output,
    archive_name,
    archive_sha,
    binary_sha,
) = sys.argv[1:]
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
    "checks": {
        "downloadedPublicAssets": True,
        "checksumsVerified": True,
        "archiveVerified": True,
        "installedToTemporaryBin": True,
        "versionPassed": True,
        "helpPassed": True,
        "doctorPassed": True,
        "firstPackSmokePassed": True,
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
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
fi

printf 'public archive install verified: %s %s\n' "$tag" "$target_label"
