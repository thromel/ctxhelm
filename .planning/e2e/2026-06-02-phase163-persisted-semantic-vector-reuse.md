# Phase 163: Persisted Semantic Vector Reuse

Date: 2026-06-02

## Goal

Reduce `local_fastembed` fresh-process overhead by reusing persisted source-free
document vectors instead of embedding every semantic candidate document on each
query.

## Changes

- Added a read-only semantic vector loader for SQLite storage.
- Changed semantic search to reuse exact path/safe-hash/provider/model/dimension
  vector matches and embed only query plus document misses.
- Added `ctxhelm index --semantic-limit` so large repositories can seed bounded
  semantic vectors.
- Tightened semantic vector upsert behavior to the schema uniqueness key
  `repo_id + path + provider + model`, so changed safe hashes update rows instead
  of failing or leaving duplicate stale vectors.
- Added tests proving source-free storage readback, hash-change update behavior,
  and semantic search reuse diagnostics.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Fresh semantic store:

```bash
rm -rf /tmp/ctxhelm-phase163-home
mkdir -p /tmp/ctxhelm-phase163-home
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase163-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  cargo run -p ctxhelm --features local-embeddings --locked -- \
  index --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed --semantic-limit 16
```

Result:

```text
Semantic storage sync
- created records: 16
- semantic vector records: 16
real 42.26
```

Seeded-store semantic search:

```bash
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase163-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed --limit 5
```

Result:

```text
real 6.03
top result: src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
```

Empty-store comparison:

```bash
rm -rf /tmp/ctxhelm-phase163-empty-home
mkdir -p /tmp/ctxhelm-phase163-empty-home
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase163-empty-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed --limit 5
```

Result:

```text
real 10.56
top result: src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
```

## Interpretation

Persisted source-free vectors reduce direct semantic search latency for this
RefactoringMiner query by avoiding repeated document embeddings across CLI
processes. The remaining cost is mostly local model/query initialization and
candidate misses. This phase improves runtime, but it does not prove
semantic-only recall lift; Phase 162's `semanticContribution` gate still showed
no unique target hits for the measured task.

## Focused Validation

```bash
cargo test -p ctxhelm-index semantic --locked
cargo check -p ctxhelm --features local-embeddings --locked
```

Both passed before the full validation stack.
