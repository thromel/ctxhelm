---
phase: 02-trust-layer-operational-diagnostics
verified: 2026-05-13T14:20:31Z
status: passed
score: "5/5 phase success criteria verified; 16/16 plan must-have truths covered"
---

# Phase 02: Trust Layer & Operational Diagnostics Verification Report

**Phase Goal:** Users can trust that ctxhelm read paths are fresh, privacy-gated, and explicit about partial or degraded results.
**Verified:** 2026-05-13T14:20:31Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

Phase 2 achieved the goal. The codebase now has a typed diagnostic contract, a centralized privacy/source-read policy, trusted inventory freshness checks, fresh-or-diagnostic index/compiler/MCP read paths, non-fatal trace/cache write behavior for read operations, and automated coverage for weak/degraded scenarios.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | User-facing search, planning, symbols, tests, dependency, pack, card, and MCP read paths detect stale inventory before returning results. | VERIFIED | `load_or_refresh_inventory` is implemented in `crates/ctxhelm-index/src/freshness.rs` and is called by search, symbols, related tests, dependencies, git report paths, compiler planning, packs/cards, CLI/MCP surfaces, and MCP resources. Workspace tests include stale inventory refresh fixtures. |
| 2 | Read paths either rebuild stale inventory or return structured stale-cache diagnostics explaining what changed. | VERIFIED | `InventoryLoadReport` carries `diagnostics`, `freshness`, and `cache_status`; stale reasons include created/deleted/changed files, ignore drift, policy drift, options drift, repo-root drift, and missing metadata. CLI and MCP stale-cache tests pass. |
| 3 | Pack, file-resource, and card generation exclude unsafe, generated, binary, oversized, unreadable, and non-UTF-8 inputs through a centralized tested policy. | VERIFIED | `policy.rs` owns `classify_path` and `read_safe_source`; packs and MCP file resources call `read_safe_source`; cards use fresh inventory/report APIs and remain source-free. |
| 4 | CLI and MCP outputs expose stable diagnostics for weak plans, stale cache, missing/timed-out git, skipped files, parse gaps, and partial graph/test/history coverage. | VERIFIED | `ContextPlan` and `ContextPack` have additive `diagnostics`; compiler projects warning/error diagnostics into `riskFlags`; MCP tool responses expose diagnostics in `structuredContent` or result-level fields where compatibility requires. |
| 5 | Context retrieval remains usable in constrained home-directory environments with trace/cache writes visible, controllable, and non-fatal when possible. | VERIFIED | `try_append_eval_trace` returns `TraceStatus` with `trace_write_failed` diagnostics; CLI has `--no-trace`; MCP has `recordTrace`; CLI constrained-home tests pass. |

**Score:** 5/5 phase success criteria verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/ctxhelm-core/src/contracts.rs` | Additive diagnostic/cache/trace contracts on public plan/pack JSON | VERIFIED | Defines `Diagnostic`, `DiagnosticSeverity`, `CacheStatus`, `TraceStatus`, and `diagnostics` fields on `ContextPlan`/`ContextPack` while preserving `riskFlags` and `warnings`. |
| `crates/ctxhelm-index/src/policy.rs` | Central privacy/source-read policy | VERIFIED | Defines `POLICY_VERSION`, `SOURCE_READ_MAX_BYTES`, `SourceRead`, typed skip reasons, `classify_path`, and `read_safe_source`; table-driven tests cover credential/generated/binary/non-UTF-8/oversized/unreadable cases. |
| `crates/ctxhelm-index/src/freshness.rs` | Inventory metadata, freshness checks, and trusted load report | VERIFIED | Implements `InventoryFreshness`, `InventoryStaleReason`, `InventoryLoadReport`, and `load_or_refresh_inventory`; rebuilds stale cache and emits source-free diagnostics. |
| `crates/ctxhelm-index/src/search.rs` | Fresh lexical search with source-read diagnostics | VERIFIED | Calls `load_or_refresh_inventory` and `read_safe_source`; returns `SearchReport` with diagnostics. |
| `crates/ctxhelm-index/src/symbols.rs` | Fresh symbol extraction/search with parse/read diagnostics | VERIFIED | Calls `load_or_refresh_inventory` and `read_safe_source`; emits `parse_gap` diagnostics. |
| `crates/ctxhelm-index/src/related_tests.rs` | Fresh related-test/test-map reads with diagnostics | VERIFIED | Calls `load_or_refresh_inventory` and `read_safe_source`; emits `test_map_partial` diagnostics. |
| `crates/ctxhelm-index/src/dependencies.rs` | Fresh dependency graph reads with diagnostics | VERIFIED | Calls `load_or_refresh_inventory` and `read_safe_source`; emits `graph_partial` diagnostics. |
| `crates/ctxhelm-index/src/git.rs` | Structured git partial/missing/timeout diagnostics | VERIFIED | Report APIs produce `git_missing`, `git_timeout`, `history_partial`, and current-diff/history diagnostics while preserving legacy compatibility. |
| `crates/ctxhelm-index/src/traces.rs` | Non-fatal trace append helper | VERIFIED | `try_append_eval_trace` wraps trace writes in `TraceStatus` and emits `trace_write_failed` diagnostics instead of failing read commands. |
| `crates/ctxhelm-compiler/src/planning.rs` | Plan diagnostics and `riskFlags` compatibility projection | VERIFIED | Merges diagnostic report APIs into plans, deduplicates diagnostics, and projects warning/error diagnostics into existing risk flags. |
| `crates/ctxhelm-compiler/src/packs.rs` | Pack snippet revalidation | VERIFIED | Loads fresh inventory and calls `read_safe_source` before snippet reads; skipped snippets become diagnostics/warnings. |
| `crates/ctxhelm-compiler/src/cards.rs` | Source-free cards from fresh inventory with diagnostics | VERIFIED | Uses `load_or_refresh_inventory`, symbol/test/dependency report APIs, and writes source-free card Markdown with report diagnostics. |
| `crates/ctxhelm/src/main.rs` | CLI diagnostics projection and trace controls | VERIFIED | Adds `--no-trace`, emits diagnostics in plan/pack JSON, and uses `try_append_eval_trace` for read-oriented trace recording. |
| `crates/ctxhelm/tests/cli_compat.rs` | Binary diagnostics and constrained-home guardrails | VERIFIED | Tests diagnostics fields, stale inventory rebuild diagnostics, no-trace help, and trace-write failure behavior. |
| `crates/ctxhelm-mcp/src/tools.rs` | MCP structured diagnostics and trace controls | VERIFIED | Uses diagnostic report APIs, `try_append_eval_trace`, and exposes diagnostics in MCP tool results. |
| `crates/ctxhelm-mcp/src/resources.rs` | Fresh, policy-gated file resources | VERIFIED | File resource reads call `load_or_refresh_inventory` and `read_safe_source` before returning source text. |
| `crates/ctxhelm-mcp/src/schemas.rs` | MCP `recordTrace` schema controls | VERIFIED | `prepare_task` and `get_pack` schemas expose additive `recordTrace`. |
| `crates/ctxhelm-mcp/src/lib.rs` | MCP protocol/resource/tool tests | VERIFIED | Tests cover diagnostic fields, stale cache rebuild, file-resource revalidation, trace failures, and unchanged six-tool surface. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `inventory.rs` | `policy.rs` | `classify_path` delegation | VERIFIED | `inventory.rs` imports and calls `classify_path`; policy owns role classification. |
| `contracts.rs` | Public plan/pack JSON | serde-defaulted additive fields | VERIFIED | Diagnostics are defaulted additive fields; old JSON deserialization tests exist. |
| `freshness.rs` | `inventory.rs` | build/read/write inventory lifecycle | VERIFIED | Freshness code calls inventory build/load/write helpers and compares metadata manifests. |
| `freshness.rs` | `policy.rs` | policy version drift | VERIFIED | Freshness compares cached metadata policy version against `POLICY_VERSION`. |
| Index read modules | `freshness.rs` | fresh inventory load | VERIFIED | Search, symbols, related tests, dependencies, and git report APIs call `load_or_refresh_inventory`. |
| Index read modules | `policy.rs` | safe source reads | VERIFIED | Search, symbols, related tests, and dependencies call `read_safe_source`. The gsd key-link helper could not expand the wildcard path, so this was manually verified by concrete file grep. |
| `planning.rs` | index diagnostic reports | report API consumption | VERIFIED | Compiler planning consumes symbol/search/test/dependency/git report diagnostics. |
| `packs.rs` | `policy.rs` | safe snippet reads | VERIFIED | Pack snippets call `read_safe_source`. |
| `cards.rs` | `freshness.rs` | fresh safe inventory | VERIFIED | Card generation calls `load_or_refresh_inventory` and report APIs. |
| `main.rs` | `traces.rs` | non-fatal trace append | VERIFIED | CLI read commands call `try_append_eval_trace`. |
| `resources.rs` | `policy.rs` | safe MCP file-resource reads | VERIFIED | MCP resources call `read_safe_source`. |
| `tools.rs` | `contracts.rs` | structured diagnostics | VERIFIED | MCP tool structured content carries serialized diagnostics. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `freshness.rs` | `InventoryLoadReport.diagnostics` | Current repo scan plus cached `RepoInventory.metadata` comparison | Yes | FLOWING |
| `search.rs` | `SearchReport.results`, `diagnostics` | Fresh inventory entries and `read_safe_source` text | Yes | FLOWING |
| `symbols.rs` | `SymbolExtractionReport.symbols`, `diagnostics` | Fresh inventory entries and safe source parsing | Yes | FLOWING |
| `related_tests.rs` | `RelatedTestsReport.results`, `diagnostics` | Fresh inventory test/source entries and safe source reads | Yes | FLOWING |
| `dependencies.rs` | `DependencyEdgesReport.edges`, `diagnostics` | Fresh inventory source/config entries and safe source reads | Yes | FLOWING |
| `git.rs` | report diagnostics | Local git command results plus freshness diagnostics | Yes | FLOWING |
| `planning.rs` | `ContextPlan.diagnostics` | Index report diagnostics, low-information checks, anchor availability, git/dependency reports | Yes | FLOWING |
| `packs.rs` | `ContextPack.diagnostics`, snippets | Current safe inventory plus `read_safe_source` at snippet time | Yes | FLOWING |
| `cards.rs` | `ContextCardsReport.diagnostics` | Fresh inventory plus symbol/test/dependency report diagnostics | Yes | FLOWING |
| `main.rs` | CLI JSON diagnostics | Compiler plan/pack diagnostics plus `try_append_eval_trace` status | Yes | FLOWING |
| `tools.rs` | MCP `structuredContent.diagnostics` | Compiler/index report diagnostics plus trace status | Yes | FLOWING |
| `resources.rs` | MCP file resource text | Fresh inventory plus `read_safe_source` | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Full workspace remains valid after Phase 2 | `cargo test --workspace` | 3 CLI unit tests, 10 CLI compatibility tests, 23 compiler tests, 27 core tests, 44 index tests, 42 MCP tests, and doctests passed. | PASS |
| CLI help still lists core commands including MCP | `cargo run -p ctxhelm -- --help` | Help output lists `serve-mcp` alongside existing commands. | PASS |
| Low-information plan exposes diagnostics and keeps riskFlags | `cargo run -q -p ctxhelm -- prepare-task "Fixes #1061" --repo . --mode bug-fix --no-trace` parsed with Node | Output: `hasDiagnostics=true`, `hasLowInfo=true`, `targetCount=8`, `hasRiskFlags=true`. | PASS |
| Pack JSON carries diagnostics and remains budgeted | `cargo run -q -p ctxhelm -- get-pack "Fixes #1061" --repo . --mode bug-fix --budget brief --format json --no-trace` parsed with Node | Output: `hasDiagnostics=true`, `budget=brief`, `warnings=2`, `sourceSnippetSections=2`. | PASS |
| MCP stale-cache diagnostics are exposed | `cargo test -p ctxhelm-mcp diagnostics_search_reports_stale_cache_rebuild -- --nocapture` | 1 test passed. | PASS |
| MCP file resources revalidate against current safe inventory | `cargo test -p ctxhelm-mcp file_resource_revalidates_against_current_safe_inventory -- --nocapture` | 1 test passed. | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| SAFE-01 | Plans 02, 03, 04, 05 | Read paths detect stale inventory metadata before returning results. | SATISFIED | `load_or_refresh_inventory` is wired through index, compiler, CLI, MCP tools, cards, packs, and file resources; stale-refresh tests pass. |
| SAFE-02 | Plans 02, 03, 04, 05 | Read paths rebuild stale inventory or return structured stale-cache diagnostics. | SATISFIED | `InventoryLoadReport` includes freshness/cache diagnostics; CLI and MCP stale-cache tests pass. |
| SAFE-03 | Plans 01, 02 | Sensitive/generated files use a centralized policy with table-driven tests. | SATISFIED | `policy.rs` centralizes classification and source reads; tests cover credentials, auth files, SSH keys, generated/vendor, binary, non-UTF-8, oversized, unreadable. |
| SAFE-04 | Plans 04, 05 | Pack, file-resource, and card generation revalidate source-bearing paths before reading. | SATISFIED | `packs.rs` and `resources.rs` call `read_safe_source`; `cards.rs` uses fresh inventory/report APIs; focused tests pass. |
| SAFE-05 | Plans 03, 04, 05 | Unreadable/non-UTF-8/oversized/skipped/external failures produce diagnostics. | SATISFIED | `read_safe_source` and report APIs emit typed diagnostics; index/compiler/MCP tests cover source-read and partial-signal cases. |
| SAFE-06 | Plans 02, 05 | Trace/cache writes are visible, controllable, and non-fatal for read paths. | SATISFIED | `load_or_refresh_inventory` makes read-path cache persistence non-fatal; `try_append_eval_trace`, CLI `--no-trace`, and MCP `recordTrace` are implemented and tested. |
| DIAG-01 | Plans 01, 02, 03, 04, 05 | Context plans show structured diagnostics for weak/degraded conditions. | SATISFIED | Compiler planning merges low-information, stale, skipped, parse gap, git, graph/test/history diagnostics into `ContextPlan.diagnostics`. |
| DIAG-02 | Plans 01, 05 | CLI and MCP expose stable diagnostics while preserving riskFlags compatibility. | SATISFIED | Public contracts keep `riskFlags`; CLI/MCP diagnostics fields are additive; MCP `related_tests` preserves array-shaped structured content with result-level diagnostics. |
| DIAG-04 | Plans 02, 03, 04, 05 | Weak-plan scenarios are covered by deterministic fixtures. | SATISFIED | Tests cover low-information plans, stale cache, partial graph/test/history, missing git, source-read failures, trace-write failure, and MCP file-resource revalidation. |

No orphaned Phase 2 requirements were found. `DIAG-03` remains Phase 3 by roadmap/requirements traceability and is correctly out of Phase 2 scope.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/ctxhelm-index/src/related_tests.rs` | 288, 372 | `placeholder` in `is_placeholder_test_script` helper | Info | Not a stub; it intentionally filters placeholder package-manager test scripts. |
| `crates/ctxhelm-mcp/src/resources.rs` | 138 | `not available` in session-scoped pack-resource error text | Info | Not a stub; this is the intentional MCP session-scoped resource diagnostic. |

No blocker or warning anti-patterns were found in the Phase 2 implementation files.

### Human Verification Required

None required for phase-goal verification. Visual/UI testing is not applicable; ctxhelm is CLI/MCP/library code, and the observable behaviors are covered by tests and command spot-checks.

### Gaps Summary

No gaps found. Phase 2 satisfies the trust-layer goal without introducing Phase 3 retrieval-ranking, parser-upgrade, or eval-failure-grouping scope.

---

_Verified: 2026-05-13T14:20:31Z_
_Verifier: Claude (gsd-verifier)_
