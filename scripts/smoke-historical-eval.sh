#!/usr/bin/env bash
set -euo pipefail

smoke_repo="${CTXPACK_SMOKE_REPO:-$PWD}"
smoke_limit="${CTXPACK_SMOKE_LIMIT:-3}"
smoke_budget="${CTXPACK_SMOKE_BUDGET:-10}"

run_smoke() {
  local label="$1"
  local repo="$2"
  local report
  report="$(mktemp)"

  echo "ctxpack smoke: ${label} repo=${repo} limit=${smoke_limit} budget=${smoke_budget}"
  cargo run -p ctxpack -- eval history \
    --repo "$repo" \
    --limit "$smoke_limit" \
    --budget "$smoke_budget" \
    --format json >"$report"

  python3 - "$report" "$smoke_budget" "$label" <<'PY'
import json
import sys

path, expected_budget, label = sys.argv[1], int(sys.argv[2]), sys.argv[3]
with open(path, "r", encoding="utf-8") as handle:
    report = json.load(handle)

required = [
    "evalRangeId",
    "repoId",
    "evaluatedCommits",
    "budget",
    "effectiveFilters",
    "refs",
    "rankingComparison",
    "signalAblations",
    "retrievalGapSummaries",
    "commits",
    "privacyStatus",
]
missing = [key for key in required if key not in report]
if missing:
    raise SystemExit(f"{label}: missing report fields: {', '.join(missing)}")

filters = report["effectiveFilters"]
if filters.get("rankingBudget") != expected_budget:
    raise SystemExit(
        f"{label}: effectiveFilters.rankingBudget={filters.get('rankingBudget')} "
        f"does not match CTXPACK_SMOKE_BUDGET={expected_budget}"
    )

ranking = report["rankingComparison"]
if ranking.get("k") != expected_budget:
    raise SystemExit(
        f"{label}: rankingComparison.k={ranking.get('k')} "
        f"does not match CTXPACK_SMOKE_BUDGET={expected_budget}"
    )
for group in ("combined", "lexicalBaseline"):
    metrics = ranking.get(group, {})
    for metric in ("recallAtK", "precisionAtK", "mrrAtK"):
        if metric not in metrics:
            raise SystemExit(f"{label}: missing rankingComparison.{group}.{metric}")

if not isinstance(report.get("signalAblations"), list):
    raise SystemExit(f"{label}: signalAblations is not an array")
if not isinstance(report.get("retrievalGapSummaries"), list):
    raise SystemExit(f"{label}: retrievalGapSummaries is not an array")
if report.get("privacyStatus", {}).get("localOnly") is not True:
    raise SystemExit(f"{label}: privacyStatus.localOnly is not true")

for commit in report.get("commits", []):
    if commit.get("sourceTextLogged") is not False:
        sha = commit.get("sha", "<unknown>")
        raise SystemExit(f"{label}: commit {sha} has sourceTextLogged != false")

blocked_keys = {
    "sourceText",
    "source_text",
    "source",
    "snippet",
    "prompt",
    "task",
    "taskText",
    "commitSubject",
    "commit_subject",
}

def walk(value, trail="$"):
    if isinstance(value, dict):
        for key, child in value.items():
            if key in blocked_keys:
                raise SystemExit(f"{label}: source or prompt text field present at {trail}.{key}")
            walk(child, f"{trail}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk(child, f"{trail}[{index}]")

walk(report)
print(
    f"ctxpack smoke ok: {label} commits={report['evaluatedCommits']} "
    f"budget={filters['rankingBudget']} range={report['evalRangeId']}"
)
PY

  rm -f "$report"
}

if [[ ! -d "$smoke_repo" ]]; then
  echo "CTXPACK_SMOKE_REPO does not exist: $smoke_repo" >&2
  exit 1
fi

run_smoke "primary" "$smoke_repo"

if [[ -n "${CTXPACK_REFACTORINGMINER_REPO:-}" ]]; then
  if [[ -d "$CTXPACK_REFACTORINGMINER_REPO" ]]; then
    run_smoke "refactoringminer" "$CTXPACK_REFACTORINGMINER_REPO"
  else
    echo "ctxpack smoke: skipping RefactoringMiner; CTXPACK_REFACTORINGMINER_REPO does not exist: $CTXPACK_REFACTORINGMINER_REPO"
  fi
else
  echo "ctxpack smoke: skipping RefactoringMiner; CTXPACK_REFACTORINGMINER_REPO is not set"
fi
