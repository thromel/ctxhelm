#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: measure-memory-generalization.sh --repo PATH [--pairs N] [--scan-commits N] [--semantic] [--semantic-provider PROVIDER] [--output PATH]

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
semantic_enabled="false"
semantic_provider="local_hash"

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
    --semantic)
      semantic_enabled="true"
      shift
      ;;
    --semantic-provider)
      semantic_provider="${2:-}"
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

python3 - "$ctxhelm_bin" "$repo_path" "$pairs" "$scan_commits" "$output_path" "$work_dir" "$semantic_enabled" "$semantic_provider" <<'PY'
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
semantic_enabled = sys.argv[7] == "true"
semantic_provider = sys.argv[8]

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
    candidates = []
    for sha in commits:
        if not commit_parent(sha):
            continue
        for path in changed_files(sha):
            if not source_like(path) or not path_exists_at_head(path):
                continue
            older = previous_by_path.get(path)
            if older:
                candidates.append(
                    {
                        "olderSha": older,
                        "newerSha": sha,
                        "targetFile": path,
                    }
                )
            previous_by_path[path] = sha
    selected = []
    selected_paths = set()
    for pair in candidates:
        path = pair["targetFile"]
        if path in selected_paths:
            continue
        selected.append(pair)
        selected_paths.add(path)
        if len(selected) >= requested_pairs:
            break
    if len(selected) < requested_pairs:
        selected_keys = {(pair["olderSha"], pair["newerSha"], pair["targetFile"]) for pair in selected}
        for pair in candidates:
            key = (pair["olderSha"], pair["newerSha"], pair["targetFile"])
            if key in selected_keys:
                continue
            selected.append(pair)
            selected_keys.add(key)
            if len(selected) >= requested_pairs:
                break
    return selected, len(candidates), len({pair["targetFile"] for pair in candidates})


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


def eval_history_args(base, head):
    args = [
        ctxhelm_bin,
        "eval",
        "history",
        "--repo",
        str(repo),
        "--base",
        base,
        "--head",
        head,
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
    ]
    if semantic_enabled:
        args.extend(["--semantic", "--semantic-provider", semantic_provider])
    return args


def selected_signal_profiles(report):
    profiles = []
    for commit in report.get("commits", []):
        profiles.extend(commit.get("selectedSignalProfiles", []))
    return profiles


def selected_signal_count(report, signals, field):
    signal_set = set(signals)
    return sum(
        int(profile.get(field, 0) or 0)
        for profile in selected_signal_profiles(report)
        if profile.get("signal") in signal_set
    )


def signal_ablation_delta(report, signal):
    current = report.get("fileRecallAt10")
    if current is None:
        return None
    for ablation in report.get("signalAblations", []):
        if ablation.get("disabledSignal") != signal:
            continue
        disabled = ablation.get("metrics", {}).get("recallAtK")
        if disabled is None:
            return None
        return round(float(current) - float(disabled), 7)
    return None


def graph_edge_target_hit_count(report):
    return sum(
        int(profile.get("retrievalTargetHitAt10Count", 0) or 0)
        for profile in report.get("graphEdgeProfiles", [])
    )


def graph_edge_removed_target_hit_count(report):
    return sum(
        int(ablation.get("removedTargetHitAt10Count", 0) or 0)
        for ablation in report.get("graphEdgeAblations", [])
    )


def memory_signal_comparison(report, memory_summary):
    memory_target_hits = int(memory_summary.get("memoryTargetHitAt10Count", 0) or 0)
    memory_unique_target_hits = int(memory_summary.get("memoryUniqueTargetHitCount", 0) or 0)
    graph_target_hits = graph_edge_target_hit_count(report)
    graph_signal_targets = selected_signal_count(
        report,
        ["dependency", "co_change", "related_test"],
        "retrievalTargetSelectedAt10Count",
    )
    semantic_target_hits = selected_signal_count(
        report,
        ["semantic"],
        "retrievalTargetSelectedAt10Count",
    )
    lexical_target_hits = selected_signal_count(
        report,
        ["lexical"],
        "retrievalTargetSelectedAt10Count",
    )
    graph_support = max(graph_target_hits, graph_signal_targets)
    semantic_support = semantic_target_hits
    graph_or_semantic_support = max(graph_support, semantic_support)
    return {
        "semanticMeasured": semantic_enabled,
        "semanticProvider": semantic_provider if semantic_enabled else None,
        "memorySelectedCount": int(memory_summary.get("memorySelectedAt10Count", 0) or 0),
        "memoryTargetHitCount": memory_target_hits,
        "memoryUniqueTargetHitCount": memory_unique_target_hits,
        "memoryUniqueNonTargetCount": int(memory_summary.get("memoryUniqueNonTargetCount", 0) or 0),
        "memoryUniqueTargetHitCurrentSupportSignalCounts": memory_summary.get(
            "memoryUniqueTargetHitCurrentSupportSignalCounts", {}
        ),
        "memoryUniqueNonTargetCurrentSupportSignalCounts": memory_summary.get(
            "memoryUniqueNonTargetCurrentSupportSignalCounts", {}
        ),
        "lexicalSelectedTargetCount": lexical_target_hits,
        "graphSelectedTargetCount": graph_signal_targets,
        "graphEdgeTargetHitCount": graph_target_hits,
        "semanticSelectedTargetCount": semantic_target_hits,
        "graphEdgeAblationRemovedTargetHitCount": graph_edge_removed_target_hit_count(report),
        "dependencyAblationRecallDeltaAt10": signal_ablation_delta(report, "dependency"),
        "semanticAblationRecallDeltaAt10": signal_ablation_delta(report, "semantic")
        if semantic_enabled
        else None,
        "memoryTargetHitsWithGraphSupportUpperBound": min(memory_target_hits, graph_support),
        "memoryTargetHitsWithSemanticSupportUpperBound": min(memory_target_hits, semantic_support),
        "memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound": min(
            memory_unique_target_hits,
            graph_or_semantic_support,
        ),
        "memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound": max(
            0,
            memory_target_hits - graph_or_semantic_support,
        ),
    }


def ordered_difference(left, right):
    right_set = set(right)
    return [value for value in left if value not in right_set]


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
            eval_history_args(base, pair["newerSha"]),
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
            eval_history_args(base, pair["newerSha"]),
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
    signal_comparison = memory_signal_comparison(after, after_memory)
    target_in_after_combined = target_in(after, target, "recommendedContextFiles")
    target_in_after_lexical = target_in(after, target, "lexicalBaselineFiles")
    target_in_before_combined = target_in(before, target, "recommendedContextFiles")
    target_in_before_lexical = target_in(before, target, "lexicalBaselineFiles")
    unique_lift = after_memory.get("memoryUniqueTargetHitCount", 0) > before_memory.get(
        "memoryUniqueTargetHitCount", 0
    )
    before_context_files = before.get("commits", [{}])[0].get("recommendedContextFiles", [])
    after_context_files = after.get("commits", [{}])[0].get("recommendedContextFiles", [])
    target_files = set(retrieval_targets(after))
    pack_added_files = ordered_difference(after_context_files, before_context_files)
    pack_removed_files = ordered_difference(before_context_files, after_context_files)
    pack_added_target_files = [path for path in pack_added_files if path in target_files]
    pack_added_non_target_files = [path for path in pack_added_files if path not in target_files]
    memory_unique_non_targets = int(after_memory.get("memoryUniqueNonTargetCount", 0) or 0)
    pack_added_non_target_count = len(pack_added_non_target_files)
    signal_only_non_target_count = max(
        0,
        memory_unique_non_targets - pack_added_non_target_count,
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
            "uniqueNonTargetWithoutCurrentSupport": after_memory.get(
                "memoryUniqueNonTargetWithoutCurrentSupportCount", 0
            ),
            "uniqueTargetHitCurrentSupportSignalCounts": after_memory.get(
                "memoryUniqueTargetHitCurrentSupportSignalCounts", {}
            ),
            "uniqueNonTargetCurrentSupportSignalCounts": after_memory.get(
                "memoryUniqueNonTargetCurrentSupportSignalCounts", {}
            ),
            "uniqueTargetHitWithoutCurrentSupport": after_memory.get(
                "memoryUniqueTargetHitWithoutCurrentSupportCount", 0
            ),
            "targetMissedByMemory": after_memory.get("memoryTargetMissedAt10Count", 0),
            "packChangedByMemory": bool(pack_added_files or pack_removed_files),
            "packAddedFileCount": len(pack_added_files),
            "packRemovedFileCount": len(pack_removed_files),
            "packAddedTargetCount": len(pack_added_target_files),
            "packAddedNonTargetCount": pack_added_non_target_count,
            "signalOnlyNonTargetCount": signal_only_non_target_count,
            "packAddedFiles": pack_added_files,
            "packRemovedFiles": pack_removed_files,
            "packAddedTargetFiles": pack_added_target_files,
            "packAddedNonTargetFiles": pack_added_non_target_files,
        },
        "signalComparison": signal_comparison,
        "runtimeSeconds": timings,
        "experience": {
            "generatedCards": len(experience.get("cards", [])),
            "approvedExperienceCardId": experience_id,
        },
    }


pairs, candidate_pair_count, candidate_target_file_count = discover_pairs()
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
evaluated_target_file_count = len({result["targetFile"] for result in results})
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
total_unique_non_targets_without_current_support = sum(
    result["lift"]["uniqueNonTargetWithoutCurrentSupport"] for result in results
)
total_unique_non_targets_with_current_support = max(
    0,
    total_unique_non_targets - total_unique_non_targets_without_current_support,
)
total_unique_target_hits = sum(result["lift"]["memoryUniqueTargetHitDelta"] for result in results)
total_unique_target_hits_without_current_support = sum(
    result["lift"]["uniqueTargetHitWithoutCurrentSupport"] for result in results
)
total_graph_supported_memory_target_hits = sum(
    result["signalComparison"]["memoryTargetHitsWithGraphSupportUpperBound"] for result in results
)
total_semantic_supported_memory_target_hits = sum(
    result["signalComparison"]["memoryTargetHitsWithSemanticSupportUpperBound"] for result in results
)
total_graph_or_semantic_supported_unique_targets = sum(
    result["signalComparison"]["memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound"]
    for result in results
)
total_memory_targets_without_graph_or_semantic = sum(
    result["signalComparison"]["memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound"]
    for result in results
)
total_graph_edge_removed_targets = sum(
    result["signalComparison"]["graphEdgeAblationRemovedTargetHitCount"] for result in results
)
memory_pack_changed_pairs = sum(1 for result in results if result["lift"]["packChangedByMemory"])
memory_pack_target_gain_pairs = sum(
    1 for result in results if result["lift"]["packAddedTargetCount"] > 0
)
total_pack_added_files = sum(result["lift"]["packAddedFileCount"] for result in results)
total_pack_removed_files = sum(result["lift"]["packRemovedFileCount"] for result in results)
total_pack_added_targets = sum(result["lift"]["packAddedTargetCount"] for result in results)
total_pack_added_non_targets = sum(
    result["lift"]["packAddedNonTargetCount"] for result in results
)
total_signal_only_non_targets = sum(
    result["lift"]["signalOnlyNonTargetCount"] for result in results
)


def merge_count_maps(results, field):
    merged = {}
    for result in results:
        for key, value in result["lift"].get(field, {}).items():
            merged[key] = merged.get(key, 0) + int(value or 0)
    return dict(sorted(merged.items()))


total_unique_target_hit_current_support_signal_counts = merge_count_maps(
    results,
    "uniqueTargetHitCurrentSupportSignalCounts",
)
total_unique_non_target_current_support_signal_counts = merge_count_maps(
    results,
    "uniqueNonTargetCurrentSupportSignalCounts",
)


def dominant_signals(counts):
    return [
        key
        for key, _ in sorted(
            counts.items(),
            key=lambda item: (-int(item[1] or 0), item[0]),
        )[:3]
    ]


supported_memory_noise_dominant_signals = dominant_signals(
    total_unique_non_target_current_support_signal_counts
)
weak_supported_memory_noise = any(
    total_unique_non_target_current_support_signal_counts.get(signal, 0) > 0
    for signal in ["lexical_expansion", "symbol"]
)
semantic_target_pairs = sum(
    1 for result in results if result["signalComparison"]["semanticSelectedTargetCount"] > 0
)
semantic_ablation_lift_pairs = sum(
    1
    for result in results
    if (result["signalComparison"]["semanticAblationRecallDeltaAt10"] or 0) > 0
)
total_after_seconds = round(
    sum(result["runtimeSeconds"].get("afterEval", 0.0) for result in results), 2
)
larger_pair_validation_target_met = (
    requested_pairs >= 5
    and evaluated >= 5
    and evaluated_target_file_count >= 5
)

unsupported_memory_precision_needs_work = (
    (total_pack_added_non_targets > 0 and total_unique_non_targets_without_current_support > 0)
    or total_unique_target_hits_without_current_support > 0
)
supported_memory_noise_needs_review = (
    total_pack_added_non_targets > 0
    and total_unique_non_targets_with_current_support > 0
    and total_unique_non_targets_without_current_support == 0
)
if unsupported_memory_precision_needs_work:
    recommended_next_r_and_d = [
        "demote_uncorroborated_memory_candidates",
        "test_memory_candidate_corroboration_policy",
    ]
elif not larger_pair_validation_target_met:
    recommended_next_r_and_d = ["increase_real_corpus_pair_count"]
elif supported_memory_noise_needs_review:
    if weak_supported_memory_noise:
        recommended_next_r_and_d = ["tune_memory_weight_against_supported_signal_pressure"]
    else:
        recommended_next_r_and_d = ["inspect_remaining_strong_signal_memory_overlap"]
else:
    recommended_next_r_and_d = ["expand_repository_diversity"]
if total_unique_non_targets > 0:
    if total_pack_added_non_targets > 0:
        recommended_next_r_and_d.append("compare_memory_noise_against_current_signal_roles")
    else:
        recommended_next_r_and_d.append("track_signal_only_memory_overlap")
if semantic_enabled:
    recommended_next_r_and_d.append("compare_against_lexical_graph_semantic_ablations")
if "measure_real_agent_outcome_lift" not in recommended_next_r_and_d:
    recommended_next_r_and_d.append("measure_real_agent_outcome_lift")

status = "measured" if evaluated else "insufficient_evidence"
payload = {
    "schemaVersion": "ctxhelm-memory-generalization-measurement-v2",
    "status": status,
    "workflowKind": "multi-pair-experience-memory-generalization",
    "repo": {
        "label": repo.name,
        "pathStored": False,
        "headShaPrefix": git("rev-parse", "HEAD").strip()[:12],
        "scanCommits": scan_commits,
        "requestedPairs": requested_pairs,
        "candidatePairCount": candidate_pair_count,
        "candidateTargetFileCount": candidate_target_file_count,
        "discoveredPairs": len(pairs),
        "evaluatedPairs": evaluated,
        "evaluatedTargetFileCount": evaluated_target_file_count,
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
        "memoryUniqueNonTargetWithCurrentSupportCount": total_unique_non_targets_with_current_support,
        "memoryUniqueTargetHitWithoutCurrentSupportCount": total_unique_target_hits_without_current_support,
        "memoryUniqueNonTargetWithoutCurrentSupportCount": total_unique_non_targets_without_current_support,
        "memoryUniqueTargetHitCurrentSupportSignalCounts": total_unique_target_hit_current_support_signal_counts,
        "memoryUniqueNonTargetCurrentSupportSignalCounts": total_unique_non_target_current_support_signal_counts,
        "memoryTargetHitsWithGraphSupportUpperBound": total_graph_supported_memory_target_hits,
        "memoryTargetHitsWithSemanticSupportUpperBound": total_semantic_supported_memory_target_hits,
        "memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound": total_graph_or_semantic_supported_unique_targets,
        "memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound": total_memory_targets_without_graph_or_semantic,
        "graphEdgeAblationRemovedTargetHitCount": total_graph_edge_removed_targets,
        "memoryPackChangedPairs": memory_pack_changed_pairs,
        "memoryPackTargetGainPairs": memory_pack_target_gain_pairs,
        "memoryPackAddedFileCount": total_pack_added_files,
        "memoryPackRemovedFileCount": total_pack_removed_files,
        "memoryPackAddedTargetCount": total_pack_added_targets,
        "memoryPackAddedNonTargetCount": total_pack_added_non_targets,
        "memorySignalOnlyNonTargetCount": total_signal_only_non_targets,
        "semanticSelectedTargetPairs": semantic_target_pairs,
        "semanticAblationLiftPairs": semantic_ablation_lift_pairs,
        "afterEvalRuntimeSecondsTotal": total_after_seconds,
    },
    "interpretation": {
        "generalizationProven": unique_lift_pairs > 1,
        "singlePairLiftObserved": unique_lift_pairs == 1,
        "precisionNeedsWork": total_pack_added_non_targets > total_pack_added_targets,
        "unsupportedMemoryPrecisionNeedsWork": unsupported_memory_precision_needs_work,
        "supportedMemoryNoiseNeedsReview": supported_memory_noise_needs_review,
        "weakSupportedMemoryNoiseNeedsTuning": weak_supported_memory_noise,
        "supportedMemoryNoiseDominantSignals": supported_memory_noise_dominant_signals,
        "memoryNeedsCorroboration": total_memory_targets_without_graph_or_semantic > 0
        or total_pack_added_non_targets > 0,
        "signalOnlyMemoryOverlapObserved": total_signal_only_non_targets > 0,
        "semanticMeasured": semantic_enabled,
        "semanticUsefulForMemoryTasks": semantic_target_pairs > 0 or semantic_ablation_lift_pairs > 0,
        "lexicalStillStrong": lexical_covered_pairs > 0,
        "pairDiversityMeasured": candidate_target_file_count > 1 or evaluated_target_file_count > 1,
        "largerPairValidationTargetMet": larger_pair_validation_target_met,
        "recommendedNextRAndD": recommended_next_r_and_d,
    },
    "semantic": {
        "enabled": semantic_enabled,
        "provider": semantic_provider if semantic_enabled else None,
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
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
