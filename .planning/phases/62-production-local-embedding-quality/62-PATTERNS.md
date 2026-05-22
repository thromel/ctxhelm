# Phase 62 Patterns

## Same Corpus, New Variant

Use the Phase 61 corpus before changing retrieval policy. If the corpus changes at the same time as the embedding variant, quality deltas are not interpretable.

## Local First Means Local

Do not use cloud embeddings or cloud rerankers in Phase 62. Any model download/cache behavior must be explicit and reported.

## Runtime Is A Quality Metric

A Recall@10 gain that doubles runtime may still be a bad default. Always report runtime and cache state alongside recall.

## Keep `local_hash` Honest

`local_hash` is deterministic scaffold behavior. Keep it useful for tests, but label it clearly and do not treat it as semantic-quality evidence.
