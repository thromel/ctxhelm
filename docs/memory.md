# Repo Memory

ctxpack memory is a local, source-free layer for durable repo lessons. It is
selected per task and budget; it is not injected into every prompt.

## Generate Domain Memory

```bash
ctxpack cards generate --repo "$REPO"
```

This writes `.ctxpack/cards/*.md` and stores matching source-free memory-card
metadata in local SQLite. Domain cards include IDs, titles, summaries, source
links, input hashes, freshness, review status, confidence, and privacy status.
They summarize safe inventory, tests, dependency edges, and path domains. They
do not store raw file contents, source snippets, secrets, or raw prompts.

## Generate Experience Memory

```bash
ctxpack memory generate-experience --repo "$REPO" --limit 20
ctxpack memory list --repo "$REPO"
ctxpack memory show experience:<task-hash> --repo "$REPO"
ctxpack memory regenerate --repo "$REPO"
```

Experience cards are derived from local eval trace metadata only: task hashes,
task type, target agent, budget, recommended files, recommended tests, and
recommended commands. They do not store raw prompts, terminal logs, source text,
or model transcripts. Repo memory does not store raw prompts.

Experience cards default to `pending` review. Pending, rejected, disabled,
stale, or degraded cards are blocked from pack inclusion by default.

## Review Controls

```bash
ctxpack memory approve experience:<task-hash> --repo "$REPO"
ctxpack memory reject experience:<task-hash> --repo "$REPO"
ctxpack memory disable experience:<task-hash> --repo "$REPO"
```

`approve` makes a fresh card eligible for task-conditioned selection. `reject`
and `disable` keep the card record for auditability but prevent pack inclusion.

## Plan And Pack Selection

`prepare-task` returns selected memory under `selectedMemory` when a fresh,
approved or deterministic card overlaps the current task or selected target
files. `get-pack` renders the result in a separate `Selected memory` section so
memory cannot crowd out target files, tests, or constraints.

MCP exposes the same data additively through existing plan/pack flows and the
resource URI:

```text
ctxpack://repo/memory
```

## Release Coverage

The release gate runs `scripts/smoke-memory.sh`. The smoke proves local-only
storage, source-free persistence, stale/pending review blocking, selected-memory
pack output, and approve/disable review controls.
