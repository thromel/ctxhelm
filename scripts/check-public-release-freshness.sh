#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: check-public-release-freshness.sh --tag TAG [--repo OWNER/REPO] [--current-commit COMMIT] [--release-json PATH] [--output PATH] [--require-current]

Checks whether the public GitHub release tag points at the current commit and
writes source-free freshness metadata. The script reads release metadata through
gh unless --release-json is provided. It does not publish, tag, upload, install,
or mutate repository state.
EOF
}

repo="thromel/ctxpack"
tag=""
current_commit=""
release_json=""
output_path=""
require_current=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --tag)
      tag="${2:-}"
      shift 2
      ;;
    --current-commit)
      current_commit="${2:-}"
      shift 2
      ;;
    --release-json)
      release_json="${2:-}"
      shift 2
      ;;
    --output)
      output_path="${2:-}"
      shift 2
      ;;
    --require-current)
      require_current=1
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

if [[ -z "$tag" || -z "$repo" ]]; then
  usage
  exit 64
fi

if [[ -z "$current_commit" ]]; then
  current_commit="$(git rev-parse HEAD)"
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
  gh release view "$tag" \
    --repo "$repo" \
    --json tagName,targetCommitish,name,isDraft,isPrerelease,url,publishedAt \
    >"$metadata_path"
fi

if [[ ! -f "$metadata_path" ]]; then
  echo "release metadata not found: $metadata_path" >&2
  exit 66
fi

relation="unknown"
commits_ahead=""
if git cat-file -e "${current_commit}^{commit}" >/dev/null 2>&1 \
  && git cat-file -e "$(python3 - "$metadata_path" <<'PY'
import json
import pathlib
import sys
print(json.loads(pathlib.Path(sys.argv[1]).read_text()).get("targetCommitish", ""))
PY
)^{commit}" >/dev/null 2>&1; then
  release_target="$(python3 - "$metadata_path" <<'PY'
import json
import pathlib
import sys
print(json.loads(pathlib.Path(sys.argv[1]).read_text()).get("targetCommitish", ""))
PY
)"
  if [[ "$release_target" == "$current_commit" ]]; then
    relation="same"
    commits_ahead="0"
  elif git merge-base --is-ancestor "$release_target" "$current_commit" >/dev/null 2>&1; then
    relation="current_descends_from_release"
    commits_ahead="$(git rev-list --count "${release_target}..${current_commit}")"
  elif git merge-base --is-ancestor "$current_commit" "$release_target" >/dev/null 2>&1; then
    relation="release_descends_from_current"
    commits_ahead="0"
  else
    relation="diverged"
  fi
fi

payload="$(python3 - "$metadata_path" "$repo" "$tag" "$current_commit" "$relation" "$commits_ahead" <<'PY'
import json
import pathlib
import sys

metadata_path, repo, expected_tag, current_commit, relation, commits_ahead = sys.argv[1:]
payload = json.loads(pathlib.Path(metadata_path).read_text())
target = payload.get("targetCommitish")
if payload.get("tagName") != expected_tag:
    raise SystemExit(f"release tag mismatch: {payload.get('tagName')} != {expected_tag}")
if payload.get("isDraft") is not False:
    raise SystemExit("release must not be a draft")
if payload.get("isPrerelease") is not False:
    raise SystemExit("release must not be a prerelease")
if not target:
    raise SystemExit("release targetCommitish is required")
status = "current" if target == current_commit else "outdated"
result = {
    "schemaVersion": 1,
    "repo": repo,
    "tag": expected_tag,
    "releaseUrl": payload.get("url"),
    "publishedAt": payload.get("publishedAt"),
    "releaseTargetCommit": target,
    "currentCommit": current_commit,
    "status": status,
    "gitRelation": relation,
    "commitsAhead": int(commits_ahead) if commits_ahead else None,
    "sourceFree": True,
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "unsupportedActions": [
        "publishing",
        "tag creation",
        "asset upload",
        "global install",
        "global agent config mutation",
    ],
}
print(json.dumps(result, indent=2, sort_keys=True))
PY
)"

if [[ -n "$output_path" ]]; then
  mkdir -p "$(dirname "$output_path")"
  printf '%s\n' "$payload" >"$output_path"
else
  printf '%s\n' "$payload"
fi

if [[ "$require_current" == "1" ]]; then
  status="$(python3 -c 'import json, sys; print(json.load(sys.stdin)["status"])' <<<"$payload")"
  if [[ "$status" != "current" ]]; then
    echo "public release is not current for $tag" >&2
    exit 1
  fi
fi
