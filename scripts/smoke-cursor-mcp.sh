#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxpack_root="${CTXPACK_ROOT:-$(cd "$script_dir/.." && pwd -P)}"
protocol_script="$ctxpack_root/scripts/smoke-mcp-protocol.sh"

resolve_ctxpack_bin() {
  if [[ -n "${CTXPACK_BIN:-}" ]]; then
    if [[ ! "$CTXPACK_BIN" = /* ]]; then
      echo "CTXPACK_BIN must be absolute: $CTXPACK_BIN" >&2
      exit 64
    fi
    if [[ ! -x "$CTXPACK_BIN" ]]; then
      echo "CTXPACK_BIN is not executable: $CTXPACK_BIN" >&2
      exit 64
    fi
    printf '%s/%s\n' "$(cd "$(dirname "$CTXPACK_BIN")" && pwd -P)" "$(basename "$CTXPACK_BIN")"
    return
  fi
  cargo build -p ctxpack >/dev/null
  printf '%s/target/debug/ctxpack\n' "$ctxpack_root"
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
  python3 - "$evidence_file" "$repo" "$client_version_value" "$ctxpack_version" <<'PY'
import json
import pathlib
import sys

path, repo, client_version, ctxpack_version = sys.argv[1:]
evidence = {
    "client": "cursor",
    "clientVersion": client_version,
    "ctxpackVersion": ctxpack_version,
    "repo": repo,
    "setupCheck": True,
    "deterministicProtocol": True,
    "realClientToolCalls": False,
    "proofBoundary": "Cursor setup and MCP protocol proof only; no machine-checkable Cursor tool-call transcript is claimed.",
}
payload = json.dumps(evidence, sort_keys=True)
if path:
    target = pathlib.Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(payload + "\n", encoding="utf-8")
else:
    print("ctxpack Cursor setup evidence: " + payload)
PY
}

ctxpack_bin="$(resolve_ctxpack_bin)"
ctxpack_version="$("$ctxpack_bin" --version)"
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
git -C "$repo" config user.email "ctxpack@example.com"
git -C "$repo" config user.name "ctxpack"
printf 'export function requireSession() { return true; }\n' >"$repo/src/auth/session.ts"
printf 'import { requireSession } from "../../src/auth/session";\n' >"$repo/tests/auth/session.test.ts"
git -C "$repo" add .
git -C "$repo" commit -m fixture >/dev/null

CTXPACK_HOME="$home" "$ctxpack_bin" init --repo "$repo" --cursor >/dev/null
CTXPACK_HOME="$home" "$ctxpack_bin" setup-check --repo "$repo" --cursor >/dev/null

CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$ctxpack_root" \
  CTXPACK_SMOKE_REPO="$repo" \
  CTXPACK_SMOKE_TASK="verify Cursor ctxpack setup proof" \
  CTXPACK_SMOKE_PATH="src/auth/session.ts" \
  CTXPACK_SMOKE_QUERY="requireSession" \
  CTXPACK_HOME="$home" \
  bash "$protocol_script"

evidence_path=""
if [[ -n "${CTXPACK_REAL_CLIENT_EVIDENCE_DIR:-}" ]]; then
  evidence_path="${CTXPACK_REAL_CLIENT_EVIDENCE_DIR}/cursor-setup-evidence.json"
fi
write_evidence "$evidence_path"
echo "ctxpack Cursor setup smoke passed"
