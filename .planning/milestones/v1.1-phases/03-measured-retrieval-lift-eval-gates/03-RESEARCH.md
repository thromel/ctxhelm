# Phase 3: Measured Retrieval Lift & Eval Gates - Research

**Researched:** 2026-05-13
**Domain:** Rust local retrieval ranking, source-free historical eval, git metadata labeling
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
## Implementation Decisions

### Candidate Model And Ranking
- Introduce a typed candidate layer before `ContextPlan` projection rather than attaching more ad hoc fields directly to target files.
- Preserve public JSON compatibility where possible by adding fields instead of replacing existing `targetFiles`, `relatedTests`, `riskFlags`, diagnostics, or provenance shapes.
- Keep recommended-file and related-test attribution source-free: path, role, signal kind, scores, edge labels, commit ids, counts, and reason codes are acceptable; source snippets and prompt text are not.
- Make graph expansion budgeted, shallow, and non-recursive by default. Lift must come from better ranking, not larger packs.

### Evaluation
- Evaluate retrieval changes against a lexical baseline at fixed budgets. If a slice does not show lift, the report should explain the signal gaps instead of hiding the result.
- Historical eval fixtures should be deterministic and small enough for unit/integration tests, with large real repositories used as smoke tests rather than the only proof.
- Frozen eval ranges should record base/head refs, limit, mode, budget, repo identity, and effective filters so results can be reproduced.
- Rename, delete, historical-only files, generated files, sensitive files, tests, configs, docs, and source files should be labeled explicitly in eval outputs.

### Parser And Runtime Scope
- Parser/runtime upgrades are allowed only when they support observed retrieval gaps or are introduced behind existing contracts with tests.
- Do not start a broad Tree-sitter, Tantivy, SQLite, rayon, notify, or MCP SDK migration in this phase unless the eval evidence requires it.

### Product Positioning
- The user-facing value is measured context quality: why this file/test was ranked, which signal selected it, and whether it beats lexical baseline at the same budget.
- The system should remain local-first, read-only, and agent-native.

### Claude's Discretion
None explicitly listed in CONTEXT.md.

### Deferred Ideas (OUT OF SCOPE)
## Deferred Ideas

- Real Codex CLI and Claude Code client durability, MCP restart behavior, wrong-repo checks, and durable pack-resource semantics are Phase 4.
- Cloud embeddings, cloud reranking, local vector search, and hosted/team features remain out of scope for this milestone.
- A visual eval dashboard is out of scope; source-free CLI/JSON reports are enough for this phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DIAG-03 | Historical eval and checklist outputs summarize why retrieval failed, grouped by path role, signal gap, and repeated missing-file families. | Add source-free `RetrievalGapSummary` records grouped from candidate evidence and hidden labels. |
| RETR-01 | Context planning represents candidate files, tests, symbols, docs, commits, and config as typed candidates with evidence and per-signal scores before projecting to `ContextPlan`. | Build a compiler candidate/ranking module and additive core contract fields. |
| RETR-02 | Dependency edges, related tests, co-change hints, current-diff anchors, and symbol matches can affect ranked target files, not only risk flags. | Merge all signal reports into candidate scores before target/test selection. |
| RETR-03 | Each recommended target file and related test includes source-free signal attribution explaining why it was selected. | Add evidence records with signal kind, score, role, edge label, commit counts/ids, and reason codes. |
| RETR-04 | Graph expansion is budgeted and non-recursive by default so retrieval lift does not come from context bloat. | Use a fixed candidate selection budget and one-hop expansion over seed candidates. |
| RETR-05 | Retrieval changes are evaluated against lexical baseline at fixed budgets and must show or explain lift on at least one frozen historical range. | Extend historical eval with budgets, lexical-only baseline, ablations, and gap explanation. |
| EVAL-01 | Maintainer can run frozen historical eval ranges with reproducible base/head refs, limits, mode, and source-free reports. | Persist/report base/head refs, limit, mode, budget, repo id, filters, and signal config. |
| EVAL-02 | Historical eval handles additions, modifications, deletes, renames, generated files, sensitive files, source files, tests, configs, docs, and files that existed only in historical revisions. | Replace name-only samples with name-status records and dual inventory classification. |
| EVAL-03 | Historical eval reports Recall@K, Precision@K, MRR or equivalent ranking quality, role-aware recall, test recommendation rate, lexical baseline, and signal ablations. | Implement standard ranking formulas at K and report per-signal disabled runs. |
| EVAL-04 | Maintainer can run large-repo smoke evals, including RefactoringMiner, without pathological full-worktree checkout costs. | Keep bounded git sampling, archive only candidate paths, and make large repo smoke separate from unit fixtures. |
| EVAL-05 | Eval reports stay source-free and prompt-free while still providing enough gap detail to drive roadmap decisions. | Report hashes, paths, roles, status labels, reason codes, and counts only. |
| PARS-01 | Maintainer can add parser-backed symbol and dependency adapters behind existing contracts without changing CLI or MCP output shapes. | Keep parser/runtime work behind existing typed signal reports; no broad parser migration. |
| PARS-02 | Parser-backed improvements are introduced only for languages and constructs with observed retrieval gaps or resolver false positives. | Gate parser adapter work on eval gap categories. |
| PARS-03 | Optional indexing/runtime upgrades such as Tantivy, rayon, SQLite, notify, or MCP SDK migration are evaluated with before/after metrics before adoption. | Do not adopt new runtime dependencies in Phase 3 without measured before/after lift. |
</phase_requirements>

## Summary

Phase 3 should not add another heuristic directly to `TargetFile`. The current compiler selects anchors, symbol hits, then lexical hits into `targetFiles`, and only afterward derives tests, co-change hints, and dependency edges as related tests or risk flags. That means graph/history/test evidence is visible but cannot consistently change the primary ranking. The phase should introduce a typed candidate layer, merge every signal into source-free evidence records, rank once under a fixed budget, then project into the existing `ContextPlan` shape with additive attribution fields.

Historical eval already has useful foundations: source-free reports, lexical baseline recall, parent worktree replay, role recall for source/test, and frozen base/head options. The missing decision-grade pieces are status-aware labels, reproducibility metadata beyond refs, fixed-budget ranking metrics, ablations, gap explanations, and rename/delete/historical-only classification. Use Git's name-status and rename detection instead of path-only diffs.

**Primary recommendation:** Implement a `ctxhelm-compiler::ranking` module plus additive core contract structs for `retrievalCandidates` / `attribution`, then extend `ctxhelm-compiler::eval` to compare combined, lexical-only, and signal-ablated rankings at identical K on frozen ranges.

## Project Constraints (from CLAUDE.md)

- Keep ctxhelm local-first, read-only, and source-safe for inventory, plans, traces, historical eval reports, and generated cards.
- AGENTS.md, MCP, and thin native rules/adapters remain the primary product surface; CLI is for setup/debug/automation.
- ctxhelm must not edit source code, run user project tests, install dependencies, or auto-commit user work.
- Keep the Rust workspace architecture and typed contracts unless measured evidence justifies changing them.
- New retrieval work should be checked against source-free historical evals; RefactoringMiner is the preferred large-history smoke when practical.
- Run `cargo test --workspace` before claiming implementation complete; run `cargo run -p ctxhelm -- --help` after CLI changes.
- Use typed serializable contracts, not formatted strings, for behavior surfaces.
- Preserve camelCase JSON public contracts and add fields compatibly where possible.
- Use `thiserror` for library/domain errors and `anyhow` only in the CLI boundary.
- Keep formatting/rendering at CLI/MCP boundaries; core/index/compiler APIs should return structured values.
- Do not add ad hoc debug logging to library functions; return structured reports or typed errors.

## Standard Stack

### Core

| Library/Tool | Version | Purpose | Why Standard |
|--------------|---------|---------|--------------|
| Rust workspace | rustc 1.87.0 / Cargo 1.87.0 | Existing implementation, tests, CLI, MCP server | Project is already a Rust 2021 Cargo workspace. |
| `serde` / `serde_json` | 1.0.228 / 1.0.149 | Additive public JSON contracts, eval reports, MCP structured content | Existing contracts use serde camelCase structs; serde supports defaults for missing fields. |
| `git` CLI | 2.45.1 | Frozen ranges, commit metadata, rename/delete labels, current diff | Existing history layer already shells out to git; official diff docs support name/status and rename detection. |
| `tempfile` | 3.27.0 | Deterministic historical eval fixtures and parent snapshots | Existing compiler eval tests use temp repos/worktrees. |

### Supporting

| Library/Tool | Version | Purpose | When to Use |
|--------------|---------|---------|-------------|
| `assert_cmd` / `predicates` | 2.2.2 / 3.1.4 | CLI compatibility tests | Use for output-shape and report rendering changes. |
| `ignore` | 0.4.25 | Safe inventory walking | Reuse for any current/historical inventory, not raw filesystem walks. |
| `blake3` | 1.8.5 | Source-free hashes and task/repo identities | Use where reports need stable identity without source text. |
| `uuid` | 1.23.1 | Plan, pack, trace IDs | Preserve existing ID conventions. |
| `tar` | bsdtar 3.8.1 | Historical worktree extraction | Existing eval uses `git archive | tar`; keep bounded by paths. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Existing lexical/symbol/git signal stack | Tantivy, SQLite, rayon, Tree-sitter, MCP SDK | Locked out unless eval evidence shows current stack cannot close observed gaps. |
| Git CLI status parsing | Porcelain status text parsing | Avoid. Use `git diff-tree --name-status -z --find-renames` style metadata to preserve rename/delete semantics and weird paths. |
| Source-free CLI/JSON reports | Visual dashboard | Deferred; Markdown/JSON reports are enough for this phase. |
| Local-only heuristics | Cloud embeddings/reranking | Deferred and contradicts default local-first constraint. |

**Installation:** No new dependency is recommended for the first implementation pass.

**Version verification:** Versions above were verified from `cargo --version`, `rustc --version`, `git --version`, `tar --version`, `cargo tree --workspace --depth 1`, and `Cargo.lock` on 2026-05-13. For Rust crates, treat `Cargo.lock` as the implementation version source instead of `npm view`.

## Architecture Patterns

### Recommended Project Structure

```text
crates/
  ctxhelm-core/src/contracts.rs        # Add public attribution/eval contract structs
  ctxhelm-compiler/src/ranking.rs      # New candidate merge, scoring, budgeted selection
  ctxhelm-compiler/src/planning.rs     # Call ranking, then project to ContextPlan
  ctxhelm-compiler/src/eval.rs         # Frozen ranges, labels, metrics, ablations
  ctxhelm-index/src/git.rs             # Name-status historical labels and bounded git reads
  ctxhelm/src/main.rs                  # Render source-free eval/gap report fields
  ctxhelm-mcp/src/*                    # Serialize additive plan fields without new tools
```

### Pattern 1: Candidate Layer Before Projection

**What:** Create internal ranked candidates keyed by path plus kind (`file`, `test`, `symbol`, `doc`, `config`, `commit`, `diff`). Each candidate carries role, total score, per-signal scores, source-free evidence, and a selected/not-selected reason.

**When to use:** Always inside `prepare_context_plan_with_paths_and_history` before writing `target_files` or `related_tests`.

**Example:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceAttribution {
    pub signal: RetrievalSignal,
    pub score: f32,
    pub reason_code: String,
    #[serde(default)]
    pub edge_label: Option<String>,
    #[serde(default)]
    pub commit_ids: Vec<String>,
}
```

Planner note: put public attribution structs in `ctxhelm-core` only when exposed through `ContextPlan`; keep score-merging helpers private in `ctxhelm-compiler::ranking`.

### Pattern 2: Fixed-Budget One-Hop Expansion

**What:** Rank seed candidates from anchors, lexical, symbol, current diff, docs/config matches, then expand one hop through dependency edges, related tests, and co-change hints. Every expanded candidate consumes the same final budget as lexical candidates.

**When to use:** Planning and eval. Do not recursively expand dependencies of dependencies in Phase 3.

**Example:**

```rust
let seeds = collect_seed_candidates(signals);
let expanded = expand_one_hop(seeds.keys(), graph, tests, history);
let ranked = rank_candidates(seeds.merge(expanded), RankingBudget { target_files: 8, tests: 5 });
let plan = project_context_plan(base_plan, ranked);
```

### Pattern 3: Source-Free Frozen Eval Range

**What:** Extend `HistoricalEvalOptions` / `HistoricalEvalReport` with budget, effective filters, signal config, and ref identity. Extend commit labels from `Vec<String>` to records with path, old path, change kind, role, and availability.

**When to use:** Every `ctxhelm eval history` run.

**Example:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalChangedPath {
    pub path: String,
    pub old_path: Option<String>,
    pub change_kind: ChangeKind,
    pub role: FileRole,
    pub label_scope: LabelScope,
    pub excluded_reason: Option<String>,
}
```

### Anti-Patterns to Avoid

- **Risk-flag-only evidence:** Dependency and co-change signals must affect ranked candidates, not just `riskFlags`.
- **Budget inflation:** Do not claim lift if combined ranking returns more context than lexical baseline.
- **Source-bearing eval:** Do not serialize commit subjects, task text, snippets, symbol signatures, or source excerpts in reports.
- **Current-inventory-only labels:** Deletes, renames, and historical-only files require historical snapshot/status metadata, not only the current inventory.
- **Broad parser migration first:** Parser upgrades follow measured gap categories, not the other way around.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rename/delete detection | Ad hoc comparison of path sets | `git diff-tree` / diff options with `--name-status`, `-z`, and rename detection | Git already defines status letters and rename similarity behavior. |
| JSON compatibility | Handwritten JSON strings | serde structs with additive fields and `#[serde(default)]` where needed | Matches existing contract tests and keeps old clients tolerant. |
| Retrieval eval metric names | Custom "quality" score only | Recall@K, Precision@K, MRR@K or MAP/nDCG, plus lift vs lexical | Standard IR metrics make fixed-budget comparisons interpretable. |
| Privacy filtering | Direct reads from historical worktrees | Existing safe inventory/source-read policy | Prevents generated/sensitive leakage and keeps diagnostics consistent. |
| Graph expansion | Recursive dependency crawler | One-hop expansion over existing dependency/test/history reports | Keeps runtime and context budget bounded. |
| Runtime acceleration | Immediate rayon/SQLite/Tantivy migration | Existing in-memory deterministic ranking until eval proves bottleneck | Locked phase scope requires measured justification. |

**Key insight:** The hard part is not finding more signals; ctxhelm already has graph, history, tests, symbols, lexical search, and diff anchors. The hard part is making those signals compete fairly under one budget and proving the result without leaking source or prompt text.

## Common Pitfalls

### Pitfall 1: Future Leakage In Historical Eval
**What goes wrong:** Eval builds labels or candidate inventory from the current repo, so a parent commit can "see" files that did not exist yet.
**Why it happens:** Current eval starts from current inventory paths and extracts parent snapshots for those paths.
**How to avoid:** Build labels from commit name-status metadata and classify against parent/head snapshots separately.
**Warning signs:** Historical-only or deleted files show as `unknown` with no explicit reason, or a parent snapshot contains future-only paths.

### Pitfall 2: Lift By Context Bloat
**What goes wrong:** Combined ranking beats lexical only because it returns files plus tests plus graph neighbors beyond lexical K.
**Why it happens:** `recommended_context_files` chains target files and tests; new signals can silently increase result length.
**How to avoid:** Compare top K over one combined context ranking for every run, including lexical-only and ablations.
**Warning signs:** `averageRecommendedContextFiles` rises while Recall@K improves.

### Pitfall 3: Attribution That Leaks Source
**What goes wrong:** Evidence reasons include symbol signatures, commit subjects, snippets, or task text.
**Why it happens:** It is tempting to reuse search reasons or commit titles directly in reports.
**How to avoid:** Use reason codes, signal kinds, paths, roles, counts, line ranges, edge labels, and commit ids only.
**Warning signs:** Serialized eval JSON contains user prompt text, commit subject text, or source code tokens.

### Pitfall 4: Rename/Delete Blind Spots
**What goes wrong:** Deletes vanish, renames look like unrelated add/delete, and historical-only files cannot be explained.
**Why it happens:** Current git sampling uses name-only output, `--no-renames`, and filters away deletes.
**How to avoid:** Capture `ChangeKind::{added, modified, deleted, renamed, copied, type_changed}` and `oldPath` from name-status output; count excluded generated/sensitive files by reason.
**Warning signs:** EVAL-02 fixtures fail to expose deleted labels or renamed old paths.

### Pitfall 5: Signal Ablations That Are Not Comparable
**What goes wrong:** Ablation runs differ in task set, refs, limit, or K.
**Why it happens:** Each eval mode samples commits independently.
**How to avoid:** Freeze commit samples once, then run lexical-only, combined, and each disabled-signal ranking over the same samples.
**Warning signs:** Reports show ablation metrics without the same `evalRangeId` and commit count.

## Code Examples

Verified patterns from current code and official docs:

### Additive ContextPlan Attribution

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetFile {
    pub path: String,
    pub reason: String,
    pub line_range: Option<LineRange>,
    pub confidence: f32,
    #[serde(default)]
    pub attribution: Vec<EvidenceAttribution>,
}
```

Source: existing `TargetFile`/`ContextPlan` contracts use serde camelCase; serde documents `#[serde(default)]` for missing input fields.

### Ranking Projection

```rust
let ranked = rank_context_candidates(repo_root, task, task_type, anchor_paths, ranking_options)?;
plan.target_files = ranked
    .selected_files()
    .map(TargetFile::from_ranked_candidate)
    .collect();
plan.related_tests = ranked
    .selected_tests()
    .map(RelatedTest::from_ranked_candidate)
    .collect();
```

Source: current planning already centralizes plan construction in `crates/ctxhelm-compiler/src/planning.rs`; replace the sequential symbol-then-lexical projection with one candidate ranking step.

### Git Name-Status Labels

```rust
git diff-tree --root --no-commit-id -r --name-status -z --find-renames <sha>
```

Source: official Git docs describe `--name-status` as showing changed names and status, `-z` as NUL-delimiting pathnames, and rename detection through diff-tree/diff options.

### Fixed-Budget Metric Formula

```rust
fn precision_at_k(relevant: &BTreeSet<String>, ranked: &[String], k: usize) -> f32 {
    let hits = ranked.iter().take(k).filter(|path| relevant.contains(*path)).count();
    hits as f32 / k.max(1) as f32
}

fn reciprocal_rank_at_k(relevant: &BTreeSet<String>, ranked: &[String], k: usize) -> f32 {
    ranked.iter().take(k)
        .position(|path| relevant.contains(path))
        .map(|zero_based| 1.0 / (zero_based + 1) as f32)
        .unwrap_or(0.0)
}
```

Source: standard search-evaluation definitions use top-k Precision, Recall, MRR, MAP, and nDCG.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Lexical-only target selection | Hybrid lexical, symbol, test, dependency, history, diff signals | Already present before Phase 3 | Signals exist but are not ranked together before projection. |
| Path-only historical labels | Status-aware changed-path labels | Phase 3 target | Required for deletes, renames, generated/sensitive exclusions, and historical-only files. |
| Recall-only eval | Recall@K plus Precision@K and MRR/MAP/nDCG | Phase 3 target | Makes ranking order and fixed-budget quality visible. |
| Single combined eval | Combined vs lexical-only plus signal ablations | Phase 3 target | Shows which signal helps or fails. |
| Broad parser-first upgrades | Gap-driven parser adapters | Locked Phase 3 scope | Avoids spending the phase on infrastructure before measured need. |

**Deprecated/outdated:**
- `--name-only --no-renames` for eval labels: insufficient for rename/delete handling.
- Treating co-change and dependency edges only as `riskFlags`: insufficient for RETR-02.
- Reporting only `topMissingFiles`: insufficient for DIAG-03 because it lacks signal-gap and repeated-family grouping.

## Open Questions

1. **Public field names for attribution**
   - What we know: Public JSON compatibility favors additive fields.
   - What's unclear: Whether to expose both `retrievalCandidates` and per-item `attribution`, or only per-item attribution.
   - Recommendation: Expose per-item `attribution` in `targetFiles`/`relatedTests` first; keep full candidate lists internal unless CLI/MCP users need debug JSON.

2. **MRR vs MAP/nDCG**
   - What we know: EVAL-03 requires MRR or equivalent ranking quality.
   - What's unclear: Multiple changed files per commit make MAP or nDCG more expressive than MRR alone.
   - Recommendation: Implement Precision@K, Recall@K, MRR@K, and MAP@K if small; MRR alone satisfies the requirement but MAP better explains multi-file commits.

3. **Frozen ranges for RefactoringMiner smoke**
   - What we know: RefactoringMiner is preferred when practical.
   - What's unclear: The exact stable base/head refs should be chosen during planning/execution against the local clone state.
   - Recommendation: Use deterministic in-repo fixtures for gates and add a documented RefactoringMiner smoke command that can be skipped only when the repo is absent.

4. **PROJECT.md availability**
   - What we know: The requested `PROJECT.md` file was not present at repo root during research.
   - What's unclear: Whether it was folded into generated `CLAUDE.md`.
   - Recommendation: Treat `CLAUDE.md` project block as authoritative project context for this phase.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust/Cargo | Workspace build and tests | Yes | rustc 1.87.0, cargo 1.87.0 | None needed |
| Git CLI | History labels, frozen refs, current diff | Yes | 2.45.1 | Degraded diagnostics only; eval gates requiring history should fail/skip explicitly |
| tar | Historical parent snapshot extraction | Yes | bsdtar 3.8.1 | Replace with Rust archive extraction only if tar becomes unavailable |
| Local filesystem | Inventory, temp fixtures, traces | Yes | macOS filesystem | None |
| RefactoringMiner repo | Large-repo smoke | Not verified in this research pass | Unknown | Skip smoke with documented reason; deterministic fixtures remain required |

**Missing dependencies with no fallback:**
- None for core Phase 3 implementation in this workspace.

**Missing dependencies with fallback:**
- RefactoringMiner large-repo smoke target was not probed here; planner should add a conditional smoke step rather than make it the only gate.

## Sources

### Primary (HIGH confidence)
- `CLAUDE.md` - project constraints, stack, conventions, architecture, validation rules.
- `.planning/phases/03-measured-retrieval-lift-eval-gates/03-CONTEXT.md` - locked Phase 3 decisions and deferred scope.
- `.planning/REQUIREMENTS.md` - Phase 3 requirement IDs and acceptance behavior.
- `crates/ctxhelm-core/src/contracts.rs` - current public contracts for plans, packs, traces, diagnostics.
- `crates/ctxhelm-compiler/src/planning.rs` - current sequential planning/risk-flag fusion behavior.
- `crates/ctxhelm-compiler/src/eval.rs` - current historical eval options, report fields, lexical baseline, and source-free trace behavior.
- `crates/ctxhelm-index/src/git.rs` - current co-change, diff, and historical commit sampling implementation.
- Git official docs: https://git-scm.com/docs/git-diff-tree and https://git-scm.com/docs/diff-options - name-status, NUL-delimited paths, rename/diff metadata.
- Serde official docs: https://serde.rs/attr-default.html - default values for missing fields.

### Secondary (MEDIUM confidence)
- Lucidworks Search Evaluation Metrics: https://doc.lucidworks.com/docs/lucidworks-search/06-metrics-and-analytics/evaluation-metrics - concise current definitions for Precision@K, Recall@K, MRR@K, MAP@K, and nDCG@K.

### Tertiary (LOW confidence)
- Prior ctxhelm memory notes were used only as routing/context hints; live repo files were treated as authoritative.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - verified from local Cargo workspace, lockfile, and installed tool versions.
- Architecture: HIGH - based on current source files and locked Phase 3 context.
- Pitfalls: HIGH - tied to observed current implementation gaps and official git/serde docs.
- Metrics: MEDIUM - standard IR formulas verified from current search-evaluation documentation; exact metric set still needs planner choice.

**Research date:** 2026-05-13
**Valid until:** 2026-06-12 for local architecture; 2026-05-20 for dependency/version assumptions.
