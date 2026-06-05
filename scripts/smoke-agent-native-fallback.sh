#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ctxhelm_root="${CTXHELM_ROOT:-$(cd "$script_dir/.." && pwd -P)}"

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

reject_text() {
  local file="$1"
  local text="$2"
  if grep -Fq "$text" "$file"; then
    echo "smoke-agent-native-fallback: unsupported text '${text}' found in ${file}" >&2
    exit 1
  fi
}

require_text() {
  local file="$1"
  local text="$2"
  if ! grep -Fq "$text" "$file"; then
    echo "smoke-agent-native-fallback: missing '${text}' in ${file}" >&2
    exit 1
  fi
}

ctxhelm_bin="$(resolve_ctxhelm_bin)"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
json_out="$work_dir/fallback.json"
mkdir -p "$repo/src/auth" "$repo/tests/auth" "$home"
cat >"$repo/src/auth/session.ts" <<'SRC'
export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("CTXHELM_AGENT_NATIVE_SENTINEL auth required");
  }
  return user.id;
}
SRC
cat >"$repo/tests/auth/session.test.ts" <<'SRC'
import { requireSession } from "../../src/auth/session";

test("requireSession returns id", () => {
  expect(requireSession({ id: "u1" })).toBe("u1");
});
SRC
git -C "$repo" init -q
git -C "$repo" config user.email "ctxhelm@example.com"
git -C "$repo" config user.name "ctxhelm"
git -C "$repo" add .
git -C "$repo" commit -m fixture >/dev/null

CTXHELM_HOME="$home" "$ctxhelm_bin" init --repo "$repo" --cursor --claude --opencode >/dev/null
CTXHELM_HOME="$home" "$ctxhelm_bin" setup-check --repo "$repo" --cursor --claude --opencode >/dev/null
CTXHELM_HOME="$home" "$ctxhelm_bin" cards fallback --repo "$repo" --target-agent codex --format json >"$json_out"

python3 - "$json_out" "$repo" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
repo = pathlib.Path(sys.argv[2])
if payload.get("sourceTextLogged") is not False:
    raise SystemExit("fallback report must remain source-free")
if payload.get("privacyStatus", {}).get("localOnly") is not True:
    raise SystemExit("fallback report must remain local-only")
if payload.get("cardCount", 0) < 3:
    raise SystemExit("fallback must generate source-free cards")
guide = pathlib.Path(payload["guidePath"])
if not guide.is_file() or not str(guide).startswith(str(repo / ".ctxhelm" / "fallback")):
    raise SystemExit("fallback guide must be repo-local")
for item in payload.get("cards", []):
    path = pathlib.Path(item["path"])
    if not path.is_file() or not str(path).startswith(str(repo / ".ctxhelm" / "cards")):
        raise SystemExit("fallback card path must be repo-local")
PY

for artifact in \
  "$repo/AGENTS.md" \
  "$repo/.cursor/rules/ctxhelm.mdc" \
  "$repo/.claude/commands/ctxhelm-bugfix.md" \
  "$repo/.ctxhelm/adapters/opencode.jsonc.snippet" \
  "$repo/.ctxhelm/fallback/codex-context.md"; do
  require_text "$artifact" "ctxhelm"
  reject_text "$artifact" "CTXHELM_AGENT_NATIVE_SENTINEL"
  reject_text "$artifact" "auth required"
done

require_text "$repo/AGENTS.md" "prepare_task"
require_text "$repo/.cursor/rules/ctxhelm.mdc" "Read the first up to 5 returned target files"
require_text "$repo/.claude/commands/ctxhelm-bugfix.md" "Read the first up to 5 returned target files"
require_text "$repo/.ctxhelm/adapters/opencode.jsonc.snippet" "serve-mcp"
require_text "$repo/.ctxhelm/fallback/codex-context.md" "local ctxhelm MCP server is unavailable"
require_text "$repo/.ctxhelm/fallback/codex-context.md" "Source snippets included: \`false\`"

python3 - "$repo" <<'PY'
import pathlib
import sys

repo = pathlib.Path(sys.argv[1])
limits = {
    "AGENTS.md": 1600,
    ".cursor/rules/ctxhelm.mdc": 1600,
    ".claude/commands/ctxhelm-bugfix.md": 1600,
    ".ctxhelm/adapters/opencode.jsonc.snippet": 1100,
}
for rel, limit in limits.items():
    size = len((repo / rel).read_text(encoding="utf-8"))
    if size > limit:
        raise SystemExit(f"{rel} is no longer thin: {size} bytes > {limit}")
PY

echo "ctxhelm agent-native fallback smoke passed"
