#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_FIXTURE_ROOT="$(cd "$ROOT_DIR/.." && pwd)/ctxhelm-proof-fixtures"
FIXTURE_ROOT="${CTXHELM_PROOF_FIXTURE_ROOT:-$DEFAULT_FIXTURE_ROOT}"
REPORT_DIR="${CTXHELM_PROOF_FIXTURE_REPORT_DIR:-$FIXTURE_ROOT}"

write_fixture_report() {
  local name="$1"
  local source="$2"
  local revision="$3"
  local path="$4"
  local status="$5"
  local failure_reason="$6"
  local head_commit="$7"
  local revision_available="$8"
  local dirty_count="$9"

  mkdir -p "$REPORT_DIR"
  python3 - "$REPORT_DIR/$name.fixture-status.json" "$name" "$source" "$revision" "$path" "$status" "$failure_reason" "$head_commit" "$revision_available" "$dirty_count" <<'PY'
import json
import pathlib
import sys

(
    output_path,
    name,
    source,
    requested_revision,
    fixture_path,
    status,
    failure_reason,
    head_commit,
    revision_available,
    dirty_count,
) = sys.argv[1:]

payload = {
    "schemaVersion": 1,
    "name": name,
    "requestedRevision": requested_revision,
    "headCommit": head_commit or None,
    "sourceKind": "local_path" if source.startswith("/") else "remote_or_relative",
    "fixturePath": fixture_path,
    "status": status,
    "failureReason": failure_reason or None,
    "checks": {
        "fixtureExists": pathlib.Path(fixture_path).is_dir(),
        "gitDirectoryExists": (pathlib.Path(fixture_path) / ".git").exists(),
        "revisionAvailable": revision_available == "true",
        "dirtyCount": int(dirty_count or "0"),
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "omitted": [
        "source snippets",
        "commit subjects",
        "diffs",
        "terminal logs",
        "prompts",
        "remote URLs",
    ],
}
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
}

ensure_fixture() {
  local name="$1"
  local source="$2"
  local revision="$3"
  local path="$FIXTURE_ROOT/$name"
  local dirty_count=0
  local head_commit=""

  mkdir -p "$FIXTURE_ROOT"
  if [[ ! -d "$path/.git" ]]; then
    git clone "$source" "$path"
  else
    git -C "$path" remote set-url origin "$source"
    git -C "$path" fetch --all --tags --prune
  fi

  if ! git -C "$path" cat-file -e "${revision}^{commit}" >/dev/null 2>&1; then
    head_commit="$(git -C "$path" rev-parse HEAD 2>/dev/null || true)"
    write_fixture_report "$name" "$source" "$revision" "$path" "blocked" "requested proof fixture revision is unavailable" "$head_commit" "false" "$dirty_count"
    printf 'proof fixture blocked: %s requested revision unavailable: %s\n' "$name" "$revision" >&2
    return 65
  fi

  git -C "$path" checkout --detach "$revision"
  if status_output="$(git -C "$path" status --porcelain 2>/dev/null)" && [[ -n "$status_output" ]]; then
    dirty_count="$(printf '%s\n' "$status_output" | wc -l | tr -d ' ')"
  fi
  head_commit="$(git -C "$path" rev-parse HEAD 2>/dev/null || true)"
  write_fixture_report "$name" "$source" "$revision" "$path" "ready" "" "$head_commit" "true" "$dirty_count"
  printf '%s %s %s\n' "$name" "$revision" "$path"
}

verischema_source="${CTXHELM_VERISCHEMA_SOURCE:-git@github.com:thromel/VeriSchema.git}"
if [[ -d "$(cd "$ROOT_DIR/.." && pwd)/VeriSchema/.git" ]]; then
  verischema_source="${CTXHELM_VERISCHEMA_SOURCE:-$(cd "$ROOT_DIR/.." && pwd)/VeriSchema}"
fi

ensure_fixture "RefactoringMiner" "${CTXHELM_REFACTORINGMINER_SOURCE:-git@github.com:thromel/RefactoringMiner.git}" "${CTXHELM_REFACTORINGMINER_REVISION:-949bddcd3509a805f5e3bcc55fcdb71a691b0dac}"
ensure_fixture "ctxhelm" "${CTXHELM_CTXHELM_SOURCE:-git@github.com:thromel/ctxhelm.git}" "${CTXHELM_CTXHELM_REVISION:-7f439fa1b1f2784aed6425b695d7c35944ea7955}"
ensure_fixture "ReAgent" "${CTXHELM_REAGENT_SOURCE:-git@github.com:SRK-LLM-Research/ccia-framework.git}" "${CTXHELM_REAGENT_REVISION:-44277ff7d89a8e2e2a1cbdf8d350c409e379786d}"
ensure_fixture "VeriSchema" "$verischema_source" "${CTXHELM_VERISCHEMA_REVISION:-33578667304472d3d58be2301dcc31d07e5c9bc4}"
