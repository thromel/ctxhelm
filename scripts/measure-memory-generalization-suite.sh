#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: measure-memory-generalization-suite.sh --repo PATH [--repo PATH ...] [--pairs N] [--scan-commits N] [--semantic] [--semantic-provider PROVIDER] [--output PATH]

Runs `scripts/measure-memory-generalization.sh` across multiple local git
repositories and writes a source-free aggregate report. The suite stores repo
labels, commit prefixes, path labels, hashes, counters, and booleans only. It
does not store raw repo paths, raw commit subjects, source snippets, terminal
logs, model transcripts, or raw MCP traffic.
EOF
}

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
pairs="1"
scan_commits="120"
output_path="${CTXHELM_MEMORY_GENERALIZATION_SUITE_REPORT:-}"
semantic_enabled="false"
semantic_provider="local_hash"
repos=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repos+=("${2:-}")
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

if [[ "${#repos[@]}" -eq 0 ]]; then
  usage
  exit 64
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
single_repo_script="$script_dir/measure-memory-generalization.sh"
if [[ ! -x "$single_repo_script" ]]; then
  echo "missing executable single-repo harness: $single_repo_script" >&2
  exit 66
fi

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo_args_json="$work_dir/repos.json"
python3 - "$repo_args_json" "${repos[@]}" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
path.write_text(json.dumps(sys.argv[2:]) + "\n")
PY

python3 - "$ctxhelm_bin" "$single_repo_script" "$repo_args_json" "$pairs" "$scan_commits" "$output_path" "$work_dir" "$semantic_enabled" "$semantic_provider" <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time

ctxhelm_bin = sys.argv[1]
single_repo_script = pathlib.Path(sys.argv[2]).resolve()
repo_args = json.loads(pathlib.Path(sys.argv[3]).read_text())
pairs = sys.argv[4]
scan_commits = sys.argv[5]
output_path = sys.argv[6]
work = pathlib.Path(sys.argv[7]).resolve()
semantic_enabled = sys.argv[8] == "true"
semantic_provider = sys.argv[9]


def sha256(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def run_repo(repo_path, index):
    repo = pathlib.Path(repo_path).resolve()
    report_path = work / f"repo-{index:02d}.json"
    started = time.monotonic()
    env = os.environ.copy()
    env["CTXHELM_BIN"] = ctxhelm_bin
    args = [
        str(single_repo_script),
        "--repo",
        str(repo),
        "--pairs",
        pairs,
        "--scan-commits",
        scan_commits,
        "--output",
        str(report_path),
    ]
    if semantic_enabled:
        args.extend(["--semantic", "--semantic-provider", semantic_provider])
    result = subprocess.run(
        args,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    runtime = round(time.monotonic() - started, 2)
    if result.returncode != 0:
        return None, {
            "repoLabel": repo.name,
            "repoPathStored": False,
            "exitCode": result.returncode,
            "stderrSha256": sha256(result.stderr),
            "stdoutSha256": sha256(result.stdout),
            "runtimeSeconds": runtime,
        }
    report = json.loads(report_path.read_text())
    report["suiteRuntimeSeconds"] = runtime
    return report, None


reports = []
errors = []
for index, repo in enumerate(repo_args, start=1):
    report, error = run_repo(repo, index)
    if report is not None:
        reports.append(report)
    if error is not None:
        errors.append(error)


def aggregate_value(key):
    return sum(report.get("aggregate", {}).get(key, 0) for report in reports)


evaluated_repos = len(reports)
evaluated_pairs = sum(report.get("repo", {}).get("evaluatedPairs", 0) for report in reports)
discovered_pairs = sum(report.get("repo", {}).get("discoveredPairs", 0) for report in reports)
repo_summaries = []
for report in reports:
    repo = report.get("repo", {})
    aggregate = report.get("aggregate", {})
    interpretation = report.get("interpretation", {})
    repo_summaries.append(
        {
            "label": repo.get("label"),
            "pathStored": False,
            "headShaPrefix": repo.get("headShaPrefix"),
            "requestedPairs": repo.get("requestedPairs"),
            "discoveredPairs": repo.get("discoveredPairs"),
            "evaluatedPairs": repo.get("evaluatedPairs"),
            "errorCount": repo.get("errorCount"),
            "aggregate": aggregate,
            "interpretation": {
                "singlePairLiftObserved": interpretation.get("singlePairLiftObserved"),
                "generalizationProven": interpretation.get("generalizationProven"),
                "precisionNeedsWork": interpretation.get("precisionNeedsWork"),
                "memoryNeedsCorroboration": interpretation.get("memoryNeedsCorroboration"),
                "semanticMeasured": interpretation.get("semanticMeasured"),
                "semanticUsefulForMemoryTasks": interpretation.get("semanticUsefulForMemoryTasks"),
                "lexicalStillStrong": interpretation.get("lexicalStillStrong"),
            },
            "suiteRuntimeSeconds": report.get("suiteRuntimeSeconds"),
        }
    )

memory_unique_lift_pairs = aggregate_value("memoryUniqueLiftPairs")
memory_unique_target_hits = aggregate_value("memoryUniqueTargetHitCount")
memory_unique_non_targets = aggregate_value("memoryUniqueNonTargetCount")
memory_unique_target_hits_without_current_support = aggregate_value(
    "memoryUniqueTargetHitWithoutCurrentSupportCount"
)
memory_unique_non_targets_without_current_support = aggregate_value(
    "memoryUniqueNonTargetWithoutCurrentSupportCount"
)
memory_target_hit_pairs = aggregate_value("memoryTargetHitPairs")
memory_candidate_pairs = aggregate_value("memoryCandidatePairs")
combined_recovered_pairs = aggregate_value("combinedRecoveredPairs")
lexical_covered_pairs = aggregate_value("lexicalCoveredPairs")
graph_supported_memory_target_hits = aggregate_value("memoryTargetHitsWithGraphSupportUpperBound")
semantic_supported_memory_target_hits = aggregate_value("memoryTargetHitsWithSemanticSupportUpperBound")
graph_or_semantic_supported_unique_targets = aggregate_value(
    "memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound"
)
memory_targets_without_graph_or_semantic = aggregate_value(
    "memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound"
)
graph_edge_removed_target_hits = aggregate_value("graphEdgeAblationRemovedTargetHitCount")
semantic_selected_target_pairs = aggregate_value("semanticSelectedTargetPairs")
semantic_ablation_lift_pairs = aggregate_value("semanticAblationLiftPairs")

payload = {
    "schemaVersion": "ctxhelm-memory-generalization-suite-v2",
    "status": "measured" if evaluated_pairs else "insufficient_evidence",
    "workflowKind": "multi-repo-experience-memory-generalization",
    "suite": {
        "requestedRepositoryCount": len(repo_args),
        "evaluatedRepositoryCount": evaluated_repos,
        "errorRepositoryCount": len(errors),
        "requestedPairsPerRepo": int(pairs),
        "scanCommitsPerRepo": int(scan_commits),
        "discoveredPairs": discovered_pairs,
        "evaluatedPairs": evaluated_pairs,
    },
    "aggregate": {
        "memoryCandidatePairs": memory_candidate_pairs,
        "memoryTargetHitPairs": memory_target_hit_pairs,
        "memoryUniqueLiftPairs": memory_unique_lift_pairs,
        "combinedRecoveredPairs": combined_recovered_pairs,
        "lexicalCoveredPairs": lexical_covered_pairs,
        "memoryUniqueTargetHitCount": memory_unique_target_hits,
        "memoryUniqueNonTargetCount": memory_unique_non_targets,
        "memoryUniqueTargetHitWithoutCurrentSupportCount": memory_unique_target_hits_without_current_support,
        "memoryUniqueNonTargetWithoutCurrentSupportCount": memory_unique_non_targets_without_current_support,
        "memoryTargetHitsWithGraphSupportUpperBound": graph_supported_memory_target_hits,
        "memoryTargetHitsWithSemanticSupportUpperBound": semantic_supported_memory_target_hits,
        "memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound": graph_or_semantic_supported_unique_targets,
        "memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound": memory_targets_without_graph_or_semantic,
        "graphEdgeAblationRemovedTargetHitCount": graph_edge_removed_target_hits,
        "semanticSelectedTargetPairs": semantic_selected_target_pairs,
        "semanticAblationLiftPairs": semantic_ablation_lift_pairs,
        "memoryUniqueNonTargetPerUniqueTarget": (
            round(memory_unique_non_targets / memory_unique_target_hits, 4)
            if memory_unique_target_hits
            else None
        ),
    },
    "interpretation": {
        "multiRepoMeasured": evaluated_repos > 1,
        "generalizationProven": evaluated_repos > 1 and memory_unique_lift_pairs > 1,
        "precisionNeedsWork": memory_unique_non_targets > 0
        or memory_target_hit_pairs < memory_candidate_pairs,
        "unsupportedMemoryPrecisionNeedsWork": memory_unique_non_targets_without_current_support > 0
        or memory_unique_target_hits_without_current_support > 0,
        "memoryNeedsCorroboration": memory_targets_without_graph_or_semantic > 0
        or memory_unique_non_targets > 0,
        "semanticMeasured": semantic_enabled,
        "semanticUsefulForMemoryTasks": semantic_selected_target_pairs > 0
        or semantic_ablation_lift_pairs > 0,
        "graphCorroborationMeasured": graph_supported_memory_target_hits > 0
        or graph_edge_removed_target_hits > 0,
        "lexicalStillStrong": lexical_covered_pairs > 0,
        "recommendedNextRAndD": [
            "increase_pairs_per_repo",
            "demote_memory_without_target_or_correlated_context_support",
            "test_memory_candidate_corroboration_policy",
            "measure_real_agent_outcome_lift",
        ],
    },
    "semantic": {
        "enabled": semantic_enabled,
        "provider": semantic_provider if semantic_enabled else None,
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
    },
    "repositories": repo_summaries,
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

if evaluated_pairs == 0:
    raise SystemExit("no repositories produced measured repeated-file memory pairs")
PY

echo "memory generalization suite measurement complete"
