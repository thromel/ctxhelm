#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_FIXTURE_ROOT="$(cd "$ROOT_DIR/.." && pwd)/ctxhelm-proof-fixtures"
FIXTURE_ROOT="${CTXHELM_PROOF_FIXTURE_ROOT:-$DEFAULT_FIXTURE_ROOT}"

ensure_fixture() {
  local name="$1"
  local source="$2"
  local revision="$3"
  local path="$FIXTURE_ROOT/$name"

  mkdir -p "$FIXTURE_ROOT"
  if [[ ! -d "$path/.git" ]]; then
    git clone "$source" "$path"
  else
    git -C "$path" remote set-url origin "$source"
    git -C "$path" fetch --all --tags --prune
  fi

  git -C "$path" checkout --detach "$revision"
  printf '%s %s %s\n' "$name" "$revision" "$path"
}

verischema_source="${CTXHELM_VERISCHEMA_SOURCE:-git@github.com:thromel/VeriSchema.git}"
if [[ -d "$(cd "$ROOT_DIR/.." && pwd)/VeriSchema/.git" ]]; then
  verischema_source="${CTXHELM_VERISCHEMA_SOURCE:-$(cd "$ROOT_DIR/.." && pwd)/VeriSchema}"
fi

ensure_fixture "RefactoringMiner" "git@github.com:thromel/RefactoringMiner.git" "949bddcd3509a805f5e3bcc55fcdb71a691b0dac"
ensure_fixture "ctxhelm" "git@github.com:thromel/ctxhelm.git" "7f439fa1b1f2784aed6425b695d7c35944ea7955"
ensure_fixture "ReAgent" "git@github.com:SRK-LLM-Research/ccia-framework.git" "44277ff7d89a8e2e2a1cbdf8d350c409e379786d"
ensure_fixture "VeriSchema" "$verischema_source" "b5cfb2a551d026514f505c45863db31277bcd1ad"
