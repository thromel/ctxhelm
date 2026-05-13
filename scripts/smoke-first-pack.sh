#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxpack_root="$(cd "$script_dir/.." && pwd -P)"
protocol_script="$ctxpack_root/scripts/smoke-mcp-protocol.sh"

if [[ -n "${CTXPACK_BIN:-}" ]]; then
  if [[ ! -x "$CTXPACK_BIN" ]]; then
    echo "CTXPACK_BIN is not executable: $CTXPACK_BIN" >&2
    exit 1
  fi
  ctxpack_bin="$(cd "$(dirname "$CTXPACK_BIN")" && pwd -P)/$(basename "$CTXPACK_BIN")"
else
  if ! command -v ctxpack >/dev/null 2>&1; then
    echo "ctxpack is not on PATH; set CTXPACK_BIN=/absolute/path/to/ctxpack" >&2
    exit 1
  fi
  resolved="$(command -v ctxpack)"
  ctxpack_bin="$(cd "$(dirname "$resolved")" && pwd -P)/$(basename "$resolved")"
fi

work_dir="$(mktemp -d)"
repo="$work_dir/repo"
ctxpack_home="$work_dir/ctxpack-home"
prepare_json="$work_dir/prepare-task.json"
pack_json="$work_dir/get-pack.json"

cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

mkdir -p "$repo/src/auth" "$repo/tests/auth" "$ctxpack_home"
cat >"$repo/src/auth/session.ts" <<'SRC'
import { issueToken } from "./token";

export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("auth required");
  }
  issueToken(user.id);
  return user.id;
}
SRC
cat >"$repo/src/auth/token.ts" <<'SRC'
export function issueToken(userId: string) {
  return `token:${userId}`;
}
SRC
cat >"$repo/tests/auth/session.test.ts" <<'SRC'
import { requireSession } from "../../src/auth/session";

test("requireSession returns user id", () => {
  expect(requireSession({ id: "user-1" })).toBe("user-1");
});
SRC
cat >"$repo/package.json" <<'SRC'
{"scripts":{"test":"vitest run"}}
SRC

git -C "$repo" init >/dev/null
git -C "$repo" config user.email "ctxpack@example.com"
git -C "$repo" config user.name "ctxpack"
git -C "$repo" add .
git -C "$repo" commit -m "fixture" >/dev/null

export CTXPACK_HOME="$ctxpack_home"

"$ctxpack_bin" --version >/dev/null
"$ctxpack_bin" --help >/dev/null
"$ctxpack_bin" init --repo "$repo" --cursor --claude --opencode >/dev/null
"$ctxpack_bin" setup-check --repo "$repo" --cursor --claude --opencode >/dev/null

echo "ctxpack first-pack smoke: running scripts/smoke-mcp-protocol.sh"
CTXPACK_BIN="$ctxpack_bin" \
  CTXPACK_ROOT="$ctxpack_root" \
  CTXPACK_SMOKE_REPO="$repo" \
  CTXPACK_SMOKE_TASK="fix requireSession auth bug" \
  CTXPACK_SMOKE_PATH="src/auth/session.ts" \
  CTXPACK_SMOKE_QUERY="requireSession" \
  CTXPACK_HOME="$ctxpack_home" \
  bash "$protocol_script" >/dev/null

"$ctxpack_bin" prepare-task "fix requireSession auth bug" \
  --repo "$repo" \
  --mode bug-fix \
  --path src/auth/session.ts \
  --target-agent codex \
  --no-trace >"$prepare_json"

"$ctxpack_bin" get-pack "fix requireSession auth bug" \
  --repo "$repo" \
  --mode bug-fix \
  --budget brief \
  --format json \
  --path src/auth/session.ts \
  --target-agent codex \
  --no-trace >"$pack_json"

python3 - "$prepare_json" "$pack_json" <<'PY'
import json
import sys

prepare_path, pack_path = sys.argv[1:]
with open(prepare_path, encoding="utf-8") as handle:
    plan = json.load(handle)
with open(pack_path, encoding="utf-8") as handle:
    pack = json.load(handle)

if not plan.get("targetFiles"):
    raise SystemExit("prepare-task: targetFiles is empty")
if not plan.get("packOptions"):
    raise SystemExit("prepare-task: packOptions is empty")
if not pack.get("repoId"):
    raise SystemExit("get-pack: repoId is empty")
if not pack.get("sections"):
    raise SystemExit("get-pack: sections is empty")
PY

echo "ctxpack first-pack smoke ok: repo=$repo binary=$ctxpack_bin"
