#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: smoke-memory-history-lift.sh [--output PATH]

Builds a temporary repo and proves source-free historical memory lift:
  1. create a historical commit whose title is a related future task
  2. seed a source-free trace with the target path
  3. generate a pending experience card
  4. show historical eval has no memory lift before approval
  5. approve the card
  6. show historical eval reports a unique memory target hit beyond lexical

The report stores path labels and metadata only. It does not store raw prompts,
source snippets, terminal logs, model transcripts, or raw MCP traffic.
EOF
}

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
output_path="${CTXHELM_MEMORY_HISTORY_LIFT_REPORT:-}"

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
mkdir -p "$repo/src/payments" "$home"
git -C "$repo" init >/dev/null
git -C "$repo" config user.email ctxhelm@example.com
git -C "$repo" config user.name ctxhelm
cat >"$repo/src/payments/handler.ts" <<'SRC'
export function processWebhook() {
  return "CTXHELM_MEMORY_HISTORY_SOURCE_SENTINEL";
}
SRC
git -C "$repo" add .
git -C "$repo" commit -m "fix checkout signature regression" >/dev/null

export CTXHELM_HOME="$home"
task="fix checkout signature regression"
target_file="src/payments/handler.ts"

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
"$ctxhelm_bin" eval history \
  --repo "$repo" \
  --limit 1 \
  --budget 10 \
  --mode bug-fix \
  --target-agent claude-code \
  --format json \
  --force \
  >"$work_dir/before-history.json"

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
  --limit 1 \
  --budget 10 \
  --mode bug-fix \
  --target-agent claude-code \
  --format json \
  --force \
  >"$work_dir/after-history.json"

python3 - "$work_dir" "$home" "$task" "$target_file" "$experience_id" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
home = pathlib.Path(sys.argv[2])
task = sys.argv[3]
target_file = sys.argv[4]
experience_id = sys.argv[5]
output_path = sys.argv[6]

seed = json.loads((work / "seed-plan.json").read_text())
experience = json.loads((work / "experience.json").read_text())
approve = json.loads((work / "approve.json").read_text())
before = json.loads((work / "before-history.json").read_text())
after = json.loads((work / "after-history.json").read_text())

def summary(report):
    return report.get("memoryReuseSummary", {})

def actions(report):
    return [action.get("action") for action in report.get("recommendedResearchActions", [])]

def target_in_combined(report):
    for commit in report.get("commits", []):
        if target_file in commit.get("recommendedContextFiles", []):
            return True
    return False

def target_in_lexical(report):
    for commit in report.get("commits", []):
        if target_file in commit.get("lexicalBaselineFiles", []):
            return True
    return False

database_paths = list(home.glob("repos/*/ctxhelm.sqlite3"))
if len(database_paths) != 1:
    raise SystemExit(f"expected one storage database, found {database_paths}")
database_bytes = database_paths[0].read_bytes()
if b"CTXHELM_MEMORY_HISTORY_SOURCE_SENTINEL" in database_bytes:
    raise SystemExit("storage leaked source sentinel")

before_summary = summary(before)
after_summary = summary(after)
status = (
    "passed"
    if seed.get("targetFiles")
    and before.get("evaluatedCommits") == 1
    and after.get("evaluatedCommits") == 1
    and before_summary.get("memoryCandidateCount", 0) == 0
    and before_summary.get("memoryUniqueTargetHitCount", 0) == 0
    and after_summary.get("memoryCandidateCount", 0) > 0
    and after_summary.get("memorySelectedAt10Count", 0) > 0
    and after_summary.get("memoryTargetHitAt10Count", 0) > 0
    and after_summary.get("memoryUniqueTargetHitCount", 0) > 0
    and not target_in_lexical(after)
    and target_in_combined(after)
    and "evaluate_memory_reuse_lift" in actions(after)
    else "degraded"
)

payload = {
    "schemaVersion": "ctxhelm-memory-history-lift-eval-v1",
    "status": status,
    "workflowKind": "historical-experience-memory-lift",
    "task": {
        "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
        "rawTaskStored": False,
    },
    "targetFile": target_file,
    "experienceCardId": experience_id,
    "seed": {
        "targetHit": bool(seed.get("targetFiles")),
        "selectedMemoryCount": len(seed.get("selectedMemory", [])),
    },
    "beforeApproval": {
        "evaluatedCommits": before.get("evaluatedCommits"),
        "targetInCombined": target_in_combined(before),
        "targetInLexical": target_in_lexical(before),
        "memoryReuseSummary": before_summary,
        "recommendedResearchActions": actions(before),
    },
    "afterApproval": {
        "evaluatedCommits": after.get("evaluatedCommits"),
        "targetInCombined": target_in_combined(after),
        "targetInLexical": target_in_lexical(after),
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
    raise SystemExit("historical memory lift proof did not pass")
PY

echo "memory history lift smoke passed"
