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
bash -n "$repo_root/scripts/verify-github-release.sh"
bash -n "$repo_root/scripts/check-public-release-freshness.sh"
bash -n "$repo_root/scripts/verify-public-archive-install.sh"
bash -n "$repo_root/scripts/smoke-public-real-clients.sh"

if bash "$repo_root/scripts/release-candidate-status.sh" create --output "$work_dir/ready-without-proof.json" --status ready >/dev/null 2>&1; then
  echo "ready release candidate status unexpectedly succeeded without --proof-summary" >&2
  exit 1
fi

for status in ready deferred blocked; do
  out="$work_dir/${status}.json"
  if [[ "$status" == "ready" ]]; then
    proof_summary="$work_dir/release-proof-summary.json"
    cat >"$proof_summary" <<'JSON'
{
  "binaryIdentity": {
    "sha256": "binary-sha256",
    "source": "archive"
  },
  "optionalProofs": {
    "cleanColdFixtureProductProof": "passed",
    "cleanColdFixtureRequired": true,
    "resourceBackedGapSummaryContract": "checked"
  },
  "releaseArchive": {
    "name": "ctxhelm-v1.1.9-aarch64-apple-darwin.tar.gz",
    "sha256": "archive-sha256"
  },
  "requiredChecks": [
    {"name": "cargo test --workspace", "status": "passed"}
  ],
  "status": "passed"
}
JSON
    bash "$repo_root/scripts/release-candidate-status.sh" create --output "$out" --status "$status" --proof-summary "$proof_summary" >/dev/null
  else
    bash "$repo_root/scripts/release-candidate-status.sh" create --output "$out" --status "$status" >/dev/null
  fi
  bash "$repo_root/scripts/release-candidate-status.sh" validate --input "$out" >/dev/null
done

assets_dir="$work_dir/assets"
mkdir -p "$assets_dir"
printf 'archive\n' >"$assets_dir/ctxhelm-v1.1.9-test.tar.gz"
printf 'manifest\n' >"$assets_dir/ctxhelm-v1.1.9-test.manifest.json"
release_json="$work_dir/github-release.json"
python3 - "$release_json" "$assets_dir" <<'PY'
import hashlib
import json
import pathlib
import sys

out, assets_dir = sys.argv[1:]
assets = pathlib.Path(assets_dir)
archive_digest = hashlib.sha256((assets / "ctxhelm-v1.1.9-test.tar.gz").read_bytes()).hexdigest()
manifest_digest = hashlib.sha256((assets / "ctxhelm-v1.1.9-test.manifest.json").read_bytes()).hexdigest()
payload = {
    "assets": [
        {
            "digest": f"sha256:{archive_digest}",
            "name": "ctxhelm-v1.1.9-test.tar.gz",
            "state": "uploaded",
        },
        {
            "digest": f"sha256:{manifest_digest}",
            "name": "ctxhelm-v1.1.9-test.manifest.json",
            "state": "uploaded",
        },
    ],
    "isDraft": False,
    "isPrerelease": False,
    "publishedAt": "2026-06-01T00:00:00Z",
    "tagName": "v1.1.9",
    "targetCommitish": "abc123",
    "url": "https://github.com/thromel/ctxhelm/releases/tag/v1.1.9",
}
pathlib.Path(out).write_text(json.dumps(payload, sort_keys=True) + "\n")
PY
bash "$repo_root/scripts/verify-github-release.sh" \
  --tag v1.1.9 \
  --target abc123 \
  --assets-dir "$assets_dir" \
  --release-json "$release_json" >/dev/null

bash "$repo_root/scripts/check-public-release-freshness.sh" \
  --tag v1.1.9 \
  --current-commit def456 \
  --release-json "$release_json" \
  --output "$work_dir/release-freshness.json" >/dev/null
python3 - "$work_dir/release-freshness.json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert payload["status"] == "outdated"
assert payload["releaseTargetCommit"] == "abc123"
assert payload["currentCommit"] == "def456"
assert payload["sourceFree"] is True
PY

candidate_dir="$work_dir/candidate"
mkdir -p "$candidate_dir"
touch "$candidate_dir/.ctxhelm-release-candidate"
printf 'artifact placeholder\n' >"$candidate_dir/ctxhelm-v1.1.9-aarch64-apple-darwin.tar.gz"
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
