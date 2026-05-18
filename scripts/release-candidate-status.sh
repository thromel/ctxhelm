#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  release-candidate-status.sh create --output PATH --status ready|deferred|blocked [--proof-level deterministic|real-client]
  release-candidate-status.sh validate --input PATH

Writes or validates source-free release candidate status metadata. It does not
publish, tag, upload, install, or mutate global agent configuration.
EOF
}

cmd="${1:-}"
if [[ -z "$cmd" ]]; then
  usage
  exit 64
fi
shift

status=""
output_path=""
input_path=""
proof_level="deterministic"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --status)
      status="${2:-}"
      shift 2
      ;;
    --output)
      output_path="${2:-}"
      shift 2
      ;;
    --input)
      input_path="${2:-}"
      shift 2
      ;;
    --proof-level)
      proof_level="${2:-}"
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

case "$cmd" in
  create)
    if [[ -z "$output_path" || -z "$status" ]]; then
      usage
      exit 64
    fi
    case "$status" in
      ready|deferred|blocked) ;;
      *) echo "unsupported candidate status: $status" >&2; exit 64 ;;
    esac
    case "$proof_level" in
      deterministic|real-client) ;;
      *) echo "unsupported proof level: $proof_level" >&2; exit 64 ;;
    esac
    mkdir -p "$(dirname "$output_path")"
    python3 - "$output_path" "$status" "$proof_level" <<'PY'
import datetime as dt
import json
import pathlib
import sys

output_path, status, proof_level = sys.argv[1:]
payload = {
    "schemaVersion": 1,
    "package": "ctxpack",
    "version": "1.1.0",
    "status": status,
    "createdAt": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat(),
    "proofLevel": proof_level,
    "checks": {
        "workspaceTests": "required",
        "releaseDocs": "required",
        "releasePackage": "required",
        "archiveVerification": "required",
        "demoArtifacts": "required",
        "distributionMetadata": "required",
        "benchmarkProof": "optional",
        "codexRealClient": "optional",
        "claudeRealClient": "optional",
        "cursorRealClient": "not_claimed",
        "opencodeRealClient": "not_claimed",
    },
    "knownLimitations": [
        "Cursor and OpenCode real-client proof is not claimed for v1.1.0.",
        "Package-manager publication, signed installers, self-update, and hosted sync are future work.",
    ],
    "privacyStatus": {
        "localOnly": True,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
        "sourceTextLogged": False,
    },
    "unsupportedActions": [
        "publishing",
        "tag creation",
        "global agent config mutation",
        "user project test execution",
        "cloud upload",
    ],
}
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
    echo "wrote release candidate status: $output_path"
    ;;
  validate)
    if [[ -z "$input_path" ]]; then
      usage
      exit 64
    fi
    python3 - "$input_path" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
if payload.get("schemaVersion") != 1:
    raise SystemExit("schemaVersion must be 1")
if payload.get("package") != "ctxpack":
    raise SystemExit("package must be ctxpack")
if payload.get("status") not in {"ready", "deferred", "blocked"}:
    raise SystemExit("status must be ready, deferred, or blocked")
if payload.get("proofLevel") not in {"deterministic", "real-client"}:
    raise SystemExit("proofLevel must be deterministic or real-client")
checks = payload.get("checks", {})
for key in ["workspaceTests", "releaseDocs", "releasePackage", "archiveVerification", "demoArtifacts", "distributionMetadata"]:
    if checks.get(key) != "required":
        raise SystemExit(f"{key} must be required")
if checks.get("cursorRealClient") != "not_claimed" or checks.get("opencodeRealClient") != "not_claimed":
    raise SystemExit("Cursor/OpenCode real-client proof must be not_claimed")
privacy = payload.get("privacyStatus", {})
if privacy.get("localOnly") is not True or privacy.get("sourceTextLogged") is not False:
    raise SystemExit("privacy status must be local-only and source-free")
text = path.read_text()
for forbidden in ["/Users/", "BEGIN PRIVATE KEY", "GITHUB_TOKEN", "API_KEY=", "promptText", '"sourceText":']:
    if forbidden in text:
        raise SystemExit(f"forbidden token in candidate status: {forbidden}")
PY
    echo "release candidate status is valid: $input_path"
    ;;
  *)
    usage
    exit 64
    ;;
esac

