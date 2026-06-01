#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: smoke-memory-reuse.sh [--output PATH]

Builds a temporary repo and proves source-free experience memory reuse:
  1. seed a prior trace with an explicit target path
  2. generate a pending experience card from the trace
  3. show pending memory is blocked from later target selection
  4. approve the card
  5. show approved memory promotes the source-linked target on a related task

The report stores path labels and metadata only. It does not store raw prompts,
source snippets, terminal logs, model transcripts, or raw MCP traffic.
EOF
}

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
output_path="${CTXHELM_MEMORY_REUSE_REPORT:-}"

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
export function handlePayment() {
  return "CTXHELM_MEMORY_REUSE_SOURCE_SENTINEL";
}
SRC
git -C "$repo" add .
git -C "$repo" commit -m "add payment handler" >/dev/null

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
"$ctxhelm_bin" prepare-task "$task" \
  --repo "$repo" \
  --mode bug-fix \
  --no-trace \
  >"$work_dir/before.json"

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
"$ctxhelm_bin" prepare-task "$task" \
  --repo "$repo" \
  --mode bug-fix \
  --no-trace \
  >"$work_dir/after.json"

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
before = json.loads((work / "before.json").read_text())
after = json.loads((work / "after.json").read_text())
approve = json.loads((work / "approve.json").read_text())

def target_hit(plan):
    return any(target.get("path") == target_file for target in plan.get("targetFiles", []))

def memory_signal_count(plan):
    count = 0
    for candidate in plan.get("retrievalCandidates", []):
        if candidate.get("path") != target_file:
            continue
        for score in candidate.get("signalScores", []):
            if score.get("signal") == "memory":
                count += 1
    return count

database_paths = list(home.glob("repos/*/ctxhelm.sqlite3"))
if len(database_paths) != 1:
    raise SystemExit(f"expected one storage database, found {database_paths}")
database_bytes = database_paths[0].read_bytes()
if b"CTXHELM_MEMORY_REUSE_SOURCE_SENTINEL" in database_bytes:
    raise SystemExit("storage leaked source sentinel")

before_hit = target_hit(before)
after_hit = target_hit(after)
after_memory_signal_count = memory_signal_count(after)
selected_memory = after.get("selectedMemory", [])
status = (
    "passed"
    if target_hit(seed)
    and not before_hit
    and after_hit
    and after_memory_signal_count > 0
    and selected_memory
    else "degraded"
)

payload = {
    "schemaVersion": "ctxhelm-memory-reuse-eval-v1",
    "status": status,
    "workflowKind": "experience-memory-reuse",
    "task": {
        "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
        "rawTaskStored": False,
    },
    "targetFile": target_file,
    "experienceCardId": experience_id,
    "seed": {
        "targetHit": target_hit(seed),
        "selectedMemoryCount": len(seed.get("selectedMemory", [])),
    },
    "beforeApproval": {
        "targetHit": before_hit,
        "selectedMemoryCount": len(before.get("selectedMemory", [])),
        "memorySignalCount": memory_signal_count(before),
    },
    "afterApproval": {
        "targetHit": after_hit,
        "selectedMemoryCount": len(selected_memory),
        "memorySignalCount": after_memory_signal_count,
        "targetCount": len(after.get("targetFiles", [])),
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
    raise SystemExit("memory reuse proof did not pass")
PY

echo "memory reuse smoke passed"
