#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage:
  release-candidate-status.sh create --output PATH --status ready|deferred|blocked [--proof-level deterministic|real-client] [--proof-summary PATH]
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
proof_summary_path=""

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
    --proof-summary)
      proof_summary_path="${2:-}"
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
    python3 - "$output_path" "$status" "$proof_level" "$proof_summary_path" <<'PY'
import datetime as dt
import hashlib
import json
import pathlib
import sys

output_path, status, proof_level, proof_summary_path = sys.argv[1:]

release_proof = {
    "status": "not_attached",
    "summarySha256": None,
    "archiveName": None,
    "archiveSha256": None,
    "binarySha256": None,
    "binarySource": None,
    "requiredChecks": 0,
    "cleanColdFixtureProductProof": "not_attached",
    "cleanColdFixtureRequired": False,
}
if status == "ready" and not proof_summary_path:
    raise SystemExit("ready candidate requires --proof-summary")
if proof_summary_path:
    proof_path = pathlib.Path(proof_summary_path)
    proof_text = proof_path.read_text()
    summary = json.loads(proof_text)
    optional = summary.get("optionalProofs", {})
    release_archive = summary.get("releaseArchive", {})
    binary = summary.get("binaryIdentity", {})
    release_proof = {
        "status": summary.get("status"),
        "summarySha256": hashlib.sha256(proof_text.encode("utf-8")).hexdigest(),
        "archiveName": release_archive.get("name"),
        "archiveSha256": release_archive.get("sha256"),
        "binarySha256": binary.get("sha256"),
        "binarySource": binary.get("source"),
        "requiredChecks": len(summary.get("requiredChecks", [])),
        "cleanColdFixtureProductProof": optional.get("cleanColdFixtureProductProof"),
        "cleanColdFixtureRequired": optional.get("cleanColdFixtureRequired") is True,
        "resourceBackedGapSummaryContract": optional.get("resourceBackedGapSummaryContract"),
    }
    if status == "ready":
        if summary.get("status") != "passed":
            raise SystemExit("ready candidate requires passed release proof summary")
        if optional.get("cleanColdFixtureProductProof") != "passed":
            raise SystemExit("ready candidate requires passed clean fixture proof")
        if optional.get("cleanColdFixtureRequired") is not True:
            raise SystemExit("ready candidate requires clean fixture proof to be required")
        if binary.get("source") != "archive":
            raise SystemExit("ready candidate requires archive binary proof")

payload = {
    "schemaVersion": 1,
    "package": "ctxhelm",
    "version": "1.1.9",
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
    "releaseProof": release_proof,
    "distributionDecision": {
        "primaryChannel": "local_archive",
        "localArchive": "ready" if status == "ready" else "deferred",
        "homebrewFormula": "ready" if status == "ready" else "deferred",
        "cratesIo": "deferred",
        "signedInstaller": "deferred",
        "selfUpdate": "not_implemented",
        "reason": "v1.1.9 production candidate supports local archives and the Apple Silicon Homebrew tap; crates.io publication, signed installers, and self-update remain future work.",
    },
    "knownLimitations": [
        "Cursor and OpenCode real-client proof is not claimed for v1.1.9.",
        "crates.io publication, signed installers, self-update, and hosted sync are future work.",
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
if payload.get("package") != "ctxhelm":
    raise SystemExit("package must be ctxhelm")
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
release_proof = payload.get("releaseProof", {})
if payload.get("status") == "ready":
    if release_proof.get("status") != "passed":
        raise SystemExit("ready status requires passed release proof")
    if release_proof.get("binarySource") != "archive":
        raise SystemExit("ready status requires archive binary proof")
    if release_proof.get("cleanColdFixtureProductProof") != "passed":
        raise SystemExit("ready status requires passed clean fixture proof")
    if release_proof.get("cleanColdFixtureRequired") is not True:
        raise SystemExit("ready status requires clean fixture proof to be required")
    if not release_proof.get("archiveSha256") or not release_proof.get("binarySha256"):
        raise SystemExit("ready status requires archive and binary checksums")
distribution = payload.get("distributionDecision", {})
if distribution.get("primaryChannel") != "local_archive":
    raise SystemExit("primaryChannel must be local_archive")
expected_homebrew = "ready" if payload.get("status") == "ready" else "deferred"
if distribution.get("homebrewFormula") != expected_homebrew:
    raise SystemExit(f"homebrewFormula must be {expected_homebrew} for v1.1.9")
for deferred in ["cratesIo", "signedInstaller"]:
    if distribution.get(deferred) != "deferred":
        raise SystemExit(f"{deferred} must be deferred for v1.1.9")
if distribution.get("selfUpdate") != "not_implemented":
    raise SystemExit("selfUpdate must be not_implemented")
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
