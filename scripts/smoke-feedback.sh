#!/usr/bin/env bash
set -euo pipefail

ctxpack_bin="${CTXPACK_BIN:-ctxpack}"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src/auth" "$repo/tests/auth" "$home"
git -C "$repo" init >/dev/null
git -C "$repo" config user.email ctxpack@example.com
git -C "$repo" config user.name ctxpack
cat >"$repo/src/auth/session.ts" <<'SRC'
export function requireSession() {
  return "CTXPACK_FEEDBACK_SOURCE_SENTINEL";
}
SRC
cat >"$repo/tests/auth/session.test.ts" <<'SRC'
import { requireSession } from "../../src/auth/session";
console.log(requireSession);
SRC
git -C "$repo" add .
git -C "$repo" commit -m "add auth session" >/dev/null

export CTXPACK_HOME="$home"

"$ctxpack_bin" eval feedback record \
  --repo "$repo" \
  --task-hash task-hash-1 \
  --mode bug-fix \
  --target-agent codex \
  --budget brief \
  --outcome passed \
  --recommended-file src/auth/session.ts \
  --recommended-test tests/auth/session.test.ts \
  --recommended-command "pnpm test tests/auth/session.test.ts" \
  --read-file src/auth/session.ts \
  --edited-file src/auth/session.ts \
  --tested-file tests/auth/session.test.ts \
  --tested-command "pnpm test tests/auth/session.test.ts" \
  --tag accepted_fix \
  --format json >"$work_dir/record.json"

"$ctxpack_bin" eval feedback list --repo "$repo" --format json >"$work_dir/list.json"
"$ctxpack_bin" eval feedback summary --repo "$repo" --format json >"$work_dir/summary.json"
"$ctxpack_bin" eval policy report --repo "$repo" --format json >"$work_dir/policy-report.json"
"$ctxpack_bin" eval policy tune --repo "$repo" --format json >"$work_dir/profile.json"
profile_id="$(python3 - "$work_dir/profile.json" <<'PY'
import json
import pathlib
import sys
print(json.loads(pathlib.Path(sys.argv[1]).read_text())["id"])
PY
)"
"$ctxpack_bin" eval policy apply "$profile_id" --repo "$repo" --format json >"$work_dir/apply.json"
"$ctxpack_bin" eval policy rollback --repo "$repo" --format json >"$work_dir/rollback.json"
"$ctxpack_bin" eval outcome compare --repo "$repo" --format json >"$work_dir/outcome.json"

python3 - "$work_dir" "$home" <<'PY'
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
home = pathlib.Path(sys.argv[2])
record = json.loads((work / "record.json").read_text())
events = json.loads((work / "list.json").read_text())
summary = json.loads((work / "summary.json").read_text())
policy = json.loads((work / "policy-report.json").read_text())
profile = json.loads((work / "profile.json").read_text())
apply = json.loads((work / "apply.json").read_text())
rollback = json.loads((work / "rollback.json").read_text())
outcome = json.loads((work / "outcome.json").read_text())

if record["event"]["sourceTextLogged"]:
    raise SystemExit("feedback record logged source text")
if len(events) != 1 or events[0]["sourceTextLogged"]:
    raise SystemExit("feedback list missing source-free event")
if summary["eventCount"] != 1 or summary["sourceTextLogged"]:
    raise SystemExit("feedback summary did not stay source-free")
if policy["eventCount"] != 1 or policy["sourceTextLogged"]:
    raise SystemExit("policy report did not stay source-free")
if not profile["weights"] or not profile["safetyFloors"]:
    raise SystemExit("policy profile missing weights or safety floors")
if profile["sourceTextLogged"]:
    raise SystemExit("policy profile logged source text")
if apply["activeProfileId"] != profile["id"]:
    raise SystemExit("policy apply did not activate profile")
if rollback["activeProfileId"] is not None:
    raise SystemExit("policy rollback left an active profile")
if outcome["eventCount"] != 1 or outcome["sourceTextLogged"]:
    raise SystemExit("outcome report did not stay source-free")

for path in home.glob("repos/*/*"):
    data = path.read_bytes()
    if b"CTXPACK_FEEDBACK_SOURCE_SENTINEL" in data:
        raise SystemExit(f"feedback storage leaked source sentinel in {path}")
PY

echo "feedback smoke passed"
