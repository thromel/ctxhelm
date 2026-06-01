#!/usr/bin/env bash
set -euo pipefail

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

run_ctxhelm() {
  if [[ -n "${CTXHELM_BIN:-}" ]]; then
    "$CTXHELM_BIN" "$@"
  else
    cargo run -q -p ctxhelm -- "$@"
  fi
}

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src/auth" "$repo/tests/auth" "$home"
git -C "$repo" init -q
git -C "$repo" config user.email ctxhelm@example.com
git -C "$repo" config user.name ctxhelm

cat >"$repo/src/auth/session.ts" <<'SRC'
export function requireSession(user?: { id: string }) {
  if (!user) {
    return "CTXHELM_V23_SOURCE_SENTINEL";
  }
  return user.id;
}
SRC
cat >"$repo/tests/auth/session.test.ts" <<'SRC'
import { requireSession } from "../../src/auth/session";

console.log(requireSession({ id: "user_1" }));
SRC
git -C "$repo" add .
git -C "$repo" commit -m "add auth session" >/dev/null

cat >"$repo/src/auth/session.ts" <<'SRC'
export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("missing session");
  }
  return user.id;
}
SRC
cat >"$repo/tests/auth/session.test.ts" <<'SRC'
import { requireSession } from "../../src/auth/session";

console.log(requireSession({ id: "user_1" }));
try {
  requireSession();
} catch (error) {
  console.log(error);
}
SRC
git -C "$repo" add .
git -C "$repo" commit -m "fix missing auth session handling" >/dev/null

export CTXHELM_HOME="$home"

run_ctxhelm eval features export \
  --repo "$repo" \
  --mode bug-fix \
  --target-agent codex \
  --limit 20 \
  --format json \
  "fix requireSession auth handling" >"$work_dir/features.json"

run_ctxhelm eval feedback record \
  --repo "$repo" \
  --task-hash v23-smoke-task \
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
  --format json >"$work_dir/feedback.json"

run_ctxhelm eval policy learn \
  --repo "$repo" \
  --min-gold-or-selected-rows 0 \
  --format json >"$work_dir/learned-policy.json"

run_ctxhelm eval baselines \
  --repo "$repo" \
  --limit 2 \
  --budget 10 \
  --format json >"$work_dir/baselines.json"

python3 - "$work_dir/suite.json" "$repo" <<'PY'
import json
import pathlib
import sys

suite_path = pathlib.Path(sys.argv[1])
repo = pathlib.Path(sys.argv[2])
suite = {
    "manifestVersion": "ctxhelm-benchmark-corpus-v2.3",
    "name": "v23-eval-smoke",
    "corpusId": "ctxhelm-v23-local-smoke",
    "privacyLabel": "local-source-free-smoke",
    "defaults": {
        "limit": 2,
        "rankingBudget": 10,
        "mode": "bug_fix",
        "targetAgent": "codex",
        "parallelism": 1,
    },
    "repositories": [
        {
            "name": "v23-smoke-repo",
            "path": str(repo),
        }
    ],
}
suite_path.write_text(json.dumps(suite, indent=2) + "\n", encoding="utf-8")
PY

run_ctxhelm eval proof \
  --config "$work_dir/suite.json" \
  --format json >"$work_dir/proof.json"

python3 - "$work_dir" <<'PY'
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
sentinel = "CTXHELM_V23_SOURCE_SENTINEL"

def load(name):
    path = work / name
    text = path.read_text(encoding="utf-8")
    if sentinel in text:
        raise SystemExit(f"v2.3 eval smoke leaked source sentinel in {name}")
    return json.loads(text)

features_payload = load("features.json")
feedback = load("feedback.json")
learned = load("learned-policy.json")
baselines = load("baselines.json")
proof = load("proof.json")

features = features_payload.get("export", features_payload)
privacy = features.get("privacyStatus", {})
if not privacy.get("localOnly") or features.get("sourceTextLogged"):
    raise SystemExit("feature export privacy contract failed")
if not features.get("rows"):
    raise SystemExit("feature export produced no rows")
if feedback.get("event", {}).get("sourceTextLogged"):
    raise SystemExit("feedback record logged source text")

if learned.get("profileSchemaVersion") != 2:
    raise SystemExit("learned profile schema version was not 2")
if "baselineThresholds" not in learned:
    raise SystemExit("learned profile missing baseline thresholds")
if not isinstance(learned.get("defaultEligible"), bool):
    raise SystemExit("learned profile defaultEligible was not a bool")
if learned.get("sourceTextLogged"):
    raise SystemExit("learned profile logged source text")

if not baselines.get("rows"):
    raise SystemExit("paired baseline report produced no rows")
if not baselines.get("tokenRoi"):
    raise SystemExit("paired baseline report produced no token ROI")
if not baselines.get("signalSaturation"):
    raise SystemExit("paired baseline report produced no signal saturation")
if not baselines.get("privacyStatus", {}).get("localOnly"):
    raise SystemExit("paired baseline privacyStatus.localOnly was not true")

summary = proof.get("v23EvalSummary", {})
if summary.get("manifestVersion") != "ctxhelm-benchmark-corpus-v2.3":
    raise SystemExit("product proof manifest version mismatch")
if summary.get("fixedCorpusId") != "ctxhelm-v23-local-smoke":
    raise SystemExit("product proof fixed corpus mismatch")
if not summary.get("pairedBaselineVerdicts"):
    raise SystemExit("product proof paired baseline verdicts were empty")
if not isinstance(summary.get("runtimeTotalMillis"), int):
    raise SystemExit("product proof runtimeTotalMillis was not an integer")

feature_privacy = summary.get("featureExportPrivacy", {})
if not feature_privacy.get("localOnly") or feature_privacy.get("sourceTextLogged"):
    raise SystemExit("product proof feature export privacy failed")
if not feature_privacy.get("sourceFreeLabelsOnly"):
    raise SystemExit("product proof did not mark source-free labels only")

learned_status = summary.get("learnedPolicyStatus", {})
if learned_status.get("profileSchemaVersion") != 2:
    raise SystemExit("product proof learned policy schema mismatch")
if not learned_status.get("defaultRequiresThresholds"):
    raise SystemExit("product proof did not require learned-policy thresholds")
if learned_status.get("silentDefaultAllowed"):
    raise SystemExit("product proof allowed silent learned-policy defaults")
if "world-class claims require repeated lift" not in summary.get("proofBoundary", ""):
    raise SystemExit("product proof boundary language missing")
PY

echo "v2.3 eval smoke passed"
