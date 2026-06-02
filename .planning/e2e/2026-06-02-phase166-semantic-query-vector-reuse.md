# Phase 166: Semantic Query Vector Reuse

Date: 2026-06-02

## Goal

Continue local semantic R&D after Phase 165 by reducing repeated fresh-process
`local_fastembed` search overhead. Phase 165 persisted document vectors and
made index failures loud, but each fresh CLI/MCP process still had to embed the
query and could trigger a second semantic-document pass while expanding stored
global candidates.

## Changes

- Added storage schema v4 with a source-free `semantic_query_vectors` table.
- Persisted query vectors by query hash, provider, model, dimensions, and
  distance metric.
- Kept raw query text out of SQLite; storage tests assert a query sentinel is
  absent from database bytes.
- Semantic search now reuses persisted query vectors before calling the
  embedding backend.
- `local_fastembed` stored-candidate expansion now reuses the first semantic
  document set instead of running a second large-repo document pass.
- Preserved the existing `local_hash` fallback path for stored global candidates.
- `ctxhelm storage status` now reports `semantic query vector records`.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Feature-enabled binary:

```bash
cargo build -p ctxhelm --features local-embeddings --locked
```

Fresh bounded seed:

```bash
rm -rf /tmp/ctxhelm-phase166-single-pass-home
mkdir -p /tmp/ctxhelm-phase166-single-pass-home
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase166-single-pass-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm index \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q
```

Observed:

```text
semantic vector records: 16
schema version: 4
real 55.65
```

Back-to-back fresh-process searches:

```bash
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase166-single-pass-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic \
  --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q \
  --limit 5
```

Observed:

```text
first search:  27.28s
second search: 12.08s
top result both times: src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
semantic vector records after write-through: 31
semantic query vector records after write-through: 1
```

Comparison against Phase 165:

```text
Phase 165 AllMini second fresh-process search: 16.92s
Phase 166 AllMini second fresh-process search: 12.08s
```

## Interpretation

This is a real latency improvement for repeated conceptual semantic searches in
fresh CLI/MCP processes. The result quality on the known RefactoringMiner probe
is unchanged, and the persistent query cache remains source-free because it
stores only a query hash and vector metadata.

Semantic retrieval is still not ready for default-on promotion. The same fixture
shows lexical search and semantic status still spend roughly 14-15s on the
large-repo safe inventory/status path, so the next R&D bottleneck is reducing
large-fixture inventory/search setup overhead rather than embedding the query
again.

## Validation

Focused checks:

```bash
cargo test -p ctxhelm-index semantic_search_reuses_persisted_query_vector --locked
cargo test -p ctxhelm-index semantic_search_adds_persisted_candidates_outside_lexical_prefilter --locked
cargo test -p ctxhelm-index semantic_search_reuses_persisted_query_vector --features local-embeddings --locked
cargo test -p ctxhelm-index persists_semantic_query_vectors_without_prompt_text --locked
cargo test -p ctxhelm-index migration_history_is_idempotent --locked
```

Final gates:

```bash
cargo fmt --check
bash scripts/check-release-docs.sh
cargo run -p ctxhelm --locked -- --help
cargo test -p ctxhelm-index --locked
cargo test -p ctxhelm local_semantic_provider_selection_is_source_free_and_policy_visible --locked
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
git diff --check
```
