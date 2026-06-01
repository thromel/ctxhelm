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
  echo "workspace smoke failed: CTXHELM_BIN is not executable: $ctxhelm_bin" >&2
  exit 64
fi

repo_a="$work_dir/repo-a"
repo_b="$work_dir/repo-b"
home="$work_dir/ctxhelm-home"
sentinel="CTXHELM_WORKSPACE_SOURCE_SENTINEL_DO_NOT_LEAK"

create_repo() {
  local repo="$1"
  local name="$2"
  mkdir -p "$repo/src" "$repo/tests" "$repo/dist"
  printf 'export function %s() { return "%s"; }\n' "$name" "$name" >"$repo/src/$name.ts"
  printf 'test("%s", () => {});\n' "$name" >"$repo/tests/$name.test.ts"
  printf '%s\n' "$sentinel" >"$repo/.env"
  printf '%s\n' "$sentinel" >"$repo/dist/generated.min.js"
  git -C "$repo" init >/dev/null
  git -C "$repo" config user.email ctxhelm@example.com
  git -C "$repo" config user.name ctxhelm
  git -C "$repo" add .
  git -C "$repo" commit -m "fixture $name" >/dev/null
}

create_repo "$repo_a" "alpha"
create_repo "$repo_b" "beta"

export CTXHELM_HOME="$home"

init_json="$work_dir/workspace-init.json"
"$ctxhelm_bin" workspace init --repo "$repo_a" --member "$repo_b" --label primary --format json >"$init_json"

manifest="$repo_a/.ctxhelm/workspace.json"
if [[ ! -f "$manifest" ]]; then
  echo "workspace smoke failed: manifest was not created" >&2
  exit 1
fi

status_json="$work_dir/workspace-status.json"
"$ctxhelm_bin" workspace status --repo "$repo_a" --format json >"$status_json"

plan_json="$work_dir/workspace-plan.json"
"$ctxhelm_bin" workspace prepare-task "fix beta redirect" --repo "$repo_a" --mode bug-fix --format json >"$plan_json"
pack_json="$work_dir/workspace-pack.json"
"$ctxhelm_bin" workspace get-pack "fix beta redirect" --repo "$repo_a" --mode bug-fix --budget brief --target-agent codex --format json >"$pack_json"

python3 - "$init_json" "$status_json" "$plan_json" "$pack_json" "$manifest" "$sentinel" "$home" <<'PY'
import json
import pathlib
import sys

init_path = pathlib.Path(sys.argv[1])
status_path = pathlib.Path(sys.argv[2])
plan_path = pathlib.Path(sys.argv[3])
pack_path = pathlib.Path(sys.argv[4])
manifest_path = pathlib.Path(sys.argv[5])
sentinel = sys.argv[6]
home = pathlib.Path(sys.argv[7])

for label, path in [("init", init_path), ("status", status_path)]:
    text = path.read_text()
    if sentinel in text:
        raise SystemExit(f"workspace smoke failed: {label} output leaked source sentinel")
    data = json.loads(text)
    if data["sourceTextLogged"] is not False:
        raise SystemExit(f"workspace smoke failed: {label} sourceTextLogged was not false")
    if data["privacyStatus"]["localOnly"] is not True:
        raise SystemExit(f"workspace smoke failed: {label} privacyStatus.localOnly was not true")
    if data["repoCount"] != 2:
        raise SystemExit(f"workspace smoke failed: {label} repoCount was not 2")
    if data["availableRepoCount"] != 2:
        raise SystemExit(f"workspace smoke failed: {label} availableRepoCount was not 2")
    if any(repo["privacyStatus"]["sourceTextLogged"] for repo in data["repos"]):
        raise SystemExit(f"workspace smoke failed: {label} repo leaked sourceTextLogged=true")

manifest = json.loads(manifest_path.read_text())
if manifest["schemaVersion"] != 1:
    raise SystemExit("workspace smoke failed: manifest schemaVersion was not 1")
if len(manifest["repos"]) != 2:
    raise SystemExit("workspace smoke failed: manifest did not contain 2 repos")

plan_text = plan_path.read_text()
if sentinel in plan_text:
    raise SystemExit("workspace smoke failed: workspace plan leaked source sentinel")
plan = json.loads(plan_text)
if plan["sourceTextLogged"] is not False:
    raise SystemExit("workspace smoke failed: workspace plan sourceTextLogged was not false")
if plan["selectedRepoCount"] < 1:
    raise SystemExit("workspace smoke failed: workspace plan did not select any repos")
if not plan["privacyStatus"]["localOnly"]:
    raise SystemExit("workspace smoke failed: workspace plan privacyStatus.localOnly was not true")

pack_text = pack_path.read_text()
if sentinel in pack_text:
    raise SystemExit("workspace smoke failed: workspace pack leaked source sentinel")
pack = json.loads(pack_text)
if pack["sourceTextLogged"] is not False:
    raise SystemExit("workspace smoke failed: workspace pack sourceTextLogged was not false")
if pack["selectedRepoCount"] < 1 or not pack["repoPacks"]:
    raise SystemExit("workspace smoke failed: workspace pack did not include repo packs")
if pack["targetAgent"] != "codex":
    raise SystemExit("workspace smoke failed: workspace pack targetAgent was not codex")
if not all("repoId" in repo and "contextPack" in repo for repo in pack["repoPacks"]):
    raise SystemExit("workspace smoke failed: workspace pack lost repo boundaries")

for path in home.rglob("*"):
    if path.is_file() and sentinel in path.read_text(errors="ignore"):
        raise SystemExit(f"workspace smoke failed: source sentinel persisted in {path}")
PY

"$ctxhelm_bin" prepare-task "verify single repo still works" --repo "$repo_a" --no-trace >/dev/null

echo "workspace smoke passed"
