# Phase 175 - Semantic Missed Target Gap Families

## Goal

Close the Phase 168 semantic R&D diagnostic gap. Semantic gates already reported
that `local_fastembed` had zero semantic-only target hits and some unique
non-targets, but they did not explain whether semantic-missed targets were
better addressed by graph/history/fusion or by improving semantic document/query
text.

## Changes

- Added `semanticContribution.semanticMissedTargetGapFamilies` to
  `SemanticContributionSummary`.
- Added source-free semantic diagnostics:
  - `semantic_contribution_missed_targets_coupled`
  - `semantic_contribution_missed_targets_no_signal`
- Added Markdown rendering for semantic missed-target gap families in
  `ctxhelm eval gate`.
- Documented the new fields in `docs/benchmarking.md` and `docs/semantic.md`.

## RefactoringMiner Proof

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase175-semantic-gap-home \
  cargo run -p ctxhelm --locked -- eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 2 \
  --budget 10 \
  --semantic-provider local_hash \
  --format json > /tmp/ctxhelm-rd/phase175-semantic-missed-gap-gate.json
```

Result:

```text
decision: hold
decisionReason: Held: local semantic recall delta +0.000, precision delta +0.000; keep opt-in.
semanticMissedTargetGapFamilies:
  semantic_miss_area_context_only:
    missedCount: 1
    examplePaths:
      - src/main/java/gr/uom/java/xmi/diff/UMLClassBaseDiff.java
semantic diagnostics:
  semantic_contribution_no_unique_target_hits
  semantic_contribution_unique_non_targets
```

Interpretation:

The current clean RefactoringMiner semantic gate still does not justify default
semantic promotion. The remaining semantic missed target is an area-context-only
miss, while semantic-only additions are non-targets. That points the next
quality work toward graph/history/fusion and broad-area ranking, not broader
embedding text alone.

## Focused Validation

```bash
cargo test -p ctxhelm-compiler semantic_contribution --locked -- --nocapture
```

## Full Validation

```bash
cargo fmt --check
bash scripts/check-release-docs.sh
cargo run -p ctxhelm --locked -- --help
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
git diff --check
```
