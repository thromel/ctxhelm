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
    echo "smoke-graph: missing '${text}' in ${file}" >&2
    exit 1
  }
}

reject_text() {
  local file="$1"
  local text="$2"
  if grep -F -- "$text" "$file" >/dev/null; then
    echo "smoke-graph: unsupported source leak '${text}' in ${file}" >&2
    exit 1
  fi
}

REPO="${TMP_DIR}/repo"
mkdir -p "${REPO}/src/auth" "${REPO}/tests/auth"
git -C "${REPO}" init >/dev/null
git -C "${REPO}" config user.email ctxpack@example.com
git -C "${REPO}" config user.name ctxpack
cat >"${REPO}/src/auth/cookies.ts" <<'EOF'
export function parseCookie() {
  return 'GRAPH_SOURCE_SENTINEL';
}
EOF
cat >"${REPO}/src/auth/session.ts" <<'EOF'
import { parseCookie } from './cookies';
export function requireSession() {
  return parseCookie();
}
EOF
cat >"${REPO}/tests/auth/session.test.ts" <<'EOF'
import { requireSession } from '../../src/auth/session';
test('session', () => requireSession());
EOF
git -C "${REPO}" add .
git -C "${REPO}" commit -m "add auth graph" >/dev/null

JSON_OUT="${TMP_DIR}/graph.json"
MD_OUT="${TMP_DIR}/graph.md"

(
  cd "${ROOT_DIR}"
  run_ctxpack graph neighborhood "fix requireSession graph" \
    --repo "${REPO}" \
    --mode bug-fix \
    --path src/auth/session.ts \
    --format json >"${JSON_OUT}"
  run_ctxpack graph neighborhood "fix requireSession graph" \
    --repo "${REPO}" \
    --mode bug-fix \
    --path src/auth/session.ts \
    --format markdown >"${MD_OUT}"
)

require_text "${JSON_OUT}" '"sourceTextLogged": false'
require_text "${JSON_OUT}" '"nodes"'
require_text "${JSON_OUT}" '"edges"'
require_text "${JSON_OUT}" '"communities"'
require_text "${JSON_OUT}" 'src/auth/session.ts'
require_text "${JSON_OUT}" 'tests/auth/session.test.ts'
require_text "${MD_OUT}" "ctxpack Graph Neighborhood"
require_text "${MD_OUT}" "Communities"
require_text "${MD_OUT}" "Edges"
reject_text "${JSON_OUT}" "GRAPH_SOURCE_SENTINEL"
reject_text "${MD_OUT}" "GRAPH_SOURCE_SENTINEL"

echo "smoke-graph passed"
