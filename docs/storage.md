# Storage

ctxhelm uses local SQLite storage for durable, source-free repository intelligence.
The store is an implementation cache for agent-native workflows; it is not a
cloud sync layer and it does not make ctxhelm an editing agent.

## Location

By default, storage lives under:

```bash
$CTXHELM_HOME/repos/<repo-id>/ctxhelm.sqlite3
```

If `CTXHELM_HOME` is unset, ctxhelm uses:

```bash
~/.ctxhelm/repos/<repo-id>/ctxhelm.sqlite3
```

Some commands also accept an explicit storage path.

## Initialize And Inspect

```bash
ctxhelm storage init --repo "$REPO"
ctxhelm storage status --repo "$REPO"
```

`storage init` creates the schema and records schema, ranking, compiler, ctxhelm,
and privacy metadata. `storage status` reports compatibility and source-free
record counts.

## Incremental Index Sync

```bash
ctxhelm index --repo "$REPO" --store
```

The first run creates file metadata records. Later runs compare safe path,
language, role, content hash, size, generated/ignored status, and policy-derived
inventory state. The output reports source-free counts for reused, created,
updated, deleted, skipped, generated, and sensitive paths.

## Semantic Vector Metadata

```bash
ctxhelm index --repo "$REPO" --semantic
```

Semantic indexing is explicit and local-only. It stores provider name, model,
dimensions, distance metric, safe file hash, privacy label, and numeric vector
metadata for both the deterministic `local_hash` scaffold and optional
`local_fastembed` backend. It uses the same safe inventory and source-read policy
as packs, and it does not store raw file contents, prompt text, secrets, or cloud
payloads.

## Pack And Eval Metadata

```bash
ctxhelm get-pack "fix auth redirect" --repo "$REPO" --store
ctxhelm eval history --repo "$REPO" --limit 10 --store
ctxhelm eval benchmark --config suite.json --store
ctxhelm eval proof --config suite.json --store
```

These commands persist only source-free metadata: task hashes, pack IDs, budgets,
target agents, confidence, benchmark metrics, gap families, proof headlines, and
privacy status. They do not persist prompt text or source snippets by default.

## Semantic Vector Metadata

```bash
ctxhelm index --repo "$REPO" --semantic --semantic-provider local_fastembed --semantic-limit 128
ctxhelm semantic status --repo "$REPO" --semantic-provider local_fastembed --format json
```

Semantic vector storage is source-free. Rows contain path, safe hash, provider,
model, dimensions, distance metric, privacy label, and numeric vector JSON, not
raw source text. Search reuses only exact path/hash/provider/model/dimension
matches and falls back to embedding candidate misses. The schema uniqueness key
is `repo_id + path + provider + model`, so changed file hashes update the row
instead of leaving duplicate stale vectors behind.

## Memory Card Metadata

```bash
ctxhelm cards generate --repo "$REPO"
ctxhelm memory generate-experience --repo "$REPO"
ctxhelm memory list --repo "$REPO"
```

Repo memory records store card IDs, kinds, titles, summaries, source-link paths,
input hashes, freshness, review status, disable state, confidence, reason, and
privacy labels. They remain source-free: no raw file contents, source snippets,
raw prompts, terminal logs, or model transcripts are persisted.

## Repair, Cleanup, And Reset

```bash
ctxhelm storage repair --repo "$REPO"
ctxhelm storage vacuum --repo "$REPO"
ctxhelm storage reset --repo "$REPO"
ctxhelm storage reset --repo "$REPO" --yes
```

`repair` is non-destructive and reinitializes missing schema metadata or tables
where possible. `vacuum` compacts the SQLite database. `reset` is a dry run unless
`--yes` is provided.

## Privacy Boundary

The storage schema stores hashes, paths that passed ctxhelm policy, roles,
counts, metrics, IDs, and JSON metadata. It does not store raw file contents,
source snippets, prompt text, secrets, or cloud embedding data by default.

The release gate runs `scripts/smoke-storage.sh`, `scripts/smoke-memory.sh`, and
`scripts/smoke-semantic.sh`, which check repeated indexing, source-free memory
and vector metadata, and that source or secret sentinels are not persisted into
`CTXHELM_HOME`.
