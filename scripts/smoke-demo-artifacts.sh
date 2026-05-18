#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "$script_dir/.." && pwd -P)"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

demo_dir="$work_dir/demo-artifacts"
CTXPACK_DEMO_DIR="$demo_dir" bash "$repo_root/scripts/generate-demo-artifacts.sh" >/dev/null

for required in \
  README.md \
  pack-inspector.json pack-inspector.html \
  retrieval-health.json retrieval-health.md \
  graph-neighborhood.json graph-neighborhood.md \
  policy-embedding.json policy-embedding.md \
  agent-preview.json agent-preview.md
do
  if [[ ! -f "$demo_dir/$required" ]]; then
    echo "demo artifacts smoke failed: missing $required" >&2
    exit 1
  fi
done

python3 - "$demo_dir" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
for path in root.glob("*.json"):
    payload = json.loads(path.read_text())
    privacy = payload.get("privacyStatus", {})
    if privacy.get("localOnly") is not True:
        raise SystemExit(f"{path.name}: privacyStatus.localOnly was not true")
    if payload.get("sourceTextLogged") is not False and privacy.get("sourceTextLogged") is not False:
        raise SystemExit(f"{path.name}: sourceTextLogged was not false")

for path in root.iterdir():
    if not path.is_file():
        continue
    text = path.read_text(errors="ignore")
    forbidden = [
        "/Users/",
        "BEGIN PRIVATE KEY",
        "GITHUB_TOKEN",
        "API_KEY=",
        "pub fn ",
        "function getSession",
        '"sourceText":',
        '"promptText":',
    ]
    for token in forbidden:
        if token in text:
            raise SystemExit(f"{path.name}: forbidden token {token!r}")
PY

echo "demo artifacts smoke passed"
