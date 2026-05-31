#!/usr/bin/env bash
set -euo pipefail

CTXPACK_ROOT="${CTXPACK_ROOT:-$PWD}"
CTXPACK_SMOKE_REPO="${CTXPACK_SMOKE_REPO:-$PWD}"
CTXPACK_SMOKE_TASK="${CTXPACK_SMOKE_TASK:-fix requireSession auth bug}"
CTXPACK_SMOKE_PATH="${CTXPACK_SMOKE_PATH:-crates/ctxpack-mcp/src/tools.rs}"
CTXPACK_SMOKE_QUERY="${CTXPACK_SMOKE_QUERY:-prepare_task}"

if [[ ! -d "$CTXPACK_ROOT" ]]; then
  echo "CTXPACK_ROOT does not exist: $CTXPACK_ROOT" >&2
  exit 1
fi

if [[ ! -d "$CTXPACK_SMOKE_REPO" ]]; then
  echo "CTXPACK_SMOKE_REPO does not exist: $CTXPACK_SMOKE_REPO" >&2
  exit 1
fi

if [[ -n "${CTXPACK_BIN:-}" && ! -x "$CTXPACK_BIN" ]]; then
  echo "CTXPACK_BIN is not executable: $CTXPACK_BIN" >&2
  exit 1
fi

created_home=0
if [[ -z "${CTXPACK_HOME:-}" ]]; then
  CTXPACK_HOME="$(mktemp -d)"
  created_home=1
fi

server_cwd="$(mktemp -d)"
diff_repo="$(mktemp -d)"
smoke_diff_path="src/ctxpack_smoke_current_diff_$$.rs"
smoke_diff_file="$diff_repo/$smoke_diff_path"

cleanup() {
  rm -rf "$diff_repo"
  rm -rf "$server_cwd"
  if [[ "$created_home" == "1" ]]; then
    rm -rf "$CTXPACK_HOME"
  fi
}
trap cleanup EXIT

git -C "$diff_repo" init -q
mkdir -p "$(dirname "$smoke_diff_file")"
printf 'pub fn ctxpack_smoke_current_diff() {}\n' >"$smoke_diff_file"
export CTXPACK_HOME

python3 - "$CTXPACK_ROOT" "$CTXPACK_SMOKE_REPO" "$CTXPACK_SMOKE_TASK" "$CTXPACK_SMOKE_PATH" "$CTXPACK_SMOKE_QUERY" "$server_cwd" "$diff_repo" "$smoke_diff_path" "${CTXPACK_BIN:-}" <<'PY'
import json
import os
import subprocess
import sys
from pathlib import Path
from urllib.parse import quote

root, repo, task, anchor_path, query, server_cwd, diff_repo, smoke_diff_path, ctxpack_bin = sys.argv[1:]
repo_path = Path(repo).resolve()
diff_repo_path = Path(diff_repo).resolve()
anchor = repo_path / anchor_path
if not anchor.exists():
    raise SystemExit(f"anchor path does not exist in CTXPACK_SMOKE_REPO: {anchor_path}")

env = os.environ.copy()
if ctxpack_bin:
    command = [ctxpack_bin, "serve-mcp"]
else:
    command = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(Path(root) / "Cargo.toml"),
        "-p",
        "ctxpack",
        "--",
        "serve-mcp",
    ]
server = subprocess.Popen(
    command,
    cwd=server_cwd,
    env=env,
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
)

next_id = 1

def request(method, params):
    global next_id
    payload = {
        "jsonrpc": "2.0",
        "id": next_id,
        "method": method,
        "params": params,
    }
    next_id += 1
    server.stdin.write(json.dumps(payload, separators=(",", ":")) + "\n")
    server.stdin.flush()
    line = server.stdout.readline()
    if not line:
        raise SystemExit(f"server exited before responding to {method}")
    response = json.loads(line)
    if "error" in response:
        raise SystemExit(f"{method} failed: {response['error']}")
    return response["result"]

def tool(name, arguments):
    return request("tools/call", {"name": name, "arguments": arguments})

def structured(result, label):
    value = result.get("structuredContent")
    if value is None:
        raise SystemExit(f"{label}: missing structuredContent")
    return value

def require_list(value, label):
    if not isinstance(value, list) or not value:
        raise SystemExit(f"{label}: expected a non-empty array")
    return value

def require_path(items, expected, label):
    require_list(items, label)
    if not any(item.get("path") == expected for item in items if isinstance(item, dict)):
        raise SystemExit(f"{label}: missing path {expected}")

def require_repo_relative_path(path, label):
    if not path:
        raise SystemExit(f"{label}: blank path")
    if not (repo_path / path).exists():
        raise SystemExit(f"{label}: path is not in explicit repo: {path}")

def context_area_for_path(path):
    components = [component for component in path.split("/") if component]
    if not components:
        return "."
    if components[0].startswith("."):
        if len(components) >= 2:
            return f"{components[0]}/{components[1]}"
        return components[0]
    if len(components) == 1:
        return "."
    if len(components) == 2 and "." in components[1]:
        return components[0]
    return "/".join(components[:2])

def resource_json(uri, label):
    resource = request("resources/read", {"uri": uri})
    contents = require_list(resource.get("contents"), f"{label}.contents")
    text = contents[0].get("text", "")
    if not text:
        raise SystemExit(f"{label}: empty resource text")
    try:
        return json.loads(text)
    except json.JSONDecodeError as error:
        raise SystemExit(f"{label}: invalid JSON resource text: {error}")

try:
    initialize = request("initialize", {})
    if initialize.get("serverInfo", {}).get("name") != "ctxpack":
        raise SystemExit("initialize: unexpected serverInfo.name")

    common = {
        "task": task,
        "repo": str(repo_path),
        "mode": "bug_fix",
        "paths": [anchor_path],
        "targetAgent": "codex",
        "recordTrace": False,
    }
    plan = structured(tool("prepare_task", common), "prepare_task")
    require_path(plan.get("targetFiles"), anchor_path, "prepare_task.targetFiles")
    pack_options = require_list(plan.get("packOptions"), "prepare_task.packOptions")
    pack_uri = pack_options[0].get("resourceUri")
    if not pack_uri:
        raise SystemExit("prepare_task.packOptions[0]: missing resourceUri")

    context_areas = resource_json("ctxpack://repo/context-areas", "resources/read.contextAreas")
    if context_areas.get("sourceTextLogged") is not False:
        raise SystemExit("resources/read.contextAreas: sourceTextLogged was not false")
    context_areas_scope = context_areas.get("resourceScope", {})
    if context_areas_scope.get("kind") != "safeInventoryArea":
        raise SystemExit("resources/read.contextAreas: missing safeInventoryArea resourceScope")
    if context_areas_scope.get("taskConditioned") is not False:
        raise SystemExit("resources/read.contextAreas: resourceScope must be taskConditioned=false")
    if context_areas_scope.get("countsSource") != "safeInventory":
        raise SystemExit("resources/read.contextAreas: resourceScope countsSource must be safeInventory")
    if not isinstance(context_areas.get("areas"), list):
        raise SystemExit("resources/read.contextAreas: missing areas array")
    if not context_areas["areas"]:
        raise SystemExit("resources/read.contextAreas: empty areas array")
    first_area_scope = context_areas["areas"][0].get("resourceScope", {})
    if first_area_scope.get("kind") != "safeInventoryArea":
        raise SystemExit("resources/read.contextAreas.areas[0]: missing safeInventoryArea resourceScope")
    context_area = context_area_for_path(anchor_path)
    context_area_uri = "ctxpack://repo/context-area/" + quote(context_area, safe="")
    context_area_resource = resource_json(
        context_area_uri, "resources/read.contextArea"
    )
    if context_area_resource.get("resourceUri") != context_area_uri:
        raise SystemExit("resources/read.contextArea: resourceUri mismatch")
    if context_area_resource.get("sourceTextLogged") is not False:
        raise SystemExit("resources/read.contextArea: sourceTextLogged was not false")
    context_area_scope = context_area_resource.get("resourceScope", {})
    if context_area_scope.get("kind") != "safeInventoryArea":
        raise SystemExit("resources/read.contextArea: missing safeInventoryArea resourceScope")
    if context_area_scope.get("taskConditioned") is not False:
        raise SystemExit("resources/read.contextArea: resourceScope must be taskConditioned=false")
    if context_area_scope.get("pathSource") != "safeInventory":
        raise SystemExit("resources/read.contextArea: resourceScope pathSource must be safeInventory")
    if context_area_resource.get("pathCount", 0) <= 0:
        raise SystemExit("resources/read.contextArea: expected paths for anchor area")
    next_read_batches = require_list(
        context_area_resource.get("nextReadBatches"),
        "resources/read.contextArea.nextReadBatches",
    )
    if not any(batch.get("paths") for batch in next_read_batches if isinstance(batch, dict)):
        raise SystemExit("resources/read.contextArea: nextReadBatches had no paths")

    pack = structured(
        tool(
            "get_pack",
            {
                **common,
                "budget": "brief",
                "format": "json",
            },
        ),
        "get_pack",
    )
    if not pack.get("repoId"):
        raise SystemExit("get_pack: blank repoId")
    require_list(pack.get("sections"), "get_pack.sections")

    search = structured(
        tool(
            "search",
            {
                "query": query,
                "repo": str(repo_path),
                "limit": 10,
            },
        ),
        "search",
    )
    files = require_list(search.get("files"), "search.files")
    require_repo_relative_path(files[0].get("path"), "search.files[0]")

    related = structured(
        tool(
            "related",
            {
                "path": anchor_path,
                "repo": str(repo_path),
                "include": ["tests", "dependencies"],
                "limit": 10,
            },
        ),
        "related",
    )
    related_tests = require_list(related.get("relatedTests"), "related.relatedTests")
    require_repo_relative_path(related_tests[0].get("path"), "related.relatedTests[0]")

    direct_tests = structured(
        tool(
            "related_tests",
            {
                "paths": [anchor_path],
                "repo": str(repo_path),
            },
        ),
        "related_tests",
    )
    require_list(direct_tests, "related_tests.structuredContent")
    require_repo_relative_path(direct_tests[0].get("path"), "related_tests[0]")

    current_diff = structured(
        tool(
            "current_diff",
            {
                "repo": str(diff_repo_path),
                "includeUntracked": True,
            },
        ),
        "current_diff",
    )
    if smoke_diff_path not in current_diff.get("untracked", []):
        raise SystemExit(
            f"current_diff: expected explicit repo untracked path {smoke_diff_path}"
        )

    resource = request("resources/read", {"uri": f"{pack_uri}.json"})
    contents = require_list(resource.get("contents"), "resources/read.contents")
    text = contents[0].get("text", "")
    if not text:
        raise SystemExit("resources/read: empty pack resource text")
    resource_pack = json.loads(text)
    if resource_pack.get("budget") != "brief":
        raise SystemExit("resources/read: pack budget is not brief")
    if not resource_pack.get("repoId"):
        raise SystemExit("resources/read: blank repoId")
    require_list(resource_pack.get("sections"), "resources/read.sections")

    print(
        "ctxpack MCP protocol smoke ok: "
        f"repo={repo_path} anchor={anchor_path} packUri={pack_uri}"
    )
finally:
    if server.stdin:
        server.stdin.close()
    try:
        server.wait(timeout=5)
    except subprocess.TimeoutExpired:
        server.kill()
        server.wait(timeout=5)
PY
