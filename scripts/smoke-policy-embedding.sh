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
    echo "smoke-policy-embedding: missing '${text}' in ${file}" >&2
    exit 1
  }
}

reject_text() {
  local file="$1"
  local text="$2"
  if grep -F -- "$text" "$file" >/dev/null; then
    echo "smoke-policy-embedding: unsupported source leak '${text}' in ${file}" >&2
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
  return 'POLICY_EMBED_SOURCE_SENTINEL';
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
git -C "${REPO}" commit -m "add auth policy baseline" >/dev/null
printf "\nexport function requireLogin() { return requireSession(); }\n" >>"${REPO}/src/auth/session.ts"
git -C "${REPO}" add .
git -C "${REPO}" commit -m "fix requireSession policy bug" >/dev/null

STATUS_JSON="${TMP_DIR}/semantic-status.json"
EXPERIMENT_JSON="${TMP_DIR}/policy-experiments.json"

(
  cd "${ROOT_DIR}"
  run_ctxpack semantic status \
    --repo "${REPO}" \
    --query "fix requireSession policy bug" \
    --mode bug-fix \
    --format json >"${STATUS_JSON}"
  run_ctxpack eval policy experiments "fix requireSession policy bug" \
    --repo "${REPO}" \
    --limit 2 \
    --format json >"${EXPERIMENT_JSON}"
)

require_text "${STATUS_JSON}" '"providerKind": "local_hash"'
require_text "${STATUS_JSON}" '"providerRole": "deterministic_scaffold"'
require_text "${STATUS_JSON}" '"qualityBackend": false'
require_text "${STATUS_JSON}" '"localOnly": true'
require_text "${STATUS_JSON}" '"providerAvailable": true'
require_text "${STATUS_JSON}" '"enabledByDefault": false'
require_text "${STATUS_JSON}" '"cloudEmbeddingsAllowed": false'
require_text "${STATUS_JSON}" '"cloudRerankingAllowed": false'
require_text "${STATUS_JSON}" '"providerPolicy"'
require_text "${STATUS_JSON}" '"allowCloudEmbeddings": false'
require_text "${STATUS_JSON}" '"allowCloudReranking": false'
require_text "${STATUS_JSON}" '"allowSourceTransfer": false'
require_text "${STATUS_JSON}" '"enableLocalFixtureReranker": false'
require_text "${STATUS_JSON}" '"provider": "cloud_embedding"'
require_text "${STATUS_JSON}" '"status": "denied"'
require_text "${STATUS_JSON}" '"provider": "local_fixture"'
require_text "${STATUS_JSON}" '"status": "disabled"'
require_text "${STATUS_JSON}" '"sourceTextLogged": false'
require_text "${EXPERIMENT_JSON}" '"lexical_only"'
require_text "${EXPERIMENT_JSON}" '"hybrid_local_semantic"'
require_text "${EXPERIMENT_JSON}" '"graph_neighborhood"'
require_text "${EXPERIMENT_JSON}" '"policy_experiment_default_unchanged"'
require_text "${EXPERIMENT_JSON}" '"providerPolicy"'
require_text "${EXPERIMENT_JSON}" '"sourceTextLogged": false'
reject_text "${STATUS_JSON}" "POLICY_EMBED_SOURCE_SENTINEL"
reject_text "${EXPERIMENT_JSON}" "POLICY_EMBED_SOURCE_SENTINEL"

echo "smoke-policy-embedding passed"
