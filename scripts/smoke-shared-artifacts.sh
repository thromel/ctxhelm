#!/usr/bin/env bash
set -euo pipefail

work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

ctxhelm_bin="${CTXHELM_BIN:-}"
if [[ -z "$ctxhelm_bin" ]]; then
  ctxhelm_bin="$(pwd -P)/target/debug/ctxhelm"
fi
if [[ ! -x "$ctxhelm_bin" ]]; then
  echo "shared artifacts smoke failed: CTXHELM_BIN is not executable: $ctxhelm_bin" >&2
  exit 64
fi

repo="$work_dir/repo"
home="$work_dir/ctxhelm-home"
sentinel="CTXHELM_SHARED_ARTIFACT_SOURCE_SENTINEL_DO_NOT_LEAK"

mkdir -p "$repo/src" "$repo/.ctxhelm/cards" "$repo/dist"
printf 'export function authSummary() { return true; }\n' >"$repo/src/auth.ts"
printf 'source-free auth card\n' >"$repo/.ctxhelm/cards/auth.md"
printf '{"sourceTextLogged":false,"eventCount":0}\n' >"$repo/.ctxhelm/feedback-summary.json"
printf '%s\n' "$sentinel" >"$repo/.env"
printf '%s\n' "$sentinel" >"$repo/dist/generated.min.js"

git -C "$repo" init >/dev/null
git -C "$repo" config user.email ctxhelm@example.com
git -C "$repo" config user.name ctxhelm
git -C "$repo" add .
git -C "$repo" commit -m "fixture shared artifacts" >/dev/null

export CTXHELM_HOME="$home"

policy_init_json="$work_dir/policy-init.json"
"$ctxhelm_bin" workspace policy init --repo "$repo" --format json >"$policy_init_json"

workspace_init_json="$work_dir/workspace-init.json"
"$ctxhelm_bin" workspace init --repo "$repo" --format json >"$workspace_init_json"

policy_status_json="$work_dir/policy-status.json"
"$ctxhelm_bin" workspace policy status --repo "$repo" --format json >"$policy_status_json"

manifest_json="$work_dir/shared-artifacts.json"
"$ctxhelm_bin" workspace artifacts export --repo "$repo" --format json >"$manifest_json"

manifest_path="$repo/.ctxhelm/shared-artifacts.json"
inspect_json="$work_dir/inspect.json"
"$ctxhelm_bin" workspace artifacts inspect "$manifest_path" --format json >"$inspect_json"

import_json="$work_dir/import.json"
"$ctxhelm_bin" workspace artifacts import "$manifest_path" --repo "$repo" --format json >"$import_json"

python3 - "$policy_init_json" "$policy_status_json" "$manifest_json" "$inspect_json" "$import_json" "$manifest_path" "$repo/.ctxhelm/imported-shared-artifacts.json" "$sentinel" "$home" "$ctxhelm_bin" "$repo" <<'PY'
import json
import pathlib
import subprocess
import sys

policy_init_path = pathlib.Path(sys.argv[1])
policy_status_path = pathlib.Path(sys.argv[2])
manifest_output_path = pathlib.Path(sys.argv[3])
inspect_path = pathlib.Path(sys.argv[4])
import_path = pathlib.Path(sys.argv[5])
manifest_path = pathlib.Path(sys.argv[6])
imported_path = pathlib.Path(sys.argv[7])
sentinel = sys.argv[8]
home = pathlib.Path(sys.argv[9])
ctxhelm_bin = pathlib.Path(sys.argv[10])
repo = pathlib.Path(sys.argv[11])

def load_source_free(label, path):
    text = path.read_text()
    if sentinel in text:
        raise SystemExit(f"shared artifacts smoke failed: {label} leaked source sentinel")
    data = json.loads(text)
    if data.get("sourceTextLogged") is not False:
        raise SystemExit(f"shared artifacts smoke failed: {label} sourceTextLogged was not false")
    privacy = data.get("privacyStatus", {})
    if privacy.get("localOnly") is not True:
        raise SystemExit(f"shared artifacts smoke failed: {label} privacyStatus.localOnly was not true")
    return data

policy_init = load_source_free("policy init", policy_init_path)
policy_status = load_source_free("policy status", policy_status_path)
for label, data in [("policy init", policy_init), ("policy status", policy_status)]:
    policy = data["policy"]
    if policy["allowCloudEmbeddings"] or policy["allowCloudReranking"]:
        raise SystemExit(f"shared artifacts smoke failed: {label} allowed cloud retrieval by default")
    if policy["allowSourceSnippetsInSharedArtifacts"]:
        raise SystemExit(f"shared artifacts smoke failed: {label} allowed source snippets")

manifest = load_source_free("manifest export", manifest_output_path)
if manifest["schemaVersion"] != 1:
    raise SystemExit("shared artifacts smoke failed: manifest schemaVersion was not 1")
if not any(artifact["kind"] == "context_cards" and artifact["status"] == "present" for artifact in manifest["artifacts"]):
    raise SystemExit("shared artifacts smoke failed: context cards artifact was not present")
if not manifest_path.exists():
    raise SystemExit("shared artifacts smoke failed: manifest file was not written")
if sentinel in manifest_path.read_text():
    raise SystemExit("shared artifacts smoke failed: manifest file leaked source sentinel")

inspect = load_source_free("manifest inspect", inspect_path)
if inspect["compatible"] is not True:
    raise SystemExit("shared artifacts smoke failed: manifest was not compatible")

imported = load_source_free("manifest import", import_path)
if imported["compatible"] is not True:
    raise SystemExit("shared artifacts smoke failed: imported manifest was not compatible")
if not imported_path.exists():
    raise SystemExit("shared artifacts smoke failed: imported manifest file was not written")
if sentinel in imported_path.read_text():
    raise SystemExit("shared artifacts smoke failed: imported manifest leaked source sentinel")

for path in home.rglob("*"):
    if path.is_file() and sentinel in path.read_text(errors="ignore"):
        raise SystemExit(f"shared artifacts smoke failed: source sentinel persisted in {path}")

server = subprocess.Popen(
    [str(ctxhelm_bin), "serve-mcp"],
    cwd=repo,
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
)
next_id = 1

def request(method, params):
    global next_id
    payload = {"jsonrpc": "2.0", "id": next_id, "method": method, "params": params}
    next_id += 1
    server.stdin.write(json.dumps(payload, separators=(",", ":")) + "\n")
    server.stdin.flush()
    line = server.stdout.readline()
    if not line:
        raise SystemExit(f"shared artifacts smoke failed: MCP server exited before {method}")
    response = json.loads(line)
    if "error" in response:
        raise SystemExit(f"shared artifacts smoke failed: {method} returned {response['error']}")
    return response["result"]

try:
    initialize = request("initialize", {})
    if initialize.get("serverInfo", {}).get("name") != "ctxhelm":
        raise SystemExit("shared artifacts smoke failed: MCP initialize server name mismatch")

    resources = request("resources/list", {}).get("resources", [])
    uris = {resource.get("uri") for resource in resources}
    for uri in ["ctxhelm://workspace/status", "ctxhelm://workspace/shared-artifacts"]:
        if uri not in uris:
            raise SystemExit(f"shared artifacts smoke failed: missing MCP resource {uri}")

    for label, uri in [
        ("workspace status resource", "ctxhelm://workspace/status"),
        ("shared artifacts resource", "ctxhelm://workspace/shared-artifacts"),
    ]:
        resource = request("resources/read", {"uri": uri})
        contents = resource.get("contents", [])
        if not contents:
            raise SystemExit(f"shared artifacts smoke failed: {label} returned no contents")
        text = contents[0].get("text", "")
        if sentinel in text:
            raise SystemExit(f"shared artifacts smoke failed: {label} leaked source sentinel")
        data = json.loads(text)
        if data.get("sourceTextLogged") is not False:
            raise SystemExit(f"shared artifacts smoke failed: {label} sourceTextLogged was not false")
        if not data.get("privacyStatus", {}).get("localOnly"):
            raise SystemExit(f"shared artifacts smoke failed: {label} privacyStatus.localOnly was not true")
finally:
    if server.stdin:
        server.stdin.close()
    try:
        server.wait(timeout=5)
    except subprocess.TimeoutExpired:
        server.kill()
        server.wait(timeout=5)
PY

echo "shared artifacts smoke passed"
