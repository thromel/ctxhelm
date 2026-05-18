#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: verify-release-archive.sh --archive PATH --manifest PATH [--checksums PATH]

Verifies a ctxpack release archive from a clean temporary extraction directory.
The script does not install binaries, publish artifacts, mutate global agent
configuration, or run user project tests.
EOF
}

archive_path=""
manifest_path=""
checksums_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --archive)
      archive_path="${2:-}"
      shift 2
      ;;
    --manifest)
      manifest_path="${2:-}"
      shift 2
      ;;
    --checksums)
      checksums_path="${2:-}"
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

if [[ -z "$archive_path" || -z "$manifest_path" ]]; then
  usage
  exit 64
fi

if [[ ! -f "$archive_path" ]]; then
  echo "archive not found: $archive_path" >&2
  exit 66
fi
if [[ ! -f "$manifest_path" ]]; then
  echo "manifest not found: $manifest_path" >&2
  exit 66
fi
if [[ -n "$checksums_path" && ! -f "$checksums_path" ]]; then
  echo "checksums file not found: $checksums_path" >&2
  exit 66
fi

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    sha256sum "$1" | awk '{ print $1 }'
  fi
}

if [[ -n "$checksums_path" ]]; then
  checksums_dir="$(cd "$(dirname "$checksums_path")" && pwd -P)"
  checksums_name="$(basename "$checksums_path")"
  (
    cd "$checksums_dir"
    if command -v shasum >/dev/null 2>&1; then
      shasum -a 256 -c "$checksums_name" >/dev/null
    else
      sha256sum -c "$checksums_name" >/dev/null
    fi
  )
fi

manifest_archive_name="$(python3 - "$manifest_path" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
privacy = payload.get("privacyStatus", {})
if privacy.get("localOnly") is not True:
    raise SystemExit("manifest privacyStatus.localOnly was not true")
if privacy.get("sourceTextLogged") is not False:
    raise SystemExit("manifest privacyStatus.sourceTextLogged was not false")
archive = payload.get("archive", {})
binary = payload.get("binary", {})
if not archive.get("name") or not archive.get("sha256"):
    raise SystemExit("manifest archive name/sha256 is missing")
if binary.get("name") != "ctxpack" or not binary.get("sha256"):
    raise SystemExit("manifest binary name/sha256 is missing")
print(archive["name"])
PY
)"

if [[ "$(basename "$archive_path")" != "$manifest_archive_name" ]]; then
  echo "archive basename does not match manifest: $(basename "$archive_path") != $manifest_archive_name" >&2
  exit 65
fi

archive_sha="$(sha256_file "$archive_path")"
manifest_sha="$(python3 - "$manifest_path" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload["archive"]["sha256"])
PY
)"
if [[ "$archive_sha" != "$manifest_sha" ]]; then
  echo "archive checksum does not match manifest" >&2
  exit 65
fi

extract_dir="$work_dir/extracted"
mkdir -p "$extract_dir"
tar -xzf "$archive_path" -C "$extract_dir"

ctxpack_bin="$(find "$extract_dir" -type f -name ctxpack -perm -111 | head -n 1)"
if [[ -z "$ctxpack_bin" ]]; then
  echo "no executable ctxpack binary found in archive" >&2
  exit 65
fi

binary_sha="$(sha256_file "$ctxpack_bin")"
manifest_binary_sha="$(python3 - "$manifest_path" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload["binary"]["sha256"])
PY
)"
if [[ "$binary_sha" != "$manifest_binary_sha" ]]; then
  echo "binary checksum does not match manifest" >&2
  exit 65
fi

"$ctxpack_bin" --version >/dev/null
"$ctxpack_bin" --help >/dev/null

doctor_json="$work_dir/doctor.json"
"$ctxpack_bin" doctor \
  --binary "$ctxpack_bin" \
  --release-manifest "$manifest_path" \
  --format json >"$doctor_json"

python3 - "$doctor_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("passed") is not True:
    raise SystemExit("doctor passed flag was not true")
privacy = payload.get("privacyStatus", {})
if privacy.get("localOnly") is not True:
    raise SystemExit("doctor privacyStatus.localOnly was not true")
if payload.get("mutatesGlobalAgentConfig") is not False:
    raise SystemExit("doctor unexpectedly mutates global agent config")
PY

echo "release archive verification passed: $(basename "$archive_path")"
