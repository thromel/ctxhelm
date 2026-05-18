#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"

required_files=(
  "$repo_root/packaging/homebrew/ctxpack.rb.template"
  "$repo_root/packaging/crates/README.md"
  "$repo_root/packaging/release/update-metadata.schema.json"
  "$repo_root/packaging/release/update-metadata.example.json"
  "$repo_root/scripts/verify-release-archive.sh"
  "$repo_root/docs/distribution.md"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "distribution metadata smoke failed: missing ${file#"$repo_root"/}" >&2
    exit 1
  fi
done

bash -n "$repo_root/scripts/verify-release-archive.sh"

python3 - "$repo_root/packaging/release/update-metadata.schema.json" "$repo_root/packaging/release/update-metadata.example.json" <<'PY'
import json
import pathlib
import sys

schema = json.loads(pathlib.Path(sys.argv[1]).read_text())
example = json.loads(pathlib.Path(sys.argv[2]).read_text())
required = set(schema.get("required", []))
missing = sorted(required - set(example))
if missing:
    raise SystemExit(f"example missing required keys: {missing}")
privacy = example.get("privacyStatus", {})
if privacy.get("localOnly") is not True:
    raise SystemExit("example privacyStatus.localOnly was not true")
if privacy.get("sourceTextLogged") is not False:
    raise SystemExit("example privacyStatus.sourceTextLogged was not false")
if example.get("selfUpdateImplemented") is not False:
    raise SystemExit("example must not claim self-update")
if example.get("signedInstaller") is not False:
    raise SystemExit("example must not claim signed installers")
PY

for file in "${required_files[@]}"; do
  text="$(cat "$file")"
  for forbidden in "/Users/" "BEGIN PRIVATE KEY" "GITHUB_TOKEN" "API_KEY=" "brew install ctxpack" "cargo install ctxpack" "self-update is implemented" "signed installer is ready"; do
    if grep -F -- "$forbidden" "$file" >/dev/null; then
      echo "distribution metadata smoke failed: forbidden token '$forbidden' in ${file#"$repo_root"/}" >&2
      exit 1
    fi
  done
done

grep -F -- "Homebrew formula template" "$repo_root/packaging/homebrew/ctxpack.rb.template" >/dev/null
grep -F -- "crates.io preparation" "$repo_root/packaging/crates/README.md" >/dev/null
grep -F -- "scripts/verify-release-archive.sh" "$repo_root/docs/distribution.md" >/dev/null
grep -F -- "not a self-update implementation" "$repo_root/docs/distribution.md" >/dev/null
grep -F -- "signing and notarization gaps" "$repo_root/docs/distribution.md" >/dev/null

echo "distribution metadata smoke passed"

