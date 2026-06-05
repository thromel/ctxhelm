#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

run_ctxhelm() {
  if [[ -n "${CTXHELM_BIN:-}" ]]; then
    "${CTXHELM_BIN}" "$@"
  else
    cargo run -q -p ctxhelm -- "$@"
  fi
}

REPO="${TMP_DIR}/repo"
HOME_DIR="${TMP_DIR}/home"
mkdir -p "${REPO}/src/auth" "${REPO}/tests/auth" "${HOME_DIR}"
git -C "${REPO}" init -b main >/dev/null
git -C "${REPO}" config user.email ctxhelm@example.com
git -C "${REPO}" config user.name ctxhelm
cat >"${REPO}/src/auth/session.ts" <<'EOF'
export function requireSession() {
  return "CTXHELM_GOVERNOR_SOURCE_SENTINEL";
}
EOF
cat >"${REPO}/tests/auth/session.test.ts" <<'EOF'
import { requireSession } from "../../src/auth/session";
test("session", () => requireSession());
EOF
git -C "${REPO}" add .
git -C "${REPO}" commit -m "add auth session" >/dev/null

export CTXHELM_HOME="${HOME_DIR}"

(
  cd "${ROOT_DIR}"
  run_ctxhelm eval feedback record \
    --repo "${REPO}" \
    --task-hash governor-task-1 \
    --mode bug-fix \
    --target-agent codex \
    --budget standard \
    --outcome passed \
    --recommended-file src/auth/session.ts \
    --recommended-test tests/auth/session.test.ts \
    --recommended-command "pnpm test tests/auth/session.test.ts" \
    --read-file src/auth/session.ts \
    --edited-file src/auth/session.ts \
    --tested-file tests/auth/session.test.ts \
    --tested-command "pnpm test tests/auth/session.test.ts" \
    --tag accepted_fix \
    --format json >"${TMP_DIR}/feedback.json"

  run_ctxhelm governor decide "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --path src/auth/session.ts \
    --format json >"${TMP_DIR}/governor-before.json"

  run_ctxhelm eval policy tune \
    --repo "${REPO}" \
    --format json >"${TMP_DIR}/profile.json"
)

PROFILE_ID="$(python3 - "${TMP_DIR}/profile.json" <<'PY'
import json
import pathlib
import sys
print(json.loads(pathlib.Path(sys.argv[1]).read_text())["id"])
PY
)"

(
  cd "${ROOT_DIR}"
  run_ctxhelm eval policy apply "${PROFILE_ID}" \
    --repo "${REPO}" \
    --format json >"${TMP_DIR}/apply.json"

  run_ctxhelm governor decide "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --path src/auth/session.ts \
    --semantic \
    --semantic-provider local_hash \
    --format json >"${TMP_DIR}/governor-active.json"

  run_ctxhelm eval policy rollback \
    --repo "${REPO}" \
    --format json >"${TMP_DIR}/rollback.json"

  run_ctxhelm governor decide "fix requireSession sentinel" \
    --repo "${REPO}" \
    --mode bug-fix \
    --path src/auth/session.ts \
    --format json >"${TMP_DIR}/governor-rollback.json"
)

python3 - "${TMP_DIR}" "${HOME_DIR}" "${PROFILE_ID}" <<'PY'
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
home = pathlib.Path(sys.argv[2])
profile_id = sys.argv[3]

before = json.loads((work / "governor-before.json").read_text())
active = json.loads((work / "governor-active.json").read_text())
rollback = json.loads((work / "governor-rollback.json").read_text())
apply = json.loads((work / "apply.json").read_text())
rollback_action = json.loads((work / "rollback.json").read_text())

required_areas = {"retrieval", "budget", "memory", "validation", "semantic", "policy_profile"}
for name, report in [
    ("before", before),
    ("active", active),
    ("rollback", rollback),
]:
    if report["schemaVersion"] != "ctxhelm-context-governor-report-v1":
        raise SystemExit(f"{name}: unexpected schema version")
    if report["sourceTextLogged"]:
        raise SystemExit(f"{name}: governor logged source text")
    if not report["privacyStatus"]["localOnly"]:
        raise SystemExit(f"{name}: governor privacy status is not local-only")
    if report["recommendedBudget"] not in {"brief", "standard", "deep"}:
        raise SystemExit(f"{name}: missing recommended budget")
    areas = {decision["area"] for decision in report["decisions"]}
    missing = required_areas - areas
    if missing:
        raise SystemExit(f"{name}: missing decision areas {sorted(missing)}")
    labels = {item["label"] for item in report["selectedEvidence"]}
    if "src/auth/session.ts" not in labels:
        raise SystemExit(f"{name}: selected evidence missing anchored source")
    if not isinstance(report["omittedEvidence"], list):
        raise SystemExit(f"{name}: omitted evidence is not an array")
    actions = {control["action"] for control in report["rolloutControls"]}
    if {"learn", "compare", "apply", "rollback"} - actions:
        raise SystemExit(f"{name}: missing rollout controls")

if before["selectedPolicyProfileId"] is not None:
    raise SystemExit("expected no active profile before apply")
if apply["activeProfileId"] != profile_id:
    raise SystemExit("policy apply did not activate profile")
if active["selectedPolicyProfileId"] != profile_id:
    raise SystemExit("governor did not report active policy profile")
if rollback_action["activeProfileId"] is not None:
    raise SystemExit("policy rollback left active profile")
if rollback["selectedPolicyProfileId"] is not None:
    raise SystemExit("governor did not observe rollback")

for path in work.glob("*.json"):
    if "CTXHELM_GOVERNOR_SOURCE_SENTINEL" in path.read_text(errors="ignore"):
        raise SystemExit(f"governor output leaked source sentinel in {path}")
for path in home.glob("repos/*/*"):
    if not path.is_file():
        continue
    if b"CTXHELM_GOVERNOR_SOURCE_SENTINEL" in path.read_bytes():
        raise SystemExit(f"governor storage leaked source sentinel in {path}")
PY

echo "smoke-governor passed"
