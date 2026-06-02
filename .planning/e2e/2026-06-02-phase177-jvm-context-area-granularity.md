# Phase 177 - JVM Context-Area Granularity

## Goal

Make Phase 176 focused context-area guidance useful in Java/Kotlin repositories.
The RefactoringMiner proof showed that `area_context_only` misses were now
actionable, but the area resource was still too broad:

```text
ctxhelm://repo/context-area/src%2Fmain
```

For a Maven/Gradle repository with tens of thousands of files, `src/main` is not
a practical progressive read scope.

## Change

`context_area_for_path` now recognizes JVM source roots:

```text
src/main/java
src/test/java
src/main/kotlin
src/test/kotlin
```

For those paths, context areas use the source root plus up to four package
components. Ordinary paths keep the previous behavior:

```text
src/auth/session.ts -> src/auth
crates/ctxhelm/src/main.rs -> crates/ctxhelm
.github/workflows/ci.yml -> .github/workflows
```

Example JVM grouping:

```text
src/main/java/gr/uom/java/xmi/diff/UMLClassBaseDiff.java
  -> src/main/java/gr/uom/java/xmi
```

## Proof

Repository:

```text
/tmp/ctxhelm-rd/RefactoringMiner-phase176-fresh
```

Command:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase177-jvm-area-home \
  cargo run -p ctxhelm --locked -- eval history \
  --repo /tmp/ctxhelm-rd/RefactoringMiner-phase176-fresh \
  --base 3c276966e \
  --head 77760c1a \
  --limit 2 \
  --budget 10 \
  --format json \
  --force \
  > /tmp/ctxhelm-rd/phase177-refminer-jvm-context-areas.json
```

Metrics:

```text
fileRecallAt10: 0.75
sourceRecallAt10: 0.75
lexicalBaselineRecallAt10: 0.5833334
ctxhelmLiftAt10: 0.16666663
```

Before:

```text
signalGap: area_context_only
contextArea: src/main
contextAreaResourceUri: ctxhelm://repo/context-area/src%2Fmain
```

After:

```text
signalGap: area_context_only
contextArea: src/main/java/gr/uom/java/xmi
contextAreaResourceUri: ctxhelm://repo/context-area/src%2Fmain%2Fjava%2Fgr%2Fuom%2Fjava%2Fxmi
examplePath: src/main/java/gr/uom/java/xmi/diff/UMLClassBaseDiff.java
```

## Interpretation

This phase intentionally preserves target-file Recall@10. The remaining missed
file still lacks enough source-free task, lexical, graph, co-change, semantic,
or symbol evidence for honest top-10 promotion.

The improvement is that the progressive read resource is now scoped to the
relevant Java package area. An agent has a much smaller source-free resource to
inspect after the ranked target files are insufficient.

## Targeted Tests

```text
cargo test -p ctxhelm-core --locked context_area_resource_uri_round_trips_source_free_area_names -- --nocapture
cargo test -p ctxhelm-compiler --locked context_ -- --nocapture
```

Both targeted test runs passed.
