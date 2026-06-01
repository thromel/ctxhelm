#!/usr/bin/env bash
set -euo pipefail

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src/auth" "$repo/tests/auth" "$home"
git -C "$repo" init >/dev/null
git -C "$repo" config user.email ctxhelm@example.com
git -C "$repo" config user.name ctxhelm
cat >"$repo/src/auth/session.ts" <<'SRC'
export function requireSession() {
  return "CTXHELM_MEMORY_SOURCE_SENTINEL";
}
SRC
cat >"$repo/tests/auth/session.test.ts" <<'SRC'
import { requireSession } from "../../src/auth/session";
console.log(requireSession);
SRC
git -C "$repo" add .
git -C "$repo" commit -m "add auth session" >/dev/null

export CTXHELM_HOME="$home"

"$ctxhelm_bin" cards generate --repo "$repo" --format json >"$work_dir/cards.json"
"$ctxhelm_bin" storage status --repo "$repo" --format json >"$work_dir/status.json"
"$ctxhelm_bin" prepare-task "fix requireSession auth session bug" --repo "$repo" --mode bug-fix >"$work_dir/plan.json"
"$ctxhelm_bin" get-pack "fix requireSession auth session bug" --repo "$repo" --mode bug-fix --budget brief >"$work_dir/pack.md"
"$ctxhelm_bin" memory generate-experience --repo "$repo" --format json >"$work_dir/experience.json"
"$ctxhelm_bin" memory list --repo "$repo" --include-disabled --format json >"$work_dir/memory-before.json"

python3 - "$work_dir" "$home" <<'PY'
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
home = pathlib.Path(sys.argv[2])
cards = json.loads((work / "cards.json").read_text())
status = json.loads((work / "status.json").read_text())
plan = json.loads((work / "plan.json").read_text())
pack = (work / "pack.md").read_text()
experience = json.loads((work / "experience.json").read_text())
memory = json.loads((work / "memory-before.json").read_text())

if not cards["privacyStatus"]["localOnly"]:
    raise SystemExit("cards were not local-only")
if status["memoryCardRecords"] < 1:
    raise SystemExit("memory cards were not stored")
if not plan.get("selectedMemory"):
    raise SystemExit("prepare-task did not select fresh deterministic memory")
if "## Selected memory" not in pack:
    raise SystemExit("get-pack did not render selected memory section")
if experience["cards"] and experience["cards"][0]["reviewStatus"] != "pending":
    raise SystemExit("experience cards must default to pending review")
pending = [card for card in memory if card["kind"] == "experience"]
if not pending:
    raise SystemExit("experience memory card was not listed")
database_paths = list(home.glob("repos/*/ctxhelm.sqlite3"))
if len(database_paths) != 1:
    raise SystemExit(f"expected one storage database, found {database_paths}")
database_bytes = database_paths[0].read_bytes()
if b"CTXHELM_MEMORY_SOURCE_SENTINEL" in database_bytes:
    raise SystemExit("storage leaked source sentinel")
print(pending[0]["id"])
PY

experience_id="$(python3 - "$work_dir/memory-before.json" <<'PY'
import json
import pathlib
import sys
cards = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(next(card["id"] for card in cards if card["kind"] == "experience"))
PY
)"

"$ctxhelm_bin" memory show "$experience_id" --repo "$repo" --format json >"$work_dir/show.json"
"$ctxhelm_bin" memory approve "$experience_id" --repo "$repo" --format json >"$work_dir/approve.json"
"$ctxhelm_bin" memory disable "$experience_id" --repo "$repo" --format json >"$work_dir/disable.json"

python3 - "$work_dir/show.json" "$work_dir/approve.json" "$work_dir/disable.json" <<'PY'
import json
import pathlib
import sys
show = json.loads(pathlib.Path(sys.argv[1]).read_text())
approve = json.loads(pathlib.Path(sys.argv[2]).read_text())
disable = json.loads(pathlib.Path(sys.argv[3]).read_text())
if show["kind"] != "experience":
    raise SystemExit("show did not return the experience card")
if approve["memoryCardRecords"] < 1:
    raise SystemExit("approve did not preserve memory records")
if disable["memoryCardRecords"] < 1:
    raise SystemExit("disable did not preserve memory records")
PY

echo "memory smoke passed"
