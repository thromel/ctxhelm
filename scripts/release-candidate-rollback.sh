#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: release-candidate-rollback.sh --candidate-dir PATH [--metadata PATH --previous-metadata PATH]

Removes a local release candidate artifact directory and optionally restores
previous release metadata. The candidate directory must contain the marker file
.ctxhelm-release-candidate. The script refuses dangerous paths and does not
touch repository source files.
EOF
}

candidate_dir=""
metadata_path=""
previous_metadata_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --candidate-dir)
      candidate_dir="${2:-}"
      shift 2
      ;;
    --metadata)
      metadata_path="${2:-}"
      shift 2
      ;;
    --previous-metadata)
      previous_metadata_path="${2:-}"
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

if [[ -z "$candidate_dir" ]]; then
  usage
  exit 64
fi
if [[ ! -d "$candidate_dir" ]]; then
  echo "candidate directory not found: $candidate_dir" >&2
  exit 66
fi
if [[ ! -f "$candidate_dir/.ctxhelm-release-candidate" ]]; then
  echo "refusing rollback: missing .ctxhelm-release-candidate marker in $candidate_dir" >&2
  exit 65
fi

resolved_candidate="$(cd "$candidate_dir" && pwd -P)"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
home_dir="$(cd ~ && pwd -P)"

case "$resolved_candidate" in
  "/"|"$repo_root"|"$home_dir")
    echo "refusing rollback of dangerous path: $resolved_candidate" >&2
    exit 65
    ;;
esac

if [[ "$resolved_candidate" == "$repo_root"/* && "$resolved_candidate" != "$repo_root/dist"* ]]; then
  echo "refusing rollback inside repository source tree: $resolved_candidate" >&2
  exit 65
fi

if [[ -n "$metadata_path" || -n "$previous_metadata_path" ]]; then
  if [[ -z "$metadata_path" || -z "$previous_metadata_path" ]]; then
    echo "--metadata and --previous-metadata must be provided together" >&2
    exit 64
  fi
  if [[ ! -f "$previous_metadata_path" ]]; then
    echo "previous metadata not found: $previous_metadata_path" >&2
    exit 66
  fi
  mkdir -p "$(dirname "$metadata_path")"
  cp "$previous_metadata_path" "$metadata_path"
fi

rm -rf "$resolved_candidate"
echo "rolled back release candidate artifacts: $resolved_candidate"

