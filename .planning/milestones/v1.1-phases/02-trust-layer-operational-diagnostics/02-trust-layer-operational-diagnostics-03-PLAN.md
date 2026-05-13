---
phase: 02-trust-layer-operational-diagnostics
plan: 03
type: execute
wave: 3
depends_on: ["02-02"]
files_modified:
  - crates/ctxpack-index/src/lib.rs
  - crates/ctxpack-index/src/search.rs
  - crates/ctxpack-index/src/symbols.rs
  - crates/ctxpack-index/src/related_tests.rs
  - crates/ctxpack-index/src/dependencies.rs
  - crates/ctxpack-index/src/git.rs
  - crates/ctxpack-index/src/policy.rs
autonomous: true
requirements: [SAFE-01, SAFE-02, SAFE-05, DIAG-01, DIAG-04]
must_haves:
  truths:
    - "Search, symbols, related tests, dependency graph, current diff, and history helpers use fresh inventory before returning inventory-derived results."
    - "Unreadable, non-UTF-8, oversized, binary, skipped, or externally unavailable inputs become diagnostics instead of silent empty matches."
    - "Weak index-layer signals expose partial coverage reasons without changing ranking/eval behavior."
  artifacts:
    - path: "crates/ctxpack-index/src/search.rs"
      provides: "Fresh lexical search with source-read diagnostics"
      contains: "load_or_refresh_inventory"
    - path: "crates/ctxpack-index/src/symbols.rs"
      provides: "Fresh symbol extraction/search with parse/read diagnostics"
      contains: "read_safe_source"
    - path: "crates/ctxpack-index/src/related_tests.rs"
      provides: "Fresh related-test/test-map reads with source-read diagnostics"
      contains: "read_safe_source"
    - path: "crates/ctxpack-index/src/dependencies.rs"
      provides: "Fresh dependency graph reads with parse/read diagnostics"
      contains: "read_safe_source"
    - path: "crates/ctxpack-index/src/git.rs"
      provides: "Structured git missing/timeout/partial diagnostics"
      contains: "git_timeout"
  key_links:
    - from: "crates/ctxpack-index/src/search.rs"
      to: "crates/ctxpack-index/src/freshness.rs"
      via: "fresh inventory load"
      pattern: "load_or_refresh_inventory"
    - from: "crates/ctxpack-index/src/*"
      to: "crates/ctxpack-index/src/policy.rs"
      via: "safe source reads"
      pattern: "read_safe_source"
---

<objective>
Wire the trusted inventory and policy layer through index read paths before compiler/CLI/MCP surfaces consume them.

Purpose: SAFE-01, SAFE-02, SAFE-05, DIAG-01, and DIAG-04 require the lower-level read APIs to stop silently using stale cache or converting source read failures into empty content.
Output: Freshness-aware search/symbol/test/dependency/git helpers plus deterministic read-failure and partial-signal tests.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/STATE.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-CONTEXT.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-RESEARCH.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-01-SUMMARY.md
@.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-02-SUMMARY.md
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md

<interfaces>
From Plan 02:
```rust
pub struct InventoryLoadReport {
    pub inventory: RepoInventory,
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: /* typed status */,
}
pub fn load_or_refresh_inventory(...) -> Result<InventoryLoadReport, InventoryError>;
```

Existing result types to preserve unless adding additive diagnostics:
```rust
pub struct SearchResult { pub path: String, pub score: f32, ... }
pub struct SymbolSearchResult { pub symbol: CodeSymbol, pub score: f32 }
pub struct RelatedTestResult { pub path: String, pub command: Option<String>, ... }
pub struct DependencyEdge { pub source_path: String, pub target_path: String, ... }
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Route index retrieval through trusted inventory</name>
  <files>crates/ctxpack-index/src/search.rs, crates/ctxpack-index/src/symbols.rs, crates/ctxpack-index/src/related_tests.rs, crates/ctxpack-index/src/dependencies.rs, crates/ctxpack-index/src/git.rs, crates/ctxpack-index/src/lib.rs</files>
  <read_first>
    - `crates/ctxpack-index/src/search.rs`
    - `crates/ctxpack-index/src/symbols.rs`
    - `crates/ctxpack-index/src/related_tests.rs`
    - `crates/ctxpack-index/src/dependencies.rs`
    - `crates/ctxpack-index/src/git.rs`
  </read_first>
  <behavior>
    - SAFE-01: search, symbol extraction/search, related tests, test map, dependency edges, co-change/current-diff/history safe labels use `load_or_refresh_inventory`.
    - SAFE-02: stale inventory rebuild diagnostics are retained in additive result/report fields where the existing result shape allows it.
    - DIAG-01: partial graph/test/history coverage diagnostics are available to compiler/MCP callers.
  </behavior>
  <action>
    Replace read-path uses of `load_or_build_inventory` with `load_or_refresh_inventory` for user-facing APIs. Preserve existing function names and return shapes where Phase 1 compatibility requires them; introduce additive diagnostic-bearing report variants only where necessary for compiler/MCP use. Do not change ranking weights, candidate scoring, eval metrics, or graph expansion behavior. Keep old `load_or_build_inventory` only as a compatibility wrapper if needed.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-index lexical_search -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index symbol -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index related_tests -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index dependency -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index current_diff -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - Existing index tests still pass.
    - New stale-cache fixtures prove mutated files and ignore changes affect search/symbol/test/dependency results without running `ctxpack index`.
    - Diagnostic-bearing variants expose freshness/partial signal data for downstream plans.
  </acceptance_criteria>
  <done>Index read APIs use the trusted freshness path and preserve current retrieval behavior except for freshness and diagnostics.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Replace silent source read failures with diagnostics</name>
  <files>crates/ctxpack-index/src/search.rs, crates/ctxpack-index/src/symbols.rs, crates/ctxpack-index/src/related_tests.rs, crates/ctxpack-index/src/dependencies.rs, crates/ctxpack-index/src/git.rs, crates/ctxpack-index/src/policy.rs</files>
  <read_first>
    - `crates/ctxpack-index/src/policy.rs`
    - `crates/ctxpack-index/src/search.rs`
    - `crates/ctxpack-index/src/symbols.rs`
    - `crates/ctxpack-index/src/dependencies.rs`
    - `crates/ctxpack-index/src/related_tests.rs`
  </read_first>
  <behavior>
    - SAFE-05: unreadable, non-UTF-8, oversized, binary, deleted-after-inventory, or policy-skipped files emit structured diagnostics and are not treated as empty successful reads.
    - DIAG-01: git unavailable and timeout cases map to `git_missing`, `git_timeout`, `history_partial`, `graph_partial`, `test_map_partial`, or `parse_gap` diagnostics.
    - DIAG-04: weak or partial index-layer scenarios have deterministic fixtures.
  </behavior>
  <action>
    Replace `fs::read_to_string(...).unwrap_or_default()` and similar silent fallbacks in index modules with the central `read_safe_source` policy. Aggregate diagnostics without source snippets. Extend git helper diagnostics for missing git/timeout and partial history results without shelling through ad hoc process calls. Add tests for invalid UTF-8, oversized source-like files, deleted-after-inventory files, unreadable files on Unix, and partial graph/test/history coverage.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-index source_read -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index diagnostics -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index partial -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - No user-facing index read path silently converts read failures into empty content.
    - Diagnostic codes match the Phase 2 research list and are source-free.
    - Existing scoring order is not intentionally changed by this task.
  </acceptance_criteria>
  <done>Index read failures and external-signal partial failures are visible as typed diagnostics instead of silent weak matches.</done>
</task>

</tasks>

<verification>
```bash
cargo test -p ctxpack-index lexical_search -- --nocapture
cargo test -p ctxpack-index symbol -- --nocapture
cargo test -p ctxpack-index related_tests -- --nocapture
cargo test -p ctxpack-index dependency -- --nocapture
cargo test -p ctxpack-index current_diff -- --nocapture
cargo test -p ctxpack-index source_read -- --nocapture
cargo test -p ctxpack-index diagnostics -- --nocapture
cargo test -p ctxpack-index partial -- --nocapture
cargo test -p ctxpack-index
```
</verification>

<success_criteria>
SAFE-01/SAFE-02 freshness is wired through the index layer, SAFE-05 skips are diagnostic, and DIAG-04 has deterministic fixtures for stale/read-failure/partial-signal behavior.
</success_criteria>

<output>
After completion, create `.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-03-SUMMARY.md`.
</output>
