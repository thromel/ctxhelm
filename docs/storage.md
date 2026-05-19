# Storage

ctxpack uses local SQLite storage for durable, source-free repository intelligence.
The store is an implementation cache for agent-native workflows; it is not a
cloud sync layer and it does not make ctxpack an editing agent.

## Location

By default, storage lives under:

```bash
$CTXPACK_HOME/repos/<repo-id>/ctxpack.sqlite3
```

If `CTXPACK_HOME` is unset, ctxpack uses:

```bash
~/.ctxpack/repos/<repo-id>/ctxpack.sqlite3
```

Some commands also accept an explicit storage path.

## Initialize And Inspect

```bash
ctxpack storage init --repo "$REPO"
ctxpack storage status --repo "$REPO"
```

`storage init` creates the schema and records schema, ranking, compiler, ctxpack,
and privacy metadata. `storage status` reports compatibility and source-free
record counts.

## Incremental Index Sync

```bash
ctxpack index --repo "$REPO" --store
```

The first run creates file metadata records. Later runs compare safe path,
language, role, content hash, size, generated/ignored status, and policy-derived
inventory state. The output reports source-free counts for reused, created,
updated, deleted, skipped, generated, and sensitive paths.

## Semantic Vector Metadata

```bash
ctxpack index --repo "$REPO" --semantic
```

Semantic indexing is explicit and local-only. It stores provider name, model,
dimensions, distance metric, safe file hash, privacy label, and numeric vector
metadata for both the deterministic `local_hash` scaffold and optional
`local_fastembed` backend. It uses the same safe inventory and source-read policy
as packs, and it does not store raw file contents, prompt text, secrets, or cloud
payloads.

## Pack And Eval Metadata

```bash
ctxpack get-pack "fix auth redirect" --repo "$REPO" --store
ctxpack eval history --repo "$REPO" --limit 10 --store
ctxpack eval benchmark --config suite.json --store
ctxpack eval proof --config suite.json --store
```

These commands persist only source-free metadata: task hashes, pack IDs, budgets,
target agents, confidence, benchmark metrics, gap families, proof headlines, and
privacy status. They do not persist prompt text or source snippets by default.

## Memory Card Metadata

```bash
ctxpack cards generate --repo "$REPO"
ctxpack memory generate-experience --repo "$REPO"
ctxpack memory list --repo "$REPO"
```

Repo memory records store card IDs, kinds, titles, summaries, source-link paths,
input hashes, freshness, review status, disable state, confidence, reason, and
privacy labels. They remain source-free: no raw file contents, source snippets,
raw prompts, terminal logs, or model transcripts are persisted.

## Repair, Cleanup, And Reset

```bash
ctxpack storage repair --repo "$REPO"
ctxpack storage vacuum --repo "$REPO"
ctxpack storage reset --repo "$REPO"
ctxpack storage reset --repo "$REPO" --yes
```

`repair` is non-destructive and reinitializes missing schema metadata or tables
where possible. `vacuum` compacts the SQLite database. `reset` is a dry run unless
`--yes` is provided.

## Privacy Boundary

The storage schema stores hashes, paths that passed ctxpack policy, roles,
counts, metrics, IDs, and JSON metadata. It does not store raw file contents,
source snippets, prompt text, secrets, or cloud embedding data by default.

The release gate runs `scripts/smoke-storage.sh`, `scripts/smoke-memory.sh`, and
`scripts/smoke-semantic.sh`, which check repeated indexing, source-free memory
and vector metadata, and that source or secret sentinels are not persisted into
`CTXPACK_HOME`.
