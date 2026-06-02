# Phase 165: Fastembed Default And Loud Index Errors

Date: 2026-06-02

## Goal

Continue the local embedding R&D loop after Phase 164. The remaining measured
problem was fresh-process local embedding cost. The next hypothesis was that the
faster `AllMiniLML6V2Q` backend could preserve useful ranking while reducing
query overhead compared with the previous Jina default.

## Findings

The first `AllMiniLML6V2Q` run exposed a correctness bug: semantic vector
indexing swallowed provider failures with `unwrap_or_default()` and reported a
successful zero-vector index. In a feature-enabled build, the documented model id
also failed through the generic `fastembed::EmbeddingModel::from_str` path.

## Changes

- Semantic vector indexing now returns a hard error when the requested provider
  is unavailable or embedding fails.
- Added explicit model-id mapping for documented local fastembed ids:
  `AllMiniLML6V2`, `AllMiniLML6V2Q`, `AllMiniLML12V2`,
  `AllMiniLML12V2Q`, and `JinaEmbeddingsV2BaseCode`.
- Changed the default `local_fastembed` model from
  `JinaEmbeddingsV2BaseCode`/768 dimensions to `AllMiniLML6V2Q`/384
  dimensions.
- Kept Jina available through explicit
  `--semantic-model JinaEmbeddingsV2BaseCode`.

## RefactoringMiner Proof

Fixture:

```text
/Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean
```

Feature-enabled default status:

```bash
env CTXHELM_HOME=/tmp/ctxhelm-phase165-allmini-fixed-home \
  cargo run -q -p ctxhelm --features local-embeddings --locked -- \
  semantic status \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic-provider local_fastembed \
  --format json
```

Observed:

```json
{
  "providerKind": "local_fastembed",
  "modelId": "AllMiniLML6V2Q",
  "dimensions": 384,
  "providerAvailable": true,
  "storedVectorCount": 31
}
```

AllMini default seed:

```bash
rm -rf /tmp/ctxhelm-phase165-allmini-fixed-home
mkdir -p /tmp/ctxhelm-phase165-allmini-fixed-home
/usr/bin/time -p env \
  CTXHELM_HOME=/tmp/ctxhelm-phase165-allmini-fixed-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  cargo run -p ctxhelm --features local-embeddings --locked -- \
  index --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q
```

Observed:

```text
semantic vector records: 16
real 63.41
```

Back-to-back fresh-process searches:

```bash
/usr/bin/time -p env CTXHELM_HOME=/tmp/ctxhelm-phase165-allmini-fixed-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q --limit 5

/usr/bin/time -p env CTXHELM_HOME=/tmp/ctxhelm-phase165-allmini-fixed-home \
  CTXHELM_FASTEMBED_DOCUMENT_LIMIT=16 \
  target/debug/ctxhelm search "Improvement in TypeScriptVisitor" \
  --repo /Users/romel/Documents/GitHub/ctxhelm-proof-fixtures/RefactoringMiner-phase157-clean \
  --semantic --semantic-provider local_fastembed \
  --semantic-model AllMiniLML6V2Q --limit 5
```

Observed:

```text
first search:  33.15s
second search: 16.92s
top result both times: src/main/java/gr/uom/java/xmi/decomposition/TypeScriptVisitor.java
semantic vector records after write-through: 31
```

Comparison against Phase 164 Jina proof:

```text
Jina second fresh-process search:    20.86s
AllMini second fresh-process search: 16.92s
```

## Interpretation

AllMini is a more practical default for local-first interactive use. It is still
not fast enough to promote semantic retrieval as a default path, but it preserves
the known RefactoringMiner top result and lowers repeated query overhead.

The more important correctness fix is that semantic index failures now fail
loudly. R&D measurements can no longer accidentally treat an unavailable or
misconfigured embedding model as a successful zero-vector semantic index.

## Validation

Focused checks run during the phase:

```bash
cargo test -p ctxhelm-index semantic --locked
cargo test -p ctxhelm-index semantic --features local-embeddings --locked
cargo test -p ctxhelm local_semantic_provider_selection_is_source_free_and_policy_visible --locked
```

Final gates:

```bash
cargo fmt --check
bash scripts/check-release-docs.sh
cargo run -p ctxhelm --locked -- --help
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
git diff --check
```
