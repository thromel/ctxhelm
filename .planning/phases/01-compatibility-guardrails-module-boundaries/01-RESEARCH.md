# Phase 01: Compatibility Guardrails & Module Boundaries - Research

**Researched:** 2026-05-13
**Domain:** Rust CLI integration tests, MCP protocol compatibility, serde JSON contracts, safe module refactoring
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
## Implementation Decisions

### Compatibility strategy
- **D-01:** Characterization tests come before module splitting. Downstream planning should first lock current behavior through binary CLI tests, MCP tests, and contract-shape tests, then perform module extraction behind existing public functions.
- **D-02:** Preserve current public JSON field names, enum encodings, tool names, resource URI shapes, prompt names, and Markdown/report headings unless a test intentionally documents an additive backward-compatible change.
- **D-03:** Treat `ContextPlan`, `ContextPack`, `EvalTrace`, historical eval reports, MCP `structuredContent`, and major CLI JSON outputs as public compatibility surfaces for this phase.

### CLI guardrails
- **D-04:** Add binary-level CLI tests for representative core commands rather than only renderer/helper tests. The minimum command set should include `--help`, `index`, `prepare-task`, `get-pack`, `search`, `related-tests`, `dependencies`, `eval history`, and `serve-mcp` startup/protocol smoke where practical.
- **D-05:** CLI tests should use real temporary repositories and temporary `CTXPACK_HOME`, matching the existing fixture style in `.planning/codebase/TESTING.md`.
- **D-06:** Exact test harness choice is flexible, but the default planning assumption should be `assert_cmd`-style binary tests or an equivalent Cargo-supported approach that exercises the compiled binary rather than only direct function calls.

### MCP guardrails
- **D-07:** MCP compatibility tests must preserve the deliberately small implemented tool surface: `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff`.
- **D-08:** MCP tests should cover both human-readable `content[0].text` and machine-readable `structuredContent` for changed tools, because real clients may consume either.
- **D-09:** Resource compatibility should cover repository resources, file/symbol resources, and session-scoped pack resources. It is acceptable in Phase 1 to characterize current session-scoped behavior rather than fix it; durability changes belong to Phase 4.

### Module boundaries
- **D-10:** Module splitting should follow existing crate responsibilities: keep public facades in each crate stable, move behavior into private modules, and avoid changing call sites outside the owning crate unless tests require a public facade update.
- **D-11:** Preferred first splits are by current concern, not by abstract utility: inventory/privacy-ish classification, lexical scoring/search, symbols, dependency graph, related tests, git/history/current diff, traces, planning, pack rendering, cards/eval, MCP schemas, MCP tools, MCP resources, and MCP prompts.
- **D-12:** Do not combine module splitting with new retrieval behavior. If a refactor reveals a bug, add a failing characterization/regression test first and keep the behavioral fix narrowly scoped.

### the agent's Discretion
- The planner may decide exact crate/module names, test crate layout, fixture helper names, and whether to use `assert_cmd`, `trycmd`, or direct `std::process::Command` binary tests, provided the result exercises real binary behavior and preserves existing public surfaces.
- The planner may decide how many golden fixtures are necessary, but must avoid brittle source-text snapshots where structured field assertions are more stable.

### Deferred Ideas (OUT OF SCOPE)
## Deferred Ideas

- Inventory freshness, safe source revalidation, privacy-policy expansion, and structured operational diagnostics belong to Phase 2.
- Candidate fusion, graph-ranked targets, signal attribution, parser-backed precision, and historical-eval metric expansion belong to Phase 3.
- Codex/Claude smoke scripts, MCP reconnect behavior, and pack-resource durability belong to Phase 4.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CONT-01 | Maintainer can run binary-level CLI tests that exercise core commands and verify output shape, repo path handling, and write side effects. | Use Cargo integration tests under `crates/ctxpack/tests/` with `assert_cmd`, real temp git repos, command-local `CTXPACK_HOME`, and JSON parsing assertions. |
| CONT-02 | Maintainer can change internal modules without changing public JSON contracts for context plans, packs, eval traces, MCP structured content, and CLI outputs. | Add field-set/type-shape tests using `serde_json::Value` over `ContextPlan`, `ContextPack`, `EvalTrace`, `HistoricalEvalReport`, CLI JSON outputs, and MCP `structuredContent`. |
| CONT-03 | Maintainer can run MCP handler/resource/prompt tests that verify current tool names, resource URI shapes, session behavior, and error responses. | Extend existing `ctxpack-mcp` in-process JSON-RPC tests for exact tool/resource/prompt lists, all six implemented tools, file/symbol/repo/pack resources, and protocol error codes. |
| CONT-04 | Maintainer can split large modules into focused submodules while preserving existing CLI, MCP, and library behavior. | Use crate-root facade modules with `pub use`, move code in small concern slices after guardrails pass, and rerun focused plus workspace tests after each split. |
</phase_requirements>

## Summary

Phase 1 should be planned as a protection phase, not a product behavior phase. The existing repo already has good inline unit and integration-style tests, but it does not yet have binary-level tests for the compiled `ctxpack` executable. The highest-leverage first step is to add CLI integration tests in `crates/ctxpack/tests/` using `assert_cmd` and the repo's existing temp-repo fixture style. These tests should parse JSON where possible and inspect stable field names, paths, side effects, and protocol lines instead of snapshotting entire dynamic outputs.

For MCP and JSON compatibility, the current code is already close to the right shape: `ctxpack-mcp` has in-process JSON-RPC tests, `ctxpack-core` has explicit serde field-name tests, and all protocol responses are built with `serde_json::Value`. Phase 1 should extend these into compatibility tests before moving code. MCP tests must preserve the intentionally small implemented tool set and must assert both `structuredContent` and `content[0].text`, matching the MCP specification's backwards-compatibility guidance for structured tool results.

**Primary recommendation:** Plan Wave 0 as compatibility harness work, then split modules behind unchanged crate-root facades one crate at a time: `ctxpack-index`, then `ctxpack-compiler`, then `ctxpack-mcp`.

## Project Constraints (from CLAUDE.md and AGENTS.md)

- Keep ctxpack local-first, source-safe, and read-only; do not add autonomous editing, cloud indexing, cloud embeddings, or cloud reranking.
- Product surface remains agent-native: AGENTS.md, MCP, and thin native adapters. CLI is for setup, debugging, and automation.
- Prefer small typed contracts over stringly typed output.
- Preserve the Rust workspace and typed-contract architecture unless measurement justifies a change.
- Add focused tests for context selection, privacy, generated agent instructions, and now public compatibility surfaces.
- Do not run user project tests automatically from ctxpack; ctxpack may recommend commands only.
- Validation for implementation work remains `cargo test --workspace`; after CLI changes also run `cargo run -p ctxpack -- --help`.
- Project-local skill directories `.claude/skills/` and `.agents/skills/` were not present during research.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust built-in test harness | rustc/cargo 1.87.0 locally | Unit and integration test execution | Cargo integration tests are the standard way to run package-level tests and Cargo sets `CARGO_BIN_EXE_<name>` for binary execution. |
| `assert_cmd` | 2.2.2, published 2026-05-11 | Binary-level CLI tests | Official docs describe `Command::cargo_bin`, env/current-dir setup, stdin, timeouts, and stdout/stderr assertions. This matches CONT-01. |
| `predicates` | 3.1.4, published 2026-02-11 | Text predicates for help/Markdown output | Pairs with `assert_cmd` for stable substring/contains assertions without full-output snapshots. |
| `serde_json` | existing workspace dep, lockfile has 1.0.149 | Structured JSON parsing and field-shape assertions | Already used in contracts and MCP tests; avoids string matching for public JSON. |
| `tempfile` | existing workspace dep, lockfile has 3.27.0 | Isolated repos and `CTXPACK_HOME` fixtures | Already used throughout index/compiler/MCP tests; keep fixture style consistent. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `snapbox` | 1.2.1, published 2026-04-02 | Snapshot command output and filesystem changes | Optional only if side-effect directory comparisons become repetitive. Its docs position it for CLI stdout/stderr and filesystem changes. |
| `trycmd` | 1.2.0, published 2026-03-23 | Markdown/TOML-driven CLI snapshot suites | Do not use as the default for this phase. Reserve for stable help examples or README command snippets after binary tests exist. |
| `insta` | 1.47.2, published 2026-03-30 | Snapshot testing with redactions/filters | Optional for recursive JSON shape snapshots with redactions, but structured `serde_json::Value` assertions are less brittle for Phase 1. |
| `schemars` | 1.2.1, published 2026-02-01 | JSON Schema generation | Do not add in Phase 1 unless the team decides to publish MCP `outputSchema` or external schema files. Field-shape tests are enough now. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `assert_cmd` | Direct `std::process::Command` plus `env!("CARGO_BIN_EXE_ctxpack")` | Fewer dependencies, but more boilerplate for assertions, timeouts, and output inspection. |
| Structured JSON assertions | Full `insta` snapshots | Snapshots catch broad drift but churn on UUIDs, hashes, paths, ordering, and formatting unless carefully redacted. |
| Hand-written per-command fixtures | `trycmd` cases | `trycmd` is strong for many simple commands, but this phase needs dynamic temp repos, temp homes, and side-effect checks. |

**Installation:**

```bash
cargo add --dev assert_cmd@2.2.2 predicates@3.1.4 -p ctxpack
# Optional only if the plan adds snapshot side-effect tests:
cargo add --dev snapbox@1.2.1 -p ctxpack
```

**Version verification:** Verified with `cargo search`, `cargo info`, and crates.io API on 2026-05-13. Local toolchain is `rustc 1.87.0`, `cargo 1.87.0`, and `git 2.45.1`.

## Architecture Patterns

### Recommended Project Structure

```text
crates/ctxpack/
|-- Cargo.toml                 # add assert_cmd/predicates dev-deps here
|-- src/main.rs                # keep command definitions and renderer helpers
`-- tests/
    |-- cli_compat.rs          # binary-level compatibility tests
    `-- common/
        `-- mod.rs             # temp repo, git, JSON shape helpers

crates/ctxpack-core/src/
|-- contracts.rs               # extend contract serialization shape tests
`-- lib.rs                     # unchanged public exports

crates/ctxpack-index/src/
|-- lib.rs                     # facade: mod declarations + pub use
|-- inventory.rs
|-- search.rs
|-- symbols.rs
|-- related_tests.rs
|-- dependencies.rs
|-- git.rs
`-- traces.rs

crates/ctxpack-compiler/src/
|-- lib.rs                     # facade: public API preserved
|-- planning.rs
|-- packs.rs
|-- cards.rs
|-- eval.rs
`-- render.rs

crates/ctxpack-mcp/src/
|-- lib.rs                     # facade: run_stdio_server/run_server preserved
|-- protocol.rs
|-- schemas.rs
|-- tools.rs
|-- resources.rs
`-- prompts.rs
```

### Pattern 1: Binary Tests Live Beside the CLI Package

**What:** Add integration tests under `crates/ctxpack/tests/`, not the workspace root and not inline in `main.rs`.

**When to use:** Any test that must execute the compiled `ctxpack` binary, validate Clap wiring, verify stdout/stderr, or inspect CLI side effects.

**Why:** Cargo integration tests are separate binaries. Cargo automatically builds binary targets for package integration tests and exposes `CARGO_BIN_EXE_<name>` for locating them. `assert_cmd::Command::cargo_bin("ctxpack")` wraps this pattern.

**Plan implication:** Put CLI compatibility tests in one integration test target, e.g. `crates/ctxpack/tests/cli_compat.rs`, to avoid many separate integration-test crates and to share fixture helpers.

### Pattern 2: Structured Assertions First, Golden Snapshots Second

**What:** Parse JSON outputs with `serde_json::from_slice::<Value>` and assert key sets, enum values, array element shapes, and selected stable values. Only use text predicates for help/Markdown headings and only use snapshots after dynamic fields are redacted.

**When to use:** `prepare-task`, `get-pack --format json`, `search`, `related-tests`, `dependencies`, `eval history --format json`, MCP `structuredContent`, and contract structs.

**Example shape guard:** Assert field names such as `taskId`, `taskType`, `targetFiles`, `packOptions`, `repoId`, `taskHash`, `targetAgent`, and `sourceTextLogged`. Assert dynamic UUID/hash values by type or non-empty string, not exact value.

### Pattern 3: MCP Compatibility Tests Stay In-Process First

**What:** Extend the existing `handle_line`/`run_server` tests in `ctxpack-mcp` for protocol behavior, then add one CLI binary `serve-mcp` smoke.

**When to use:** Tool/resource/prompt lists, handler dispatch, session-scoped pack resources, JSON-RPC errors, and `structuredContent` plus text compatibility.

**Why:** In-process tests are deterministic and already match local style. Binary `serve-mcp` tests should only prove that the compiled executable starts and speaks newline-delimited JSON-RPC over stdio.

### Pattern 4: Crate-Root Facade During Module Splits

**What:** Split implementation into private modules while preserving the crate root's public functions, structs, and imports.

**Example:**

```rust
// crates/ctxpack-index/src/lib.rs
mod dependencies;
mod inventory;
mod search;

pub use dependencies::{dependency_edges, related_dependency_edges, DependencyEdge, DependencyOptions};
pub use inventory::{
    build_inventory, load_inventory, load_or_build_inventory, write_inventory, FileInventoryEntry,
    InventoryOptions, InventoryReport, RepoInventory,
};
pub use search::{lexical_search, SearchOptions, SearchResult};
```

**When to use:** Every split in `ctxpack-index`, `ctxpack-compiler`, and `ctxpack-mcp`.

**Plan implication:** Do not update CLI or MCP call sites just because code moved. If a public API must change, add a characterization test first and treat the API change as a separate explicit compatibility decision.

### Recommended Plan Decomposition

1. **Wave 0 - Compatibility harness:** add CLI integration test deps, common temp repo helpers, JSON shape helpers, and first `--help` plus `index` binary tests.
2. **Wave 1 - CLI command coverage:** cover `prepare-task`, `get-pack`, `search`, `related-tests`, `dependencies`, `eval history`, and `serve-mcp` binary smoke.
3. **Wave 2 - Public JSON contracts:** extend core/compiler contract tests for `ContextPlan`, `ContextPack`, `EvalTrace`, `HistoricalEvalReport`, and representative CLI JSON outputs.
4. **Wave 3 - MCP compatibility:** harden in-process MCP tests for exact tool/resource/prompt surfaces, all six tool `structuredContent` shapes, text fallbacks, resource reads, session pack resources, and error codes.
5. **Wave 4 - Module splitting:** split `ctxpack-index` first, then `ctxpack-compiler`, then `ctxpack-mcp`, rerunning focused tests after each crate and `cargo test --workspace` at the end.
6. **Wave 5 - Cleanup and docs:** update only internal code organization notes if needed; do not change user-facing behavior or README command promises unless a test documents an intentional additive compatibility guarantee.

### Anti-Patterns to Avoid

- **Snapshot everything:** Full stdout snapshots will churn on UUIDs, task hashes, absolute temp paths, and formatting.
- **Move code before guardrails:** This defeats the phase goal. Characterization tests must fail on public drift before module work starts.
- **Refactor across crate boundaries:** Keep changes inside the owning crate unless a test exposes an existing public facade gap.
- **Adopt a full MCP SDK during this phase:** That is protocol migration, not compatibility guardrails.
- **Fix pack-resource durability now:** Phase 1 should characterize current session-scoped behavior; durability belongs to Phase 4.

## CLI Guardrail Details

Minimum binary tests for CONT-01:

| Command | Assertions |
|---------|------------|
| `ctxpack --help` | exits 0; contains command names; no exact full help snapshot. |
| `ctxpack index --repo <repo>` | uses explicit repo; writes inventory under command-local `CTXPACK_HOME`; excludes sensitive/generated defaults; stdout reports counts. |
| `ctxpack prepare-task ... --repo <repo> --mode bug-fix --target-agent codex` | stdout parses as `ContextPlan`; stable camelCase fields; expected target/test paths; trace append side effect exists. |
| `ctxpack get-pack ... --format json` | stdout parses as `ContextPack`; includes `repoId`, `taskHash`, `targetAgent`, `budget`, sections, warnings, privacy status. |
| `ctxpack get-pack ... --format markdown` | contains stable headings such as `# Context Pack` and provenance header fields; avoid full snapshot. |
| `ctxpack search <query> --repo <repo>` | JSON array shape and source-free summary fields. |
| `ctxpack related-tests <path> --repo <repo>` | JSON array shape and targeted command presence. |
| `ctxpack dependencies <path> --repo <repo>` | JSON array shape with `sourcePath`, `targetPath`, and `kind`. |
| `ctxpack eval history --repo <repo> --limit 1 --format json` | JSON report shape, source-free fields, bounded result count. |
| `ctxpack serve-mcp` | feed `initialize` and `tools/list` over stdin; parse one JSON-RPC response per line. |

## MCP Compatibility Guardrails

Preserve exact implemented tool names and order unless an additive change is intentionally documented:

```text
prepare_task
search
related
get_pack
related_tests
current_diff
```

MCP test matrix for CONT-03:

| Surface | Required Assertions |
|---------|---------------------|
| `initialize` | `protocolVersion` stays current local value, `serverInfo.name == "ctxpack"`, capabilities include tools/resources/prompts with `listChanged: false`. |
| `tools/list` | exactly six tools, stable names/order, stable required args and enum encodings, `includeCurrentDiff` camelCase input field preserved. |
| `tools/call prepare_task` | `structuredContent` is a `ContextPlan`; `content[0].text` contains serialized JSON fallback; pack resource URIs include brief/standard/deep. |
| `tools/call get_pack` | `structuredContent` is a `ContextPack`; Markdown mode text fallback and JSON mode structured output both work. |
| `tools/call search` | structured `files`, `symbols`, and `privacyStatus` shape; source text not returned for search summaries. |
| `tools/call related` | structured `resolvedPaths`, `symbolMatches`, `relatedTests`, `coChangeHints`, `dependencyEdges`, and warnings shape. |
| `tools/call related_tests` | structured array of test results and text fallback. |
| `tools/call current_diff` | structured `staged`, `unstaged`, `untracked`, `excluded`, `privacyStatus`; source text remains false. |
| `resources/list` | exact repo resources: `ctxpack://repo/summary`, `ctxpack://repo/test-map`, `ctxpack://repo/dependency-graph`, `ctxpack://pack/guide`. |
| `resources/read` | repo resources, safe file slices, `ctxpack://symbol/<name>`, and same-process `ctxpack://pack/<id>` reads. |
| `prompts/list` and `prompts/get` | six prompt names and stable message/content shape. |
| Errors | parse error `-32700`, method not found `-32601`, invalid params `-32602`, unknown tool/resource errors. |

Important MCP source finding: the official 2025-06-18 tool spec says a tool returning `structuredContent` should also return serialized JSON in a text content block for backwards compatibility. Phase 1 should make that an explicit invariant for current tool responses.

## Public JSON Contract Guardrails

Use field-shape tests for these surfaces:

| Surface | Stable Fields |
|---------|---------------|
| `ContextPlan` | `taskId`, `taskType`, `confidence`, `targetFiles`, `relatedTests`, `recommendedCommands`, `packOptions`, `missingInfoQuestions`, `riskFlags`, `privacyStatus`. |
| `ContextPack` | `id`, `taskId`, `repoId`, `taskHash`, `taskType`, `targetAgent`, `budget`, `sections`, `tokenEstimate`, `confidence`, `warnings`, `privacyStatus`. |
| `EvalTrace` | `id`, `repoId`, `taskHash`, `taskType`, `packId`, `targetAgent`, `budget`, `recommendedFiles`, `recommendedTests`, `recommendedCommands`, `createdAtUnixSeconds`, `sourceTextLogged`. |
| `HistoricalEvalReport` | current compiler report fields with `camelCase`; assert source-free properties and metric field names. |
| CLI JSON outputs | same shape as underlying structs or arrays; tests should parse stdout and assert no snake_case public field drift. |
| MCP `structuredContent` | same struct shape plus MCP wrapper shape; text fallback must remain present. |

Recommended helper:

```rust
fn object_keys(value: &serde_json::Value) -> Vec<&str> {
    let mut keys = value.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}
```

Prefer key-set and selected-value assertions over JSON Schema generation for Phase 1. JSON Schema is useful later if ctxpack publishes schemas or MCP `outputSchema`, but it is extra dependency and maintenance surface right now.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Finding and running the compiled CLI binary | Custom target-dir path guessing | `assert_cmd::Command::cargo_bin("ctxpack")` | Cargo already exposes binary paths for integration tests; path guessing breaks across profiles and workspaces. |
| CLI stdout/stderr assertions | Manual process wrappers everywhere | `assert_cmd` plus `predicates` | Gives success/failure, stdin, timeout, env, cwd, stdout, and stderr support. |
| Temp repo/home fixtures | Shared persistent fixture dirs | `tempfile::TempDir` plus real `git` commands | Existing tests already rely on isolated real repos and avoid persistent local state. |
| JSON contract checks | String contains checks for JSON | `serde_json::Value` shape assertions | Protects field names and enum encodings without formatting brittleness. |
| MCP protocol parsing in tests | String splitting JSON-RPC responses | Existing `handle_line`/`run_server` plus `serde_json` | The server already has deterministic in-process boundaries. |
| Module split public API tracking | Broad call-site rewrites | crate-root `mod` plus `pub use` facades | Preserves existing imports while allowing internal files to shrink. |

**Key insight:** The complex part is not executing commands; it is deciding which outputs are public contracts and keeping dynamic fields from making guardrails noisy. Structured assertions should be the default compatibility mechanism.

## Runtime State Inventory

This is a module-boundary/refactor phase, so runtime state was audited for non-source places that could keep old behavior or names after files move.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | Existing ctxpack local data under `CTXPACK_HOME` or `~/.ctxpack/repos/<repo-id>/`: `inventory.json` and `traces.jsonl` use public JSON contracts such as repo IDs, task hashes, file paths, and eval trace fields. No Rust module names are stored. | No data migration for module splits. Add compatibility tests so code moves do not change serialized field names. |
| Live service config | None found in repo. MCP client configs may refer to the `ctxpack serve-mcp` command, not Rust module paths. | Preserve CLI command name and `serve-mcp` behavior. No external service patch in Phase 1. |
| OS-registered state | None found in repo for launchd/systemd/pm2/task scheduler style registrations. | None. |
| Secrets/env vars | `CTXPACK_HOME` is a public env var used in tests/runtime. No secret key names tied to module names found. | Preserve `CTXPACK_HOME`; binary tests should set it per command, not globally when possible. |
| Build artifacts | Cargo `target/` may contain stale incremental artifacts after moves; no committed build artifacts. | Rerun Cargo tests after each split. If Cargo gets stuck or stale, planner may include `cargo clean -p <crate>` as a troubleshooting fallback, not a default task. |

## Common Pitfalls

### Pitfall 1: Tests Exercise Helpers Instead Of The Binary

**What goes wrong:** CLI behavior breaks in Clap wiring, argument defaults, current-dir handling, or stdout rendering while unit tests still pass.

**Why it happens:** Existing CLI tests are renderer/helper tests in `main.rs`, not compiled-binary tests.

**How to avoid:** Put representative command tests in `crates/ctxpack/tests/cli_compat.rs` using `assert_cmd`.

**Warning signs:** A plan claims CONT-01 with only direct Rust function calls.

### Pitfall 2: Dynamic Fields Make Golden Tests Noisy

**What goes wrong:** Snapshots fail on UUIDs, hashes, absolute temp paths, timestamps, or ordering.

**Why it happens:** Public JSON contains dynamic `taskId`, `id`, `repoId`, `taskHash`, temp paths, and sometimes git-derived order.

**How to avoid:** Assert shape and selected stable values; redact dynamic fields if snapshots are introduced.

**Warning signs:** Full JSON snapshots include temp directory paths or UUID literals.

### Pitfall 3: Global Environment Leaks Across Tests

**What goes wrong:** Tests pass or fail depending on execution order because `CTXPACK_HOME` or process cwd remains changed.

**Why it happens:** Existing inline tests use global env and sometimes cwd with an `env_lock`; binary tests can avoid this by setting command-local env and cwd.

**How to avoid:** Prefer `.env("CTXPACK_HOME", temp_home)` and `.current_dir(repo)` on `assert_cmd::Command`. Use a mutex only when mutating process-global env/cwd in in-process tests.

**Warning signs:** New tests call `std::env::set_var` without a guard or cleanup.

### Pitfall 4: MCP Pack Resources Are Session-Scoped

**What goes wrong:** A test prepares a pack in one process and reads the resource in another, then concludes resources are broken.

**Why it happens:** Current pack-resource cache is process-local in `ctxpack-mcp`.

**How to avoid:** Characterize same-process/session behavior in Phase 1. Do not require cross-process resource durability until Phase 4.

**Warning signs:** A binary smoke launches separate `serve-mcp` commands for `prepare_task` and `resources/read`.

### Pitfall 5: Module Moves Accidentally Change Public Imports

**What goes wrong:** CLI/MCP call sites or external crate imports change because types moved into submodules without re-export.

**Why it happens:** Splits are treated as redesigns instead of private implementation moves.

**How to avoid:** Keep crate-root facades stable with `pub use`; run compiler tests and binary/MCP compatibility tests after each move.

**Warning signs:** A module split plan includes broad edits to `crates/ctxpack/src/main.rs` or `ctxpack-mcp` call sites before tests fail.

### Pitfall 6: MCP Spec Drift Becomes Product Scope

**What goes wrong:** The phase turns into an MCP SDK/protocol migration because current code uses a local protocol version and hand-written JSON-RPC.

**Why it happens:** Official MCP docs are active and protocol versions evolve.

**How to avoid:** Preserve current behavior in Phase 1. Only document spec-backed invariants that already align with the product, especially structured content plus text fallback.

**Warning signs:** Tasks propose replacing the stdio server with a new SDK or changing protocol version without a compatibility requirement.

## Code Examples

Verified patterns from official docs and current repo style.

### Binary CLI Test With Command-Local State

```rust
// Source: assert_cmd docs and Cargo integration-test binary behavior.
use assert_cmd::Command;
use serde_json::Value;

#[test]
fn prepare_task_binary_preserves_context_plan_shape() {
    let fixture = FixtureRepo::new();

    let output = Command::cargo_bin("ctxpack")
        .unwrap()
        .env("CTXPACK_HOME", &fixture.home)
        .args([
            "prepare-task",
            "fix requireSession bug",
            "--repo",
            fixture.repo.to_str().unwrap(),
            "--mode",
            "bug-fix",
            "--target-agent",
            "codex",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["taskType"], "bug_fix");
    assert_eq!(value["targetFiles"][0]["path"], "src/auth/session.ts");
    assert!(value.get("task_id").is_none());
}
```

### MCP Structured And Text Compatibility

```rust
// Source: current ctxpack-mcp handler style and MCP structuredContent guidance.
#[test]
fn prepare_task_returns_structured_content_and_text_fallback() {
    let response = handle_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"prepare_task","arguments":{"task":"fix bug","repo":"/tmp/repo","mode":"bug_fix"}}}"#)
        .unwrap();

    assert_eq!(response["result"]["structuredContent"]["taskType"], "bug_fix");
    assert!(response["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("\"taskType\""));
}
```

### Stable JSON Shape Helper

```rust
fn assert_object_keys(value: &serde_json::Value, expected: &[&str]) {
    let mut actual = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    actual.sort_unstable();

    let mut expected = expected.to_vec();
    expected.sort_unstable();

    assert_eq!(actual, expected);
}
```

### Facade-Preserving Module Split

```rust
// crates/ctxpack-mcp/src/lib.rs
mod protocol;
mod resources;
mod tools;

pub use protocol::{run_server, run_stdio_server};
```

Keep the public functions visible from the crate root. Move tests with the private implementation when they test internals; keep compatibility tests at the facade level.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Guessing binary paths in integration tests | Cargo integration tests plus `CARGO_BIN_EXE_<name>` and `assert_cmd::cargo_bin` | Stable Cargo behavior; official docs current in 2026 | Use package integration tests for real CLI coverage. |
| Full golden snapshots for CLI JSON | Structured `serde_json::Value` field-shape assertions, snapshots only with redactions | Snapshot tooling is mature, but dynamic agent outputs need restraint | Less test churn and clearer compatibility failures. |
| Tool result text only | MCP `structuredContent` plus serialized JSON text fallback | MCP 2025-06-18 spec documents this compatibility pattern | Tests must assert both machine and human-readable paths. |
| One huge module refactor | Facade-preserving internal modules with characterization tests first | Standard Rust crate organization pattern | Keeps public APIs stable while reducing internal file size. |

**Deprecated/outdated:**

- `assert_cli`: Do not use. `assert_cmd` docs identify it as the successor.
- Full source-text snapshots for search/plan/eval outputs: avoid because ctxpack's product contract emphasizes source-free summaries for many surfaces.
- MCP SDK migration as compatibility work: out of scope for Phase 1.

## Open Questions

1. **Should `snapbox` be added now or deferred?**
   - What we know: `snapbox` is current and useful for filesystem side effects.
   - What's unclear: Phase 1 may get enough coverage from `assert_cmd`, `tempfile`, and `serde_json`.
   - Recommendation: Start without `snapbox`; add it only if side-effect assertions become repetitive.

2. **How many exact JSON shape tests are enough?**
   - What we know: Required public surfaces are named in D-03 and CONT-02.
   - What's unclear: Whether every CLI command should have a full field-set helper or selected representative outputs.
   - Recommendation: Cover every named public contract plus representative CLI wrappers; avoid exhaustive duplicate tests that mirror lower-layer tests.

3. **Should MCP `outputSchema` be added?**
   - What we know: MCP spec supports optional `outputSchema`, and `schemars` is current.
   - What's unclear: Adding schemas changes the discoverable MCP contract and increases maintenance scope.
   - Recommendation: Do not add `outputSchema` in Phase 1 unless the user explicitly chooses it as an additive compatibility surface.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust compiler | Workspace build/tests | yes | `rustc 1.87.0` | None needed |
| Cargo | Test runner and dependency management | yes | `cargo 1.87.0` | None needed |
| git CLI | Fixture repos, index/history/current diff tests | yes | `git 2.45.1` | No good fallback for this phase |
| crates.io access | Version verification and adding test deps | yes | API reachable 2026-05-13 | Manual Cargo.toml edit if `cargo add` is unavailable |
| Node.js | GSD tooling only | yes | `node` ran init/version scripts | Not required by ctxpack tests |

**Missing dependencies with no fallback:**
- None found.

**Missing dependencies with fallback:**
- None found.

## Implementation Validation Commands

`.planning/config.json` explicitly sets `workflow.nyquist_validation` to `false`, so no Nyquist validation architecture is included. Phase 1 still needs these implementation validation commands:

```bash
cargo test -p ctxpack --test cli_compat
cargo test -p ctxpack-core contracts
cargo test -p ctxpack-mcp
cargo test --workspace
cargo run -p ctxpack -- --help
```

## Sources

### Primary (HIGH confidence)

- Local project files: `.planning/phases/01-compatibility-guardrails-module-boundaries/01-CONTEXT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`, `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/STRUCTURE.md`, `.planning/codebase/TESTING.md`.
- Local code: `Cargo.toml`, `crates/ctxpack/src/main.rs`, `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`.
- Cargo Book, Cargo Targets: https://doc.rust-lang.org/cargo/reference/cargo-targets.html
- assert_cmd 2.2.2 docs: https://docs.rs/assert_cmd/latest/assert_cmd/
- MCP 2025-06-18 Tools spec: https://modelcontextprotocol.io/specification/2025-06-18/server/tools
- MCP 2025-06-18 Resources spec: https://modelcontextprotocol.io/specification/2025-06-18/server/resources
- MCP 2025-06-18 Prompts spec: https://modelcontextprotocol.io/specification/2025-06-18/server/prompts
- crates.io API and `cargo info` version checks for `assert_cmd`, `predicates`, `insta`, `trycmd`, `snapbox`, and `schemars`.

### Secondary (MEDIUM confidence)

- trycmd docs: https://docs.rs/trycmd/
- snapbox docs: https://docs.rs/snapbox
- Insta docs: https://insta.rs/docs/

### Tertiary (LOW confidence)

- None used for recommendations.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - Current versions verified through Cargo/crates.io and official docs; matches phase constraints.
- Architecture: HIGH - Based on live repo structure, Cargo behavior, and existing test conventions.
- Pitfalls: HIGH - Derived from current code patterns, official MCP/Cargo behavior, and prior ctxpack execution history.
- Module split order: MEDIUM-HIGH - Strongly grounded in crate ownership, but exact file boundaries should be adjusted during implementation to minimize diff risk.

**Research date:** 2026-05-13
**Valid until:** 2026-06-12 for Rust test harness recommendations; recheck MCP spec and crate versions before changing protocol/schema behavior.
