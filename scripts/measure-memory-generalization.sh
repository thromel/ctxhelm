#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: measure-memory-generalization.sh --repo PATH [--pairs N] [--scan-commits N] [--output PATH]

Measures source-free experience-memory reuse across multiple repeated-file
historical pairs in a real local repository. For each pair, the script:
  1. finds an older and newer commit that touched the same source-like file
  2. evaluates the newer commit before memory approval
  3. seeds one approved experience card from the older task and target path
  4. evaluates the newer commit again
  5. aggregates target lift, lexical coverage, memory noise, and runtime

This is an R&D measurement harness, not a pass/fail release smoke. It stores
commit/task hashes, path labels, counters, and booleans only. It does not store
raw commit subjects, source snippets, terminal logs, model transcripts, or raw
MCP traffic.
EOF
}

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
repo_path=""
output_path="${CTXHELM_MEMORY_GENERALIZATION_REPORT:-}"
pairs="3"
scan_commits="300"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo_path="${2:-}"
      shift 2
      ;;
    --pairs)
      pairs="${2:-}"
      shift 2
      ;;
    --scan-commits)
      scan_commits="${2:-}"
      shift 2
      ;;
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

if [[ -z "$repo_path" ]]; then
  usage
  exit 64
fi
if [[ ! -d "$repo_path/.git" ]]; then
  echo "repo is not a git checkout: $repo_path" >&2
  exit 66
fi

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

python3 - "$ctxhelm_bin" "$repo_path" "$pairs" "$scan_commits" "$output_path" "$work_dir" <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time

ctxhelm_bin = sys.argv[1]
repo = pathlib.Path(sys.argv[2]).resolve()
requested_pairs = int(sys.argv[3])
scan_commits = int(sys.argv[4])
output_path = sys.argv[5]
work = pathlib.Path(sys.argv[6]).resolve()

source_suffixes = {
    ".c",
    ".cc",
    ".cpp",
    ".cs",
    ".go",
    ".java",
    ".js",
    ".jsx",
    ".kt",
    ".php",
    ".py",
    ".rb",
    ".rs",
    ".scala",
    ".ts",
    ".tsx",
}

skip_parts = {
    ".git",
    ".idea",
    ".vscode",
    "build",
    "dist",
    "node_modules",
    "target",
    "vendor",
}


def run(cmd, *, env=None, cwd=None, capture=True):
    result = subprocess.run(
        cmd,
        cwd=str(cwd or repo),
        env=env,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(map(str, cmd))}\n{result.stderr}"
        )
    return result.stdout if capture else ""


def git(*args):
    return run(["git", "-C", str(repo), *args])


def sha256(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def source_like(path):
    p = pathlib.PurePosixPath(path)
    if any(part in skip_parts for part in p.parts):
        return False
    return p.suffix.lower() in source_suffixes


def path_exists_at_head(path):
    result = subprocess.run(
        ["git", "-C", str(repo), "cat-file", "-e", f"HEAD:{path}"],
        text=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def commit_subject(sha):
    return git("show", "-s", "--format=%s", sha).strip()


def commit_parent(sha):
    parents = git("rev-list", "--parents", "-n", "1", sha).strip().split()
    return parents[1] if len(parents) > 1 else None


def changed_files(sha):
    text = git("diff-tree", "--no-commit-id", "--name-only", "-r", sha)
    return [line.strip() for line in text.splitlines() if line.strip()]


def discover_pairs():
    commits = git(
        "rev-list",
        "--no-merges",
        f"--max-count={scan_commits}",
        "HEAD",
    ).splitlines()
    commits = list(reversed([commit.strip() for commit in commits if commit.strip()]))
    previous_by_path = {}
    pairs = []
    for sha in commits:
        if not commit_parent(sha):
            continue
        for path in changed_files(sha):
            if not source_like(path) or not path_exists_at_head(path):
                continue
            older = previous_by_path.get(path)
            if older:
                pairs.append(
                    {
                        "olderSha": older,
                        "newerSha": sha,
                        "targetFile": path,
                    }
                )
                if len(pairs) >= requested_pairs:
                    return pairs
            previous_by_path[path] = sha
    return pairs


def load_json(path):
    return json.loads(path.read_text())


def target_in(report, target, key):
    return any(target in commit.get(key, []) for commit in report.get("commits", []))


def retrieval_targets(report):
    targets = []
    for commit in report.get("commits", []):
        targets.extend(commit.get("retrievalTargetFiles", []))
    return targets


def actions(report):
    return [
        action.get("action")
        for action in report.get("recommendedResearchActions", [])
        if action.get("action")
    ]


def evaluate(pair, index):
    pair_dir = work / f"pair-{index:02d}"
    pair_dir.mkdir(parents=True, exist_ok=True)
    home = pair_dir / "home"
    home.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["CTXHELM_HOME"] = str(home)

    target = pair["targetFile"]
    older_subject = commit_subject(pair["olderSha"])
    newer_subject = commit_subject(pair["newerSha"])
    base = commit_parent(pair["newerSha"])
    if not base:
        raise RuntimeError(f"newer commit has no parent: {pair['newerSha']}")

    timings = {}
    before_path = pair_dir / "before-history.json"
    start = time.monotonic()
    before_path.write_text(
        run(
            [
                ctxhelm_bin,
                "eval",
                "history",
                "--repo",
                str(repo),
                "--base",
                base,
                "--head",
                pair["newerSha"],
                "--limit",
                "1",
                "--budget",
                "10",
                "--mode",
                "bug-fix",
                "--target-agent",
                "claude-code",
                "--format",
                "json",
                "--force",
            ],
            env=env,
        )
    )
    timings["beforeEval"] = round(time.monotonic() - start, 2)

    seed_path = pair_dir / "seed-plan.json"
    start = time.monotonic()
    seed_path.write_text(
        run(
            [
                ctxhelm_bin,
                "prepare-task",
                older_subject,
                "--repo",
                str(repo),
                "--mode",
                "bug-fix",
                "--path",
                target,
                "--target-agent",
                "claude-code",
            ],
            env=env,
        )
    )
    timings["seedPrepareTask"] = round(time.monotonic() - start, 2)

    experience_path = pair_dir / "experience.json"
    start = time.monotonic()
    experience_path.write_text(
        run(
            [
                ctxhelm_bin,
                "memory",
                "generate-experience",
                "--repo",
                str(repo),
                "--limit",
                "5",
                "--format",
                "json",
            ],
            env=env,
        )
    )
    timings["generateExperience"] = round(time.monotonic() - start, 2)

    experience = load_json(experience_path)
    cards = [card for card in experience.get("cards", []) if card.get("kind") == "experience"]
    if not cards:
        raise RuntimeError(f"no experience card generated for {target}")
    experience_id = cards[0]["id"]

    approve_path = pair_dir / "approve.json"
    start = time.monotonic()
    approve_path.write_text(
        run(
            [
                ctxhelm_bin,
                "memory",
                "approve",
                experience_id,
                "--repo",
                str(repo),
                "--format",
                "json",
            ],
            env=env,
        )
    )
    timings["approve"] = round(time.monotonic() - start, 2)

    after_path = pair_dir / "after-history.json"
    start = time.monotonic()
    after_path.write_text(
        run(
            [
                ctxhelm_bin,
                "eval",
                "history",
                "--repo",
                str(repo),
                "--base",
                base,
                "--head",
                pair["newerSha"],
                "--limit",
                "1",
                "--budget",
                "10",
                "--mode",
                "bug-fix",
                "--target-agent",
                "claude-code",
                "--format",
                "json",
                "--force",
            ],
            env=env,
        )
    )
    timings["afterEval"] = round(time.monotonic() - start, 2)

    before = load_json(before_path)
    after = load_json(after_path)
    combined_text = json.dumps({"before": before, "after": after}, sort_keys=True)
    if older_subject and older_subject in combined_text:
        raise RuntimeError("history report leaked older raw task text")
    if newer_subject and newer_subject in combined_text:
        raise RuntimeError("history report leaked newer raw task text")

    before_memory = before.get("memoryReuseSummary", {})
    after_memory = after.get("memoryReuseSummary", {})
    target_in_after_combined = target_in(after, target, "recommendedContextFiles")
    target_in_after_lexical = target_in(after, target, "lexicalBaselineFiles")
    target_in_before_combined = target_in(before, target, "recommendedContextFiles")
    target_in_before_lexical = target_in(before, target, "lexicalBaselineFiles")
    unique_lift = after_memory.get("memoryUniqueTargetHitCount", 0) > before_memory.get(
        "memoryUniqueTargetHitCount", 0
    )

    return {
        "pairIndex": index,
        "targetFile": target,
        "olderShaPrefix": pair["olderSha"][:12],
        "newerShaPrefix": pair["newerSha"][:12],
        "olderTaskSha256": sha256(older_subject),
        "newerTaskSha256": sha256(newer_subject),
        "rawTaskStored": False,
        "evaluatedCommits": after.get("evaluatedCommits"),
        "retrievalTargetMatched": target in retrieval_targets(after),
        "beforeApproval": {
            "targetInCombined": target_in_before_combined,
            "targetInLexical": target_in_before_lexical,
            "memoryReuseSummary": before_memory,
            "recommendedResearchActions": actions(before),
        },
        "afterApproval": {
            "targetInCombined": target_in_after_combined,
            "targetInLexical": target_in_after_lexical,
            "memoryReuseSummary": after_memory,
            "recommendedResearchActions": actions(after),
        },
        "lift": {
            "memoryCandidateDelta": after_memory.get("memoryCandidateCount", 0)
            - before_memory.get("memoryCandidateCount", 0),
            "memoryTargetHitDelta": after_memory.get("memoryTargetHitAt10Count", 0)
            - before_memory.get("memoryTargetHitAt10Count", 0),
            "memoryUniqueTargetHitDelta": after_memory.get("memoryUniqueTargetHitCount", 0)
            - before_memory.get("memoryUniqueTargetHitCount", 0),
            "uniqueTargetLiftBeyondLexical": unique_lift,
            "combinedRecoveredTarget": (not target_in_before_combined)
            and target_in_after_combined,
            "lexicalAlreadyCoveredTarget": target_in_after_lexical,
            "uniqueNonTargetNoise": after_memory.get("memoryUniqueNonTargetCount", 0),
            "targetMissedByMemory": after_memory.get("memoryTargetMissedAt10Count", 0),
        },
        "runtimeSeconds": timings,
        "experience": {
            "generatedCards": len(experience.get("cards", [])),
            "approvedExperienceCardId": experience_id,
        },
    }


pairs = discover_pairs()
results = []
errors = []
for index, pair in enumerate(pairs, start=1):
    try:
        results.append(evaluate(pair, index))
    except Exception as exc:
        errors.append(
            {
                "pairIndex": index,
                "targetFile": pair.get("targetFile"),
                "olderShaPrefix": pair.get("olderSha", "")[:12],
                "newerShaPrefix": pair.get("newerSha", "")[:12],
                "errorSha256": sha256(str(exc)),
            }
        )

evaluated = len(results)
unique_lift_pairs = sum(1 for result in results if result["lift"]["uniqueTargetLiftBeyondLexical"])
target_hit_pairs = sum(
    1
    for result in results
    if result["afterApproval"]["memoryReuseSummary"].get("memoryTargetHitAt10Count", 0) > 0
)
candidate_pairs = sum(
    1
    for result in results
    if result["afterApproval"]["memoryReuseSummary"].get("memoryCandidateCount", 0) > 0
)
lexical_covered_pairs = sum(1 for result in results if result["lift"]["lexicalAlreadyCoveredTarget"])
noise_pairs = sum(1 for result in results if result["lift"]["uniqueNonTargetNoise"] > 0)
combined_recovered_pairs = sum(1 for result in results if result["lift"]["combinedRecoveredTarget"])
total_unique_non_targets = sum(result["lift"]["uniqueNonTargetNoise"] for result in results)
total_unique_target_hits = sum(result["lift"]["memoryUniqueTargetHitDelta"] for result in results)
total_after_seconds = round(
    sum(result["runtimeSeconds"].get("afterEval", 0.0) for result in results), 2
)

status = "measured" if evaluated else "insufficient_evidence"
payload = {
    "schemaVersion": "ctxhelm-memory-generalization-measurement-v1",
    "status": status,
    "workflowKind": "multi-pair-experience-memory-generalization",
    "repo": {
        "label": repo.name,
        "pathStored": False,
        "headShaPrefix": git("rev-parse", "HEAD").strip()[:12],
        "scanCommits": scan_commits,
        "requestedPairs": requested_pairs,
        "discoveredPairs": len(pairs),
        "evaluatedPairs": evaluated,
        "errorCount": len(errors),
    },
    "aggregate": {
        "memoryCandidatePairs": candidate_pairs,
        "memoryTargetHitPairs": target_hit_pairs,
        "memoryUniqueLiftPairs": unique_lift_pairs,
        "combinedRecoveredPairs": combined_recovered_pairs,
        "lexicalCoveredPairs": lexical_covered_pairs,
        "memoryNoisePairs": noise_pairs,
        "memoryUniqueTargetHitCount": total_unique_target_hits,
        "memoryUniqueNonTargetCount": total_unique_non_targets,
        "afterEvalRuntimeSecondsTotal": total_after_seconds,
    },
    "interpretation": {
        "generalizationProven": unique_lift_pairs > 1,
        "singlePairLiftObserved": unique_lift_pairs == 1,
        "precisionNeedsWork": noise_pairs > 0 or total_unique_non_targets > total_unique_target_hits,
        "lexicalStillStrong": lexical_covered_pairs > 0,
        "recommendedNextRAndD": [
            "increase_real_corpus_pair_count",
            "reduce_memory_unique_non_target_noise",
            "compare_against_lexical_graph_semantic_ablations",
            "measure_real_agent_outcome_lift",
        ],
    },
    "pairs": results,
    "errors": errors,
    "privacyStatus": {
        "localOnly": True,
        "repoPathStored": False,
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

if not evaluated:
    raise SystemExit("no repeated-file memory generalization pairs were evaluated")
PY

echo "memory generalization measurement complete"
