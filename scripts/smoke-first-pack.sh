#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxhelm_root="$(cd "$script_dir/.." && pwd -P)"
protocol_script="$ctxhelm_root/scripts/smoke-mcp-protocol.sh"

if [[ -n "${CTXHELM_BIN:-}" ]]; then
  if [[ ! -x "$CTXHELM_BIN" ]]; then
    echo "CTXHELM_BIN is not executable: $CTXHELM_BIN" >&2
    exit 1
  fi
  ctxhelm_bin="$(cd "$(dirname "$CTXHELM_BIN")" && pwd -P)/$(basename "$CTXHELM_BIN")"
else
  if ! command -v ctxhelm >/dev/null 2>&1; then
    echo "ctxhelm is not on PATH; set CTXHELM_BIN=/absolute/path/to/ctxhelm" >&2
    exit 1
  fi
  resolved="$(command -v ctxhelm)"
  ctxhelm_bin="$(cd "$(dirname "$resolved")" && pwd -P)/$(basename "$resolved")"
fi

work_dir="$(mktemp -d)"
repo="$work_dir/repo"
ctxhelm_home="$work_dir/ctxhelm-home"
prepare_json="$work_dir/prepare-task.json"
pack_json="$work_dir/get-pack.json"

cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

mkdir -p "$repo/src/auth" "$repo/tests/auth" "$ctxhelm_home"
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
git -C "$repo" config user.email "ctxhelm@example.com"
git -C "$repo" config user.name "ctxhelm"
git -C "$repo" add .
git -C "$repo" commit -m "fixture" >/dev/null

export CTXHELM_HOME="$ctxhelm_home"

"$ctxhelm_bin" --version >/dev/null
"$ctxhelm_bin" --help >/dev/null
"$ctxhelm_bin" init --repo "$repo" --cursor --claude --opencode >/dev/null
"$ctxhelm_bin" setup-check --repo "$repo" --cursor --claude --opencode >/dev/null

echo "ctxhelm first-pack smoke: running scripts/smoke-mcp-protocol.sh"
CTXHELM_BIN="$ctxhelm_bin" \
  CTXHELM_ROOT="$ctxhelm_root" \
  CTXHELM_SMOKE_REPO="$repo" \
  CTXHELM_SMOKE_TASK="fix requireSession auth bug" \
  CTXHELM_SMOKE_PATH="src/auth/session.ts" \
  CTXHELM_SMOKE_QUERY="requireSession" \
  CTXHELM_HOME="$ctxhelm_home" \
  bash "$protocol_script" >/dev/null

"$ctxhelm_bin" prepare-task "fix requireSession auth bug" \
  --repo "$repo" \
  --mode bug-fix \
  --path src/auth/session.ts \
  --target-agent codex \
  --no-trace >"$prepare_json"

"$ctxhelm_bin" get-pack "fix requireSession auth bug" \
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

echo "ctxhelm first-pack smoke ok: repo=$repo binary=$ctxhelm_bin"
