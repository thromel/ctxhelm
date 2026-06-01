---
phase: 03-measured-retrieval-lift-eval-gates
verified: 2026-05-13T15:53:31Z
status: passed
score: 5/5 must-haves verified
---

# Phase 3: Measured Retrieval Lift & Eval Gates Verification Report

**Phase Goal:** Users and maintainers can see ranked, attributed evidence that improves over lexical retrieval at fixed budgets and remains source-free in reports.
**Verified:** 2026-05-13T15:53:31Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Context planning ranks typed candidates for files, tests, symbols, docs, commits, and config using evidence and per-signal scores before projecting to `ContextPlan`. | VERIFIED | `TargetFile`, `RelatedTest`, and `ContextPlan` carry additive attribution/candidate fields in `crates/ctxhelm-core/src/contracts.rs:25`, `crates/ctxhelm-core/src/contracts.rs:36`, and `crates/ctxhelm-core/src/contracts.rs:195`. Required candidate kinds are explicit at `crates/ctxhelm-core/src/contracts.rs:47`. Planner calls `rank_candidates` before projection at `crates/ctxhelm-compiler/src/planning.rs:215`. Ranking materializes doc/config/commit candidates and tests assert the required kinds at `crates/ctxhelm-compiler/src/ranking.rs:808`. |
| 2 | Dependency edges, related tests, co-change hints, current-diff anchors, and symbol matches affect ranked targets with source-free attribution for every recommended file and test. | VERIFIED | Planning collects symbol, lexical, related-test, co-change, dependency, and current-diff signals before ranking at `crates/ctxhelm-compiler/src/planning.rs:85`, `crates/ctxhelm-compiler/src/planning.rs:95`, `crates/ctxhelm-compiler/src/planning.rs:124`, `crates/ctxhelm-compiler/src/planning.rs:147`, `crates/ctxhelm-compiler/src/planning.rs:153`, and `crates/ctxhelm-compiler/src/planning.rs:189`. Ranking converts those signals into weighted evidence at `crates/ctxhelm-compiler/src/ranking.rs:41`, `crates/ctxhelm-compiler/src/ranking.rs:69`, `crates/ctxhelm-compiler/src/ranking.rs:90`, `crates/ctxhelm-compiler/src/ranking.rs:128`, `crates/ctxhelm-compiler/src/ranking.rs:156`, and `crates/ctxhelm-compiler/src/ranking.rs:176`; attribution is copied to selected files/tests at `crates/ctxhelm-compiler/src/ranking.rs:362`. CLI and MCP tests assert attributed structured output. |
| 3 | Graph expansion stays budgeted and non-recursive by default, so retrieval lift is measured at fixed context budgets rather than through context bloat. | VERIFIED | Expansion seed paths are built once from anchors/symbol/lexical results at `crates/ctxhelm-compiler/src/planning.rs:136`; dependency edges are accepted only when one endpoint is a seed at `crates/ctxhelm-compiler/src/ranking.rs:128`. Selection applies fixed file/test budgets at `crates/ctxhelm-compiler/src/ranking.rs:209`. Tests prove one-hop non-recursion and fixed budgets at `crates/ctxhelm-compiler/src/ranking.rs:702` and `crates/ctxhelm-compiler/src/ranking.rs:756`. |
| 4 | Maintainer can run frozen historical eval ranges, including large-repo smokes, with reproducible refs, role-aware labels, rename/delete handling, lexical baselines, ranking metrics, and signal ablations. | VERIFIED | `HistoricalEvalOptions` includes limit, ranking budget, task type, target agent, base, and head at `crates/ctxhelm-compiler/src/eval.rs:22`. Reports include eval range id, repo id, effective filters, refs, ranking comparison, ablations, and role recall at `crates/ctxhelm-compiler/src/eval.rs:52` and `crates/ctxhelm-compiler/src/eval.rs:104`. Git history labels include change kind, old path, role, label scope, and exclusions at `crates/ctxhelm-index/src/git.rs:103`; sampling is bounded with `rev-list --max-count` and `diff-tree --name-status -z -M` at `crates/ctxhelm-index/src/git.rs:623`. Bounded smoke passed for this repo and `/Users/romel/Documents/GitHub/RefactoringMiner`. |
| 5 | Eval and checklist reports remain source-free and prompt-free while summarizing retrieval failures by path role, signal gap, and repeated missing-file family. | VERIFIED | `RetrievalGapSummary` groups by role, signal gap, path family, and examples at `crates/ctxhelm-compiler/src/eval.rs:94`; grouped gaps are built at `crates/ctxhelm-compiler/src/eval.rs:1095`. CLI history and checklist rendering includes grouped failures at `crates/ctxhelm/src/main.rs:699` and `crates/ctxhelm/src/main.rs:751`. Source-free guard tests reject source/prompt keys at `crates/ctxhelm/tests/cli_compat.rs:547`; the smoke script repeats the source-free check at `scripts/smoke-historical-eval.sh:77`. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/ctxhelm-core/src/contracts.rs` | Additive public candidate and attribution contracts | VERIFIED | Contains `RetrievalCandidateKind`, `RetrievalSignalKind`, scores, evidence, candidate structs, defaulted plan fields, compatibility/source-free tests. |
| `crates/ctxhelm-compiler/src/ranking.rs` | Candidate collection, signal fusion, one-hop expansion, fixed-budget selection | VERIFIED | Substantive implementation with signal weights, accumulation, attribution projection, deterministic ranking tests. |
| `crates/ctxhelm-compiler/src/planning.rs` | ContextPlan projection from ranked candidates | VERIFIED | Collects reports, computes anchors/current diff, invokes `rank_candidates`, applies `select_ranked_candidates`, preserves diagnostics/risk flags. |
| `crates/ctxhelm-index/src/git.rs` | Status-aware historical sampling | VERIFIED | Uses name-status records, rename detection, delete labels, safe/generated/sensitive/historical-only scopes, bounded rev traversal. |
| `crates/ctxhelm-compiler/src/eval.rs` | Frozen eval metadata, labels, metrics, ablations, gaps | VERIFIED | Computes fixed-K combined vs lexical metrics, role recall, ablations over same range id, grouped gaps. |
| `crates/ctxhelm/src/main.rs` | CLI eval/checklist rendering and `--budget` | VERIFIED | Wires `eval history --budget`, renders source-free metrics, ablations, grouped retrieval failures. |
| `crates/ctxhelm/tests/cli_compat.rs` | Binary compatibility checks | VERIFIED | Asserts retrieval candidates, attribution, eval metadata, ablations, source-free reports. |
| `crates/ctxhelm-mcp/src/lib.rs` | MCP compatibility checks | VERIFIED | `prepare_task` structured content exposes attributed targets/tests and retrieval candidates without new tools. |
| `scripts/smoke-historical-eval.sh` | Bounded source-free smoke validation | VERIFIED | Validates budget metadata, ranking comparison, ablations, grouped gaps, source-free fields, optional RefactoringMiner run. |

GSD artifact verification reported all 12 declared Phase 3 artifacts present and substantive. No artifact was missing, stubbed, or orphaned.

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `contracts.rs` | `planning.rs` | Additive `TargetFile`/`RelatedTest`/`ContextPlan` fields with defaults | VERIFIED | Manual check: contracts define serde-defaulted fields and planning/base literals initialize them. The GSD regex check for Plan 01 was a false negative because the plan pattern was double-escaped. |
| `planning.rs` | `ranking.rs` | Collect signals, rank candidates, project target files/tests | VERIFIED | `planning.rs` imports and calls `rank_candidates` and `select_ranked_candidates`. |
| `git.rs` | `eval.rs` | `HistoricalCommitSample.changed_paths` labels | VERIFIED | Eval clones `sample.changed_paths` into per-commit labels and metrics. |
| `eval.rs` | ranking outputs | Same ranked plan path feeds combined, lexical, and ablated eval rankings | VERIFIED | Eval replays `prepare_context_plan_with_paths_and_history`, extracts plan signals, computes fixed-budget comparison and ablations. |
| CLI/MCP surfaces | Compatibility tests and smoke script | Binary JSON and structuredContent contract checks | VERIFIED | CLI tests, MCP tests, and smoke script all passed. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `planning.rs` | `plan.retrieval_candidates`, `target_files`, `related_tests` | Live index reports: lexical, symbol, related tests, co-change, dependency, current diff | Yes | FLOWING |
| `ranking.rs` | `RankedSelection` | `RankingInput` from planner signal reports | Yes | FLOWING |
| `eval.rs` | `HistoricalEvalReport` | `historical_commit_samples`, parent worktrees, `prepare_context_plan`, lexical baseline search | Yes | FLOWING |
| `main.rs` | eval/checklist Markdown and JSON | `evaluate_historical_commits` and `list_eval_traces` | Yes | FLOWING |
| `scripts/smoke-historical-eval.sh` | JSON smoke report | `cargo run -p ctxhelm -- eval history --format json` | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Workspace validation | `cargo test --workspace` | 169 tests passed across CLI, compiler, core, index, MCP, and doc tests. | PASS |
| CLI help after CLI changes | `cargo run -p ctxhelm -- --help` | Listed `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards`, `eval`, `serve-mcp`. | PASS |
| Ranking and historical eval targeted tests | `cargo test -p ctxhelm-compiler ranking -- --nocapture && cargo test -p ctxhelm-compiler historical_eval -- --nocapture` | 5 ranking-related and 6 historical-eval-related tests passed. | PASS |
| History labels and MCP prepare_task targeted tests | `cargo test -p ctxhelm-index historical_commit -- --nocapture && cargo test -p ctxhelm-mcp prepare_task -- --nocapture` | 6 historical-commit and 6 MCP prepare_task tests passed. | PASS |
| Source-free historical eval smoke | `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=2 CTXHELM_SMOKE_BUDGET=10 bash scripts/smoke-historical-eval.sh` | Primary repo smoke passed; optional RefactoringMiner skipped because env was unset. | PASS |
| Large-repo RefactoringMiner smoke | `CTXHELM_SMOKE_REPO="$PWD" CTXHELM_SMOKE_LIMIT=1 CTXHELM_SMOKE_BUDGET=10 CTXHELM_REFACTORINGMINER_REPO=/Users/romel/Documents/GitHub/RefactoringMiner bash scripts/smoke-historical-eval.sh` | Primary and RefactoringMiner smokes passed with budget 10 and one bounded commit each. | PASS |
| Runtime dependency scope | `cargo tree --workspace --depth 1 > /tmp/ctxhelm-phase3-cargo-tree-verify.txt && ! rg "tantivy|rayon|rusqlite|notify|tree-sitter|mcp-sdk" /tmp/ctxhelm-phase3-cargo-tree-verify.txt` | No broad parser/search/runtime dependency was added. | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| DIAG-03 | Plans 04, 05 | Eval/checklist outputs summarize failures by role, signal gap, missing family | SATISFIED | `RetrievalGapSummary`, eval/checklist renderer, CLI tests, smoke script. |
| RETR-01 | Plans 01, 02 | Typed candidates for files/tests/symbols/docs/commits/config before projection | SATISFIED | Core contracts and ranking/planning flow. |
| RETR-02 | Plan 02 | Dependency/test/co-change/current-diff/symbol signals affect targets | SATISFIED | Ranking signal weights and planner signal collection. |
| RETR-03 | Plans 01, 02 | Recommended target/test attribution is source-free | SATISFIED | `RetrievalEvidence`, target/test attribution projection, CLI/MCP tests. |
| RETR-04 | Plan 02 | Graph expansion budgeted and non-recursive | SATISFIED | Seed-only dependency expansion and fixed-budget selection tests. |
| RETR-05 | Plans 04, 05 | Retrieval evaluated against lexical baseline at fixed budgets with lift/gap explanation | SATISFIED | `rankingComparison`, lexical baseline metrics, ablations, grouped gaps, smoke script. |
| EVAL-01 | Plan 03 | Frozen ranges with reproducible refs, limits, mode, reports | SATISFIED | `HistoricalEvalOptions`, `HistoricalEvalEffectiveFilters`, `HistoricalEvalRefs`, `evalRangeId`. |
| EVAL-02 | Plan 03 | Add/modify/delete/rename/generated/sensitive/docs/config/tests/source/historical-only labels | SATISFIED | `HistoricalChangedPath`, `ChangeKind`, `LabelScope`, name-status parsing tests. |
| EVAL-03 | Plan 04 | Recall/precision/MRR, role recall, test recommendation rate, lexical baseline, ablations | SATISFIED | `RankingMetrics`, `EvalComparison`, `SignalAblationResult`. |
| EVAL-04 | Plan 05 | Large-repo smoke evals without pathological full-worktree costs | SATISFIED | Bounded smoke script passed for RefactoringMiner with limit 1; history sampling uses `rev-list --max-count`. |
| EVAL-05 | Plans 03, 04, 05 | Source-free and prompt-free reports with actionable gap detail | SATISFIED | Source-free tests and smoke validation reject source/prompt fields. |
| PARS-01 | Plans 01, 02 | Parser-backed adapters can sit behind existing contracts | SATISFIED | Additive typed contracts and no CLI/MCP surface widening. |
| PARS-02 | Plan 04 | Parser improvements gated by observed gaps | SATISFIED | Gap summaries provide the gating surface; no broad parser migration happened. |
| PARS-03 | Plan 04 | Optional runtime upgrades require before/after metrics | SATISFIED | No Tantivy/rayon/SQLite/notify/tree-sitter/MCP SDK dependency found; metrics now exist for future comparison. |

No orphaned Phase 3 requirements were found: all Phase 3 IDs in `REQUIREMENTS.md` are claimed by at least one Phase 3 plan.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | - | - | - | Anti-pattern scan found no TODO/FIXME/placeholders/not-implemented markers in Phase 3 implementation files. `Vec::new()` matches are constructors, defaults, or empty-case handling backed by real data paths and tests, not stubs. |

### Human Verification Required

None for Phase 3 goal achievement. Visual or interactive client durability is Phase 4 scope, and the Phase 3 source-free CLI/MCP/report behavior is covered by automated checks.

### Gaps Summary

No blocking gaps found. Phase 3 achieved the goal: ctxhelm now has typed, attributed retrieval candidates before `ContextPlan` projection; non-lexical signals affect ranked output under fixed budgets; graph expansion is shallow and budgeted; historical eval reports are frozen, status-aware, source-free, and metric-rich; and bounded smoke validation works for both the local repo and RefactoringMiner.

---

_Verified: 2026-05-13T15:53:31Z_
_Verifier: Claude (gsd-verifier)_
