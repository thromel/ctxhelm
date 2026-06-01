#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

ctxhelm_bin="${CTXHELM_BIN:-${ROOT_DIR}/target/debug/ctxhelm}"
if [[ ! -x "$ctxhelm_bin" ]]; then
  (cd "$ROOT_DIR" && cargo build -q -p ctxhelm)
fi

repo="$TMP_DIR/repo"
mkdir -p "$repo/src/auth" "$repo/tests/auth"
git -C "$repo" init >/dev/null
git -C "$repo" config user.email ctxhelm@example.com
git -C "$repo" config user.name ctxhelm

cat >"$repo/src/auth/session.ts" <<'TS'
export function requireSession(user?: string) {
  return user ?? null;
}
TS
cat >"$repo/tests/auth/session.test.ts" <<'TS'
import { requireSession } from "../../src/auth/session";
test("session", () => requireSession("alice"));
TS
git -C "$repo" add .
git -C "$repo" commit -m "add auth session baseline" >/dev/null

cat >>"$repo/src/auth/session.ts" <<'TS'
export function requireAdmin(role?: string) {
  return role === "admin";
}
TS
git -C "$repo" add .
git -C "$repo" commit -m "add requireAdmin auth gate" >/dev/null

gate_json="$TMP_DIR/v24-gate.json"
"$ctxhelm_bin" eval gate \
  --repo "$repo" \
  --limit 2 \
  --budget 10 \
  --format json >"$gate_json"

python3 - "$gate_json" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    report = json.load(handle)

if report.get("decision") not in {"promote", "hold", "block"}:
    raise SystemExit("v2.4 gate missing decision")
if not report.get("privacyStatus", {}).get("localOnly"):
    raise SystemExit("v2.4 gate privacyStatus.localOnly was not true")
if report.get("sourceTextLogged"):
    raise SystemExit("v2.4 gate logged source text")
variants = {variant.get("name"): variant for variant in report.get("variants", [])}
for required in [
    "lexical_baseline",
    "ctxhelm_default",
    "local_semantic",
    "precision_enriched_semantic",
    "semantic_precision_full_hybrid",
    "policy_allowed_reranked",
]:
    if required not in variants:
        raise SystemExit(f"v2.4 gate missing variant {required}")
if variants["policy_allowed_reranked"].get("status") != "skipped":
    raise SystemExit("policy reranked variant should be skipped by default")
policy = report.get("providerPolicy", {}).get("policy", {})
if policy.get("allowCloudEmbeddings") or policy.get("allowCloudReranking") or policy.get("allowSourceTransfer"):
    raise SystemExit("v2.4 gate allowed unsafe cloud/source policy")
if "precisionStatus" not in report:
    raise SystemExit("v2.4 gate missing precisionStatus")
for key in ["namedWins", "namedRegressions", "namedMisses"]:
    if key not in report:
        raise SystemExit(f"v2.4 gate missing {key}")
PY

if grep -F "requireSession(user" "$gate_json" >/dev/null; then
  echo "smoke-v24-gate failed: source body leaked into gate report" >&2
  exit 1
fi

echo "v2.4 semantic/precision gate smoke passed"
