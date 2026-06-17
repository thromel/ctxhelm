#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
SERVER_PID=""
cleanup() {
  if [[ -n "${SERVER_PID}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
    wait "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

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
git -C "${REPO}" config user.email ctxhelm@example.com
git -C "${REPO}" config user.name ctxhelm
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
PROOF_REPORT="${TMP_DIR}/agent-run-proof.json"
PROOF_MD_OUT="${TMP_DIR}/proof-inspector.md"
PROOF_JSON_OUT="${TMP_DIR}/proof-inspector.json"
PRODUCT_PROOF_REPORT="${TMP_DIR}/product-proof.json"
PRODUCT_PROOF_JSON_OUT="${TMP_DIR}/product-proof-inspector.json"
PROOF_BUNDLE_JSON_OUT="${TMP_DIR}/proof-bundle-inspector.json"

cat >"${PROOF_REPORT}" <<'EOF'
{
  "schemaVersion": "ctxhelm-agent-run-eval-v1",
  "status": "passed",
  "workflowKind": "paired-agent-context-suite",
  "client": {
    "name": "codex",
    "version": "codex-cli smoke"
  },
  "aggregate": {
    "taskCount": 2,
    "comparisonEligibleCount": 2,
    "comparableCtxhelmLaneCount": 1,
    "targetCoverageDeltaAverage": 0.25,
    "targetReadCoverageDeltaAverage": 0.25,
    "irrelevantReadDeltaSum": -2,
    "forbiddenToolCallsObserved": false,
    "missingRequiredCtxhelmCallsObserved": false,
    "invalidRequiredCtxhelmCallsObserved": false,
    "clientFailuresObserved": false,
    "rateLimitsObserved": false,
    "ctxhelmEvidenceMissesObserved": false,
    "ctxhelmEvidenceOnlyTargetsObserved": true,
    "ctxhelmUnderReadTargetsObserved": false,
    "retryCost": {
      "retryTriggeredLanes": 1,
      "retrySelectedLanes": 1,
      "avgReadFilesBeforeRetry": 4.0,
      "avgReadFilesAfterRetry": 5.0,
      "avgIrrelevantReadsBeforeRetry": 2.0,
      "avgIrrelevantReadsAfterRetry": 1.0,
      "targetReadCoverageBeforeRetry": 0.5,
      "targetReadCoverageAfterRetry": 0.75,
      "evidenceOnlyTargetsBeforeRetry": 1,
      "evidenceOnlyTargetsAfterRetry": 0
    },
    "readEfficiency": {
      "analysisAvailable": true,
      "baselineLane": "baseline",
      "efficientCtxhelmLane": "ctxhelm-brief",
      "efficientTargetReadCoverage": 0.75,
      "efficientTargetReadPrecision": 0.6,
      "efficientIrrelevantReadCount": 1
    },
    "outcomeClaim": "ctxhelm_improved",
    "laneSummaries": [
      {
        "lane": "baseline",
        "averageTargetReadCoverage": 0.5,
        "targetReadPrecision": 0.25,
        "irrelevantReadCount": 3,
        "ctxhelmEvidenceOnlyTargetCount": 0
      },
      {
        "lane": "ctxhelm-brief",
        "averageTargetReadCoverage": 0.75,
        "targetReadPrecision": 0.6,
        "irrelevantReadCount": 1,
        "ctxhelmEvidenceOnlyTargetCount": 0
      }
    ]
  },
  "privacyStatus": {
    "localOnly": true,
    "sourceTextLogged": false,
    "rawPromptStored": false,
    "rawTranscriptStored": false,
    "rawMcpTrafficStored": false,
    "remoteEmbeddingsUsed": false,
    "remoteRerankingUsed": false
  }
}
EOF

cat >"${PRODUCT_PROOF_REPORT}" <<'EOF'
{
  "suiteName": "inspector-product-proof-smoke",
  "suiteId": "suite-hash",
  "evaluatedRepositoryCount": 1,
  "evaluatedCommitCount": 3,
  "releaseGate": {
    "decision": "promote",
    "defaultPromotionAllowed": true,
    "decisionReason": "Promote: source-free proof is clean.",
    "lexicalComparison": {
      "contextClaim": "beats_all_corpora",
      "agentEvidenceClaim": "beats_all_corpora",
      "allFileClaim": "mixed",
      "averageContextDeltaAt10": 0.2,
      "averageAgentEvidenceDeltaAt10": 0.3,
      "averageFileDeltaAt10": 0.1
    },
    "corpusVerdicts": [
      {
        "repository": "fixture",
        "status": "beat",
        "protectedEvidenceTargetMissRateAt10": 0.0
      }
    ]
  },
  "privacyStatus": {
    "localOnly": true,
    "sourceTextLogged": false,
    "rawPromptStored": false,
    "rawTranscriptStored": false,
    "rawMcpTrafficStored": false,
    "remoteEmbeddingsUsed": false,
    "remoteRerankingUsed": false
  }
}
EOF

(
  cd "${ROOT_DIR}"
  run_ctxhelm inspector export "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --format json \
    --output "${JSON_OUT}"
  run_ctxhelm inspector export "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --format html \
    --output "${HTML_OUT}"
  run_ctxhelm inspector proof \
    --repo "${REPO}" \
    --report "${PROOF_REPORT}" \
    --output "${PROOF_MD_OUT}"
  run_ctxhelm inspector proof \
    --repo "${REPO}" \
    --report "${PROOF_REPORT}" \
    --format json \
    --output "${PROOF_JSON_OUT}"
  run_ctxhelm inspector proof \
    --repo "${REPO}" \
    --report "${PRODUCT_PROOF_REPORT}" \
    --format json \
    --output "${PRODUCT_PROOF_JSON_OUT}"
  run_ctxhelm inspector proof \
    --repo "${REPO}" \
    --report "${PRODUCT_PROOF_REPORT}" \
    --report "${PROOF_REPORT}" \
    --format json \
    --output "${PROOF_BUNDLE_JSON_OUT}"
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

require_text "${PROOF_MD_OUT}" '# ctxhelm Proof Inspector'
require_text "${PROOF_MD_OUT}" 'Report kind: `agent_run_suite`'
require_text "${PROOF_MD_OUT}" 'Claim: `ctxhelm_improved`'
require_text "${PROOF_MD_OUT}" 'Evidence-only targets after retry: `0`'
require_text "${PROOF_MD_OUT}" 'Summary source-free: `true`'
require_text "${PROOF_MD_OUT}" 'Use as source-free outcome evidence'
reject_text "${PROOF_MD_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

require_text "${PROOF_JSON_OUT}" '"schemaVersion": "ctxhelm-proof-inspector-v1"'
require_text "${PROOF_JSON_OUT}" '"reportKind": "agent_run_suite"'
require_text "${PROOF_JSON_OUT}" '"claim": "ctxhelm_improved"'
require_text "${PROOF_JSON_OUT}" '"evidenceOnlyTargetsAfterRetry": 0'
require_text "${PROOF_JSON_OUT}" '"sourceFreeSummary": true'
reject_text "${PROOF_JSON_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

require_text "${PRODUCT_PROOF_JSON_OUT}" '"reportKind": "product_proof"'
require_text "${PRODUCT_PROOF_JSON_OUT}" '"releaseGateDecision": "promote"'
require_text "${PRODUCT_PROOF_JSON_OUT}" '"contextClaim": "beats_all_corpora"'
require_text "${PRODUCT_PROOF_JSON_OUT}" '"maxProtectedTargetMissRateAt10": 0.0'
require_text "${PRODUCT_PROOF_JSON_OUT}" '"sourceFreeSummary": true'
reject_text "${PRODUCT_PROOF_JSON_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

require_text "${PROOF_BUNDLE_JSON_OUT}" '"schemaVersion": "ctxhelm-proof-inspector-bundle-v1"'
require_text "${PROOF_BUNDLE_JSON_OUT}" '"maturityVerdict": "release_and_agent_outcome_evidence_ready"'
require_text "${PROOF_BUNDLE_JSON_OUT}" '"cleanProductProofCount": 1'
require_text "${PROOF_BUNDLE_JSON_OUT}" '"cleanAgentOutcomeCount": 1'
require_text "${PROOF_BUNDLE_JSON_OUT}" '"privacyBoundaryFailed": false'
reject_text "${PROOF_BUNDLE_JSON_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

PORT="$(python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"
SHELL_OUT="${TMP_DIR}/shell.html"
SHELL_INSPECTOR_JSON="${TMP_DIR}/shell-inspector.json"
SHELL_GRAPH_HTML="${TMP_DIR}/shell-graph.html"
SHELL_GRAPH_JSON="${TMP_DIR}/shell-graph.json"
SHELL_SETUP_JSON="${TMP_DIR}/shell-setup.json"
SHELL_HEALTH_JSON="${TMP_DIR}/shell-health.json"
SERVER_LOG="${TMP_DIR}/server.log"

(
  cd "${ROOT_DIR}"
  run_ctxhelm inspector serve "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --target-agent codex \
    --port "${PORT}" >"${SERVER_LOG}" 2>&1
) &
SERVER_PID="$!"

fetch_url() {
  local path="$1"
  local output="$2"
  python3 - "$PORT" "$path" "$output" <<'PY'
import pathlib
import sys
import urllib.request

port, path, output = sys.argv[1:]
with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=2) as response:
    pathlib.Path(output).write_bytes(response.read())
PY
}

for _ in $(seq 1 50); do
  if fetch_url "/health.json" "${SHELL_HEALTH_JSON}" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "${SERVER_PID}" >/dev/null 2>&1; then
    cat "${SERVER_LOG}" >&2 || true
    echo "smoke-inspector: inspector shell exited before health check" >&2
    exit 1
  fi
  sleep 0.1
done

fetch_url "/" "${SHELL_OUT}"
fetch_url "/pack-inspector.json" "${SHELL_INSPECTOR_JSON}"
fetch_url "/graph.html" "${SHELL_GRAPH_HTML}"
fetch_url "/graph.json" "${SHELL_GRAPH_JSON}"
fetch_url "/setup-status.json" "${SHELL_SETUP_JSON}"
fetch_url "/health.json" "${SHELL_HEALTH_JSON}"

require_text "${SERVER_LOG}" "ctxhelm inspector shell listening on http://127.0.0.1:${PORT}"
require_text "${SHELL_OUT}" 'data-inspector-shell-source-free="true"'
require_text "${SHELL_OUT}" 'Pack Inspector'
require_text "${SHELL_OUT}" 'Graph Neighborhood'
require_text "${SHELL_OUT}" 'Setup Status JSON'
require_text "${SHELL_OUT}" 'daily coding stays inside existing agents'
reject_text "${SHELL_OUT}" "INSPECTOR_UI_SOURCE_SENTINEL"

require_text "${SHELL_GRAPH_HTML}" 'data-graph-source-free="true"'
require_text "${SHELL_GRAPH_HTML}" 'id="graphFilterText"'
require_text "${SHELL_GRAPH_HTML}" 'id="graphKindFilter"'
require_text "${SHELL_GRAPH_HTML}" 'Communities'
reject_text "${SHELL_GRAPH_HTML}" "INSPECTOR_UI_SOURCE_SENTINEL"

require_text "${SHELL_INSPECTOR_JSON}" '"sourceTextLogged": false'
require_text "${SHELL_GRAPH_JSON}" '"sourceTextLogged": false'
require_text "${SHELL_SETUP_JSON}" '"repoRoot"'
require_text "${SHELL_HEALTH_JSON}" '"readOnly": true'
require_text "${SHELL_HEALTH_JSON}" '"sourceTextLogged": false'
reject_text "${SHELL_INSPECTOR_JSON}" "INSPECTOR_UI_SOURCE_SENTINEL"
reject_text "${SHELL_GRAPH_JSON}" "INSPECTOR_UI_SOURCE_SENTINEL"
reject_text "${SHELL_SETUP_JSON}" "INSPECTOR_UI_SOURCE_SENTINEL"
reject_text "${SHELL_HEALTH_JSON}" "INSPECTOR_UI_SOURCE_SENTINEL"

echo "smoke-inspector passed"
