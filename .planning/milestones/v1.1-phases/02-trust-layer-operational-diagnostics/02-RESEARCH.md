# Phase 02: Trust Layer & Operational Diagnostics - Research

**Researched:** 2026-05-13
**Domain:** Rust local repository inventory, privacy-gated source reads, CLI/MCP diagnostics
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
## Implementation Decisions

### Freshness Policy
- Prefer a centralized inventory freshness check over scattered per-call timestamp checks.
- User-facing read paths should rebuild stale inventory when safe and cheap enough, but return structured stale-cache diagnostics when rebuild fails or is disabled.
- Freshness should account for repository files, ignore files, inventory options, and deleted/renamed/sensitive/generated path changes.
- Preserve existing command behavior where possible; add diagnostics as compatible fields rather than replacing current outputs.

### Privacy And Source Reads
- Move sensitive/generated/binary/oversized/unreadable classification toward a centralized tested policy used by inventory, packs, file resources, cards, current diff, and historical labels.
- Revalidate every source-bearing path immediately before reading snippets for packs, MCP file resources, and generated cards.
- Treat package-manager auth files, SSH private keys, cloud credential JSON, credential-like dotfiles, vendored/generated output, binaries, non-UTF-8 files, oversized files, and unreadable files conservatively by default.
- Do not introduce cloud uploads, cloud embeddings, or remote reranking.

### Diagnostics Contract
- Add stable structured diagnostics for stale inventory, weak/low-information plans, missing git, git timeouts, unreadable files, skipped files, parse gaps, partial graph/test/history coverage, and cache/trace write failures.
- Preserve existing `riskFlags` compatibility while making diagnostics richer and more machine-readable for CLI and MCP clients.
- Diagnostics should be source-free: include paths, roles, reason codes, counts, hashes, and command/error categories, but not source snippets or prompt text.
- Prefer typed contracts in `ctxpack-core` over ad hoc strings in CLI/MCP renderers.

### Cache And Trace Writes
- Context retrieval should remain usable in constrained home-directory environments.
- Trace/cache write failures should be visible and non-fatal for read-oriented operations unless the user explicitly requested a write operation.
- Expose enough cache/trace status for users and tests to understand where local state is written and whether it was skipped.

### the agent's Discretion
- The agent may choose the exact Rust type names, module placement, and command flags if they preserve public compatibility and stay within the requirements.
- The agent may stage the work across multiple plans to protect the Phase 1 guardrails.

### Claude's Discretion
### the agent's Discretion
- The agent may choose the exact Rust type names, module placement, and command flags if they preserve public compatibility and stay within the requirements.
- The agent may stage the work across multiple plans to protect the Phase 1 guardrails.

### Deferred Ideas (OUT OF SCOPE)
## Deferred Ideas

- DIAG-03 historical eval failure grouping belongs to Phase 3.
- Ranking lift, signal ablations, parser upgrades, and RefactoringMiner fixed-range eval gates belong to Phase 3.
- Real Codex/Claude client durability and MCP pack-resource persistence semantics belong to Phase 4.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SAFE-01 | User-facing read paths detect stale inventory metadata before returning search, symbol, test, dependency, pack, card, or MCP results. | Use one `load_or_refresh_inventory`/freshness envelope in `ctxpack-index`, then route all read APIs through it. |
| SAFE-02 | User-facing read paths either rebuild stale inventory or return structured stale-cache diagnostics that explain what changed. | Return `InventoryLoadReport` with `status`, `rebuildAttempted`, `rebuildSucceeded`, and source-free stale reasons. |
| SAFE-03 | ctxpack classifies sensitive and generated files through a centralized privacy policy with table-driven tests for common credential, auth, generated, vendored, and binary path families. | Move path/content classification from `inventory.rs` helpers into a tested source policy module used by inventory and source reads. |
| SAFE-04 | Pack, file-resource, and card generation revalidate every source-bearing path against the current safe inventory immediately before reading source text. | Replace direct `fs::read_to_string` in pack/resource code with `read_safe_source_slice`. |
| SAFE-05 | Unreadable, non-UTF-8, oversized, skipped, or externally unavailable inputs produce structured diagnostics instead of silently becoming empty matches. | Stop `unwrap_or_default` content reads in search/symbol/dependency/test paths; collect diagnostics by reason code. |
| SAFE-06 | Trace/cache writes are visible and controllable enough that context retrieval can remain usable in read-only or constrained home-directory environments. | Add non-fatal trace/cache write wrappers and expose `cacheStatus`/`traceStatus` diagnostics in CLI/MCP surfaces. |
| DIAG-01 | User can see structured diagnostics on context plans for low-information tasks, stale cache, missing git, git timeout, unreadable files, skipped files, parse gaps, and partial graph/test/history coverage. | Add `diagnostics: Vec<Diagnostic>` to core contracts and emit typed diagnostics from compiler/index/git layers. |
| DIAG-02 | CLI and MCP outputs expose diagnostics in stable structured fields while preserving existing risk-flag compatibility. | Add fields, do not remove `riskFlags`; mirror legacy risk-like diagnostics into `riskFlags`. |
| DIAG-04 | Maintainer can test weak-plan scenarios with deterministic fixtures instead of relying on manual interpretation. | Extend Phase 1 CLI/MCP guardrails and inline fixture repos with stale, weak-task, read failure, and constrained-home cases. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Keep ctxpack local-first, read-only in the product sense, and agent-native through AGENTS.md, MCP, and thin native adapter surfaces.
- Do not add autonomous editing, cloud indexing, cloud embeddings, or cloud reranking by default.
- Prefer small typed contracts over stringly typed command output.
- Add focused tests for context selection, privacy, and generated agent instruction behavior.
- Preserve the Rust workspace architecture and typed contract layer unless there is measured reason to change.
- Use `thiserror` in library/domain errors and `anyhow::Result` at the CLI boundary.
- Return structured data contracts from library layers; keep formatting in CLI/MCP boundary helpers.
- Do not add ad hoc library logging; return structured reports or typed errors.
- Run `cargo test --workspace` before claiming implementation complete.
- Run `cargo run -p ctxpack -- --help` after CLI changes.
- No project-local `.claude/skills` or `.agents/skills` were found during research.

## Summary

Phase 2 should not add a new retrieval stack. The standard implementation path is to harden the existing Phase 1 module split by adding a typed trust layer around inventory loading and source reads. The trust layer belongs mostly in `ctxpack-index`, with stable additive contract fields in `ctxpack-core`, pack/card revalidation in `ctxpack-compiler`, and CLI/MCP projection in `ctxpack` and `ctxpack-mcp`.

The current high-risk facts are concrete: `load_or_build_inventory` reuses any existing inventory JSON without checking filesystem or ignore-policy freshness; several read paths convert unreadable or invalid UTF-8 files into empty strings; `prepare-task`, `get-pack`, and MCP equivalents fail if trace writes fail; packs read snippets directly from plan paths without current safe-inventory revalidation. These are Phase 2's implementation targets.

**Primary recommendation:** implement one typed `InventoryLoadReport + Diagnostic + SourceReadPolicy` path and make every user-facing read operation call it before returning source-bearing or inventory-derived data.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust workspace | rustc/cargo 1.87.0 | Existing implementation language and package manager | Matches current product and Phase 1 guardrails. |
| serde | 1.0.228 | Stable camelCase JSON contracts | Already used for `ContextPlan`, `ContextPack`, inventory, traces, MCP payloads. |
| serde_json | 1.0.149 | CLI JSON, MCP JSON-RPC, local JSON files | Existing public JSON behavior is Phase 1 protected. |
| ignore | 0.4.25 | Gitignore/custom-ignore aware repository walking | Already handles `.gitignore`, `.ctxpackignore`, `.cursorignore`; do not bypass it. |
| blake3 | 1.8.5 | File hashes, task hashes, repo-local state fingerprints | Fast deterministic hashes already in inventory and trace paths. |
| uuid | 1.23.1 | Repo/plan/pack/trace IDs | Existing contracts use UUIDs; preserve generated ID semantics. |
| thiserror | 2.0.18 | Library error enums | Existing pattern for typed domain errors. |
| anyhow | 1.0.102 | CLI boundary errors | Existing CLI pattern only; do not push `anyhow` into library contracts. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| clap | 4.6.1 | CLI flags such as optional trace/cache controls | Only for user-facing CLI additions. |
| tempfile | 3.27.0 | Temp repos, temp homes, historical worktrees | Use for deterministic stale/cache/privacy fixtures. |
| assert_cmd | 2.2.2 | Binary-level CLI compatibility tests | Extend `crates/ctxpack/tests/cli_compat.rs`. |
| predicates | 3.1.4 | CLI stdout assertions | Use sparingly with JSON shape assertions. |
| system git | 2.45.1 | diff/current-diff/history signals | Keep calls centralized in `git.rs`; surface timeout/missing-tool diagnostics. |
| system tar | bsdtar 3.8.1 | Historical eval archive extraction | Phase 2 should only diagnose availability; Phase 3 owns eval upgrades. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Existing `ignore` walker | Custom ignore parser | Do not use. Custom parsing will miss `.git/info/exclude`, parent ignores, and edge cases already handled by `ignore`. |
| Additive diagnostics on current contracts | Replace `riskFlags` | Do not replace. MCP/CLI clients already consume `riskFlags`; add `diagnostics` and mirror compatibility flags. |
| Direct `fs::read_to_string` in each module | One `read_safe_source_slice` policy API | Use the central API. It gives one privacy, UTF-8, size, unreadable, and diagnostic path. |
| Fatal trace/cache writes | Existing `append_eval_trace` everywhere | Keep `append_eval_trace` for explicit trace commands, but add non-fatal wrappers for read-oriented commands. |

**Installation:**
```bash
# No new crates are recommended for Phase 2.
cargo test --workspace
```

**Version verification:**
```bash
cargo metadata --format-version 1 --no-deps
cargo tree --workspace --depth 1
cargo info ignore serde serde_json clap tempfile blake3 thiserror uuid anyhow assert_cmd predicates
rustc --version
cargo --version
git --version
tar --version
```
Versions above were verified from `Cargo.lock`, `cargo tree`, `cargo info`, and local tool probes on 2026-05-13. `cargo info` does not expose publish dates; registry version metadata and docs URLs were verified, but publish dates were not available from the local tool.

## Architecture Patterns

### Recommended Project Structure

```text
crates/ctxpack-core/src/contracts.rs        # Add additive Diagnostic/CacheStatus/TraceStatus contracts
crates/ctxpack-index/src/policy.rs          # Central path/content/source-read policy
crates/ctxpack-index/src/freshness.rs       # Inventory metadata, freshness check, rebuild/load report
crates/ctxpack-index/src/inventory.rs       # Inventory build/write/load facade uses policy + freshness
crates/ctxpack-index/src/search.rs          # Uses trusted inventory + diagnostic-aware source reads
crates/ctxpack-index/src/symbols.rs         # Same read-policy path
crates/ctxpack-index/src/related_tests.rs   # Same read-policy path
crates/ctxpack-index/src/dependencies.rs    # Same read-policy path
crates/ctxpack-index/src/git.rs             # Git diagnostics stay centralized
crates/ctxpack-index/src/traces.rs          # Add non-fatal trace write status
crates/ctxpack-compiler/src/planning.rs     # Attach diagnostics; preserve riskFlags projection
crates/ctxpack-compiler/src/packs.rs        # Revalidate snippets before reading
crates/ctxpack-compiler/src/cards.rs        # Use trusted inventory and diagnostics
crates/ctxpack-mcp/src/tools.rs             # Expose diagnostics in structuredContent
crates/ctxpack-mcp/src/resources.rs         # Revalidate file resources with same policy
crates/ctxpack/src/main.rs                  # CLI projection, optional no-trace/cache flags if added
```

### Pattern 1: Additive Diagnostics Contract

**What:** Add source-free, camelCase diagnostics to stable core contracts while leaving existing fields intact.

**When to use:** Every result that can be stale, partial, weak, skipped, or degraded. Start with `ContextPlan`, `ContextPack`, MCP search/related/current_diff JSON, inventory reports, and cards reports.

**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub paths: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}
```

Keep this intentionally small. Avoid embedding arbitrary errors or source text. Add `diagnostics: Vec<Diagnostic>` with `#[serde(default)]` only if backwards deserialization is needed in tests.

### Pattern 2: Trusted Inventory Load Envelope

**What:** Replace direct `load_or_build_inventory` use on read paths with an envelope that returns both data and trust metadata.

**When to use:** `lexical_search`, `symbol_search`, `related_tests`, `test_map`, `dependency_edges`, `prepare_context_plan`, pack/card generation, MCP repo resources, and current diff filtering.

**Example:**
```rust
pub struct InventoryLoadReport {
    pub inventory: RepoInventory,
    pub diagnostics: Vec<Diagnostic>,
    pub freshness: InventoryFreshness,
    pub cache_status: CacheStatus,
}

pub fn load_or_refresh_inventory(
    repo_root: impl AsRef<Path>,
    options: &InventoryOptions,
) -> Result<InventoryLoadReport, InventoryError> {
    // 1. Load existing cache if present.
    // 2. Compare schema/policy/options/ignore fingerprints and file manifest.
    // 3. Rebuild if stale.
    // 4. If rebuild works but cache write fails, return the in-memory inventory plus warning.
}
```

### Pattern 3: Central Source-Read Policy

**What:** Make source-bearing reads go through one policy API that checks current safe inventory, role, generated/sensitive status, size cap, bytes, binary heuristics, and UTF-8 decoding.

**When to use:** Pack target/test snippets, MCP file resources, search content scoring, symbol extraction, dependency parsing, related-test scanning, and any future card content.

**Example:**
```rust
pub struct SourceRead {
    pub path: String,
    pub text: String,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn read_safe_source(
    repo_root: &Path,
    inventory: &RepoInventory,
    path: &str,
    max_bytes: u64,
) -> Result<Option<SourceRead>, InventoryError> {
    // Return Ok(None) with diagnostics for sensitive/generated/binary/oversized/unreadable/nonUtf8.
}
```

### Pattern 4: Compatibility Projection To `riskFlags`

**What:** Treat `diagnostics` as the richer source of truth and keep `riskFlags` as a legacy compatibility view for plan/pack clients.

**When to use:** Plan construction and pack rendering.

**Example:**
```rust
fn project_diagnostics_to_risk_flags(diagnostics: &[Diagnostic]) -> Vec<RiskFlag> {
    diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.severity, DiagnosticSeverity::Warning | DiagnosticSeverity::Error))
        .map(|diagnostic| RiskFlag {
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
        })
        .collect()
}
```

### Anti-Patterns to Avoid

- **Scattered timestamp checks:** stale cache behavior must be one code path, or CLI and MCP will drift.
- **More stringly `riskFlags`:** use typed diagnostics and project to risk flags only for compatibility.
- **Silent `unwrap_or_default` reads:** this is the current cause of unreadable/non-UTF-8 files looking like weak matches.
- **Reading snippets from plan paths:** plans can be stale; snippets must be revalidated at read time.
- **Failing context retrieval on trace writes:** trace/cache failures should be visible diagnostics, not fatal for read-oriented operations.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ignore-file semantics | Custom `.gitignore` parser | Existing `ignore::WalkBuilder` | Handles git ignore hierarchy and custom ignore files already. |
| Public JSON rendering | Manual JSON strings | `serde` contracts with camelCase tests | Phase 1 guards contract shape through serialization tests. |
| Source-read policy | Per-module `fs::read_to_string` branches | Central `SourceReadPolicy` in index | Prevents privacy drift between inventory, packs, cards, MCP resources, and search. |
| Git diagnostics | Ad hoc process calls outside `git.rs` | Existing `git_stdout_with_timeout` wrapper, extended diagnostics | Keeps missing git/timeouts/fallback behavior consistent. |
| CLI compatibility tests | Shell scripts only | `assert_cmd` binary tests plus fixture repos | Existing guardrails verify real binary behavior and `CTXPACK_HOME` side effects. |

**Key insight:** ctxpack's trust failure mode is not one bad branch; it is divergence between inventory, source reads, cache writes, CLI JSON, and MCP JSON. Central contracts are cheaper than trying to synchronize ad hoc checks later.

## Current Code Findings

| Area | Current Behavior | Phase 2 Action | Confidence |
|------|------------------|----------------|------------|
| Inventory freshness | `load_or_build_inventory` returns cached inventory if `inventory.json` exists. | Add freshness metadata/check and refresh-on-read envelope. | HIGH |
| Ignore files | Walker uses `.gitignore`, `.ctxpackignore`, `.cursorignore`, git excludes. | Include ignore-file fingerprints in freshness. | HIGH |
| Sensitive paths | Denylist covers `.env`, cert/key extensions, dumps, `secret`, `credentials`. | Expand in centralized policy with table-driven tests. | HIGH |
| Generated paths | Denylist covers node_modules, target, dist, build, coverage, vendor, selected resource directories, lockfiles. | Centralize, add binary/oversized/non-UTF-8/generated fixtures. | HIGH |
| Search/symbol/dependency/test reads | Several modules use `read_to_string(...).unwrap_or_default()`. | Return diagnostics for unreadable/non-UTF-8/oversized instead of empty content. | HIGH |
| Pack snippets | `packs.rs` reads `repo_root.join(path)` directly. | Revalidate every snippet path against fresh safe inventory. | HIGH |
| MCP file resource | Re-checks safe inventory, but still uses stale `load_or_build_inventory`. | Switch to trusted freshness envelope and source-read policy. | HIGH |
| Trace writes | CLI/MCP `prepare-task` and `get-pack` fail on `append_eval_trace` errors. | Add non-fatal write status for read-oriented operations. | HIGH |

## Freshness Design

Use a two-layer design:

1. **Inventory metadata:** add `metadata` to `RepoInventory` with `schemaVersion`, `policyVersion`, `options`, `builtAtUnixSeconds`, `repoRoot`, `ignoreFingerprints`, and a file manifest keyed by relative path with size, modified time where available, and hash for included files.
2. **Load policy:** new `load_or_refresh_inventory` checks metadata first. If stale or missing, rebuild. If rebuild succeeds but cache write fails, return in-memory inventory with `cache_write_failed`. If rebuild fails, return old inventory only when explicitly allowed and emit `stale_inventory_rebuild_failed`; otherwise error for explicit write commands.

Freshness invalidation points:

- File created, deleted, renamed, or size/mtime/hash changed.
- File moved into or out of sensitive/generated/vendor/binary/oversized classification.
- `.gitignore`, `.git/info/exclude`, `.ctxpackignore`, or `.cursorignore` changed.
- Inventory options changed (`includeGenerated`, `includeSensitive`, future size limits).
- Source policy version changed.
- Inventory schema version changed.
- Repo root/repo ID mismatch after path moves.

Read-path policy:

- `ctxpack index` remains an explicit write and should fail if inventory cannot be written.
- Search, symbols, related tests, dependencies, prepare-task, get-pack, cards, MCP tools/resources should prefer fresh rebuild.
- If cache write fails but in-memory rebuild succeeds, continue with diagnostics.
- If rebuild fails and stale cache exists, return stale diagnostics only for operations where returning stale data is explicitly accepted by the new API/flag; default should be conservative for source-bearing reads.

## Privacy And Source Policy

Centralize these categories:

| Category | Examples | Default Action |
|----------|----------|----------------|
| Package-manager auth | `.npmrc`, `.yarnrc.yml`, `.pypirc`, `.netrc` | Sensitive, exclude |
| SSH/private keys | `id_rsa`, `id_ed25519`, `*.pem`, `*.key`, `*.p12`, `*.pfx` | Sensitive, exclude |
| Cloud credentials | `serviceAccountKey.json`, `firebase-adminsdk*.json`, `.aws/credentials`, `.gcloud/*`, credential-like JSON | Sensitive, exclude |
| Secret/config state | `.env*`, `terraform.tfstate`, files with `secret`, `credentials`, `token` in credential contexts | Sensitive, exclude |
| Generated/vendor | `node_modules`, `target`, `dist`, `build`, `coverage`, `vendor`, minified assets, lockfiles | Generated, exclude |
| Binary/non-UTF-8 | NUL bytes, invalid UTF-8, source-like extension with binary bytes | Skip with diagnostic |
| Oversized | Above configured source-read cap | Skip with diagnostic |
| Unreadable | Permission denied, disappeared after inventory | Skip with diagnostic |

Conservative classification is correct for this phase. False positives can be relaxed later by explicit opt-in policy, but false negatives can leak source or credentials through pack/file-resource surfaces.

## Diagnostics Contract

Recommended fields:

```rust
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub paths: Vec<String>,
    pub count: usize,
}
```

Required codes for this phase:

| Code | Severity | Emitter |
|------|----------|---------|
| `inventory_stale` | warning | freshness check |
| `inventory_rebuilt` | info | freshness check |
| `inventory_rebuild_failed` | error/warning | freshness check |
| `cache_write_failed` | warning | inventory cache writes |
| `trace_write_failed` | warning | trace append wrapper |
| `source_unreadable` | warning | source policy |
| `source_non_utf8` | warning | source policy |
| `source_binary` | warning | source policy |
| `source_oversized` | warning | source policy |
| `source_policy_excluded` | info/warning | source policy |
| `low_information_task` | warning | compiler planning |
| `git_missing` | warning | git wrapper |
| `git_timeout` | warning | git wrapper |
| `history_partial` | warning | co-change/history |
| `graph_partial` | warning | dependency graph |
| `test_map_partial` | warning | related tests |
| `parse_gap` | info/warning | symbols/dependencies |

MCP and CLI shape:

- `ContextPlan` JSON: add `diagnostics`; keep `riskFlags`.
- `ContextPack` JSON: add `diagnostics`; keep `warnings` and risk flag section.
- MCP tool responses: `structuredContent` should contain diagnostics at the top-level value or inside the serialized plan/pack.
- CLI raw arrays for `search`, `symbols`, `related-tests`, and `dependencies` cannot gain object wrappers without breaking behavior. Prefer a new compatible format only where existing shape is not guarded, or add `--with-diagnostics` object output. For MCP, use object outputs because tool wrappers already return structured content.

## Cache And Trace Write Handling

Current write points:

- Inventory cache: `write_inventory` writes `CTXPACK_HOME/repos/<repo-id>/inventory.json`.
- Eval traces: `append_eval_trace` appends `CTXPACK_HOME/repos/<repo-id>/traces.jsonl`.
- Cards: `cards generate` writes repo-local `.ctxpack/cards/*.md`.
- MCP pack resources: in-memory session cache, not filesystem.

Recommended behavior:

- Keep explicit `ctxpack index` and `ctxpack cards generate` write failures fatal.
- Make trace writes non-fatal for `prepare-task`, `get-pack`, MCP `prepare_task`, and MCP `get_pack`.
- Make inventory cache write failures non-fatal when a read path can build a fresh in-memory inventory.
- Expose `cacheStatus` and `traceStatus` in diagnostics, including target path and reason category but not source or prompt text.
- Add an opt-out control for traces, preferably `--no-trace` on CLI read commands and a `recordTrace`/`noTrace` MCP argument if needed. The exact flag name is discretionary; keep default behavior compatible.

## Focused Test Strategy

Use Phase 1 guardrails rather than a new test harness.

| Test Area | Location | Fixtures |
|-----------|----------|----------|
| Freshness create/delete/rename | `crates/ctxpack-index/src/lib.rs` or `freshness.rs` tests | temp repo, stale inventory, file mutation after cache |
| Ignore invalidation | index tests | mutate `.ctxpackignore`, `.cursorignore`, `.gitignore`, `.git/info/exclude` |
| Privacy corpus | policy tests | `.npmrc`, `.pypirc`, `.netrc`, `id_rsa`, service account JSON, tfstate, generated/vendor/binary |
| Source-read diagnostics | policy/search/symbol/dependency tests | unreadable file on Unix, invalid UTF-8, oversized file, deleted-after-inventory |
| Pack revalidation | `crates/ctxpack-compiler/src/lib.rs` | plan points to file later made sensitive/generated/deleted |
| MCP diagnostics shape | `crates/ctxpack-mcp/src/lib.rs` | `prepare_task`, `get_pack`, `search`, file resource, current_diff |
| CLI compatibility | `crates/ctxpack/tests/cli_compat.rs` | stale cache JSON shape, read-only `CTXPACK_HOME`, non-fatal trace write |
| Weak plan | compiler + CLI/MCP | low-information task like `Fixes #1061` and empty/near-empty repo |

Required validation commands:

```bash
cargo test -p ctxpack-index
cargo test -p ctxpack-compiler
cargo test -p ctxpack-mcp
cargo test -p ctxpack --test cli_compat
cargo test --workspace
cargo run -p ctxpack -- --help
```

## Code Examples

### Diagnostic-Aware Trace Write

```rust
pub fn try_append_eval_trace(
    repo_root: impl AsRef<Path>,
    trace: &EvalTrace,
) -> TraceWriteStatus {
    match append_eval_trace(repo_root, trace) {
        Ok(path) => TraceWriteStatus::Written { path },
        Err(error) => TraceWriteStatus::Skipped {
            diagnostic: Diagnostic {
                code: "trace_write_failed".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!("Eval trace was not recorded: {error}"),
                paths: Vec::new(),
                count: 1,
            },
        },
    }
}
```

### Pack Revalidation Shape

```rust
let load = load_or_refresh_inventory(repo_root, &InventoryOptions::default())?;
for target in &plan.target_files {
    match read_safe_source(repo_root, &load.inventory, &target.path, max_bytes)? {
        Some(source) => render_snippet(source),
        None => diagnostics.push(policy_skip_diagnostic(&target.path)),
    }
}
```

### Table-Driven Privacy Tests

```rust
#[test]
fn policy_excludes_common_credentials() {
    for path in [
        ".npmrc",
        ".pypirc",
        ".netrc",
        ".ssh/id_ed25519",
        "config/serviceAccountKey.json",
        "terraform.tfstate",
    ] {
        assert_eq!(classify_path(path), FileRole::Sensitive, "{path}");
    }
}
```

## State of the Art

| Old Approach | Current Approach For Phase 2 | When Changed | Impact |
|--------------|------------------------------|--------------|--------|
| Cache exists means cache is valid | Cache carries metadata and is checked before every user-facing read | Phase 2 | Prevents stale/deleted/sensitive files from driving results. |
| Per-module direct file reads | Central source-read policy with diagnostics | Phase 2 | Prevents silent weak matches and source leakage. |
| `riskFlags` as only diagnostics | Additive `diagnostics` plus `riskFlags` projection | Phase 2 | Machine-readable diagnostics without breaking existing clients. |
| Fatal trace writes during read operations | Non-fatal trace/cache status for context retrieval | Phase 2 | ctxpack remains usable in constrained home directories. |
| Git failures mostly string warnings | Structured `git_missing`/`git_timeout` diagnostics | Phase 2 | Users can distinguish weak task from missing external signal. |

**Deprecated/outdated:**
- Direct `fs::read_to_string(...).unwrap_or_default()` in read paths: replace with source-read policy.
- Plan-path snippet reads without revalidation: replace with safe inventory check at snippet-read time.
- Returning stale inventory silently: replace with freshness diagnostics and rebuild attempts.

## Common Pitfalls

### Pitfall 1: Freshness Checks That Do Not Cover Policy Drift
**What goes wrong:** Cache looks fresh by timestamp, but a new policy version would now classify the same path as sensitive/generated.
**Why it happens:** Freshness is tied only to file mtimes.
**How to avoid:** Store `policyVersion`, `schemaVersion`, and `InventoryOptions` in inventory metadata.
**Warning signs:** Privacy tests pass for new builds but fail after preloading an old cache.

### Pitfall 2: Diagnostics Break Existing JSON Consumers
**What goes wrong:** CLI search changes from array to object or MCP fields are renamed.
**Why it happens:** Diagnostics are added by replacing result shapes.
**How to avoid:** Add fields only to existing object contracts; for array-returning CLI commands, add explicit diagnostic modes or leave existing output shape intact.
**Warning signs:** Phase 1 CLI compatibility tests fail on shape assertions.

### Pitfall 3: Rebuild-on-Read Makes Constrained Homes Fatal
**What goes wrong:** A read path builds a fresh inventory but then fails because `CTXPACK_HOME` is read-only.
**Why it happens:** Build and cache-write are coupled.
**How to avoid:** Return fresh in-memory inventory even when cache persistence fails; emit `cache_write_failed`.
**Warning signs:** Search/prepare-task fails in read-only temp home despite repo files being readable.

### Pitfall 4: Source Policy Only Runs During Inventory
**What goes wrong:** A file is safe when planned but becomes sensitive/generated/deleted before pack/resource read.
**Why it happens:** Pack rendering trusts stale plan paths.
**How to avoid:** Revalidate every source-bearing path immediately before reading snippets.
**Warning signs:** A pack still contains a file after it is renamed to `.env` or moved under `dist/`.

### Pitfall 5: Low-Information Plans Are Treated As Retrieval Failures
**What goes wrong:** `Fixes #1061` returns weak context without explaining that the task itself lacks terms.
**Why it happens:** Existing `is_low_information_task` is not surfaced as a structured diagnostic.
**How to avoid:** Emit `low_information_task` and ask for symbol/file/error text in `missingInfoQuestions`.
**Warning signs:** Empty or generic tasks produce only empty arrays.

## Open Questions

1. **Should stale cache ever be returned by default?**
   - What we know: Context says rebuild when safe/cheap, otherwise structured stale diagnostics.
   - What's unclear: Whether default source-bearing reads may fall back to stale cache if rebuild fails.
   - Recommendation: default source-bearing pack/file reads should not use known-stale cache; non-source search may allow explicit fallback.

2. **Exact trace-control surface**
   - What we know: constrained-home behavior must be non-fatal and visible.
   - What's unclear: final CLI/MCP flag names.
   - Recommendation: use additive `--no-trace` for CLI read commands and a camelCase MCP option only if real clients need it.

3. **Oversized-file threshold**
   - What we know: current code reads full files repeatedly.
   - What's unclear: product default size cap.
   - Recommendation: start conservative, e.g. 1 MiB per source read for snippets/scoring, then make it configurable in `InventoryOptions` later.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust compiler | Build/test | yes | rustc 1.87.0 | None needed |
| Cargo | Build/test/deps | yes | cargo 1.87.0 | None needed |
| git | current diff, history, fixture commits | yes | 2.45.1 | Diagnostics for missing/timeout |
| tar | historical eval worktrees | yes | bsdtar 3.8.1 | Phase 3 owns eval fallback |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

## Sources

### Primary (HIGH confidence)
- `.planning/phases/02-trust-layer-operational-diagnostics/02-CONTEXT.md` - locked decisions, deferred scope, reusable Phase 1 assets.
- `.planning/REQUIREMENTS.md` - SAFE/DIAG requirement text.
- `.planning/ROADMAP.md` - Phase 2 success criteria and boundaries.
- `AGENTS.md` and `CLAUDE.md` - project constraints and validation commands.
- `crates/ctxpack-index/src/inventory.rs` - current cache load/build, classification, ctxpack home, inventory write behavior.
- `crates/ctxpack-index/src/search.rs`, `symbols.rs`, `related_tests.rs`, `dependencies.rs`, `git.rs`, `traces.rs` - read paths, git timeouts, trace writes.
- `crates/ctxpack-core/src/contracts.rs` - current public contract shapes and `riskFlags`.
- `crates/ctxpack-compiler/src/planning.rs`, `packs.rs`, `cards.rs` - planning, pack snippets, card generation.
- `crates/ctxpack-mcp/src/tools.rs`, `resources.rs`, `lib.rs` - MCP tool/resource structured content.
- `crates/ctxpack/tests/cli_compat.rs` and `crates/ctxpack/tests/common/mod.rs` - Phase 1 binary guardrails.
- `Cargo.lock`, `cargo tree --workspace --depth 1`, `cargo metadata --format-version 1 --no-deps` - dependency versions.

### Secondary (MEDIUM confidence)
- `cargo info` registry metadata for `ignore`, `serde`, `serde_json`, `clap`, `tempfile`, `blake3`, `thiserror`, `uuid`, `anyhow`, `assert_cmd`, `predicates` - verified crate versions and docs URLs.
- Official docs URLs from `cargo info`: `https://docs.rs/ignore`, `https://serde.rs`, `https://docs.rs/serde_json`, `https://docs.rs/clap/4.6.1`, `https://docs.rs/tempfile`, `https://docs.rs/blake3`, `https://docs.rs/thiserror`, `https://docs.rs/uuid`, `https://docs.rs/anyhow`, `https://docs.rs/assert_cmd`, `https://docs.rs/predicates`.

### Tertiary (LOW confidence)
- None used. No web-search-only claims are included.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - existing Rust workspace and locked dependency versions verified locally and via `cargo info`.
- Architecture: HIGH - based on live Phase 1 module split and current source inspection.
- Pitfalls: HIGH - each listed pitfall maps to a current code path or explicit Phase 2 requirement.
- External ecosystem: MEDIUM - no new library adoption is recommended; official crate docs were identified, but publish dates were not available from local cargo metadata.

**Research date:** 2026-05-13
**Valid until:** 2026-06-12 for codebase-specific recommendations; re-check dependency/tool versions before any package upgrade.
