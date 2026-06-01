#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: prepare-benchmark-corpus.sh [--name NAME] [--source URL_OR_PATH] [--revision SHA_OR_REF] [--worktree PATH] [--min-commits N] [--output PATH] [--refresh]

Prepares and validates a clean detached benchmark corpus worktree for source-free
ctxhelm proof runs. By default this targets the RefactoringMiner proof fixture.

The script writes source-free health metadata: revision identity, commit count,
dirty-file count, git object/history usability, and readiness status. It does
not write source snippets, commit subjects, prompts, terminal logs, or diffs.

Without --refresh, an existing dirty or corrupt worktree fails fast. With
--refresh, the target is treated as a disposable proof fixture: it is fetched,
hard-reset, cleaned, and checked out detached at the requested revision.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
parent_dir="$(cd "$repo_root/.." && pwd -P)"

name="RefactoringMiner"
source="https://github.com/tsantalis/RefactoringMiner.git"
revision="e319af8d6b51d821b61d2f735ad211631775adfb"
worktree="$parent_dir/RefactoringMiner-ctxhelm-proof"
min_commits=20
output_path=""
refresh=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --name)
      name="${2:-}"
      shift 2
      ;;
    --source)
      source="${2:-}"
      shift 2
      ;;
    --revision)
      revision="${2:-}"
      shift 2
      ;;
    --worktree)
      worktree="${2:-}"
      shift 2
      ;;
    --min-commits)
      min_commits="${2:-}"
      shift 2
      ;;
    --output)
      output_path="${2:-}"
      shift 2
      ;;
    --refresh)
      refresh=1
      shift
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

if [[ -z "$name" || -z "$source" || -z "$revision" || -z "$worktree" || -z "$min_commits" ]]; then
  usage
  exit 64
fi
if ! [[ "$min_commits" =~ ^[0-9]+$ ]]; then
  echo "--min-commits must be a non-negative integer: $min_commits" >&2
  exit 64
fi

if [[ "$worktree" != /* ]]; then
  worktree="$(pwd -P)/$worktree"
fi

status="ready"
failure_reason=""
git_history_usable=false
object_content_usable=false
dirty_count=0
commit_count=0
head_commit=""

write_report() {
  local path="$1"
  [[ -z "$path" ]] && return 0
  mkdir -p "$(dirname "$path")"
  python3 - "$path" "$name" "$source" "$revision" "$worktree" "$status" "$failure_reason" "$git_history_usable" "$object_content_usable" "$dirty_count" "$commit_count" "$head_commit" "$refresh" "$min_commits" <<'PY'
import json
import pathlib
import sys

(
    output_path,
    name,
    source,
    requested_revision,
    worktree,
    status,
    failure_reason,
    git_history_usable,
    object_content_usable,
    dirty_count,
    commit_count,
    head_commit,
    refresh,
    min_commits,
) = sys.argv[1:]

payload = {
    "schemaVersion": 1,
    "name": name,
    "requestedRevision": requested_revision,
    "headCommit": head_commit or None,
    "sourceKind": "local_path" if source.startswith("/") else "remote_or_relative",
    "worktreePath": worktree,
    "status": status,
    "failureReason": failure_reason or None,
    "checks": {
        "worktreeExists": pathlib.Path(worktree).is_dir(),
        "gitHistoryUsable": git_history_usable == "true",
        "objectContentUsable": object_content_usable == "true",
        "dirtyCount": int(dirty_count),
        "commitCount": int(commit_count),
        "minCommits": int(min_commits),
        "refreshRequested": refresh == "1",
    },
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "omitted": [
        "source snippets",
        "commit subjects",
        "diffs",
        "terminal logs",
        "prompts",
    ],
}
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
}

fail_with_report() {
  status="blocked"
  failure_reason="$1"
  write_report "$output_path"
  echo "$failure_reason" >&2
  exit 65
}

update_dirty_count() {
  local status_output
  if ! status_output="$(git -C "$worktree" status --porcelain 2>/dev/null)"; then
    fail_with_report "benchmark corpus git status failed"
  fi
  if [[ -z "$status_output" ]]; then
    dirty_count=0
  else
    dirty_count="$(printf '%s\n' "$status_output" | wc -l | tr -d ' ')"
  fi
}

if [[ ! -d "$worktree/.git" ]]; then
  mkdir -p "$(dirname "$worktree")"
  git clone "$source" "$worktree" >/dev/null 2>&1 || fail_with_report "failed to clone benchmark corpus"
elif [[ "$refresh" != "1" ]]; then
  update_dirty_count
  if [[ "$dirty_count" != "0" ]]; then
    fail_with_report "benchmark corpus worktree is dirty; rerun with --refresh only for a disposable proof fixture"
  fi
fi

if [[ "$refresh" == "1" ]]; then
  git -C "$worktree" remote set-url origin "$source" >/dev/null 2>&1 || true
  git -C "$worktree" fetch --all --tags --prune >/dev/null 2>&1 || fail_with_report "failed to fetch benchmark corpus"
  git -C "$worktree" reset --hard >/dev/null 2>&1 || fail_with_report "failed to reset benchmark corpus"
  git -C "$worktree" clean -fdx >/dev/null 2>&1 || fail_with_report "failed to clean benchmark corpus"
fi

git -C "$worktree" cat-file -e "${revision}^{commit}" >/dev/null 2>&1 || fail_with_report "requested benchmark corpus revision is unavailable"
git -C "$worktree" checkout --detach "$revision" >/dev/null 2>&1 || fail_with_report "failed to checkout benchmark corpus revision"

update_dirty_count
if [[ "$dirty_count" != "0" ]]; then
  fail_with_report "benchmark corpus worktree is dirty after checkout"
fi

head_commit="$(git -C "$worktree" rev-parse HEAD 2>/dev/null || true)"
if [[ -z "$head_commit" ]]; then
  fail_with_report "benchmark corpus HEAD is unavailable"
fi

if git -C "$worktree" fsck --no-progress --connectivity-only >/dev/null 2>&1; then
  object_content_usable=true
else
  fail_with_report "benchmark corpus git object store failed connectivity check"
fi

commit_count="$(git -C "$worktree" rev-list --count HEAD 2>/dev/null || true)"
if [[ -z "$commit_count" || ! "$commit_count" =~ ^[0-9]+$ ]]; then
  fail_with_report "benchmark corpus history count failed"
fi
if (( commit_count < min_commits )); then
  fail_with_report "benchmark corpus has fewer commits than required"
fi
git_history_usable=true

write_report "$output_path"
printf 'benchmark corpus ready: %s %s\n' "$name" "$head_commit"
