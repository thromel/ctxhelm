#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

run_ctxhelm() {
  if [[ -n "${CTXHELM_BIN:-}" ]]; then
    "${CTXHELM_BIN}" "$@"
  else
    cargo run -q -p ctxhelm -- "$@"
  fi
}

require_text() {
  local file="$1"
  local text="$2"
  grep -F -- "$text" "$file" >/dev/null || {
    echo "smoke-retrieval-health: missing '${text}' in ${file}" >&2
    exit 1
  }
}

reject_text() {
  local file="$1"
  local text="$2"
  if grep -F -- "$text" "$file" >/dev/null; then
    echo "smoke-retrieval-health: unsupported source leak '${text}' in ${file}" >&2
    exit 1
  fi
}

REPO="${TMP_DIR}/repo"
mkdir -p "${REPO}/src/auth" "${REPO}/tests/auth"
git -C "${REPO}" init >/dev/null
git -C "${REPO}" config user.email ctxhelm@example.com
git -C "${REPO}" config user.name ctxhelm
cat >"${REPO}/src/auth/session.ts" <<'EOF'
export function requireSession() {
  return true;
}
EOF
cat >"${REPO}/tests/auth/session.test.ts" <<'EOF'
import { requireSession } from '../../src/auth/session';
test('session', () => requireSession());
EOF
git -C "${REPO}" add .
git -C "${REPO}" commit -m "add auth session" >/dev/null
cat >"${REPO}/src/auth/session.ts" <<'EOF'
export function requireSession() {
  return 'RETRIEVAL_HEALTH_SOURCE_SENTINEL';
}
EOF
git -C "${REPO}" add .
git -C "${REPO}" commit -m "fix requireSession bug" >/dev/null

JSON_OUT="${TMP_DIR}/health.json"
MD_OUT="${TMP_DIR}/health.md"

(
  cd "${ROOT_DIR}"
  run_ctxhelm eval health --repo "${REPO}" --limit 2 --format json >"${JSON_OUT}"
  run_ctxhelm eval health --repo "${REPO}" --limit 2 --format markdown >"${MD_OUT}"
)

require_text "${JSON_OUT}" '"sourceTextLogged": false'
require_text "${JSON_OUT}" '"metrics"'
require_text "${JSON_OUT}" '"signalContributions"'
require_text "${JSON_OUT}" '"gapFamilies"'
require_text "${MD_OUT}" "ctxhelm Retrieval Health Report"
require_text "${MD_OUT}" "Signal Contributions"
require_text "${MD_OUT}" "Gap Families"
reject_text "${JSON_OUT}" "RETRIEVAL_HEALTH_SOURCE_SENTINEL"
reject_text "${MD_OUT}" "RETRIEVAL_HEALTH_SOURCE_SENTINEL"

echo "smoke-retrieval-health passed"
