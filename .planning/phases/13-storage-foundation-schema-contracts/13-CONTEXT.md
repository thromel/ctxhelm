# Phase 13: Storage Foundation & Schema Contracts - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Create ctxpack's first durable local SQLite storage layer: store location resolution, source-free schema contracts, version/migration metadata, and privacy tests. This phase establishes the storage foundation only. Incremental indexing behavior, storage-backed benchmark comparison, and operational repair/cleanup commands belong to Phases 14-16.

</domain>

<decisions>
## Implementation Decisions

### Store Location
- **D-01:** Default durable storage should remain user-local under `CTXPACK_HOME` or `~/.ctxpack`, matching the current inventory and trace state model.
- **D-02:** The store path must be explicit in metadata and diagnostics so users can see exactly which SQLite file is being used.
- **D-03:** Repo-local `.ctxpack/` remains for repo-owned config, generated cards, and adapter artifacts. Do not default to committing or storing the SQLite database in the repo.
- **D-04:** Allow an explicit store path override for tests, advanced users, and future repo-local/cloud-task workflows, but keep the default private and local.

### Source-Free Boundary
- **D-05:** Phase 13 should create a strict source-free schema. Do not add raw source text, prompt text, snippet bodies, commit subjects, or secret-bearing columns/tables.
- **D-06:** Store stable handles instead: repo IDs, safe paths, roles, hashes, line ranges, symbol names/signatures where already source-free enough for existing contracts, candidate IDs, warnings, privacy status, and metric values.
- **D-07:** If future phases ever need source-bearing persistence, it must be a separate explicit opt-in design with visible privacy labeling and migration boundaries. Do not pre-create source-bearing hooks in Phase 13.

### Schema Breadth
- **D-08:** Prefer a broad future-ready schema skeleton over an inventory-only store. Phase 13 should define typed tables/records for repos, files, symbols, chunks, edges, tests, git-history summaries, traces, packs, benchmark runs, proof reports, and migrations.
- **D-09:** The broad schema can start with minimal columns needed for identity, provenance, freshness, source-free metadata, and versioning. Full consumer wiring can be phased in later.
- **D-10:** Keep schema contracts typed in Rust rather than ad hoc SQL string output. Public CLI/MCP contracts should consume typed storage-facing values, not raw database rows.

### Existing JSON Transition
- **D-11:** Do not replace `inventory.json` and `traces.jsonl` immediately. Phase 13 should be additive and non-breaking.
- **D-12:** Existing JSON/JSONL files remain a fallback during the transition. SQLite import/sync should be idempotent and source-free.
- **D-13:** Avoid broad mandatory dual-write until the store contract is stable. Planner should introduce writes only where they are testable and low-risk, then Phase 14 can expand reuse and incremental behavior.

### the agent's Discretion
- Choose the SQLite Rust dependency and storage module layout during planning, as long as the choice keeps the CLI distributable and test setup simple.
- Choose exact table names and normalization details during planning, as long as all Phase 13 requirements map cleanly and privacy tests prove source-free defaults.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Scope
- `.planning/ROADMAP.md` — Phase 13 goal, requirements, success criteria, and planned plan breakdown.
- `.planning/REQUIREMENTS.md` — STORE-01 through STORE-04 and v1.3 exclusions.
- `.planning/PROJECT.md` — product constraints, local-first/read-only contract, and milestone strategy.

### Codebase Maps
- `.planning/codebase/STACK.md` — current Rust workspace, dependencies, runtime, and absence of database dependencies.
- `.planning/codebase/ARCHITECTURE.md` — current layered crate architecture and JSON/JSONL persistence flow.
- `.planning/codebase/INTEGRATIONS.md` — current local filesystem storage, MCP surfaces, and no database/remote-service integrations.

### Existing Storage Code
- `crates/ctxpack-index/src/inventory.rs` — current inventory path resolution, JSON inventory persistence, `CTXPACK_HOME` fallback, and safe inventory report.
- `crates/ctxpack-index/src/traces.rs` — current source-free trace append/list path.
- `crates/ctxpack-index/src/freshness.rs` — current inventory freshness metadata and stale-cache diagnostics.
- `crates/ctxpack-core/src/contracts.rs` — stable typed contracts that storage records must not break.
- `crates/ctxpack/src/main.rs` — CLI command boundary and existing index/eval command flows.
- `crates/ctxpack-mcp/src/resources.rs` — current session-scoped MCP pack resource cache.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ctxpack_index::ctxpack_home()` pattern in `crates/ctxpack-index/src/inventory.rs`: reuse the same `CTXPACK_HOME` / `HOME/.ctxpack` fallback model for default store location.
- `repo_id_for_path()` in `crates/ctxpack-index/src/inventory.rs`: reuse deterministic local repo IDs for storage partitioning.
- `InventoryMetadata` and freshness checks in `crates/ctxpack-index/src/freshness.rs`: reuse metadata concepts for schema version, options fingerprint, ignore-policy drift, and stale diagnostics.
- Source-free eval and pack contracts in `crates/ctxpack-core/src/contracts.rs`: use these as the privacy baseline for stored trace, benchmark, and pack metadata.

### Established Patterns
- Public data flows are typed Rust structs serialized to JSON at the CLI/MCP boundary. Storage should follow the same typed-contract pattern.
- Current state writes are local-only and best-effort where possible. Storage failures should degrade with diagnostics instead of hiding low-confidence behavior.
- Tests already use `tempfile` and isolated `CTXPACK_HOME`; storage tests should follow that fixture style.

### Integration Points
- `crates/ctxpack-index` is the natural first home for storage path resolution and repository metadata persistence.
- `crates/ctxpack-core` should own shared typed storage-facing contracts only if they become public across crates.
- `crates/ctxpack-compiler` will later use storage for benchmark/proof/pack metadata, but Phase 13 should avoid prematurely rewiring every compiler path.
- `crates/ctxpack-mcp` should surface storage diagnostics only after CLI/index storage contracts exist.

</code_context>

<specifics>
## Specific Ideas

- Add a focused storage module, likely under `crates/ctxpack-index`, with `StoreConfig`, `StorePaths`, `StorageMetadata`, `SchemaVersion`, and typed record structs.
- Use SQLite as the durable store, but keep source-free JSON/JSONL fallback behavior during Phase 13.
- Add tests that insert realistic source-like fixture text into repo files and assert the SQLite database does not contain that raw text.
- Store path diagnostics should mention whether storage came from `CTXPACK_HOME`, default home, repo-local explicit override, or test override.
- Migration history should be queryable without opening source files.

</specifics>

<deferred>
## Deferred Ideas

- Full incremental indexing and cache reuse belongs to Phase 14.
- Storage-backed benchmark trend lookup belongs to Phase 15.
- CLI repair, cleanup, migration commands, and release-gate checks belong to Phase 16.
- Any source-bearing snippet persistence is out of scope for v1.3 unless a future milestone explicitly opts in with privacy labeling.

</deferred>

---

*Phase: 13-Storage Foundation & Schema Contracts*
*Context gathered: 2026-05-14*
