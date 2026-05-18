#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

bash -n "$repo_root/scripts/release-candidate-status.sh"
bash -n "$repo_root/scripts/release-candidate-rollback.sh"

for status in ready deferred blocked; do
  out="$work_dir/${status}.json"
  bash "$repo_root/scripts/release-candidate-status.sh" create --output "$out" --status "$status" >/dev/null
  bash "$repo_root/scripts/release-candidate-status.sh" validate --input "$out" >/dev/null
done

candidate_dir="$work_dir/candidate"
mkdir -p "$candidate_dir"
touch "$candidate_dir/.ctxpack-release-candidate"
printf 'artifact placeholder\n' >"$candidate_dir/ctxpack-v1.1.0-aarch64-apple-darwin.tar.gz"
metadata="$work_dir/release-metadata.json"
previous="$work_dir/previous-release-metadata.json"
printf '{"version":"candidate"}\n' >"$metadata"
printf '{"version":"previous"}\n' >"$previous"

bash "$repo_root/scripts/release-candidate-rollback.sh" \
  --candidate-dir "$candidate_dir" \
  --metadata "$metadata" \
  --previous-metadata "$previous" >/dev/null

if [[ -e "$candidate_dir" ]]; then
  echo "release governance smoke failed: candidate directory still exists" >&2
  exit 1
fi
if ! grep -F -- '"previous"' "$metadata" >/dev/null; then
  echo "release governance smoke failed: previous metadata was not restored" >&2
  exit 1
fi

for file in \
  "$repo_root/docs/release-governance.md" \
  "$repo_root/packaging/release/release-checklist.md"
do
  if [[ ! -f "$file" ]]; then
    echo "release governance smoke failed: missing ${file#"$repo_root"/}" >&2
    exit 1
  fi
  for required in "deterministic protocol proof" "optional real-client proof" "ready" "deferred" "blocked" "rollback" "Cursor" "OpenCode"; do
    grep -F -- "$required" "$file" >/dev/null || {
      echo "release governance smoke failed: missing '$required' in ${file#"$repo_root"/}" >&2
      exit 1
    }
  done
done

echo "release governance smoke passed"

