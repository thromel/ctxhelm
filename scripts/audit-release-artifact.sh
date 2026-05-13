#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -eq 0 ]]; then
  echo "usage: $0 <archive.tar.gz> [archive.tar.gz ...]" >&2
  exit 64
fi

# Forbidden path families include .ctxpack, traces.jsonl, request logs,
# temp homes, .env, key/token-looking paths, target/, .git/, and /Users/.
FORBIDDEN_PATH_RE='(^|/)(\.ctxpack|\.git|target|cache|tmp)(/|$)|traces\.jsonl|request[-_]?logs?|requests\.jsonl|\.env|id_rsa|id_ed25519|token|secret|/Users/'
FORBIDDEN_TEXT_RE='/Users/|/private/var/folders|/tmp/ctxpack|CTXPACK_HOME=[^[:space:]]+|AWS_[A-Z0-9_]+=|GH_TOKEN=|GITHUB_TOKEN=|API_KEY=|BEGIN [A-Z ]*PRIVATE KEY|id_rsa|id_ed25519|\.env|request[-_]?logs?|requests\.jsonl|token[[:alnum:]_.-]*=|secret[[:alnum:]_.-]*='

audit_archive() {
  local archive="$1"
  local extract_dir
  extract_dir="$(mktemp -d)"

  cleanup_archive() {
    rm -rf "${extract_dir}"
  }
  trap cleanup_archive RETURN

  if [[ ! -f "${archive}" ]]; then
    echo "archive not found: ${archive}" >&2
    return 1
  fi

  case "${archive}" in
    *.tar.gz|*.tgz) ;;
    *)
      echo "unsupported archive type: ${archive}" >&2
      return 1
      ;;
  esac

  local members
  members="$(tar -tf "${archive}")"
  while IFS= read -r member; do
    [[ -z "${member}" ]] && continue
    if [[ "${member}" =~ ${FORBIDDEN_PATH_RE} ]]; then
      echo "forbidden archive member in ${archive}: ${member}" >&2
      return 1
    fi
  done <<< "${members}"

  tar -xzf "${archive}" -C "${extract_dir}"
  if LC_ALL=C grep -I -R -l -E "${FORBIDDEN_TEXT_RE}" "${extract_dir}" >/tmp/ctxpack-audit-matches.$$ 2>/dev/null; then
    while IFS= read -r path; do
      rel_path="${path#"${extract_dir}/"}"
      echo "forbidden text pattern in ${archive}: ${rel_path}" >&2
    done </tmp/ctxpack-audit-matches.$$
    rm -f /tmp/ctxpack-audit-matches.$$
    return 1
  fi
  rm -f /tmp/ctxpack-audit-matches.$$

  echo "audit passed: ${archive}"
}

for archive in "$@"; do
  audit_archive "${archive}"
done
