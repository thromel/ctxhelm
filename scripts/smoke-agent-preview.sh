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
    echo "smoke-agent-preview: missing '${text}' in ${file}" >&2
    exit 1
  }
}

reject_text() {
  local file="$1"
  local text="$2"
  if grep -F -- "$text" "$file" >/dev/null; then
    echo "smoke-agent-preview: unsupported source leak '${text}' in ${file}" >&2
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
  return 'AGENT_PREVIEW_SOURCE_SENTINEL';
}
EOF
cat >"${REPO}/tests/auth/session.test.ts" <<'EOF'
import { requireSession } from '../../src/auth/session';
test('session', () => requireSession());
EOF
git -C "${REPO}" add .
git -C "${REPO}" commit -m "add auth preview baseline" >/dev/null

JSON_OUT="${TMP_DIR}/agent-preview.json"
MD_OUT="${TMP_DIR}/agent-preview.md"

(
  cd "${ROOT_DIR}"
  run_ctxpack agent preview "fix requireSession preview bug" \
    --repo "${REPO}" \
    --mode bug-fix \
    --budget brief \
    --format json >"${JSON_OUT}"
  run_ctxpack agent preview "fix requireSession preview bug" \
    --repo "${REPO}" \
    --target-agent claude-code \
    --mode bug-fix \
    --format markdown >"${MD_OUT}"
)

require_text "${JSON_OUT}" '"targetAgent": "codex"'
require_text "${JSON_OUT}" '"targetAgent": "claude-code"'
require_text "${JSON_OUT}" '"targetAgent": "cursor"'
require_text "${JSON_OUT}" '"targetAgent": "opencode"'
require_text "${JSON_OUT}" '"targetAgent": "generic"'
require_text "${JSON_OUT}" '"prepare_task"'
require_text "${JSON_OUT}" '"get_pack"'
require_text "${JSON_OUT}" '"ctxpack://repo/summary"'
require_text "${JSON_OUT}" '"AGENTS.md"'
require_text "${JSON_OUT}" '"sourceTextLogged": false'
require_text "${JSON_OUT}" '"sourceTextIncluded": false'
require_text "${JSON_OUT}" '"localOnly": true'
require_text "${MD_OUT}" "Claude Code"
require_text "${MD_OUT}" ".claude/commands/ctxpack-bugfix.md"
require_text "${MD_OUT}" "ctxpack suggests target files"
require_text "${MD_OUT}" "native file tools"
reject_text "${JSON_OUT}" "AGENT_PREVIEW_SOURCE_SENTINEL"
reject_text "${MD_OUT}" "AGENT_PREVIEW_SOURCE_SENTINEL"

echo "smoke-agent-preview passed"
