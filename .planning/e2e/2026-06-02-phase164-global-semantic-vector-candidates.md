# Phase 164: Global Semantic Vector Candidates And Write-Through

Date: 2026-06-02

## Goal

Make persisted local semantic vectors more useful than a lexical-prefilter cache.
Phase 163 reused vectors only for documents that already survived the query
prefilter. That improved some repeated work, but it could not create
semantic-only target hits.

## Changes

- Semantic search now unions in matching persisted vectors from the local store
  even when those documents sit outside the lexical prefilter.
- Embedded candidate misses are written through to the source-free vector store
  for reuse by later fresh CLI/MCP processes.
- Storage now distinguishes full semantic-index replacement from incremental
  vector upsert, so search write-through does not prune vectors created by
  `ctxhelm index --semantic`.
- Foreground `local_fastembed` index default is bounded to 16 vectors. Larger
  local vector jobs remain available through explicit `--semantic-limit`.

## Negative R&D Evidence

A full synchronous RefactoringMiner `local_fastembed` seed was attempted with
`--semantic-limit 700` on the clean 647-file fixture. It was stopped after more
than 9 minutes without completing. A 128-vector foreground default was also
attempted and stopped after more than 4 minutes. This is not an acceptable
interactive-agent default.

Source-free symbol-enriched semantic documents were also tested. They regressed
the known RefactoringMiner query: `Improvement in TypeScriptVisitor` took
`64.10s` and no longer ranked `TypeScriptVisitor.java` first. The symbol
enrichment experiment was reverted for this phase.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Seed command:

```bash
rm -rf /tmp/ctxhelm-phase164-writethrough2-home
mkdir -p /tmp/ctxhelm-phase164-writethrough2-home
env CTXHELM_HOME=/tmp/ctxhelm-phase164-writethrough2-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm index \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed
```

Result:

```text
semantic vector records: 16
```

Back-to-back fresh-process searches:

```bash
/usr/bin/time -p env CTXHELM_HOME=/tmp/ctxhelm-phase164-writethrough2-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed --limit 5

/usr/bin/time -p env CTXHELM_HOME=/tmp/ctxhelm-phase164-writethrough2-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed --limit 5
```

Observed:

```text
first search:  31.48s
second search: 20.86s
top result both times: src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
semantic vector records after write-through: 31
```

## Interpretation

This phase makes the local vector store behave more like an actual semantic
index: persisted vectors can compete globally instead of only caching documents
that lexical prefiltering already found, and repeated searches add source-free
candidate vectors incrementally.

It still does not prove `local_fastembed` promotion. Fresh-process query/model
cost remains high, and Phase 162 still showed no semantic-only target hit on the
gate sample. The next R&D frontier is either asynchronous/background vector
jobs, a smaller/faster local model, or learned task-conditioned semantic query
construction that produces measurable unique target hits.

## Validation

Focused checks run during the phase:

```bash
cargo test -p ctxhelm-index semantic --locked
cargo test -p ctxhelm-index upserts_semantic_vectors_without_pruning_existing_index --locked
cargo test -p ctxhelm local_fastembed_index_limit_defaults_to_bounded_foreground_seed --locked
cargo check -p ctxhelm --features local-embeddings --locked
```
