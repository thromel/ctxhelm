#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"

required_files=(
  "$repo_root/packaging/homebrew/ctxpack.rb.template"
  "$repo_root/packaging/crates/README.md"
  "$repo_root/packaging/release/update-metadata.schema.json"
  "$repo_root/packaging/release/update-metadata.example.json"
  "$repo_root/scripts/render-homebrew-formula.sh"
  "$repo_root/scripts/verify-release-archive.sh"
  "$repo_root/docs/distribution.md"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "distribution metadata smoke failed: missing ${file#"$repo_root"/}" >&2
    exit 1
  fi
done

bash -n "$repo_root/scripts/render-homebrew-formula.sh"
bash -n "$repo_root/scripts/verify-release-archive.sh"

metadata_path="$repo_root/.ctxpack/distribution-metadata-smoke.json"
dist_dir="${CTXPACK_DIST_DIR:-"$repo_root/dist"}"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

cargo_metadata="$work_dir/cargo-metadata.json"
cargo metadata --no-deps --format-version 1 >"$cargo_metadata"
version="$(python3 - "$cargo_metadata" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(next(package["version"] for package in payload["packages"] if package["name"] == "ctxpack"))
PY
)"
target_label="$(rustc -vV | awk '/^host:/ { print $2 }')"
archive_name="ctxpack-v${version}-${target_label}.tar.gz"
archive_path="$dist_dir/$archive_name"
rendered_formula="$work_dir/ctxpack.rb"

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    sha256sum "$1" | awk '{ print $1 }'
  fi
}

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
  case "${file#"$repo_root"/}" in
    scripts/*)
      continue
      ;;
  esac
  for forbidden in "/Users/" "BEGIN PRIVATE KEY" "GITHUB_TOKEN" "API_KEY=" "brew install ctxpack" "cargo install ctxpack" "self-update is implemented" "signed installer is ready"; do
    if grep -F -- "$forbidden" "$file" >/dev/null; then
      echo "distribution metadata smoke failed: forbidden token '$forbidden' in ${file#"$repo_root"/}" >&2
      exit 1
    fi
  done
done

grep -F -- "Homebrew formula template" "$repo_root/packaging/homebrew/ctxpack.rb.template" >/dev/null
grep -F -- "crates.io preparation" "$repo_root/packaging/crates/README.md" >/dev/null
grep -F -- "render-homebrew-formula.sh" "$repo_root/docs/distribution.md" >/dev/null
grep -F -- "scripts/verify-release-archive.sh" "$repo_root/docs/distribution.md" >/dev/null
grep -F -- "not a self-update implementation" "$repo_root/docs/distribution.md" >/dev/null
grep -F -- "signing and notarization gaps" "$repo_root/docs/distribution.md" >/dev/null

if [[ -f "$archive_path" ]]; then
  archive_sha256="$(sha256_file "$archive_path")"
  archive_url="https://github.com/thromel/ctxpack/releases/download/v${version}/${archive_name}"
  bash "$repo_root/scripts/render-homebrew-formula.sh" \
    --version "$version" \
    --url "$archive_url" \
    --sha256 "$archive_sha256" \
    --output "$rendered_formula" >/dev/null
  grep -F -- "desc \"Local, read-only context compiler for AI coding agents\"" "$rendered_formula" >/dev/null
  grep -F -- "license \"MIT\"" "$rendered_formula" >/dev/null
  formula_status="passed"
else
  archive_sha256=""
  archive_url=""
  formula_status="skipped_no_archive"
fi

package_list="$work_dir/cargo-package-list.txt"
cargo package --manifest-path "$repo_root/crates/ctxpack/Cargo.toml" --locked --allow-dirty --list >"$package_list"
for required in Cargo.toml README.md src/main.rs tests/cli_compat.rs tests/release_packaging.rs; do
  grep -Fx -- "$required" "$package_list" >/dev/null
done
for forbidden in '.ctxpack/' '.planning/' 'target/' 'dist/' '.env' 'request-summary' 'traces.jsonl' '/Users/'; do
  if grep -F -- "$forbidden" "$package_list" >/dev/null; then
    echo "distribution metadata smoke failed: cargo package includes forbidden entry '$forbidden'" >&2
    exit 1
  fi
done

mkdir -p "$(dirname "$metadata_path")"
python3 - "$metadata_path" "$version" "$target_label" "$formula_status" "$archive_name" "$archive_sha256" <<'PY'
import json
import pathlib
import sys

metadata_path, version, target_label, formula_status, archive_name, archive_sha256 = sys.argv[1:]
payload = {
    "schemaVersion": "ctxpack-distribution-readiness-v1",
    "version": version,
    "targetLabel": target_label,
    "homebrewFormulaRender": {
        "status": formula_status,
        "archiveName": archive_name if archive_sha256 else None,
        "archiveSha256": archive_sha256 or None,
        "published": False,
    },
    "cratesPackage": {
        "status": "passed",
        "published": False,
        "sourceFreeBoundaryChecked": True,
    },
    "privacyStatus": {
        "localOnly": True,
        "sourceTextLogged": False,
        "rawPromptStored": False,
        "remoteEmbeddingsUsed": False,
        "remoteRerankingUsed": False,
    },
    "unsupportedActions": [
        "brew tap publication",
        "crates.io publish",
        "global install",
        "signed installer",
        "self-update",
    ],
}
pathlib.Path(metadata_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

echo "distribution metadata smoke passed"
