#!/usr/bin/env bash
set -euo pipefail

ctxpack_bin="${CTXPACK_BIN:-ctxpack}"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
store="$work_dir/semantic.sqlite3"
search_json="$work_dir/search.json"
plan_json="$work_dir/plan.json"
pack_json="$work_dir/pack.json"
eval_json="$work_dir/eval.json"

mkdir -p "$repo/src/payments" "$home"
git -C "$repo" init -q
git -C "$repo" config user.email "ctxpack@example.com"
git -C "$repo" config user.name "ctxpack"
printf '# Fixture\n' >"$repo/README.md"
git -C "$repo" add .
git -C "$repo" commit -m "initial fixture" >/dev/null

cat >"$repo/src/payments/webhooks.ts" <<'SRC'
export function verifyPaymentWebhook(event: string) {
  if (!event.includes("payment")) {
    throw new Error("CTXPACK_SEMANTIC_SOURCE_SENTINEL");
  }
  return "payment webhook validation";
}
SRC
printf 'SECRET=CTXPACK_SEMANTIC_SECRET_SENTINEL\n' >"$repo/.env"
git -C "$repo" add src/payments/webhooks.ts .env
git -C "$repo" commit -m "add payment webhook validation" >/dev/null

export CTXPACK_HOME="$home"

"$ctxpack_bin" index --repo "$repo" --semantic --store-path "$store" >"$work_dir/index.txt"
"$ctxpack_bin" storage status --repo "$repo" --path "$store" >"$work_dir/status.txt"
"$ctxpack_bin" semantic status --repo "$repo" --format json >"$work_dir/semantic-status.json"
"$ctxpack_bin" search "payment webhook validation" --repo "$repo" --limit 5 --semantic >"$search_json"
"$ctxpack_bin" prepare-task "fix payment webhook validation" \
  --repo "$repo" \
  --mode bug-fix \
  --semantic \
  --no-trace >"$plan_json"
"$ctxpack_bin" get-pack "fix payment webhook validation" \
  --repo "$repo" \
  --mode bug-fix \
  --budget brief \
  --format json \
  --semantic \
  --no-trace >"$pack_json"
"$ctxpack_bin" eval history \
  --repo "$repo" \
  --limit 1 \
  --budget 5 \
  --format json \
  --semantic >"$eval_json"

grep -F -- "Semantic storage sync" "$work_dir/index.txt" >/dev/null
grep -F -- "Semantic vector records:" "$work_dir/status.txt" >/dev/null

python3 - "$work_dir/semantic-status.json" "$search_json" "$plan_json" "$pack_json" "$eval_json" <<'PY'
import json
import sys

status_path, search_path, plan_path, pack_path, eval_path = sys.argv[1:]
status = json.load(open(status_path, encoding="utf-8"))
search = json.load(open(search_path, encoding="utf-8"))
plan = json.load(open(plan_path, encoding="utf-8"))
pack = json.load(open(pack_path, encoding="utf-8"))
evaluation = json.load(open(eval_path, encoding="utf-8"))

if status.get("providerKind") != "local_hash":
    raise SystemExit("semantic status did not report local_hash")
if status.get("providerRole") != "deterministic_scaffold":
    raise SystemExit("semantic status did not label local_hash as scaffold behavior")
if status.get("qualityBackend"):
    raise SystemExit("semantic status marked local_hash as a quality backend")
if not status.get("localOnly"):
    raise SystemExit("semantic status did not report localOnly")
if not status.get("providerAvailable"):
    raise SystemExit("semantic status did not report default provider availability")
if status.get("cloudEmbeddingsAllowed") or status.get("cloudRerankingAllowed"):
    raise SystemExit("semantic status allowed cloud embeddings or reranking")
if status.get("privacyStatus", {}).get("remoteEmbeddingsUsed"):
    raise SystemExit("semantic status reported remote embeddings")

if not search:
    raise SystemExit("semantic search returned no results")
if search[0]["provider"]["provider"] != "local_hash":
    raise SystemExit("semantic search did not use the local_hash provider")
if search[0]["provider"].get("providerRole") != "deterministic_scaffold":
    raise SystemExit("semantic search did not expose scaffold provider role")
if search[0]["provider"].get("qualityBackend"):
    raise SystemExit("semantic search marked local_hash as a quality backend")
if search[0]["path"] != "src/payments/webhooks.ts":
    raise SystemExit(f"unexpected top semantic path: {search[0]['path']}")

semantic_signal = False
for candidate in plan.get("retrievalCandidates", []):
    for signal in candidate.get("signalScores", []):
        semantic_signal = semantic_signal or signal.get("signal") == "semantic"
    for evidence in candidate.get("evidence", []):
        semantic_signal = semantic_signal or evidence.get("signal") == "semantic"
if not semantic_signal:
    raise SystemExit("prepare-task did not surface semantic provenance")

if not pack.get("privacyStatus", {}).get("localOnly"):
    raise SystemExit("get-pack privacyStatus.localOnly was not true")
if pack.get("privacyStatus", {}).get("remoteEmbeddingsUsed"):
    raise SystemExit("get-pack reported remote embeddings")
if not evaluation.get("effectiveFilters", {}).get("semanticEnabled"):
    raise SystemExit("eval history did not record semanticEnabled")
if not evaluation.get("privacyStatus", {}).get("localOnly"):
    raise SystemExit("eval history privacyStatus.localOnly was not true")
PY

if grep -R -- "CTXPACK_SEMANTIC_SOURCE_SENTINEL" "$home" "$store" >/dev/null 2>&1; then
  echo "semantic smoke failed: source sentinel was persisted" >&2
  exit 1
fi
if grep -R -- "CTXPACK_SEMANTIC_SECRET_SENTINEL" "$home" "$store" >/dev/null 2>&1; then
  echo "semantic smoke failed: secret sentinel was persisted" >&2
  exit 1
fi

echo "semantic smoke passed"
