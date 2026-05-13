---
phase: 01-compatibility-guardrails-module-boundaries
plan: 03
type: execute
wave: 2
depends_on:
  - 01-compatibility-guardrails-module-boundaries-01
  - 01-compatibility-guardrails-module-boundaries-02
files_modified:
  - crates/ctxpack-index/src/lib.rs
  - crates/ctxpack-index/src/inventory.rs
  - crates/ctxpack-index/src/search.rs
  - crates/ctxpack-index/src/symbols.rs
  - crates/ctxpack-index/src/related_tests.rs
  - crates/ctxpack-index/src/dependencies.rs
  - crates/ctxpack-index/src/git.rs
  - crates/ctxpack-index/src/traces.rs
autonomous: true
requirements: [CONT-04]
must_haves:
  truths:
    - "Maintainer can split ctxpack-index internals while crate-root public APIs remain import-compatible."
    - "Inventory, lexical search, symbols, related tests, dependencies, git/history/current diff, and traces live in focused modules."
    - "Existing index behavior and downstream CLI/MCP/compiler behavior remain unchanged after the split."
  artifacts:
    - path: "crates/ctxpack-index/src/lib.rs"
      provides: "Stable crate-root facade"
      contains: "pub use"
    - path: "crates/ctxpack-index/src/inventory.rs"
      provides: "Inventory and path classification implementation"
      contains: "pub fn build_inventory"
    - path: "crates/ctxpack-index/src/search.rs"
      provides: "Lexical search implementation"
      contains: "pub fn lexical_search"
    - path: "crates/ctxpack-index/src/symbols.rs"
      provides: "Symbol extraction and search implementation"
      contains: "pub fn symbol_search"
  key_links:
    - from: "crates/ctxpack-index/src/lib.rs"
      to: "crates/ctxpack-compiler/src/lib.rs"
      via: "unchanged public exports"
      pattern: "pub use .*SearchOptions"
    - from: "crates/ctxpack-index/src/lib.rs"
      to: "crates/ctxpack-mcp/src/lib.rs"
      via: "unchanged public exports"
      pattern: "pub use .*CurrentDiffOptions"
---

<objective>
Split `ctxpack-index` into focused private modules behind the existing crate-root facade.

Purpose: CONT-04 plus decisions D-10, D-11, and D-12 require module boundaries by current concern without retrieval behavior changes.
Output: Focused index modules with stable public exports and passing existing/new guardrail tests.
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
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-RESEARCH.md
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/STRUCTURE.md
@.planning/codebase/TESTING.md
@.planning/codebase/CONVENTIONS.md
@AGENTS.md
@crates/ctxpack-index/src/lib.rs

<interfaces>
`ctxpack-index` public APIs consumed by CLI, compiler, and MCP must continue to be exported from `crates/ctxpack-index/src/lib.rs`. Keep these names import-compatible: `InventoryError`, `InventoryOptions`, `FileInventoryEntry`, `RepoInventory`, `InventoryReport`, `build_inventory`, `write_inventory`, `load_inventory`, `load_or_build_inventory`, `inventory_path`, `SearchOptions`, `SearchResult`, `lexical_search`, `SymbolOptions`, `CodeSymbol`, `SymbolSearchResult`, `symbol_search`, `RelatedTestOptions`, `RelatedTestResult`, `related_tests`, `test_map`, `DependencyOptions`, `DependencyEdge`, `dependency_edges`, `related_dependency_edges`, `CoChangeOptions`, `CoChangeHint`, `co_change_hints`, `CurrentDiffOptions`, `CurrentDiffSummary`, `current_diff_summary`, `HistoricalCommitSample`, `HistoricalCommitOptions`, `historical_commit_samples`, `append_eval_trace`, and `list_eval_traces`.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Create index facade and inventory/search modules</name>
  <files>crates/ctxpack-index/src/lib.rs, crates/ctxpack-index/src/inventory.rs, crates/ctxpack-index/src/search.rs</files>
  <read_first>
    - crates/ctxpack-index/src/lib.rs
    - crates/ctxpack-compiler/src/lib.rs
    - crates/ctxpack-mcp/src/lib.rs
    - crates/ctxpack/src/main.rs
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - `ctxpack_index::build_inventory`, `write_inventory`, `load_inventory`, `load_or_build_inventory`, `inventory_path`, `InventoryOptions`, `RepoInventory`, `InventoryReport`, `FileInventoryEntry`, `SearchOptions`, `SearchResult`, and `lexical_search` remain import-compatible.
    - Existing inventory and lexical search tests still pass.
  </behavior>
  <action>
    Move inventory types/functions/path classification helpers from `crates/ctxpack-index/src/lib.rs` into `crates/ctxpack-index/src/inventory.rs`. Move lexical search types/functions/query scoring helpers into `crates/ctxpack-index/src/search.rs`. Keep `is_sensitive_path`, `is_generated_path`, `classify_path`, and `language_for_path` available to sibling modules with `pub(crate)` only where needed. Replace the moved sections in `lib.rs` with `mod inventory; mod search;` and `pub use inventory::{...}; pub use search::{...};` for the exact public API names listed in this plan. Do not change scoring constants, generated/sensitive classification behavior, inventory JSON fields, or search result shape.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-index inventory -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index lexical_search -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `test -f crates/ctxpack-index/src/inventory.rs && test -f crates/ctxpack-index/src/search.rs` succeeds.
    - `rg -n '^mod inventory;|^mod search;|pub use inventory::|pub use search::' crates/ctxpack-index/src/lib.rs` finds facade declarations.
    - `rg -n 'pub fn build_inventory|pub fn write_inventory|pub fn load_or_build_inventory|pub fn lexical_search|pub struct SearchOptions|pub struct SearchResult' crates/ctxpack-index/src/inventory.rs crates/ctxpack-index/src/search.rs` finds the moved public items.
    - `cargo test -p ctxpack-index inventory -- --nocapture` passes.
    - `cargo test -p ctxpack-index lexical_search -- --nocapture` passes.
  </acceptance_criteria>
  <done>Inventory and lexical search concerns are split while existing behavior remains stable.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Split symbols, related tests, and dependencies modules</name>
  <files>crates/ctxpack-index/src/lib.rs, crates/ctxpack-index/src/symbols.rs, crates/ctxpack-index/src/related_tests.rs, crates/ctxpack-index/src/dependencies.rs</files>
  <read_first>
    - crates/ctxpack-index/src/lib.rs
    - crates/ctxpack-index/src/inventory.rs
    - crates/ctxpack-index/src/search.rs
    - crates/ctxpack-compiler/src/lib.rs
    - crates/ctxpack-mcp/src/lib.rs
    - .planning/codebase/CONVENTIONS.md
  </read_first>
  <behavior>
    - Symbol, related-test, and dependency APIs remain exported from `ctxpack_index`.
    - Existing tests for symbol extraction, related tests, test map, dependency edges, and related dependency edges still pass.
  </behavior>
  <action>
    Move symbol extraction/search into `crates/ctxpack-index/src/symbols.rs`, test mapping and command inference into `crates/ctxpack-index/src/related_tests.rs`, and dependency/import graph logic into `crates/ctxpack-index/src/dependencies.rs`. Add `mod symbols; mod related_tests; mod dependencies;` and exact `pub use` exports in `lib.rs`. Use shared `inventory` helpers instead of duplicating path classification. Preserve struct derives, serde rename rules, package-manager command strings, dependency edge `kind` values, symbol kind strings, and result sorting.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-index symbol -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index related_tests -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index dependency -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `test -f crates/ctxpack-index/src/symbols.rs && test -f crates/ctxpack-index/src/related_tests.rs && test -f crates/ctxpack-index/src/dependencies.rs` succeeds.
    - `rg -n '^mod symbols;|^mod related_tests;|^mod dependencies;|pub use symbols::|pub use related_tests::|pub use dependencies::' crates/ctxpack-index/src/lib.rs` finds facade declarations.
    - `rg -n 'pub fn symbol_search|pub struct CodeSymbol|pub fn related_tests|pub fn test_map|pub fn dependency_edges|pub fn related_dependency_edges' crates/ctxpack-index/src/symbols.rs crates/ctxpack-index/src/related_tests.rs crates/ctxpack-index/src/dependencies.rs` finds moved APIs.
    - All three focused `cargo test -p ctxpack-index ...` commands pass.
  </acceptance_criteria>
  <done>Symbol, test, and dependency concerns are isolated without changing imports or outputs.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Split git/history/current-diff and trace modules, then run guardrails</name>
  <files>crates/ctxpack-index/src/lib.rs, crates/ctxpack-index/src/git.rs, crates/ctxpack-index/src/traces.rs</files>
  <read_first>
    - crates/ctxpack-index/src/lib.rs
    - crates/ctxpack-index/src/inventory.rs
    - crates/ctxpack-index/src/dependencies.rs
    - crates/ctxpack-compiler/src/lib.rs
    - crates/ctxpack-mcp/src/lib.rs
    - crates/ctxpack/tests/cli_compat.rs
    - .planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md
  </read_first>
  <behavior>
    - Co-change, current-diff, historical sample, and trace APIs remain exported from `ctxpack_index`.
    - Existing and Wave 1 compatibility tests pass after the split.
  </behavior>
  <action>
    Move git-backed APIs and helpers into `crates/ctxpack-index/src/git.rs`: `co_change_hints`, `current_diff_summary`, `historical_commit_samples`, commit parsing helpers, git timeout helpers, and related option/result structs. Move trace persistence into `crates/ctxpack-index/src/traces.rs`: `append_eval_trace`, `list_eval_traces`, and trace path helpers. Add `mod git; mod traces;` and exact `pub use` exports in `lib.rs`. Keep shell-free `Command` invocation, timeout values, trace JSONL shape, source-free guarantees, current-diff safe filtering, and empty-history behavior unchanged. After compile errors are fixed, run Wave 1 guardrails to prove public behavior survived.
  </action>
  <verify>
    <automated>cargo test -p ctxpack-index current_diff -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index historical_commit -- --nocapture</automated>
    <automated>cargo test -p ctxpack-index eval_traces -- --nocapture</automated>
    <automated>cargo test -p ctxpack --test cli_compat</automated>
  </verify>
  <acceptance_criteria>
    - `test -f crates/ctxpack-index/src/git.rs && test -f crates/ctxpack-index/src/traces.rs` succeeds.
    - `rg -n '^mod git;|^mod traces;|pub use git::|pub use traces::' crates/ctxpack-index/src/lib.rs` finds facade declarations.
    - `rg -n 'pub fn co_change_hints|pub fn current_diff_summary|pub fn historical_commit_samples|pub fn append_eval_trace|pub fn list_eval_traces' crates/ctxpack-index/src/git.rs crates/ctxpack-index/src/traces.rs` finds moved APIs.
    - `cargo test -p ctxpack-index` passes.
    - `cargo test -p ctxpack --test cli_compat` passes.
  </acceptance_criteria>
  <done>`ctxpack-index` is split by concern with stable crate-root exports and passing compatibility guardrails.</done>
</task>

</tasks>

<verification>
Run `cargo test -p ctxpack-index`, `cargo test -p ctxpack-compiler`, `cargo test -p ctxpack-mcp`, and `cargo test -p ctxpack --test cli_compat`.
</verification>

<success_criteria>
CONT-04 is partially satisfied for the index crate when the large index module is split by concern, `lib.rs` remains the public facade, and all existing consumers compile unchanged.
</success_criteria>

<output>
After completion, create `.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-03-SUMMARY.md`.
</output>
