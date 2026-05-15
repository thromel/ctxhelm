# Phase 15: Evaluation, Pack, and Proof Persistence - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning
**Mode:** Autonomous smart discuss

<domain>
## Phase Boundary

Phase 15 persists source-free context pack, historical eval, benchmark, retrieval-gap, and proof metadata into the SQLite store. It must not persist pack snippets, prompt text, raw source, or benchmark sample bodies.

</domain>

<decisions>
## Implementation Decisions

### Persistence Boundary
- Persist metadata records, metrics, safe IDs, safe paths, gap families, and privacy labels.
- Add opt-in `--store` flags to commands that produce persistable metadata.
- Keep existing JSON/Markdown output unchanged unless storage metadata is explicitly requested.

### Product Proof
- Benchmark and proof persistence should be reusable for later trend comparisons.
- Benchmark suite persistence should map each configured repo path to its own local store.

### the agent's Discretion
The storage layer can use source-free record contracts rather than depending on compiler report types, avoiding a crate cycle.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Compiler eval reports already expose source-free metrics and retrieval gaps.
- `ContextPack` already exposes `task_hash`, `budget`, `target_agent`, `confidence`, warnings, and privacy status.

### Established Patterns
- Eval commands already support Markdown/JSON output and should remain backward-compatible.

### Integration Points
- `get-pack --store`
- `eval history --store`
- `eval benchmark --store`
- `eval proof --store`

</code_context>

<specifics>
## Specific Ideas

Store pack metadata and benchmark/proof metadata only; source-bearing pack sections remain output material, not default persisted state.

</specifics>

<deferred>
## Deferred Ideas

Automatic benchmark trend selection can be built on the persisted records later.

</deferred>
