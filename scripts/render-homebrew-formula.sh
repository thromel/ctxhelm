#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: render-homebrew-formula.sh --version VERSION --url URL --sha256 SHA256 --output PATH

Renders packaging/homebrew/ctxhelm.rb.template into a concrete formula file.
This is a local readiness helper only: it does not create a tap, install
Homebrew, publish artifacts, or mutate global package-manager state.
EOF
}

version=""
url=""
sha256=""
output_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      version="${2:-}"
      shift 2
      ;;
    --url)
      url="${2:-}"
      shift 2
      ;;
    --sha256)
      sha256="${2:-}"
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

if [[ -z "$version" || -z "$url" || -z "$sha256" || -z "$output_path" ]]; then
  usage
  exit 64
fi
if [[ ! "$version" =~ ^[0-9]+[.][0-9]+[.][0-9]+([-.][A-Za-z0-9.]+)?$ ]]; then
  echo "invalid version: $version" >&2
  exit 65
fi
if [[ ! "$url" =~ ^https://github[.]com/thromel/ctxhelm/releases/download/v${version}/ctxhelm-v${version}-[^/]+[.]tar[.]gz$ ]]; then
  echo "unexpected release archive url: $url" >&2
  exit 65
fi
if [[ ! "$sha256" =~ ^[0-9a-f]{64}$ ]]; then
  echo "invalid sha256: $sha256" >&2
  exit 65
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
template="$repo_root/packaging/homebrew/ctxhelm.rb.template"
if [[ ! -f "$template" ]]; then
  echo "missing Homebrew formula template: $template" >&2
  exit 66
fi

mkdir -p "$(dirname "$output_path")"
python3 - "$template" "$output_path" "$version" "$url" "$sha256" <<'PY'
import pathlib
import sys

template_path, output_path, version, url, sha256 = sys.argv[1:]
text = pathlib.Path(template_path).read_text(encoding="utf-8")
text = text.replace("CTXHELM_URL", url)
text = text.replace("CTXHELM_SHA256", sha256)
if "CTXHELM_" in text:
    raise SystemExit("unrendered CTXHELM placeholder remains")
pathlib.Path(output_path).write_text(text, encoding="utf-8")
PY

grep -F -- "class Ctxhelm < Formula" "$output_path" >/dev/null
grep -F -- "url \"$url\"" "$output_path" >/dev/null
grep -F -- "sha256 \"$sha256\"" "$output_path" >/dev/null
grep -F -- "depends_on arch: :arm64" "$output_path" >/dev/null
grep -F -- "bin.install \"ctxhelm\"" "$output_path" >/dev/null
grep -F -- "shell_output(\"#{bin}/ctxhelm --version\")" "$output_path" >/dev/null
if grep -F -- "CTXHELM_" "$output_path" >/dev/null; then
  echo "rendered formula still contains template placeholders" >&2
  exit 65
fi
if grep -F -- "/Users/" "$output_path" >/dev/null; then
  echo "rendered formula contains a machine-local path" >&2
  exit 65
fi

echo "rendered $output_path"
