#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: smoke-memory-parent-snapshot-lift.sh [--output PATH]

Builds a two-commit temporary repo and proves approved source-free memory from
the source repo is visible when `eval history` evaluates a non-root commit
through a parent snapshot. This covers the real-corpus failure mode where
parent-snapshot storage has a different repo identity from the source checkout.

The report stores path labels and counts only. It does not store raw prompts,
source snippets, terminal logs, model transcripts, or raw MCP traffic.
EOF
}

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
output_path="${CTXHELM_MEMORY_PARENT_SNAPSHOT_LIFT_REPORT:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      output_path="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 64
      ;;
  esac
done

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src/internal" "$home"
git -C "$repo" init -q
git -C "$repo" config user.email ctxhelm@example.com
git -C "$repo" config user.name ctxhelm

target_file="src/internal/handler.ts"
task="fix checkout signature regression"
sentinel="CTXHELM_MEMORY_PARENT_SNAPSHOT_SOURCE_SENTINEL"

cat >"$repo/$target_file" <<SRC
export function internalHandler() {
  return "$sentinel";
}
SRC
git -C "$repo" add .
git -C "$repo" commit -m "add internal handler" >/dev/null
base_sha="$(git -C "$repo" rev-parse HEAD)"

cat >"$repo/$target_file" <<SRC
export function internalHandler() {
  return "$sentinel-updated";
}
SRC
git -C "$repo" add .
git -C "$repo" commit -m "$task" >/dev/null
head_sha="$(git -C "$repo" rev-parse HEAD)"

export CTXHELM_HOME="$home"

"$ctxhelm_bin" eval history \
  --repo "$repo" \
  --base "$base_sha" \
  --head "$head_sha" \
  --limit 1 \
  --budget 10 \
  --mode bug-fix \
  --target-agent claude-code \
  --format json \
  --force \
  >"$work_dir/before-history.json"

"$ctxhelm_bin" prepare-task "$task" \
  --repo "$repo" \
  --mode bug-fix \
  --path "$target_file" \
  --target-agent claude-code \
  >"$work_dir/seed-plan.json"
"$ctxhelm_bin" memory generate-experience \
  --repo "$repo" \
  --limit 5 \
  --format json \
  >"$work_dir/experience.json"

experience_id="$(python3 - "$work_dir/experience.json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
cards = [card for card in report.get("cards", []) if card.get("kind") == "experience"]
if not cards:
    raise SystemExit("no experience card generated")
print(cards[0]["id"])
PY
)"

"$ctxhelm_bin" memory approve "$experience_id" \
  --repo "$repo" \
  --format json \
  >"$work_dir/approve.json"

"$ctxhelm_bin" eval history \
  --repo "$repo" \
  --base "$base_sha" \
  --head "$head_sha" \
  --limit 1 \
  --budget 10 \
  --mode bug-fix \
  --target-agent claude-code \
  --format json \
  --force \
  >"$work_dir/after-history.json"

python3 - "$work_dir" "$home" "$task" "$target_file" "$sentinel" "$experience_id" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
home = pathlib.Path(sys.argv[2])
task = sys.argv[3]
target_file = sys.argv[4]
sentinel = sys.argv[5]
experience_id = sys.argv[6]
output_path = sys.argv[7]

seed = json.loads((work / "seed-plan.json").read_text())
experience = json.loads((work / "experience.json").read_text())
approve = json.loads((work / "approve.json").read_text())
before = json.loads((work / "before-history.json").read_text())
after = json.loads((work / "after-history.json").read_text())

def summary(report):
    return report.get("memoryReuseSummary", {})

def actions(report):
    return [action.get("action") for action in report.get("recommendedResearchActions", [])]

def target_in(report, key):
    return any(target_file in commit.get(key, []) for commit in report.get("commits", []))

def retrieval_targets(report):
    targets = []
    for commit in report.get("commits", []):
        targets.extend(commit.get("retrievalTargetFiles", []))
    return targets

database_paths = list(home.glob("repos/*/ctxhelm.sqlite3"))
if len(database_paths) < 2:
    raise SystemExit(f"expected source and snapshot storage databases, found {database_paths}")
for database_path in database_paths:
    data = database_path.read_bytes()
    if sentinel.encode("utf-8") in data:
        raise SystemExit("storage leaked source sentinel")

report_text = json.dumps({"before": before, "after": after}, sort_keys=True)
if sentinel in report_text:
    raise SystemExit("history report leaked source sentinel")
if task in report_text:
    raise SystemExit("history report leaked raw task text")

before_summary = summary(before)
after_summary = summary(after)
status = (
    "passed"
    if seed.get("targetFiles")
    and before.get("evaluatedCommits") == 1
    and after.get("evaluatedCommits") == 1
    and target_file in retrieval_targets(after)
    and before_summary.get("memoryCandidateCount", 0) == 0
    and before_summary.get("memoryUniqueTargetHitCount", 0) == 0
    and not target_in(before, "recommendedContextFiles")
    and not target_in(after, "lexicalBaselineFiles")
    and target_in(after, "recommendedContextFiles")
    and after_summary.get("memoryCandidateCount", 0) > 0
    and after_summary.get("memorySelectedAt10Count", 0) > 0
    and after_summary.get("memoryTargetHitAt10Count", 0) > 0
    and after_summary.get("memoryUniqueTargetHitCount", 0) > 0
    and "evaluate_memory_reuse_lift" in actions(after)
    else "degraded"
)

payload = {
    "schemaVersion": "ctxhelm-memory-parent-snapshot-lift-eval-v1",
    "status": status,
    "workflowKind": "parent-snapshot-experience-memory-lift",
    "task": {
        "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
        "rawTaskStored": False,
    },
    "targetFile": target_file,
    "experienceCardId": experience_id,
    "beforeApproval": {
        "evaluatedCommits": before.get("evaluatedCommits"),
        "targetInCombined": target_in(before, "recommendedContextFiles"),
        "targetInLexical": target_in(before, "lexicalBaselineFiles"),
        "memoryReuseSummary": before_summary,
        "recommendedResearchActions": actions(before),
    },
    "afterApproval": {
        "evaluatedCommits": after.get("evaluatedCommits"),
        "targetInCombined": target_in(after, "recommendedContextFiles"),
        "targetInLexical": target_in(after, "lexicalBaselineFiles"),
        "memoryReuseSummary": after_summary,
        "recommendedResearchActions": actions(after),
    },
    "experience": {
        "generatedCards": len(experience.get("cards", [])),
        "storedRecords": experience.get("storedRecords"),
        "approvedRecords": approve.get("memoryCardRecords"),
    },
    "privacyStatus": {
        "localOnly": True,
        "sourceTextLogged": False,
        "rawPromptStored": False,
        "rawTaskTextStored": False,
        "rawTranscriptStored": False,
        "rawMcpTrafficStored": False,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
    },
    "unsupportedActions": [
        "source edits",
        "model invocation",
        "cloud upload",
        "raw prompt storage",
    ],
}

text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output_path:
    path = pathlib.Path(output_path).resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
else:
    print(text, end="")

if status != "passed":
    raise SystemExit("parent-snapshot memory lift proof did not pass")
PY

echo "memory parent-snapshot lift smoke passed"
