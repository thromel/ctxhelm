# Phase 222 E2E: Memory Signal Ablation Suite

Date: 2026-06-05

## Goal

Measure experience-memory reuse against graph and semantic retrieval signals on
real repeated-file history pairs, without storing source text, raw task text,
repo paths, prompts, transcripts, or MCP traffic.

## Command

```bash
CTXHELM_BIN="$PWD/target/debug/ctxhelm" \
  scripts/measure-memory-generalization-suite.sh \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/VeriSchema \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ReAgent \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/ctxhelm \
  --pairs 1 \
  --scan-commits 120 \
  --semantic \
  --semantic-provider local_hash \
  --output .ctxhelm/e2e/phase222-memory-signal-ablation-suite.json
```

## Artifact

- `.ctxhelm/e2e/phase222-memory-signal-ablation-suite.json`

## Result

The suite evaluated four repositories and four repeated-file pairs with local
semantic retrieval enabled:

- `memoryUniqueLiftPairs = 1`
- `memoryUniqueTargetHitCount = 1`
- `memoryUniqueNonTargetCount = 1`
- `memoryTargetHitsWithGraphSupportUpperBound = 4`
- `memoryTargetHitsWithSemanticSupportUpperBound = 3`
- `memoryUniqueTargetsWithGraphOrSemanticSupportUpperBound = 1`
- `memoryTargetHitsWithoutGraphOrSemanticSupportLowerBound = 1`
- `semanticSelectedTargetPairs = 2`
- `semanticAblationLiftPairs = 0`
- `graphEdgeAblationRemovedTargetHitCount = 0`

Interpretation:

- `semanticMeasured = true`
- `semanticUsefulForMemoryTasks = true`
- `graphCorroborationMeasured = true`
- `generalizationProven = false`
- `precisionNeedsWork = true`
- `memoryNeedsCorroboration = true`

## Conclusion

Phase 222 closes the measurement gap from Phase 221: memory can now be compared
against graph and semantic evidence in the same source-free repeated-history
suite. The result is intentionally not promoted as a quality win. Semantic
selected target evidence appears on two repositories, but semantic ablation lift
is zero on this four-pair probe, and memory still produces one unique non-target
plus one lower-bound uncorroborated target hit. The next R&D slice should test a
stricter memory-candidate corroboration policy and increase pair counts.
