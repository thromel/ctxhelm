---
phase: 02-trust-layer-operational-diagnostics
plan: 02
type: execute
wave: 2
depends_on: ["02-01"]
files_modified:
  - crates/ctxhelm-index/src/lib.rs
  - crates/ctxhelm-index/src/inventory.rs
  - crates/ctxhelm-index/src/freshness.rs
  - crates/ctxhelm-index/src/policy.rs
autonomous: true
requirements: [SAFE-01, SAFE-02, SAFE-03, SAFE-06, DIAG-01, DIAG-04]
must_haves:
  truths:
    - "User-facing read paths can ask for a fresh inventory envelope instead of blindly trusting inventory.json."
    - "Inventory freshness explains source-free reasons for stale cache and rebuild attempts."
    - "Read operations can continue with a fresh in-memory inventory when cache persistence fails."
  artifacts:
    - path: "crates/ctxhelm-index/src/freshness.rs"
      provides: "Inventory metadata, freshness checks, and load-or-refresh report"
      contains: "load_or_refresh_inventory"
    - path: "crates/ctxhelm-index/src/inventory.rs"
      provides: "Metadata-bearing inventory serialization and cache write status"
      contains: "metadata"
    - path: "crates/ctxhelm-index/src/lib.rs"
      provides: "Freshness API re-exports"
      contains: "InventoryLoadReport"
  key_links:
    - from: "crates/ctxhelm-index/src/freshness.rs"
      to: "crates/ctxhelm-index/src/inventory.rs"
      via: "build/read/write inventory lifecycle"
      pattern: "build_inventory"
    - from: "crates/ctxhelm-index/src/freshness.rs"
      to: "crates/ctxhelm-index/src/policy.rs"
      via: "policy version and classification drift"
      pattern: "POLICY_VERSION"
---

<objective>
Implement the freshness envelope that all Phase 2 read paths will use before returning inventory-derived results.

Purpose: SAFE-01, SAFE-02, SAFE-06, DIAG-01, and DIAG-04 require one deterministic cache freshness path that can rebuild safely, diagnose stale state, and decouple in-memory freshness from cache write failures.
Output: `load_or_refresh_inventory` plus metadata/freshness tests for create/delete/rename, ignore changes, option changes, policy changes, and constrained `CTXHELM_HOME`.
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
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/CONCERNS.md
@.planning/codebase/TESTING.md
@AGENTS.md

<interfaces>
Created by Plan 01 and required here:
```rust
pub struct Diagnostic { /* source-free diagnostic fields */ }
pub enum DiagnosticSeverity { /* info/warning/error */ }
pub fn read_safe_source(...);
```

Existing inventory entry point to preserve:
```rust
pub fn load_or_build_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<RepoInventory, InventoryError>;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add inventory metadata and freshness fixtures</name>
  <files>crates/ctxhelm-index/src/inventory.rs, crates/ctxhelm-index/src/freshness.rs, crates/ctxhelm-index/src/lib.rs</files>
  <read_first>
    - `crates/ctxhelm-index/src/inventory.rs`
    - `crates/ctxhelm-index/src/lib.rs`
    - `.planning/phases/02-trust-layer-operational-diagnostics/02-RESEARCH.md` freshness design section
  </read_first>
  <behavior>
    - SAFE-01: cached inventory is stale when repository files are created, deleted, renamed, size/hash/mtime changes, or paths move into generated/sensitive classification.
    - SAFE-02: stale reasons are returned as structured diagnostics that explain what changed without source snippets.
    - SAFE-03: policy version drift invalidates inventory metadata.
    - DIAG-04: tests deterministically cover stale cache scenarios with temp repositories.
  </behavior>
  <action>
    Add `InventoryMetadata` and file-manifest fields to `RepoInventory` using serde defaults for compatibility with old cache JSON. Include schema version, policy version, options fingerprint, repo root, built-at timestamp, ignore-file fingerprints for `.gitignore`, `.git/info/exclude`, `.ctxhelmignore`, and `.cursorignore`, plus manifest entries for safe files. Create `freshness.rs` with pure comparison helpers and table-driven tests for file create/delete/rename, ignore-file mutation, options changes, policy-version changes, and repo-root mismatch. Keep `load_or_build_inventory` available for compatibility until read paths migrate.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-index freshness -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - Freshness tests fail on unmodified code and pass after metadata implementation.
    - Old inventory JSON without metadata deserializes and is treated as stale instead of trusted.
    - Diagnostics include reason codes and paths/counts but no file contents.
  </acceptance_criteria>
  <done>Inventory metadata and freshness checks can identify stale cache causes before read-path migration.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement trusted inventory load and non-fatal cache status</name>
  <files>crates/ctxhelm-index/src/inventory.rs, crates/ctxhelm-index/src/freshness.rs, crates/ctxhelm-index/src/lib.rs</files>
  <read_first>
    - `crates/ctxhelm-index/src/inventory.rs`
    - `crates/ctxhelm-index/src/freshness.rs`
    - `crates/ctxhelm-index/src/traces.rs`
  </read_first>
  <behavior>
    - SAFE-01/SAFE-02: `load_or_refresh_inventory` rebuilds stale inventory before returning results when repo files are readable.
    - SAFE-06: if cache write fails after a successful rebuild, read-oriented callers receive the fresh in-memory inventory plus `cache_write_failed` diagnostics.
    - DIAG-01: load reports include `inventory_stale`, `inventory_rebuilt`, `inventory_rebuild_failed`, and `cache_write_failed` diagnostics where applicable.
  </behavior>
  <action>
    Implement `InventoryLoadReport { inventory, diagnostics, freshness, cache_status }` and `load_or_refresh_inventory(repo_root, options)` in `freshness.rs` or `inventory.rs`, re-exported from `lib.rs`. The function should load cache, check freshness, rebuild stale/missing cache, and attempt persistence. For explicit `write_inventory`, keep write failures fatal. For `load_or_refresh_inventory`, make cache persistence failure non-fatal when in-memory rebuild succeeded. Preserve source-free diagnostics and avoid changing retrieval ranking.
  </action>
  <verify>
    <automated>cargo test -p ctxhelm-index load_or_refresh -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - Fresh file mutations trigger rebuilds and return rebuilt diagnostics.
    - Read-only or otherwise constrained `CTXHELM_HOME` does not make `load_or_refresh_inventory` fail if the repo itself can be scanned.
    - `write_inventory` remains an explicit write operation that reports write errors normally.
  </acceptance_criteria>
  <done>Trusted inventory loads are fresh-or-diagnostic and cache write failures are visible but non-fatal for read paths.</done>
</task>

</tasks>

<verification>
```bash
cargo test -p ctxhelm-index freshness -- --nocapture
cargo test -p ctxhelm-index load_or_refresh -- --nocapture
cargo test -p ctxhelm-index
```
</verification>

<success_criteria>
SAFE-01 and SAFE-02 have a central freshness implementation. SAFE-06 cache-write behavior is available to read paths. DIAG-04 has deterministic stale-cache fixtures.
</success_criteria>

<output>
After completion, create `.planning/phases/02-trust-layer-operational-diagnostics/02-trust-layer-operational-diagnostics-02-SUMMARY.md`.
</output>
