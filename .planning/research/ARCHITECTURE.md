# Architecture Research

**Domain:** local-first repository context compiler for coding agents
**Researched:** 2026-05-13
**Confidence:** HIGH for current-code architecture and next internal boundaries; MEDIUM for external ecosystem direction.

## Standard Architecture

### System Overview

Repo Context Packer should remain a layered Rust workspace where agent-facing surfaces depend on stable contracts and a compiler facade, while retrieval internals can evolve behind typed module boundaries.

```text
+--------------------------------------------------------------------+
| Agent-Native Surfaces                                               |
| CLI commands | MCP tools/resources/prompts | AGENTS/adapter files   |
+-----------------------------+--------------------------------------+
                              |
+-----------------------------v--------------------------------------+
| Public Contract Boundary                                           |
| ctxpack-core: ContextPlan, ContextPack, EvalTrace, PrivacyStatus    |
+-----------------------------+--------------------------------------+
                              |
+-----------------------------v--------------------------------------+
| Context Compiler                                                    |
| planning | candidate fusion | pack materialization | cards | eval   |
+-------------+---------------+----------------+---------------------+
              |               |                |
+-------------v---+  +--------v---------+  +---v--------------------+
| Retrieval Query |  | Diagnostics      |  | Privacy/Freshness Gate |
| lexical/symbol  |  | warnings/reasons |  | policy + cache checks  |
| graph/test/git  |  | operation report |  | safe file validation   |
+-------------+---+  +------------------+  +---+--------------------+
              |                              |
+-------------v------------------------------v-----------------------+
| Local Index And Stores                                               |
| inventory.json | graph cache | history cache | traces.jsonl | cards |
+-------------+------------------------------------------------------+
              |
+-------------v------------------------------------------------------+
| Local Repository                                                      |
| filesystem | ignore files | git history | current diff | manifests |
+--------------------------------------------------------------------+
```

The key architectural move is not a new service or product surface. It is making the current implicit subsystems explicit:

- Privacy policy and freshness checks must gate every file path before retrieval, graph expansion, snippet reads, MCP file resources, cards, and eval labels.
- Retrieval should return `ContextCandidate`-like internal records with per-signal evidence instead of directly mutating `ContextPlan` fields.
- Diagnostics should be first-class metadata emitted by every subsystem, not only ad hoc risk flags after failures.
- Eval should measure each retrieval signal and graph lift separately so tuning is evidence-driven.
- Public JSON contracts should remain stable; add optional fields only after the internal model has settled.

### Component Responsibilities

| Component | Responsibility | Recommended Implementation |
|-----------|----------------|----------------------------|
| Core contracts | Stable public structs, repo root discovery, privacy status, init artifacts. | Keep in `ctxpack-core`; preserve camelCase JSON and existing tool/resource names. |
| Privacy policy | Decide whether a path may be inventoried, searched, used as a label, or read as a snippet. | Dedicated `ctxpack-index::privacy` module with table-driven deny/allow rules and policy version. |
| Inventory freshness | Decide whether cached inventory is usable for the current repo state and options. | Dedicated `ctxpack-index::inventory` module with cache metadata, ignore-file hashes, option hash, file counts, and scan timestamp. |
| Text/source reader | Read safe text under size, UTF-8, binary, and snippet budgets. | Shared `ctxpack-index::source` helper returning content or structured skip diagnostics. |
| Lexical and symbol retrieval | Rank direct text, path, and symbol matches. | `ctxpack-index::search` and `ctxpack-index::symbols`; keep current heuristics, later add parser-backed implementations behind same result types. |
| Graph retrieval | Build and query dependency, import, symbol, test, and co-change edges. | `ctxpack-index::graph` over typed `GraphEdge` records; expose current `DependencyEdge` as a view. |
| History retrieval | Batch git changed-file extraction and co-change features. | `ctxpack-index::history`; centralize git command execution, timeouts, and partial-result diagnostics. |
| Test inference | Map files to likely tests and commands with confidence and reasons. | `ctxpack-index::tests`; return command confidence and runner evidence. |
| Candidate fusion | Merge anchors, lexical, symbols, graph, tests, history, and current diff into ranked candidates. | `ctxpack-compiler::planning`; internal candidates carry `source_scores`, `evidence`, and `diagnostics`. |
| Pack compiler | Materialize a budgeted pack from an already validated plan. | `ctxpack-compiler::packs`; revalidate snippet paths against fresh safe inventory before reading source. |
| Eval engine | Run source-free historical and trace-based quality checks. | `ctxpack-compiler::eval`; report lexical baseline, graph lift, source/test recall, missing role distribution, stale/partial diagnostics. |
| Diagnostics | Explain weak plans, stale cache, skipped files, missing tools, and partial graph/history. | Shared `Diagnostic`/`OperationReport` type used internally, projected to risk flags and CLI/MCP debug output. |
| MCP boundary | Translate JSON-RPC/MCP requests to typed compiler/index calls. | Keep in `ctxpack-mcp`; split schema, tools, resources, prompts, cache modules without changing protocol. |
| CLI boundary | Human/debug automation surface. | Keep in `ctxpack`; add binary-level tests for Clap wiring and output formats. |

## Recommended Project Structure

Split large files into internal modules first, then improve behavior. Do not split crates until module boundaries have proved stable.

```text
crates/
  ctxpack-core/src/
    contracts.rs          # public ContextPlan/ContextPack/EvalTrace contracts
    privacy.rs            # public PrivacyStatus only
    repo.rs               # repo root discovery
    init.rs               # AGENTS and adapter artifact generation

  ctxpack-index/src/
    lib.rs                # public re-exports and thin facade
    inventory.rs          # safe inventory build/load/write/freshness
    privacy_policy.rs     # path classification and policy version
    source.rs             # safe bounded text reads and snippet guards
    search.rs             # lexical search and scoring
    symbols.rs            # symbol extraction/search facade
    graph.rs              # graph edge model, expansion, dependency views
    tests.rs              # related test and command inference
    history.rs            # git process, co-change, historical samples
    diff.rs               # current diff anchors and privacy summary
    traces.rs             # source-free trace persistence and retention
    diagnostics.rs        # index-layer diagnostic types

  ctxpack-compiler/src/
    lib.rs                # public compiler facade
    planning.rs           # candidate fusion and ContextPlan projection
    candidates.rs         # internal candidate/evidence/source-score model
    packs.rs              # pack construction and Markdown rendering
    cards.rs              # source-free context card generation
    eval.rs               # historical eval and trace eval
    diagnostics.rs        # compiler diagnostics and risk-flag projection

  ctxpack-mcp/src/
    lib.rs                # stdio server facade
    protocol.rs           # JSON-RPC/MCP protocol structs
    schema.rs             # tool/resource/prompt descriptors
    tools.rs              # tool handlers
    resources.rs          # resource handlers
    prompts.rs            # prompt handlers
    cache.rs              # session pack-resource cache

  ctxpack/src/
    main.rs               # Clap command entry point
    output.rs             # CLI renderers after main.rs grows further
```

### Structure Rationale

- **Index modules own facts, not plans:** inventory, privacy, graph, tests, symbols, diff, and history should return typed evidence plus diagnostics. They should not know how many files belong in a coding-agent plan.
- **Compiler owns ranking decisions:** task-conditioned fusion, diversification, confidence, pack budgets, and eval belong together because changes here should be measured against historical retrieval quality.
- **Transport stays thin:** MCP and CLI should validate args, call facades, and serialize responses. They should not implement retrieval heuristics.
- **Public crate APIs stay stable:** keep existing public functions as compatibility facades while moving implementations behind modules.
- **Diagnostics cross layers:** each subsystem should attach structured warnings to an operation report; compiler projects high-severity items into existing `riskFlags` so public contracts remain compatible.

## Architectural Patterns

### Pattern 1: Policy-Gated Retrieval

**What:** Every candidate path flows through privacy policy and freshness validation before it can become a search result, graph neighbor, pack snippet, card entry, MCP file slice, or eval label.

**When to use:** Always for file-bearing APIs. This is the trust boundary for a local-first context compiler.

**Trade-offs:** More checks add small overhead, but the alternative is stale or unsafe source leakage. Cache freshness should be cheap and conservative; when uncertain, rebuild or emit a diagnostic.

**Example:**

```rust
pub struct RetrievalContext {
    pub inventory: RepoInventory,
    pub policy: PrivacyPolicy,
    pub freshness: FreshnessReport,
    pub diagnostics: Vec<Diagnostic>,
}

impl RetrievalContext {
    pub fn safe_candidate(&mut self, path: &str) -> Option<SafePath> {
        if !self.freshness.usable {
            self.diagnostics.push(Diagnostic::stale_inventory());
            return None;
        }
        self.policy.allow_inventory_path(path).then(|| SafePath::new(path))
    }
}
```

### Pattern 2: Internal Candidate Graph, Public Plan Projection

**What:** Retrieval subsystems emit internal candidates with signal-specific scores and evidence. The compiler fuses them and then projects the top results into the existing `ContextPlan` contract.

**When to use:** Before making graph expansion first-class. Graph edges need to affect ranking, not only produce risk/evidence text.

**Trade-offs:** Adds an internal model, but avoids breaking CLI/MCP contracts and allows eval to attribute wins or misses to lexical, symbol, graph, history, or test signals.

**Example:**

```rust
pub struct ContextCandidate {
    pub path: String,
    pub kind: CandidateKind,
    pub source_scores: SourceScores,
    pub evidence: Vec<Evidence>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn project_plan(candidates: &[ContextCandidate]) -> ContextPlan {
    // Existing public contract stays unchanged while ranking internals evolve.
    todo!("map top file/test candidates to targetFiles, relatedTests, commands, riskFlags")
}
```

### Pattern 3: Diagnostics As Data

**What:** Treat stale cache, unreadable files, skipped large files, missing `git`, timed-out history, partial parser coverage, and weak graph coverage as structured diagnostics.

**When to use:** Any operation that can silently degrade retrieval quality.

**Trade-offs:** Requires more plumbing through return types, but it tells users whether a weak plan is caused by task ambiguity or subsystem failure.

### Pattern 4: Eval-Driven Retrieval Changes

**What:** Every major ranking or graph change should report source-free historical metrics before and after: Recall@5/10, lexical baseline, graph lift, source/test recall, missing role distribution, and diagnostic counts.

**When to use:** For graph-aware retrieval, cache freshness, parser-backed symbols, history batching, and test inference changes.

**Trade-offs:** Eval can overfit to sampled repositories. Mitigate by freezing ranges with `--base`/`--head` and using at least one large external repo smoke such as RefactoringMiner.

## Data Flow

### Prepare Task Flow

```text
CLI/MCP prepare_task
  -> repo root discovery
  -> RetrievalContext::load(repo, options)
       -> check cache metadata, ignore hashes, policy version, option hash
       -> rebuild inventory when stale or unsafe
       -> attach freshness/privacy diagnostics
  -> collect anchors
       -> explicit paths, active paths, safe current diff
  -> retrieve candidates
       -> lexical search
       -> symbol search
       -> related tests
       -> dependency/import graph expansion
       -> git co-change/history hints
  -> fuse candidates
       -> normalize scores, dedupe, diversify roles, attach evidence
       -> compute confidence and diagnostics
  -> project ContextPlan
       -> targetFiles, relatedTests, commands, packOptions, riskFlags
  -> append source-free trace if enabled
  -> return JSON/structuredContent through CLI/MCP
```

### Get Pack Flow

```text
CLI/MCP get_pack
  -> prepare or receive ContextPlan
  -> reload/revalidate safe inventory before snippet reads
  -> compile sections under budget
       -> task restatement
       -> selected files and reasons
       -> validation commands
       -> bounded safe snippets
       -> related tests
       -> diagnostics/checklist
  -> append source-free trace if enabled
  -> render JSON or Markdown
```

### Graph-Aware Retrieval Flow

```text
anchors + lexical/symbol seeds
  -> seed candidates with high direct evidence
  -> graph expansion from safe seeds
       -> inbound/outbound dependency edges
       -> symbol definition/reference edges when available
       -> test-to-source edges
       -> co-change edges
  -> edge weighting
       -> direct import > test relation > co-change > weak name/path relation
  -> candidate fusion
       -> graph contributes score and explanation
       -> graph-only candidates are admitted only above a confidence threshold
  -> eval attribution
       -> report graph lift over lexical baseline
```

### Cache Freshness Flow

```text
operation requests inventory
  -> compute repo id
  -> read inventory metadata
  -> compare ctxpack version, inventory schema, options hash, policy version
  -> compare ignore-file hashes for .gitignore, .ctxpackignore, .cursorignore
  -> cheap filesystem probe for changed/deleted/sensitive/generated paths
  -> if fresh: return cached inventory with FreshnessReport::fresh
  -> if stale: rebuild inventory and return FreshnessReport::rebuilt
  -> if rebuild fails: return cached inventory only with explicit degraded diagnostic, or fail for snippet-producing APIs
```

### Privacy Flow

```text
path discovered
  -> normalize repo-relative path
  -> apply ignore rules
  -> classify generated/vendor/build artifacts
  -> classify sensitive name/path/content-shape families
  -> record exclusion counts, not source text
  -> only safe paths enter inventory, graph, cards, traces, eval labels, or snippets
```

### Eval Feedback Flow

```text
historical commit sample
  -> parent snapshot
  -> safe changed-file labels
  -> run prepare_task against parent
  -> compare plan files with labels
  -> compute source/test recall and lexical baseline
  -> attribute hits/misses to signal sources
  -> emit source-free report and top retrieval gaps
```

## Suggested Build Order And Dependencies

1. **Characterization and binary tests first**
   - Add binary-level CLI tests for existing commands and MCP shape tests around changed handlers.
   - Rationale: module decomposition and diagnostics plumbing should not accidentally change public contracts.
   - Depends on: current workspace only.

2. **Extract modules behind compatibility facades**
   - Split `ctxpack-index`, `ctxpack-compiler`, and `ctxpack-mcp` into modules without changing behavior.
   - Rationale: privacy, freshness, graph, and eval changes are risky while everything lives in 2K-3K line files.
   - Depends on: characterization tests.

3. **Centralize privacy policy and safe source reads**
   - Move classification into `privacy_policy.rs`; add corpus tests for package-manager auth files, SSH keys, cloud credentials, sensitive JSON names, unknown dotfiles, binary/non-UTF-8, and generated paths.
   - Rationale: every later graph, pack, card, resource, and eval improvement depends on safe path filtering.
   - Depends on: module extraction.

4. **Add inventory freshness metadata**
   - Store schema/tool version, policy version, options hash, ignore-file hashes, scan timestamp, and file-count/hash summary.
   - Rationale: graph-aware retrieval and MCP sessions are not trustworthy if stale inventory is the default read path.
   - Depends on: privacy policy module.

5. **Introduce diagnostics and operation reports**
   - Add structured diagnostics internally; project critical diagnostics into existing `riskFlags` and expose detailed diagnostics through CLI debug output or later optional fields.
   - Rationale: weak plans must explain stale cache, unreadable files, missing git, partial graph, skipped large files, and parser coverage.
   - Depends on: freshness and safe reader APIs.

6. **Implement internal candidate/evidence model**
   - Add `ContextCandidate`, `Evidence`, and `SourceScores` internally; project to existing `ContextPlan`.
   - Rationale: graph expansion needs to rank files, not just add risk/evidence flags.
   - Depends on: diagnostics, retrieval module boundaries.

7. **Make graph expansion first-class**
   - Treat dependency/test/history/current-diff edges as scored candidate sources; admit graph-neighbor candidates through explicit thresholds and diversity rules.
   - Rationale: this is the most direct path to measurable lift over lexical baseline.
   - Depends on: candidate model and eval attribution.

8. **Upgrade eval quality and signal attribution**
   - Add rename/delete cases, role-aware labels, graph/history lift reporting, partial-result diagnostics, and frozen large-repo ranges.
   - Rationale: retrieval improvements should be accepted only when source-free eval shows actual lift.
   - Depends on: candidate source scores and diagnostics.

9. **Parser-backed graph precision as a later phase**
   - Add Tree-sitter or SCIP-backed parsers behind current symbol/graph contracts for languages where fixtures show current line parsing is insufficient.
   - Rationale: external protocols are useful, but freshness, privacy, diagnostics, and measured graph lift should come first.
   - Depends on: graph model, eval baselines, language-specific fixtures.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Small repos | JSON inventory and on-demand search are fine; freshness checks should still run by default. |
| Medium repos | Add per-operation text cache, size caps, batched git history, and graph edge cache. |
| Large repos/monorepos | Add incremental inventory updates, manifest-aware package/workspace boundaries, cached graph partitions, and explicit operation budgets. |
| Long-running MCP sessions | Add cache diagnostics, pack-resource eviction, and explicit repo arguments; avoid assuming process-local resource state survives reconnects. |

### Scaling Priorities

1. **First bottleneck: stale and repeated full-file reads.** Fix with freshness metadata, bounded safe source reads, and per-operation content caching.
2. **Second bottleneck: git history subprocess fan-out.** Fix with batched `git log --name-status`/`--name-only` parsing, bounded ranges, and centralized timeouts.
3. **Third bottleneck: graph precision.** Fix with parser-backed edges only after eval proves graph candidates matter.

## Anti-Patterns

### Anti-Pattern 1: Transport-Driven Retrieval Logic

**What people do:** Add special ranking, path filtering, or pack behavior inside CLI or MCP handlers.

**Why it's wrong:** CLI and MCP diverge, tests miss real behavior, and public contracts become coupled to one client.

**Do this instead:** Put behavior in index/compiler modules and keep transport handlers as validation/serialization adapters.

### Anti-Pattern 2: Graph Edges As Decorations

**What people do:** Compute dependency edges but only show them as evidence or risk flags.

**Why it's wrong:** Historical eval will continue to tie lexical baseline because graph information does not affect candidate selection.

**Do this instead:** Convert edges into scored candidates and measure graph lift explicitly.

### Anti-Pattern 3: Privacy Denylist Sprawl In Retrieval Code

**What people do:** Add filename checks wherever a new reader or resource needs safety.

**Why it's wrong:** One missed path can expose safe-looking but sensitive source through packs or MCP file slices.

**Do this instead:** Centralize policy and require all source-bearing APIs to accept only `SafePath`/inventory-backed entries.

### Anti-Pattern 4: Silent Degradation

**What people do:** Treat unreadable files, invalid UTF-8, missing git, parser failures, or stale cache as empty results.

**Why it's wrong:** Users cannot distinguish a genuinely weak task from a broken context compiler.

**Do this instead:** Return partial results with structured diagnostics and surface high-impact diagnostics in plans and reports.

### Anti-Pattern 5: Parser Migration Before Measurement

**What people do:** Replace lightweight parsing with Tree-sitter/SCIP across all languages before proving which misses matter.

**Why it's wrong:** It increases install complexity and maintenance before graph-aware retrieval has shown measurable value.

**Do this instead:** Add parser-backed implementations per language only when fixtures and historical eval identify a precision/recall bottleneck.

## Integration Points

### External Tools And Standards

| Tool/Standard | Integration Pattern | Notes |
|---------------|---------------------|-------|
| MCP | Keep current small tool/resource/prompt surface; use structuredContent and stable JSON contracts. | The latest MCP spec defines hosts, clients, servers, JSON-RPC, resources, prompts, tools, and utilities for error reporting/logging. |
| Git | Centralize command execution for diff, log, archive, co-change, and historical eval. | Prefer batched history output such as `git log --name-only` or `--name-status` where possible; handle rename/delete metadata. |
| Tree-sitter | Optional parser-backed symbol/dependency implementation. | Useful for incremental and robust parse trees, but should stay behind current contracts until eval justifies it. |
| SCIP | Optional precise code-intelligence import path. | Language-agnostic symbol/reference index ecosystem; useful later for definitions/references without inventing every language parser. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI/MCP -> compiler | Public functions returning `ContextPlan`, `ContextPack`, reports. | Keep stable; add optional diagnostics after internal plumbing. |
| compiler -> index | Typed retrieval APIs returning candidates/evidence/diagnostics. | Compiler owns scoring and projection. |
| index -> filesystem/git | Centralized safe source reader and git runner. | Enforce budgets, timeouts, and diagnostics. |
| inventory/privacy -> all readers | `SafePath` or inventory entry IDs. | Prevent direct arbitrary path reads in packs/resources. |
| eval -> compiler/index | Same public prepare flow plus hidden source-free labels. | Eval should exercise real retrieval, not a special code path. |

## Roadmap Implications

Recommended phase structure:

1. **Stabilize boundaries and tests**
   - Module split, characterization tests, binary CLI tests, MCP response compatibility.
   - Avoids breaking public contracts while enabling deeper changes.

2. **Trust layer**
   - Privacy policy corpus, safe source reader, inventory freshness, local cache diagnostics.
   - Must precede graph and snippet-heavy work.

3. **Diagnostics layer**
   - Operation reports, risk-flag projection, stale/partial/skip warnings, cache visibility.
   - Makes later quality failures actionable.

4. **Graph-aware planning**
   - Internal candidates, edge-weighted expansion, source scores, graph lift attribution.
   - Directly targets the active requirement to beat lexical baseline.

5. **Eval hardening**
   - Rename/delete cases, large-repo frozen ranges, signal attribution, role-specific gap reports.
   - Turns retrieval tuning into measurable product work.

6. **Parser-backed precision**
   - Tree-sitter or SCIP-backed implementations for languages and edge types that eval proves weak.
   - Defer until graph-aware retrieval has a clear measurement loop.

## Sources

- `.planning/PROJECT.md` (HIGH): current project goals, validated requirements, active requirements, constraints.
- `.planning/codebase/ARCHITECTURE.md` (HIGH): current layered Rust workspace, data flow, state management, public entry points.
- `.planning/codebase/STRUCTURE.md` (HIGH): current crate layout and recommended code locations.
- `.planning/codebase/CONCERNS.md` (HIGH): stale cache, privacy, diagnostics, module-size, eval, MCP lifecycle, and parsing risks.
- `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md` (HIGH): original runtime flow, storage direction, stable contract intent.
- `README.md` (HIGH): current historical eval and graph/user-facing behavior.
- Model Context Protocol specification, latest version 2025-11-25 (HIGH): https://modelcontextprotocol.io/specification/2025-11-25
- Tree-sitter GitHub README and release metadata (MEDIUM): https://github.com/tree-sitter/tree-sitter
- SCIP Code Intelligence Protocol overview (MEDIUM): https://scip-code.org/
- Git log documentation for changed-file and rename/delete metadata (HIGH): https://git-scm.com/docs/git-log.html
- Codebase-Memory arXiv preprint on Tree-sitter knowledge graphs via MCP (LOW/MEDIUM, preprint): https://arxiv.org/abs/2603.27277

---
*Architecture research for: Repo Context Packer*
*Researched: 2026-05-13*
