#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: smoke-memory-benchmark-lift.sh [--output PATH]

Builds two temporary repos and proves source-free memory lift is visible through
the benchmark/product-proof aggregation path:
  1. seed one approved experience card per repo
  2. run `eval proof` over both repos
  3. prove each embedded historical report has a memory-only target hit beyond
     lexical
  4. prove product proof routes R&D to `evaluate_memory_reuse_lift`

The report stores path labels and counts only. It does not store raw prompts,
source snippets, terminal logs, model transcripts, or raw MCP traffic.
EOF
}

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
output_path="${CTXHELM_MEMORY_BENCHMARK_LIFT_REPORT:-}"

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

home="$work_dir/home"
mkdir -p "$home"
export CTXHELM_HOME="$home"

seed_repo() {
  local repo="$1"
  local target_file="$2"
  local commit_subject="$3"
  local sentinel="$4"
  local source_text="$5"

  mkdir -p "$repo/$(dirname "$target_file")"
  git -C "$repo" init -q
  git -C "$repo" config user.email ctxhelm@example.com
  git -C "$repo" config user.name ctxhelm
  printf '%b\n' "$source_text" >"$repo/$target_file"
  git -C "$repo" add .
  git -C "$repo" commit -m "$commit_subject" >/dev/null

  "$ctxhelm_bin" prepare-task "$commit_subject" \
    --repo "$repo" \
    --mode bug-fix \
    --path "$target_file" \
    --target-agent claude-code \
    >"$repo.seed-plan.json"
  "$ctxhelm_bin" memory generate-experience \
    --repo "$repo" \
    --limit 5 \
    --format json \
    >"$repo.experience.json"

  local experience_id
  experience_id="$(python3 - "$repo.experience.json" <<'PY'
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
    >"$repo.approve.json"
  printf '%s\n' "$experience_id" >"$repo.experience-id"
  printf '%s\n' "$sentinel" >"$repo.sentinel"
}

repo_a="$work_dir/repo-a"
repo_b="$work_dir/repo-b"
target_a="src/payments/handler.ts"
target_b="services/billing/retry.py"
task_a="fix checkout signature regression"
task_b="fix checkout signature regression"
sentinel_a="CTXHELM_MEMORY_BENCHMARK_SOURCE_SENTINEL_A"
sentinel_b="CTXHELM_MEMORY_BENCHMARK_SOURCE_SENTINEL_B"

seed_repo \
  "$repo_a" \
  "$target_a" \
  "$task_a" \
  "$sentinel_a" \
  "export function processWebhook() { return \"$sentinel_a\"; }"
seed_repo \
  "$repo_b" \
  "$target_b" \
  "$task_b" \
  "$sentinel_b" \
  "def retry_invoice():\n    return \"$sentinel_b\""

config_path="$work_dir/benchmark-config.json"
python3 - "$config_path" "$repo_a" "$repo_b" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
repo_a = sys.argv[2]
repo_b = sys.argv[3]
payload = {
    "name": "memory benchmark lift smoke",
    "corpusId": "ctxhelm-memory-benchmark-lift-smoke",
    "privacyLabel": "source-free-local-smoke",
    "defaults": {
        "limit": 1,
        "rankingBudget": 10,
        "mode": "bug_fix",
        "targetAgent": "claude-code",
        "forceRefresh": True,
        "cacheEnabled": False,
    },
    "repositories": [
        {"name": "memory-ts", "path": repo_a},
        {"name": "memory-python", "path": repo_b},
    ],
}
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY

proof_path="$work_dir/product-proof.json"
"$ctxhelm_bin" eval proof \
  --config "$config_path" \
  --format json \
  >"$proof_path"

python3 - "$work_dir" "$home" "$target_a" "$target_b" "$task_a" "$task_b" "$output_path" <<'PY'
import hashlib
import json
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
home = pathlib.Path(sys.argv[2])
targets = [sys.argv[3], sys.argv[4]]
tasks = [sys.argv[5], sys.argv[6]]
output_path = sys.argv[7]
proof = json.loads((work / "product-proof.json").read_text())

def actions(report):
    return [action.get("action") for action in report.get("recommendedResearchActions", [])]

def repo_summary(repo):
    report = repo.get("report") or {}
    commits = report.get("commits", [])
    target = None
    target_in_combined = False
    target_in_lexical = False
    if commits:
        retrieval_targets = commits[0].get("retrievalTargetFiles", [])
        target = retrieval_targets[0] if retrieval_targets else None
        target_in_combined = bool(target and target in commits[0].get("recommendedContextFiles", []))
        target_in_lexical = bool(target and target in commits[0].get("lexicalBaselineFiles", []))
    memory = report.get("memoryReuseSummary", {})
    return {
        "name": repo.get("name"),
        "evaluatedCommits": repo.get("evaluatedCommits"),
        "targetFile": target,
        "targetInCombined": target_in_combined,
        "targetInLexical": target_in_lexical,
        "memoryReuseSummary": memory,
        "recommendedResearchActions": actions(report),
        "privacyStatus": repo.get("privacyStatus", {}),
    }

repo_summaries = [repo_summary(repo) for repo in proof.get("benchmarkReport", {}).get("repositories", [])]
sentinel_values = [
    (work / "repo-a.sentinel").read_text().strip(),
    (work / "repo-b.sentinel").read_text().strip(),
]
proof_text = json.dumps(proof, sort_keys=True)
if any(value in proof_text for value in sentinel_values):
    raise SystemExit("product proof leaked source sentinel")
if any(task in proof_text for task in tasks):
    raise SystemExit("product proof leaked raw task text")

database_paths = list(home.glob("repos/*/ctxhelm.sqlite3"))
if len(database_paths) != 2:
    raise SystemExit(f"expected two storage databases, found {database_paths}")
for database_path in database_paths:
    data = database_path.read_bytes()
    for value in sentinel_values:
        if value.encode("utf-8") in data:
            raise SystemExit("storage leaked source sentinel")

per_repo_passed = []
for expected_target, summary in zip(targets, repo_summaries):
    memory = summary.get("memoryReuseSummary", {})
    per_repo_passed.append(
        summary.get("evaluatedCommits") == 1
        and summary.get("targetFile") == expected_target
        and summary.get("targetInCombined") is True
        and summary.get("targetInLexical") is False
        and memory.get("memoryCandidateCount", 0) > 0
        and memory.get("memorySelectedAt10Count", 0) > 0
        and memory.get("memoryTargetHitAt10Count", 0) > 0
        and memory.get("memoryUniqueTargetHitCount", 0) > 0
        and memory.get("memoryUniqueNonTargetCount", 0) == 0
    )

product_actions = actions(proof)
status = (
    "passed"
    if proof.get("evaluatedRepositoryCount") == 2
    and proof.get("evaluatedCommitCount") == 2
    and len(repo_summaries) == 2
    and all(per_repo_passed)
    and "evaluate_memory_reuse_lift" in product_actions
    and proof.get("privacyStatus", {}).get("localOnly") is True
    else "degraded"
)

payload = {
    "schemaVersion": "ctxhelm-memory-benchmark-lift-eval-v1",
    "status": status,
    "workflowKind": "benchmark-product-proof-experience-memory-lift",
    "suite": {
        "evaluatedRepositoryCount": proof.get("evaluatedRepositoryCount"),
        "evaluatedCommitCount": proof.get("evaluatedCommitCount"),
        "suiteId": proof.get("suiteId"),
    },
    "tasks": [
        {
            "taskSha256": hashlib.sha256(task.encode("utf-8")).hexdigest(),
            "rawTaskStored": False,
            "targetFile": target,
        }
        for task, target in zip(tasks, targets)
    ],
    "repositories": repo_summaries,
    "productProof": {
        "recommendedResearchActions": product_actions,
        "releaseGateDecision": proof.get("releaseGate", {}).get("decision"),
        "defaultPromotionAllowed": proof.get("releaseGate", {}).get("defaultPromotionAllowed"),
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
    raise SystemExit("benchmark memory lift proof did not pass")
PY

echo "memory benchmark lift smoke passed"
