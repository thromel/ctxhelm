#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${CTXPACK_DIST_DIR:-"${REPO_ROOT}/dist"}"
CARGO_BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-"${REPO_ROOT}/target"}"
STAGING_PARENT="$(mktemp -d)"
EXTRACT_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "${STAGING_PARENT}" "${EXTRACT_DIR}"
}
trap cleanup EXIT

cd "${REPO_ROOT}"

METADATA_PATH="${STAGING_PARENT}/cargo-metadata.json"
cargo metadata --no-deps --format-version 1 > "${METADATA_PATH}"
VERSION="$(python3 - "${METADATA_PATH}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)

print(next(package["version"] for package in data["packages"] if package["name"] == "ctxpack"))
PY
)"
TARGET_LABEL="${CTXPACK_TARGET_LABEL:-$(rustc -vV | awk '/^host:/ { print $2 }')}"
ARCHIVE_NAME="ctxpack-v${VERSION}-${TARGET_LABEL}.tar.gz"
ARCHIVE_PATH="${DIST_DIR}/${ARCHIVE_NAME}"
MANIFEST_NAME="ctxpack-v${VERSION}-${TARGET_LABEL}.manifest.json"
MANIFEST_PATH="${DIST_DIR}/${MANIFEST_NAME}"
AUDIT_REPORT_NAME="ctxpack-v${VERSION}-${TARGET_LABEL}.audit.json"
AUDIT_REPORT_PATH="${DIST_DIR}/${AUDIT_REPORT_NAME}"
SHA256SUMS_PATH="${DIST_DIR}/sha256sums.txt"

if [[ "${CTXPACK_ALLOW_DIRTY:-0}" != "1" ]]; then
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "refusing to package from a dirty checkout; set CTXPACK_ALLOW_DIRTY=1 to override" >&2
    exit 65
  fi
fi

mkdir -p "${DIST_DIR}"
cargo build -p ctxpack --release --locked

STAGING_DIR="${STAGING_PARENT}/ctxpack-v${VERSION}-${TARGET_LABEL}"
mkdir -p "${STAGING_DIR}"
cp "${CARGO_BUILD_TARGET_DIR}/release/ctxpack" "${STAGING_DIR}/ctxpack"
cp "${REPO_ROOT}/README.md" "${STAGING_DIR}/README.md"
cp "${REPO_ROOT}/LICENSE" "${STAGING_DIR}/LICENSE"
printf 'ctxpack %s\n' "${VERSION}" > "${STAGING_DIR}/VERSION"

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    sha256sum "$1" | awk '{ print $1 }'
  fi
}

rm -f "${ARCHIVE_PATH}" "${ARCHIVE_PATH}.sha256" "${MANIFEST_PATH}" "${AUDIT_REPORT_PATH}" "${SHA256SUMS_PATH}"
tar -czf "${ARCHIVE_PATH}" -C "${STAGING_PARENT}" "$(basename "${STAGING_DIR}")"
CTXPACK_AUDIT_REPORT="${AUDIT_REPORT_PATH}" "${SCRIPT_DIR}/audit-release-artifact.sh" "${ARCHIVE_PATH}"

ARCHIVE_SHA256="$(sha256_file "${ARCHIVE_PATH}")"
BINARY_SHA256="$(sha256_file "${STAGING_DIR}/ctxpack")"
python3 - "${MANIFEST_PATH}" "${VERSION}" "${TARGET_LABEL}" "${ARCHIVE_NAME}" "${ARCHIVE_SHA256}" "${BINARY_SHA256}" "${AUDIT_REPORT_NAME}" <<'PY'
import json
import sys

(
    manifest_path,
    version,
    target_label,
    archive_name,
    archive_sha256,
    binary_sha256,
    audit_report_name,
) = sys.argv[1:]
manifest = {
    "schemaVersion": 1,
    "package": "ctxpack",
    "version": version,
    "targetLabel": target_label,
    "archive": {
        "name": archive_name,
        "sha256": archive_sha256,
        "format": "tar.gz",
    },
    "binary": {
        "name": "ctxpack",
        "sha256": binary_sha256,
    },
    "includedFiles": ["ctxpack", "README.md", "LICENSE", "VERSION"],
    "auditReport": audit_report_name,
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
        "cloud upload",
    ],
}
with open(manifest_path, "w", encoding="utf-8") as handle:
    json.dump(manifest, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY

(
  cd "${DIST_DIR}"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${ARCHIVE_NAME}" "${MANIFEST_NAME}" "${AUDIT_REPORT_NAME}" > "${ARCHIVE_NAME}.sha256"
  else
    sha256sum "${ARCHIVE_NAME}" "${MANIFEST_NAME}" "${AUDIT_REPORT_NAME}" > "${ARCHIVE_NAME}.sha256"
  fi
  cp "${ARCHIVE_NAME}.sha256" sha256sums.txt
)

tar -xzf "${ARCHIVE_PATH}" -C "${EXTRACT_DIR}"
EXTRACTED_BIN="${EXTRACT_DIR}/ctxpack-v${VERSION}-${TARGET_LABEL}/ctxpack"
(
  cd "${EXTRACT_DIR}"
  "${EXTRACTED_BIN}" --version >/dev/null
  "${EXTRACTED_BIN}" --help >/dev/null
)

echo "created ${ARCHIVE_PATH}"
echo "wrote ${MANIFEST_PATH}"
echo "wrote ${AUDIT_REPORT_PATH}"
echo "wrote ${ARCHIVE_PATH}.sha256"
echo "wrote ${SHA256SUMS_PATH}"
