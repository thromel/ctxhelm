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

write_audit_report() {
  local report_path="$1"
  local archive="$2"
  local status="$3"
  local member_count="$4"
  local detail="$5"
  if [[ -z "$report_path" ]]; then
    return
  fi
  python3 - "$report_path" "$archive" "$status" "$member_count" "$detail" <<'PY'
import json
import pathlib
import sys

report_path, archive, status, member_count, detail = sys.argv[1:]
payload = {
    "schemaVersion": 1,
    "archiveName": pathlib.Path(archive).name,
    "status": status,
    "memberCount": int(member_count or "0"),
    "sourceFree": status == "passed",
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "checks": [
        "archive members do not include local ctxpack state",
        "archive members do not include git internals or target debris",
        "archive members do not include secret-looking paths",
        "text payloads do not include machine-local paths or secret-looking values",
    ],
    "detail": detail,
}
path = pathlib.Path(report_path)
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

audit_archive() {
  local archive="$1"
  local extract_dir
  extract_dir="$(mktemp -d)"

  cleanup_archive() {
    rm -rf "${extract_dir}"
  }
  trap cleanup_archive RETURN

  if [[ ! -f "${archive}" ]]; then
    write_audit_report "${CTXPACK_AUDIT_REPORT:-}" "${archive}" "failed" 0 "archive not found"
    echo "archive not found: ${archive}" >&2
    return 1
  fi

  case "${archive}" in
    *.tar.gz|*.tgz) ;;
    *)
      write_audit_report "${CTXPACK_AUDIT_REPORT:-}" "${archive}" "failed" 0 "unsupported archive type"
      echo "unsupported archive type: ${archive}" >&2
      return 1
      ;;
  esac

  local members
  members="$(tar -tf "${archive}")"
  local member_count
  member_count="$(printf '%s\n' "${members}" | sed '/^$/d' | wc -l | tr -d ' ')"
  while IFS= read -r member; do
    [[ -z "${member}" ]] && continue
    if [[ "${member}" =~ ${FORBIDDEN_PATH_RE} ]]; then
      write_audit_report "${CTXPACK_AUDIT_REPORT:-}" "${archive}" "failed" "${member_count}" "forbidden archive member: ${member}"
      echo "forbidden archive member in ${archive}: ${member}" >&2
      return 1
    fi
  done <<< "${members}"

  tar -xzf "${archive}" -C "${extract_dir}"
  if LC_ALL=C grep -I -R -l -E "${FORBIDDEN_TEXT_RE}" "${extract_dir}" >/tmp/ctxpack-audit-matches.$$ 2>/dev/null; then
    local first_match=""
    while IFS= read -r path; do
      rel_path="${path#"${extract_dir}/"}"
      if [[ -z "$first_match" ]]; then
        first_match="$rel_path"
      fi
      echo "forbidden text pattern in ${archive}: ${rel_path}" >&2
    done </tmp/ctxpack-audit-matches.$$
    rm -f /tmp/ctxpack-audit-matches.$$
    write_audit_report "${CTXPACK_AUDIT_REPORT:-}" "${archive}" "failed" "${member_count}" "forbidden text pattern: ${first_match}"
    return 1
  fi
  rm -f /tmp/ctxpack-audit-matches.$$

  write_audit_report "${CTXPACK_AUDIT_REPORT:-}" "${archive}" "passed" "${member_count}" "artifact audit passed"
  echo "audit passed: ${archive}"
}

for archive in "$@"; do
  audit_archive "${archive}"
done
