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

created_home=0
if [[ -z "${CTXPACK_HOME:-}" ]]; then
  CTXPACK_HOME="$(mktemp -d)"
  created_home=1
fi

server_cwd="$(mktemp -d)"
smoke_diff_path="ctxpack_smoke_current_diff_$$.rs"
smoke_diff_file="$CTXPACK_SMOKE_REPO/$smoke_diff_path"

cleanup() {
  rm -f "$smoke_diff_file"
  rm -rf "$server_cwd"
  if [[ "$created_home" == "1" ]]; then
    rm -rf "$CTXPACK_HOME"
  fi
}
trap cleanup EXIT

printf 'pub fn ctxpack_smoke_current_diff() {}\n' >"$smoke_diff_file"
export CTXPACK_HOME

python3 - "$CTXPACK_ROOT" "$CTXPACK_SMOKE_REPO" "$CTXPACK_SMOKE_TASK" "$CTXPACK_SMOKE_PATH" "$CTXPACK_SMOKE_QUERY" "$server_cwd" "$smoke_diff_path" <<'PY'
import json
import os
import subprocess
import sys
from pathlib import Path

root, repo, task, anchor_path, query, server_cwd, smoke_diff_path = sys.argv[1:]
repo_path = Path(repo).resolve()
anchor = repo_path / anchor_path
if not anchor.exists():
    raise SystemExit(f"anchor path does not exist in CTXPACK_SMOKE_REPO: {anchor_path}")

env = os.environ.copy()
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
                "repo": str(repo_path),
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
