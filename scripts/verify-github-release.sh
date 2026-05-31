#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: verify-github-release.sh --tag TAG --target COMMIT --assets-dir DIR [--repo OWNER/REPO] [--release-json PATH]

Verifies source-free GitHub release metadata against local release artifacts.
The script reads release metadata through gh unless --release-json is provided.
It does not create tags, publish releases, upload assets, or mutate repository
state.
EOF
}

tag=""
target=""
assets_dir=""
repo=""
release_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      tag="${2:-}"
      shift 2
      ;;
    --target)
      target="${2:-}"
      shift 2
      ;;
    --assets-dir)
      assets_dir="${2:-}"
      shift 2
      ;;
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --release-json)
      release_json="${2:-}"
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

if [[ -z "$tag" || -z "$target" || -z "$assets_dir" ]]; then
  usage
  exit 64
fi
if [[ ! -d "$assets_dir" ]]; then
  echo "assets directory not found: $assets_dir" >&2
  exit 66
fi

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

metadata_path="$release_json"
if [[ -z "$metadata_path" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    echo "gh is required unless --release-json is provided" >&2
    exit 69
  fi
  metadata_path="$work_dir/release.json"
  gh_args=(release view "$tag" --json tagName,targetCommitish,name,isDraft,isPrerelease,url,publishedAt,assets)
  if [[ -n "$repo" ]]; then
    gh_args+=(--repo "$repo")
  fi
  gh "${gh_args[@]}" >"$metadata_path"
fi
if [[ ! -f "$metadata_path" ]]; then
  echo "release metadata not found: $metadata_path" >&2
  exit 66
fi

python3 - "$metadata_path" "$tag" "$target" "$assets_dir" <<'PY'
import hashlib
import json
import pathlib
import sys

metadata_path = pathlib.Path(sys.argv[1])
expected_tag = sys.argv[2]
expected_target = sys.argv[3]
assets_dir = pathlib.Path(sys.argv[4])

payload = json.loads(metadata_path.read_text())
if payload.get("tagName") != expected_tag:
    raise SystemExit(f"release tag mismatch: {payload.get('tagName')} != {expected_tag}")
if payload.get("targetCommitish") != expected_target:
    raise SystemExit(
        f"release target mismatch: {payload.get('targetCommitish')} != {expected_target}"
    )
if payload.get("isDraft") is not False:
    raise SystemExit("release must not be a draft")
if payload.get("isPrerelease") is not False:
    raise SystemExit("release must not be a prerelease")
if not payload.get("url") or not payload.get("publishedAt"):
    raise SystemExit("release url and publishedAt are required")

asset_digests = {}
for asset in payload.get("assets", []):
    name = asset.get("name")
    digest = asset.get("digest")
    state = asset.get("state")
    if state != "uploaded":
        raise SystemExit(f"release asset is not uploaded: {name}")
    if not name or not digest:
        raise SystemExit("release asset name/digest missing")
    if not digest.startswith("sha256:"):
        raise SystemExit(f"release asset digest is not sha256: {name}")
    asset_digests[name] = digest.removeprefix("sha256:")

local_files = sorted(path for path in assets_dir.iterdir() if path.is_file())
if not local_files:
    raise SystemExit("assets directory has no files")
for path in local_files:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    release_digest = asset_digests.get(path.name)
    if release_digest is None:
        raise SystemExit(f"release missing asset: {path.name}")
    if release_digest != digest:
        raise SystemExit(f"release asset digest mismatch: {path.name}")

print(
    json.dumps(
        {
            "tag": expected_tag,
            "target": expected_target,
            "url": payload["url"],
            "assetCount": len(local_files),
            "sourceFree": True,
        },
        sort_keys=True,
    )
)
PY
