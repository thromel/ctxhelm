#!/usr/bin/env bash
set -euo pipefail

ctxhelm_bin="${CTXHELM_BIN:-ctxhelm}"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src"
git -C "$repo" init -q
printf 'pub fn demo() {}\n' >"$repo/src/lib.rs"

CTXHELM_HOME="$home" "$ctxhelm_bin" index --repo "$repo" --store >"$work_dir/index-1.txt"
CTXHELM_HOME="$home" "$ctxhelm_bin" index --repo "$repo" --store >"$work_dir/index-2.txt"
CTXHELM_HOME="$home" "$ctxhelm_bin" storage status --repo "$repo" >"$work_dir/status.txt"
CTXHELM_HOME="$home" "$ctxhelm_bin" storage vacuum --repo "$repo" >"$work_dir/vacuum.txt"

grep -F -- "created records: 1" "$work_dir/index-1.txt" >/dev/null
grep -F -- "reused records: 1" "$work_dir/index-2.txt" >/dev/null
grep -F -- "Compatibility: \`Compatible\`" "$work_dir/status.txt" >/dev/null
grep -F -- "File records: \`1\`" "$work_dir/status.txt" >/dev/null

if grep -R -- "pub fn demo" "$home" >/dev/null 2>&1; then
  echo "storage smoke failed: source body was persisted" >&2
  exit 1
fi

echo "storage smoke passed"
