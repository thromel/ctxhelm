#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: verify-homebrew-tap.sh [--tap OWNER/TAP] [--formula NAME] --expected-version VERSION --expected-url URL --expected-sha256 SHA256 [--output PATH]

Verifies the public Homebrew tap formula by tapping it, auditing/style-checking
the formula, installing it through Homebrew, running the formula test, and
checking the installed binary version. This script mutates local Homebrew state
by installing the formula, but it does not publish releases, edit files, or
mutate global agent configuration.
EOF
}

tap="thromel/tap"
formula="ctxpack"
expected_version=""
expected_url=""
expected_sha256=""
output_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tap)
      tap="${2:-}"
      shift 2
      ;;
    --formula)
      formula="${2:-}"
      shift 2
      ;;
    --expected-version)
      expected_version="${2:-}"
      shift 2
      ;;
    --expected-url)
      expected_url="${2:-}"
      shift 2
      ;;
    --expected-sha256)
      expected_sha256="${2:-}"
      shift 2
      ;;
    --output)
      output_path="${2:-}"
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

if [[ -z "$expected_version" || -z "$expected_url" || -z "$expected_sha256" ]]; then
  usage
  exit 64
fi
if [[ ! "$expected_sha256" =~ ^[0-9a-f]{64}$ ]]; then
  echo "invalid expected sha256: $expected_sha256" >&2
  exit 65
fi
if ! command -v brew >/dev/null 2>&1; then
  echo "brew is required for Homebrew tap verification" >&2
  exit 69
fi

brew tap "$tap" >/dev/null
tap_repo="$(brew --repo "$tap")"
formula_path="$tap_repo/Formula/$formula.rb"
if [[ ! -f "$formula_path" ]]; then
  echo "formula not found in tap: $formula_path" >&2
  exit 66
fi

ruby -c "$formula_path" >/dev/null
brew style "$formula_path" >/dev/null
brew audit --strict --new "$formula" >/dev/null
grep -F -- "url \"$expected_url\"" "$formula_path" >/dev/null
grep -F -- "sha256 \"$expected_sha256\"" "$formula_path" >/dev/null
grep -F -- "depends_on arch: :arm64" "$formula_path" >/dev/null

brew install "$tap/$formula" >/dev/null
brew test "$tap/$formula" >/dev/null

installed_prefix="$(brew --prefix "$tap/$formula")"
installed_version="$("$installed_prefix/bin/$formula" --version)"
if [[ "$installed_version" != "$expected_version" ]]; then
  echo "installed version mismatch: $installed_version != $expected_version" >&2
  exit 67
fi

if [[ -n "$output_path" ]]; then
  mkdir -p "$(dirname "$output_path")"
  python3 - "$output_path" "$tap" "$formula" "$formula_path" "$expected_version" "$expected_url" "$expected_sha256" "$installed_version" <<'PY'
import json
import pathlib
import sys

(
    output_path,
    tap,
    formula,
    formula_path,
    expected_version,
    expected_url,
    expected_sha256,
    installed_version,
) = sys.argv[1:]
payload = {
    "schemaVersion": "ctxpack-homebrew-tap-proof-v1",
    "tap": tap,
    "formula": formula,
    "formulaPath": formula_path,
    "expectedVersion": expected_version,
    "installedVersion": installed_version,
    "expectedUrl": expected_url,
    "expectedSha256": expected_sha256,
    "checks": {
        "tapSucceeded": True,
        "rubySyntaxPassed": True,
        "stylePassed": True,
        "auditPassed": True,
        "urlMatched": True,
        "sha256Matched": True,
        "arm64ConstraintPresent": True,
        "installPassed": True,
        "testPassed": True,
        "versionPassed": True,
    },
    "privacyStatus": {
        "localOnly": True,
        "sourceTextLogged": False,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
    },
    "unsupportedActions": [
        "crates.io publish",
        "signed installer",
        "self-update",
        "global agent config mutation",
    ],
}
pathlib.Path(output_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
fi

echo "homebrew tap verified: $tap/$formula $installed_version"
