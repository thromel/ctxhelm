# Phase 110: Clean Cold Fixture Proof

Date: 2026-05-31

## Goal

Make the four-repo production proof depend on clean detached fixtures instead of dirty interactive checkouts, and prevent stale parent-snapshot caches from producing false retrieval evidence.

## Changes

- Added fail-closed parent snapshot extraction: incomplete object extraction now reports `git_object_store_unavailable` instead of leaving a reusable partial snapshot.
- Replaced sidecar-only parent snapshot reuse with manifest-based reuse.
- Bumped the parent snapshot schema to force fresh clean snapshots after the stale-cache bug.
- Evicted the global repo cache when replacing a parent snapshot directory, so immutable eval snapshots cannot reuse an inventory from previous contents.
- Enabled parent-bounded co-change ranking from the source-free eval sidecar. The sidecar is written at the parent revision, so it exercises production history retrieval without target-commit leakage.
- Tightened symbol facet detection so commit-subject acronyms and title-case verbs do not trigger whole-repo symbol extraction.
- Added `scripts/prepare-proof-fixtures.sh` and a committed clean fixture config.

## Evidence

Command:

```bash
scripts/prepare-proof-fixtures.sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/ctxpack-phase110-target \
  cargo run -p ctxpack -- eval proof \
  --config .planning/e2e/2026-05-31-phase110-clean-cold-fixture-config.json \
  --format json > .ctxpack/e2e/phase110-clean-fixture-proof.json
```

Proof artifact:

- `.ctxpack/e2e/phase110-clean-fixture-proof.json`

Result:

- `releaseGate.decision = promote`
- RefactoringMiner: `match`, Context Recall@10 `1.0`, lexical context `1.0`, Test Recall@10 `1.0`, runtime `9254ms`
- ctxpack: `beat`, Context Recall@10 `0.3889`, lexical context `0.3056`, runtime `6331ms`
- ReAgent: `beat`, Context Recall@10 `1.0`, lexical context `0.2857`, Test Recall@10 `1.0`, runtime `12043ms`
- VeriSchema: `beat`, Context Recall@10 `0.2055`, lexical context `0.0822`, effective validation recall `1.0`

## Rejected / Diagnosed Runs

- Partial blobless fixtures now fail closed as `git_object_store_unavailable` instead of producing empty `retrievalTargetFiles`.
- Full fixtures with stale parent-snapshot inventory caches blocked on quality with zero recall for RefactoringMiner, ctxpack, and ReAgent. That was a cache correctness bug.
- Fresh snapshots before the symbol-facet fix restored quality but blocked on cold runtime: RefactoringMiner took about `64.8s` because commit-subject acronyms forced whole-repo symbol extraction.

## Remaining Work

The clean cold fixture proof now promotes, but production readiness is not complete. Next work should focus on keeping this proof in the packaged release gate, refreshing real-client evidence when client versions change, and continuing broad context-area/ranking work without weakening the source-free proof contract.
