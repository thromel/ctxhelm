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
