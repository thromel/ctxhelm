# Phase 168: Semantic Alias And Noise Diagnostics

Date: 2026-06-02

## Goal

Continue semantic R&D after Phase 167 removed the large generated-tree latency
blocker. The question for this phase was whether production-local semantic
retrieval can contribute target files beyond lexical on the clean
RefactoringMiner proof, or whether the remaining misses are caused by
non-semantic coupling that should be handled by graph/history/fusion instead.

## Baseline Measurement

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Feature-enabled gate:

```bash
CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 target/debug/ctxhelm \
  eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 3 \
  --budget 10 \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase168-refminer-fastembed-gate-limit3.json
```

Observed before code changes, using the warmed Phase 167 cache:

```text
decision: hold
decisionReason: Held: runtime ratio 3.01x with recall delta +0.000; promotion requires stronger quality lift.
evaluatedCommits: 3
semanticSelectedFileCount: 5
semanticTargetHitCount: 3
semanticOnlyTargetHitCount: 0
semanticLexicalOverlapCount: 5
semanticMissedTargetCount: 3
local_semantic File Recall@10: 0.72222227
lexical/default File Recall@10: 0.72222227
wall time: 28.39s
```

The named local-semantic misses remained:

```text
77760c1a464c -> src/main/java/gr/uom/java/xmi/diff/UMLClassBaseDiff.java
eed843034a44 -> src/main/java/gr/uom/java/xmi/diff/UMLOperationDiff.java
```

## Changes

- Added source-free identifier aliases to semantic documents:
  - path aliases split camel-case/acronym identifiers such as
    `UMLOperationDiff` into `uml operation diff`
  - symbol aliases are included in normal semantic document reports when symbol
    facets are requested
- Expanded semantic query text with identifier aliases before local embedding.
- Versioned semantic query text and semantic document vector hashes so old
  persisted vectors are not reused after the render-text contract changes.
- Added semantic contribution diagnostics for unique semantic non-targets:
  - `semanticOnlyNonTargetCount`
  - `semanticOnlyNonTargetRate`
  - `semanticOnlyNonTargets`
  - `semantic_contribution_unique_non_targets`

## Rejected Experiment

I tried enabling full symbol facets directly inside semantic search. That was
too expensive on RefactoringMiner parent snapshots: the 3-commit gate ran for
more than 4 minutes without producing a report and was stopped. This showed that
symbol-enriched semantic search needs a cached/bounded design before it can be
used in the eval loop.

The final implementation keeps semantic search on path/query aliases only and
leaves full symbol facets available through `semantic document` generation.

## RefactoringMiner Proof

Fresh alias-only gate:

```bash
rm -rf /tmp/ctxhelm-phase168-alias-semantic-home
mkdir -p /tmp/ctxhelm-phase168-alias-semantic-home

/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase168-alias-semantic-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 3 \
  --budget 10 \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase168-refminer-alias-semantic-gate-limit3.json
```

Observed:

```text
decision: hold
decisionReason: Held: local semantic recall delta +0.000, precision delta +0.000; keep opt-in.
local_semantic File Recall@10: 0.72222227
lexical/default File Recall@10: 0.72222227
semanticOnlyTargetHitCount: 0
semanticOnlyNonTargetCount: 1
semanticOnlyNonTargetRate: 1.0
wall time: 203.45s
```

Warmed diagnostic gate:

```bash
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase168-alias-semantic-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm eval gate \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --limit 3 \
  --budget 10 \
  --semantic-provider local_fastembed \
  --format json > /tmp/ctxhelm-phase168-refminer-alias-semantic-gate-limit3-warm.json
```

Observed:

```text
decision: hold
local_semantic File Recall@10: 0.72222227
lexical/default File Recall@10: 0.72222227
semanticOnlyTargetHitCount: 0
semanticOnlyNonTargetCount: 1
semanticOnlyNonTargetRate: 1.0
semanticOnlyNonTargets:
  77760c1a464c -> src/test/java/org/refactoringminer/astDiff/tests/TypeScriptDiffTest.java
local_semantic runtimeMillis: 6082
default runtimeMillis: 5355
wall time: 21.20s
```

Diagnostics now include both:

```text
semantic_contribution_no_unique_target_hits
semantic_contribution_unique_non_targets
```

## Interpretation

This phase does not justify promoting semantic retrieval. The added source-free
aliases made the semantic text more meaningful and the diagnostics more honest,
but the measured RefactoringMiner target-file recall stayed flat. The only
semantic-only file was a non-target test, while the remaining misses were
coupled source files (`UMLClassBaseDiff.java`, `UMLOperationDiff.java`) that are
better handled by graph/history/fusion improvements than by more embedding text.

Next R&D should target:

1. cached/bounded symbol-enriched semantic documents if we want symbol-level
   semantic quality without multi-minute eval runs;
2. graph/history expansion from high-confidence hits for coupled source files;
3. diagnostics that separate semantic target misses caused by coupling from
   misses caused by weak semantic text.

## Validation

Focused checks:

```bash
cargo fmt --check
cargo test -p ctxhelm-index semantic --locked
cargo test -p ctxhelm-compiler semantic_contribution --locked
cargo build -p ctxhelm --features local-embeddings --locked
```

Final gates are recorded in the Phase 168 closeout after full validation.

Final gates:

```bash
cargo fmt --check
cargo test -p ctxhelm-index semantic --locked
cargo test -p ctxhelm-compiler semantic_contribution --locked
bash scripts/check-release-docs.sh
cargo run -p ctxhelm --locked -- --help >/tmp/ctxhelm-phase168-help.txt
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
git diff --check
```

Result: all passed.
