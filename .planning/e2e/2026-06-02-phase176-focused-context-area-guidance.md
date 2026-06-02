# Phase 176 - Focused Context-Area Guidance

## Goal

Turn the remaining RefactoringMiner `area_context_only` miss into an actionable
agent-facing guidance surface without pretending the exact file had enough
source-free signal to enter the top-10 ranking.

## Change

- `prepare_task` now emits focused context areas for standard tasks when:
  - the selected files include source-like paths in an area, and
  - the same area has unselected source-like candidates worth progressive
    native reads.
- Broad tasks still keep the broader context-area behavior.
- Eval ranking no longer uses `plan.context_areas.is_empty()` to decide whether
  to reserve validation-test slots. The reserve is now tied to whether the task
  is multi-area, so focused context-area guidance cannot silently displace tests
  from Recall@10.

## Fresh RefactoringMiner Proof

Repository:

```text
/tmp/ctxhelm-rd/RefactoringMiner-phase176-fresh
```

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase176-focused-area-home \
  cargo run -p ctxhelm --locked -- eval history \
  --repo /tmp/ctxhelm-rd/RefactoringMiner-phase176-fresh \
  --base 3c276966e \
  --head 77760c1a \
  --limit 2 \
  --budget 10 \
  --format json \
  --force \
  > /tmp/ctxhelm-rd/phase176-refminer-focused-context-areas.json
```

Result:

```text
evaluatedCommits: 2
fileRecallAt10: 0.75
sourceRecallAt10: 0.75
lexicalBaselineRecallAt10: 0.5833334
ctxhelmLiftAt10: 0.16666663
```

Remaining gap:

```text
signalGap: area_context_only
pathFamily: src/main/java/gr/uom/java/xmi/diff/*.java
examplePath: src/main/java/gr/uom/java/xmi/diff/UMLClassBaseDiff.java
contextAreaResourceUri: ctxhelm://repo/context-area/src%2Fmain
```

The first evaluated commit now includes:

```text
contextAreas[0].area: src/main
contextAreas[0].reason: Focused task candidate area with 28 candidate path(s), 10 selected path(s), and 19 source-like next-read candidate(s).
contextAreas[0].resourceUri: ctxhelm://repo/context-area/src%2Fmain
```

## Interpretation

This phase does not improve Recall@10, and that is intentional. The missed
`UMLClassBaseDiff.java` file lacks direct lexical, semantic, dependency,
co-change, or symbol signal from the source-free task prompt. Forcing it into
the top 10 would overfit a historical commit.

The product improvement is that agents now receive a focused progressive
context-area resource for this kind of miss. After reading the ranked target
files, an agent can inspect `ctxhelm://repo/context-area/src%2Fmain` to explore
safe inventory families and next-read batches in the relevant area instead of
falling back to broad repo search.

## Targeted Tests

```text
cargo test -p ctxhelm-compiler --locked focused_context_areas_keep_selected_source_area_hints_only -- --nocapture
cargo test -p ctxhelm-compiler --locked context_ -- --nocapture
```

Both targeted test runs passed.
