#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: e2e-codex-memory-outcome-suite.sh --repo PATH [--repo PATH ...] [--pairs N] [--scan-commits N] [--output PATH]

Runs a source-free cross-repo Codex CLI outcome probe for experience memory:
  1. discover repeated-file historical pairs in each repo
  2. seed one approved experience card from the older pair task
  3. run the existing paired Codex read-only harness on the newer task
  4. aggregate target-read, irrelevant-read, evidence, client, and privacy fields

The report stores repo labels, commit prefixes, path labels, hashes, counters,
and booleans only. It does not store raw task text, raw prompts, raw transcripts,
raw MCP traffic, raw command output, source snippets, or raw repo paths.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
ctxhelm_bin="${CTXHELM_BIN:-}"
pairs_per_repo="1"
scan_commits="180"
output_path="${CTXHELM_CODEX_MEMORY_OUTCOME_SUITE_REPORT:-}"
repos=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repos+=("${2:-}")
      shift 2
      ;;
    --pairs)
      pairs_per_repo="${2:-}"
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

if [[ "${#repos[@]}" -eq 0 ]]; then
  usage
  exit 64
fi

if [[ -z "$ctxhelm_bin" ]]; then
  cargo build -p ctxhelm >/dev/null
  ctxhelm_bin="$repo_root/target/debug/ctxhelm"
fi
if [[ ! "$ctxhelm_bin" = /* || ! -x "$ctxhelm_bin" ]]; then
  echo "CTXHELM_BIN must be an absolute executable path: $ctxhelm_bin" >&2
  exit 64
fi
if ! command -v codex >/dev/null 2>&1; then
  echo "codex CLI is required for this real-client outcome probe" >&2
  exit 69
fi

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repos_json="$work_dir/repos.json"
python3 - "$repos_json" "${repos[@]}" <<'PY'
import json
import pathlib
import sys

pathlib.Path(sys.argv[1]).write_text(json.dumps(sys.argv[2:]) + "\n", encoding="utf-8")
PY

python3 - "$ctxhelm_bin" "$script_dir/e2e-agent-run-codex.sh" "$repos_json" "$pairs_per_repo" "$scan_commits" "$work_dir" "$output_path" <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time
from collections import defaultdict

ctxhelm_bin = pathlib.Path(sys.argv[1]).resolve()
codex_harness = pathlib.Path(sys.argv[2]).resolve()
repo_args = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
pairs_per_repo = int(sys.argv[4])
scan_commits = int(sys.argv[5])
work = pathlib.Path(sys.argv[6]).resolve()
output_path = sys.argv[7]

source_suffixes = {
    ".c", ".cc", ".cpp", ".cs", ".go", ".java", ".js", ".jsx", ".kt",
    ".php", ".py", ".rb", ".rs", ".scala", ".ts", ".tsx",
}
skip_parts = {".git", ".idea", ".vscode", "build", "dist", "node_modules", "target", "vendor"}


def sha256(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def run(cmd, *, cwd=None, env=None):
    result = subprocess.run(
        [str(part) for part in cmd],
        cwd=str(cwd) if cwd else None,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(map(str, cmd))}\n{result.stderr}"
        )
    return result.stdout


def git(repo, *args):
    return run(["git", "-C", str(repo), *args])


def source_like(path):
    p = pathlib.PurePosixPath(path)
    if any(part in skip_parts for part in p.parts):
        return False
    return p.suffix.lower() in source_suffixes


def path_exists_at_head(repo, path):
    result = subprocess.run(
        ["git", "-C", str(repo), "cat-file", "-e", f"HEAD:{path}"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def commit_parent(repo, sha):
    parents = git(repo, "rev-list", "--parents", "-n", "1", sha).strip().split()
    return parents[1] if len(parents) > 1 else None


def changed_files(repo, sha):
    text = git(repo, "diff-tree", "--no-commit-id", "--name-only", "-r", sha)
    return [line.strip() for line in text.splitlines() if line.strip()]


def commit_subject(repo, sha):
    return git(repo, "show", "-s", "--format=%s", sha).strip()


def discover_pairs(repo):
    commits = git(
        repo,
        "rev-list",
        "--no-merges",
        f"--max-count={scan_commits}",
        "HEAD",
    ).splitlines()
    commits = list(reversed([commit.strip() for commit in commits if commit.strip()]))
    previous_by_path = {}
    candidates = []
    for sha in commits:
        if not commit_parent(repo, sha):
            continue
        for path in changed_files(repo, sha):
            if not source_like(path) or not path_exists_at_head(repo, path):
                continue
            older = previous_by_path.get(path)
            if older:
                candidates.append({"olderSha": older, "newerSha": sha, "targetFile": path})
            previous_by_path[path] = sha
    selected = []
    selected_paths = set()
    for pair in candidates:
        if pair["targetFile"] in selected_paths:
            continue
        selected.append(pair)
        selected_paths.add(pair["targetFile"])
        if len(selected) >= pairs_per_repo:
            break
    if len(selected) < pairs_per_repo:
        seen = {(pair["olderSha"], pair["newerSha"], pair["targetFile"]) for pair in selected}
        for pair in candidates:
            key = (pair["olderSha"], pair["newerSha"], pair["targetFile"])
            if key in seen:
                continue
            selected.append(pair)
            seen.add(key)
            if len(selected) >= pairs_per_repo:
                break
    return selected, len(candidates), len({pair["targetFile"] for pair in candidates})


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def seed_memory(repo, pair, pair_dir):
    home = pair_dir / "home"
    home.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["CTXHELM_HOME"] = str(home)
    env["CTXHELM_BIN"] = str(ctxhelm_bin)
    older_subject = commit_subject(repo, pair["olderSha"])

    run(
        [
            ctxhelm_bin,
            "prepare-task",
            older_subject,
            "--repo",
            repo,
            "--mode",
            "bug-fix",
            "--path",
            pair["targetFile"],
            "--target-agent",
            "codex",
        ],
        env=env,
    )
    experience = json.loads(
        run(
            [
                ctxhelm_bin,
                "memory",
                "generate-experience",
                "--repo",
                repo,
                "--limit",
                "5",
                "--format",
                "json",
            ],
            env=env,
        )
    )
    cards = [card for card in experience.get("cards", []) if card.get("kind") == "experience"]
    if not cards:
        raise RuntimeError("no experience card generated")
    run(
        [
            ctxhelm_bin,
            "memory",
            "approve",
            cards[0]["id"],
            "--repo",
            repo,
            "--format",
            "json",
        ],
        env=env,
    )
    return home, cards[0]["id"], sha256(older_subject)


def run_codex_pair(repo, pair, index, repo_index):
    pair_dir = work / f"repo-{repo_index:02d}-pair-{index:02d}"
    pair_dir.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    home, card_id, older_task_hash = seed_memory(repo, pair, pair_dir)
    newer_subject = commit_subject(repo, pair["newerSha"])
    report_path = pair_dir / "codex-report.json"
    env = os.environ.copy()
    env["CTXHELM_HOME"] = str(home)
    env["CTXHELM_BIN"] = str(ctxhelm_bin)
    env["CTXHELM_RUN_REAL_CLIENT"] = "1"
    env.setdefault("CTXHELM_AGENT_RUN_TIMEOUT_SECONDS", "120")
    run(
        [
            codex_harness,
            "--repo",
            repo,
            "--task",
            newer_subject,
            "--target-file",
            pair["targetFile"],
            "--output",
            report_path,
        ],
        env=env,
    )
    report = load_json(report_path)
    return {
        "repoLabel": repo.name,
        "repoPathStored": False,
        "pairIndex": index,
        "targetFile": pair["targetFile"],
        "olderShaPrefix": pair["olderSha"][:12],
        "newerShaPrefix": pair["newerSha"][:12],
        "olderTaskSha256": older_task_hash,
        "newerTaskSha256": sha256(newer_subject),
        "rawTaskStored": False,
        "approvedExperienceCardIdSha256": sha256(card_id),
        "runtimeSeconds": round(time.monotonic() - started, 2),
        "report": report,
    }


def lane_summary(report, lane_id):
    for lane in report.get("lanes", []):
        if lane.get("lane") == lane_id:
            return lane
    return {}


def aggregate_pair(pair_result):
    report = pair_result["report"]
    comparison = report.get("comparison", {})
    baseline = lane_summary(report, "baseline")
    memory = lane_summary(report, "ctxhelm-memory")
    best = comparison.get("bestLane")
    return {
        "status": report.get("status"),
        "outcomeClaim": comparison.get("outcomeClaim"),
        "comparisonEligible": comparison.get("comparisonEligible", False),
        "bestLane": best,
        "baselineTargetReadCoverage": baseline.get("metrics", {}).get("targetReadCoverage"),
        "memoryTargetReadCoverage": memory.get("metrics", {}).get("targetReadCoverage"),
        "baselineIrrelevantReadCount": baseline.get("metrics", {}).get("irrelevantReadCount"),
        "memoryIrrelevantReadCount": memory.get("metrics", {}).get("irrelevantReadCount"),
        "baselineReadFileCount": baseline.get("metrics", {}).get("readFileCount"),
        "memoryReadFileCount": memory.get("metrics", {}).get("readFileCount"),
        "ctxhelmEvidenceMissesObserved": comparison.get("ctxhelmEvidenceMissesObserved", False),
        "ctxhelmUnderReadTargetsObserved": comparison.get("ctxhelmUnderReadTargetsObserved", False),
        "clientFailuresObserved": comparison.get("clientFailuresObserved", False),
        "rateLimitsObserved": comparison.get("rateLimitsObserved", False),
        "forbiddenCommandsObserved": comparison.get("forbiddenCommandsObserved", False),
        "missingRequiredCtxhelmCallsObserved": comparison.get("missingRequiredCtxhelmCallsObserved", False),
        "invalidRequiredCtxhelmCallsObserved": comparison.get("invalidRequiredCtxhelmCallsObserved", False),
    }


pair_results = []
errors = []
for repo_index, raw_repo in enumerate(repo_args, start=1):
    repo = pathlib.Path(raw_repo).resolve()
    if not (repo / ".git").exists():
        errors.append({
            "repoLabel": repo.name,
            "repoPathStored": False,
            "errorKind": "not_git_checkout",
            "errorSha256": sha256(str(repo)),
        })
        continue
    try:
        pairs, candidate_pair_count, candidate_target_file_count = discover_pairs(repo)
    except Exception as exc:
        errors.append({
            "repoLabel": repo.name,
            "repoPathStored": False,
            "errorKind": "pair_discovery_failed",
            "errorSha256": sha256(str(exc)),
        })
        continue
    for index, pair in enumerate(pairs, start=1):
        try:
            result = run_codex_pair(repo, pair, index, repo_index)
            result["candidatePairCount"] = candidate_pair_count
            result["candidateTargetFileCount"] = candidate_target_file_count
            pair_results.append(result)
        except Exception as exc:
            errors.append({
                "repoLabel": repo.name,
                "repoPathStored": False,
                "pairIndex": index,
                "targetFile": pair.get("targetFile"),
                "olderShaPrefix": pair.get("olderSha", "")[:12],
                "newerShaPrefix": pair.get("newerSha", "")[:12],
                "errorSha256": sha256(str(exc)),
            })

pair_summaries = []
for result in pair_results:
    summary = aggregate_pair(result)
    pair_summaries.append({
        "repoLabel": result["repoLabel"],
        "repoPathStored": False,
        "pairIndex": result["pairIndex"],
        "targetFile": result["targetFile"],
        "olderShaPrefix": result["olderShaPrefix"],
        "newerShaPrefix": result["newerShaPrefix"],
        "olderTaskSha256": result["olderTaskSha256"],
        "newerTaskSha256": result["newerTaskSha256"],
        "rawTaskStored": False,
        "approvedExperienceCardIdSha256": result["approvedExperienceCardIdSha256"],
        "runtimeSeconds": result["runtimeSeconds"],
        "candidatePairCount": result.get("candidatePairCount"),
        "candidateTargetFileCount": result.get("candidateTargetFileCount"),
        "summary": summary,
    })

repo_labels = sorted({result["repoLabel"] for result in pair_results})
evaluated_pairs = len(pair_results)
comparison_eligible_count = sum(
    1 for pair in pair_summaries if pair["summary"].get("comparisonEligible")
)
improved_pairs = sum(1 for pair in pair_summaries if pair["summary"].get("outcomeClaim") == "ctxhelm_improved")
matched_pairs = sum(1 for pair in pair_summaries if pair["summary"].get("outcomeClaim") == "ctxhelm_matched")
memory_improved_irrelevant = sum(
    1
    for pair in pair_summaries
    if (pair["summary"].get("baselineIrrelevantReadCount") is not None)
    and (pair["summary"].get("memoryIrrelevantReadCount") is not None)
    and pair["summary"]["memoryIrrelevantReadCount"] < pair["summary"]["baselineIrrelevantReadCount"]
)
memory_target_read_improved = sum(
    1
    for pair in pair_summaries
    if (pair["summary"].get("baselineTargetReadCoverage") is not None)
    and (pair["summary"].get("memoryTargetReadCoverage") is not None)
    and pair["summary"]["memoryTargetReadCoverage"] > pair["summary"]["baselineTargetReadCoverage"]
)
memory_target_read_matched_or_improved = sum(
    1
    for pair in pair_summaries
    if (pair["summary"].get("baselineTargetReadCoverage") is not None)
    and (pair["summary"].get("memoryTargetReadCoverage") is not None)
    and pair["summary"]["memoryTargetReadCoverage"] >= pair["summary"]["baselineTargetReadCoverage"]
)
memory_underread = sum(1 for pair in pair_summaries if pair["summary"].get("ctxhelmUnderReadTargetsObserved"))
client_failures = sum(1 for pair in pair_summaries if pair["summary"].get("clientFailuresObserved"))
rate_limits = sum(1 for pair in pair_summaries if pair["summary"].get("rateLimitsObserved"))
evidence_misses = sum(1 for pair in pair_summaries if pair["summary"].get("ctxhelmEvidenceMissesObserved"))
forbidden = sum(1 for pair in pair_summaries if pair["summary"].get("forbiddenCommandsObserved"))
missing_calls = sum(1 for pair in pair_summaries if pair["summary"].get("missingRequiredCtxhelmCallsObserved"))
invalid_calls = sum(1 for pair in pair_summaries if pair["summary"].get("invalidRequiredCtxhelmCallsObserved"))
privacy = {
    "localOnly": True,
    "repoPathStored": False,
    "sourceTextLogged": False,
    "rawPromptStored": False,
    "rawTaskTextStored": False,
    "rawTranscriptStored": False,
    "rawMcpTrafficStored": False,
    "rawCommandOutputStored": False,
    "remoteEmbeddingsUsed": False,
    "remoteRerankingUsed": False,
}
for result in pair_results:
    status = result["report"].get("privacyStatus", {})
    privacy["localOnly"] = privacy["localOnly"] and bool(status.get("localOnly", True))
    for key in [
        "sourceTextLogged",
        "rawPromptStored",
        "rawTranscriptStored",
        "rawMcpTrafficStored",
        "rawCommandOutputStored",
        "remoteEmbeddingsUsed",
        "remoteRerankingUsed",
    ]:
        privacy[key] = privacy[key] or bool(status.get(key, False))

recommended = []
if client_failures or rate_limits:
    recommended.append("retry_real_client_when_available")
if evidence_misses:
    recommended.append("fix_memory_evidence_retrieval")
if memory_underread:
    recommended.append("harden_memory_native_read_guidance")
if missing_calls or invalid_calls:
    recommended.append("harden_required_ctxhelm_call_guidance")
if forbidden:
    recommended.append("tighten_read_only_codex_prompt")
if evaluated_pairs and len(repo_labels) < 3:
    recommended.append("expand_memory_outcome_repository_diversity")
if comparison_eligible_count and not improved_pairs:
    recommended.append("analyze_memory_outcome_no_lift")
if not recommended and evaluated_pairs:
    recommended.append("preserve_current_memory_agent_contract")

payload = {
    "schemaVersion": "ctxhelm-codex-memory-outcome-suite-v1",
    "status": "passed"
    if evaluated_pairs
    and client_failures == 0
    and rate_limits == 0
    and evidence_misses == 0
    and forbidden == 0
    and missing_calls == 0
    and invalid_calls == 0
    else ("degraded" if evaluated_pairs else "insufficient_evidence"),
    "workflowKind": "cross-repo-codex-memory-outcome-suite",
    "client": {
        "name": "codex",
        "version": run(["codex", "--version"]).splitlines()[0],
    },
    "ctxhelmVersion": run([ctxhelm_bin, "--version"]).splitlines()[0],
    "suite": {
        "requestedRepositoryCount": len(repo_args),
        "evaluatedRepositoryCount": len(repo_labels),
        "evaluatedPairCount": evaluated_pairs,
        "comparisonEligibleCount": comparison_eligible_count,
        "pairsPerRepo": pairs_per_repo,
        "scanCommitsPerRepo": scan_commits,
        "rawTasksStored": False,
    },
    "aggregate": {
        "improvedPairCount": improved_pairs,
        "matchedPairCount": matched_pairs,
        "memoryTargetReadImprovedPairCount": memory_target_read_improved,
        "memoryTargetReadMatchedOrImprovedPairCount": memory_target_read_matched_or_improved,
        "memoryIrrelevantReadImprovedPairCount": memory_improved_irrelevant,
        "ctxhelmUnderReadPairCount": memory_underread,
        "clientFailurePairCount": client_failures,
        "rateLimitPairCount": rate_limits,
        "ctxhelmEvidenceMissPairCount": evidence_misses,
        "forbiddenCommandPairCount": forbidden,
        "missingRequiredCtxhelmCallPairCount": missing_calls,
        "invalidRequiredCtxhelmCallPairCount": invalid_calls,
        "recommendedNextRAndD": recommended,
    },
    "pairs": pair_summaries,
    "errors": errors,
    "privacyStatus": privacy,
    "unsupportedActions": [
        "source edits",
        "user project tests",
        "global agent config mutation",
        "cloud upload",
        "raw prompt storage",
        "raw transcript storage",
        "raw MCP traffic storage",
        "raw command output storage",
    ],
}

text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output_path:
    path = pathlib.Path(output_path).resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
else:
    print(text, end="")

if evaluated_pairs == 0:
    raise SystemExit("no codex memory outcome pairs were evaluated")
PY

echo "codex memory outcome suite complete"
