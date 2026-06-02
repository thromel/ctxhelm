# Phase 169: Graph Ordering And Context Balance

Date: 2026-06-02

## Goal

Fix the RefactoringMiner coupled-source miss where a selected source file had a direct dependency edge to another changed source file, but ctxhelm failed to place that dependency in the top-10 context pack.

## Evidence Source

- Repository: `/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean`
- Eval range: `3c276966e..77760c1a`
- Commits evaluated: `2`
- Ranking budget: `10`
- Final local proof JSON: `.ctxhelm/e2e/phase169-graph-ordering-refminer-proof.json`

## Before

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase169-eval-home \
  cargo run -p ctxhelm --locked -- eval history \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --base 3c276966e --head 77760c1a --limit 2 --budget 10 --format json --force
```

Results:

- File Recall@10: `0.5833334`
- Source Recall@10: `0.5833334`
- Lexical baseline Recall@10: `0.5833334`
- Lift vs lexical: `0.0`
- Gap summaries:
  - `area_context_only`: `UMLClassBaseDiff.java`
  - `ranked_below_budget_dependency`: `UMLOperationDiff.java`

The official eval selected `UMLOperationDiff.java` neither as a top-10 context file nor as a hit, even though a direct import edge from `UMLOperationBodyMapper.java` existed.

## Diagnosis

The first probe showed `TypeScriptVisitor.java` had no useful graph or co-change signal to `UMLClassBaseDiff.java`; that miss remains an `area_context_only` gap and should not be papered over with a heuristic.

The second probe showed `UMLOperationBodyMapper.java` had a direct Java import edge to `UMLOperationDiff.java`. Two ranking issues suppressed it:

1. Related dependency edges ranked broad incoming importers before outgoing imports from the selected source seed.
2. Dependency candidates lost the retriever order during ranking because all dependency edges had the same score, then path-order tie-breaking favored generic imports such as `Constants.java`.
3. The eval context pack reserved three validation-test slots in a 10-item pack, dropping the selected source file at slot 8.

## Change

- Related dependency retrieval now prefers outgoing seed imports before incoming importers.
- Related dependency ordering now uses a source-free identifier-token affinity tie-breaker, so `UMLOperationBodyMapper.java` -> `UMLOperationDiff.java` ranks ahead of generic imports.
- Ranking preserves dependency retriever order with a tiny score decay by dependency rank.
- Standard-scope target selection reserves a bounded source-dependency slot instead of letting lexical-only files fill the whole source budget.
- Validation-test reserve for 10-item eval context packs is capped at one-quarter of the budget, preserving selected source evidence at slot 8 while keeping targeted tests.

## After

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase169-eval-home-balanced \
  cargo run -p ctxhelm --locked -- eval history \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --base 3c276966e --head 77760c1a --limit 2 --budget 10 --format json --force
```

Results:

- File Recall@10: `0.75`
- Source Recall@10: `0.75`
- Lexical baseline Recall@10: `0.5833334`
- Lift vs lexical: `+0.16666663`
- `ranked_below_budget_dependency` gap: removed
- Remaining gap:
  - `area_context_only`: `UMLClassBaseDiff.java`

For commit `eed843034a443346d895d89e52c1351591a3a39e`, the final top-10 context now includes:

- `src/main/java/gr/uom/java/xmi/decomposition/UMLOperationBodyMapper.java`
- `src/main/java/gr/uom/java/xmi/diff/UMLOperationDiff.java`
- `src/main/java/org/refactoringminer/astDiff/matchers/wrappers/BodyMapperMatcher.java`

## Validation

Focused validation completed:

```bash
cargo test -p ctxhelm-index related_dependency_edges --locked
cargo test -p ctxhelm-compiler ranking::tests:: --locked
cargo test -p ctxhelm-compiler context_ranking_keeps_validation_tests_inside_budget --locked
```

Full release validation remains required before closeout.
