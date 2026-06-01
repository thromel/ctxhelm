#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxhelm_root="${CTXHELM_ROOT:-$(cd "$script_dir/.." && pwd -P)}"
protocol_script="$ctxhelm_root/scripts/smoke-mcp-protocol.sh"

resolve_ctxhelm_bin() {
  if [[ -n "${CTXHELM_BIN:-}" ]]; then
    if [[ ! "$CTXHELM_BIN" = /* ]]; then
      echo "CTXHELM_BIN must be absolute: $CTXHELM_BIN" >&2
      exit 64
    fi
    if [[ ! -x "$CTXHELM_BIN" ]]; then
      echo "CTXHELM_BIN is not executable: $CTXHELM_BIN" >&2
      exit 64
    fi
    printf '%s/%s\n' "$(cd "$(dirname "$CTXHELM_BIN")" && pwd -P)" "$(basename "$CTXHELM_BIN")"
    return
  fi
  cargo build -p ctxhelm >/dev/null
  printf '%s/target/debug/ctxhelm\n' "$ctxhelm_root"
}

client_version() {
  if command -v cursor >/dev/null 2>&1; then
    cursor --version 2>&1 | head -n 1
  else
    printf '%s\n' "not_installed"
  fi
}

write_evidence() {
  local evidence_file="$1"
  python3 - "$evidence_file" "$repo" "$client_version_value" "$ctxhelm_version" <<'PY'
import json
import pathlib
import sys

path, repo, client_version, ctxhelm_version = sys.argv[1:]
evidence = {
    "client": "cursor",
    "clientVersion": client_version,
    "ctxhelmVersion": ctxhelm_version,
    "repo": repo,
    "setupCheck": True,
    "deterministicProtocol": True,
    "deterministicContextAreaResourceRead": True,
    "realClientToolCalls": False,
    "proofBoundary": "Cursor setup and MCP protocol proof only; no machine-checkable Cursor tool-call transcript is claimed.",
}
payload = json.dumps(evidence, sort_keys=True)
if path:
    target = pathlib.Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(payload + "\n", encoding="utf-8")
else:
    print("ctxhelm Cursor setup evidence: " + payload)
PY
}

ctxhelm_bin="$(resolve_ctxhelm_bin)"
ctxhelm_version="$("$ctxhelm_bin" --version)"
client_version_value="$(client_version)"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src/auth" "$repo/tests/auth" "$home"
git -C "$repo" init >/dev/null
git -C "$repo" config user.email "ctxhelm@example.com"
git -C "$repo" config user.name "ctxhelm"
printf 'export function requireSession() { return true; }\n' >"$repo/src/auth/session.ts"
printf 'import { requireSession } from "../../src/auth/session";\n' >"$repo/tests/auth/session.test.ts"
git -C "$repo" add .
git -C "$repo" commit -m fixture >/dev/null

CTXHELM_HOME="$home" "$ctxhelm_bin" init --repo "$repo" --cursor >/dev/null
CTXHELM_HOME="$home" "$ctxhelm_bin" setup-check --repo "$repo" --cursor >/dev/null

CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$ctxhelm_root" \
  CTXHELM_SMOKE_REPO="$repo" \
  CTXHELM_SMOKE_TASK="verify Cursor ctxhelm setup proof" \
  CTXHELM_SMOKE_PATH="src/auth/session.ts" \
  CTXHELM_SMOKE_QUERY="requireSession" \
  CTXHELM_HOME="$home" \
  bash "$protocol_script"

evidence_path=""
if [[ -n "${CTXHELM_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
  evidence_path="${CTXHELM_REAL_CLIENT_EVIDENCE_DIR}/cursor-setup-evidence.json"
fi
write_evidence "$evidence_path"
echo "ctxhelm Cursor setup smoke passed"
