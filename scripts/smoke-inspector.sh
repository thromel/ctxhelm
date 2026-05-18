#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

run_ctxpack() {
  if [[ -n "${CTXPACK_BIN:-}" ]]; then
    "${CTXPACK_BIN}" "$@"
  else
    cargo run -q -p ctxpack -- "$@"
  fi
}

require_text() {
  local file="$1"
  local text="$2"
  grep -F -- "$text" "$file" >/dev/null || {
    echo "smoke-inspector: missing '${text}' in ${file}" >&2
    exit 1
  }
}

reject_text() {
  local file="$1"
  local text="$2"
  if grep -F -- "$text" "$file" >/dev/null; then
    echo "smoke-inspector: unsupported source leak '${text}' in ${file}" >&2
    exit 1
  fi
}

REPO="${TMP_DIR}/repo"
mkdir -p "${REPO}/src/auth" "${REPO}/tests/auth"
git -C "${REPO}" init >/dev/null
git -C "${REPO}" config user.email ctxpack@example.com
git -C "${REPO}" config user.name ctxpack
cat >"${REPO}/src/auth/session.ts" <<'EOF'
export function requireSession() {
  return 'INSPECTOR_UI_SOURCE_SENTINEL';
}
EOF
cat >"${REPO}/tests/auth/session.test.ts" <<'EOF'
import { requireSession } from '../../src/auth/session';
test('session', () => requireSession());
EOF
git -C "${REPO}" add .
git -C "${REPO}" commit -m "add session" >/dev/null

JSON_OUT="${TMP_DIR}/inspector.json"
HTML_OUT="${TMP_DIR}/inspector.html"

(
  cd "${ROOT_DIR}"
  run_ctxpack inspector export "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --format json \
    --output "${JSON_OUT}"
  run_ctxpack inspector export "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --format html \
    --output "${HTML_OUT}"
)

require_text "${JSON_OUT}" '"sourceTextLogged": false'
require_text "${JSON_OUT}" '"sourceBearing": true'
require_text "${JSON_OUT}" '"targetFiles"'
reject_text "${JSON_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

require_text "${HTML_OUT}" 'data-inspector-source-free="true"'
require_text "${HTML_OUT}" 'id="filterText"'
require_text "${HTML_OUT}" 'id="kindFilter"'
require_text "${HTML_OUT}" 'id="sourceOnly"'
require_text "${HTML_OUT}" 'Retrieval Candidates'
require_text "${HTML_OUT}" 'Selected Memory'
require_text "${HTML_OUT}" 'hidden-by-filter'
reject_text "${HTML_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

echo "smoke-inspector passed"
