---
phase: 03-measured-retrieval-lift-eval-gates
plan: 03
type: execute
wave: 3
depends_on: [01, 02]
files_modified:
  - crates/ctxpack-index/src/git.rs
  - crates/ctxpack-index/src/lib.rs
  - crates/ctxpack-compiler/src/eval.rs
  - crates/ctxpack-compiler/src/lib.rs
autonomous: true
requirements: [EVAL-01, EVAL-02, EVAL-05]
must_haves:
  truths:
    - "Historical eval reports identify the frozen range, budget, mode, repo identity, and effective filters."
    - "Commit labels include source-free path status and role information for additions, modifications, deletes, renames, generated files, sensitive files, and historical-only files."
    - "Historical eval avoids future leakage by classifying paths against parent/head snapshots instead of current inventory only."
  artifacts:
    - path: "crates/ctxpack-index/src/git.rs"
      provides: "Status-aware historical commit sampling from git name-status metadata"
    - path: "crates/ctxpack-compiler/src/eval.rs"
      provides: "Frozen range metadata and role/status labels in eval reports"
  key_links:
    - from: "crates/ctxpack-index/src/git.rs"
      to: "crates/ctxpack-compiler/src/eval.rs"
      via: "HistoricalCommitSample changed path records"
      pattern: "changed_paths"
---

<objective>
Make historical eval labels reproducible and status-aware.

Purpose: Ranking metrics are only useful if the labels know whether paths were added, modified, deleted, renamed, generated, sensitive, current-only, or historical-only.
Output: Source-free historical labels and frozen-range metadata.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-CONTEXT.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-RESEARCH.md
@.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-02-SUMMARY.md
@.planning/phases/01-compatibility-guardrails-module-boundaries/01-compatibility-guardrails-module-boundaries-03-SUMMARY.md

<decision_trace>
- D-06: Historical eval fixtures must be deterministic and small; large repositories are smoke tests.
- D-07: Frozen eval ranges record base/head refs, limit, mode, budget, repo identity, and effective filters.
- D-08: Rename, delete, historical-only, generated, sensitive, tests, configs, docs, and source files are explicitly labeled.
- D-09: Reports stay source-free and prompt-free.
</decision_trace>

<interfaces>
Current index history sample:
```rust
pub struct HistoricalCommitSample {
    pub sha: String,
    pub parent_sha: Option<String>,
    pub title: String,
    pub safe_changed_files: Vec<String>,
    pub excluded_changed_file_count: usize,
}
```

Current eval options:
```rust
pub struct HistoricalEvalOptions {
    pub limit: usize,
    pub task_type: TaskType,
    pub target_agent: String,
    pub base: Option<String>,
    pub head: Option<String>,
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Replace path-only history samples with name-status records</name>
  <files>crates/ctxpack-index/src/git.rs, crates/ctxpack-index/src/lib.rs</files>
  <behavior>
    - Test 1: A rename commit yields one changed-path record with `changeKind=renamed`, `path`, and `oldPath`.
    - Test 2: A delete commit yields a deleted label instead of disappearing from the sample.
    - Test 3: Generated and sensitive changed paths are counted with exclusion reasons and source text is never serialized.
    - Test 4: `--limit` bounds git log/diff traversal before expensive per-commit processing.
  </behavior>
  <action>Use git name-status metadata with rename detection, such as `git diff-tree --name-status -z -M`, instead of name-only diffs per D-08. Add public index structs for `HistoricalChangedPath`, `ChangeKind`, `LabelScope`, and exclusion reason codes. Preserve `safe_changed_files` as a compatibility projection, but make the rich records the source of truth. Keep commit titles internal for task replay only and never expose them in serialized reports per D-09.</action>
  <verify>
    <automated>cargo test -p ctxpack-index historical_commit -- --nocapture</automated>
  </verify>
  <done>Index history samples retain rename/delete/status metadata, safe path projections, exclusion counts, and bounded git traversal.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add frozen eval range metadata and role/status labels</name>
  <files>crates/ctxpack-compiler/src/eval.rs, crates/ctxpack-compiler/src/lib.rs</files>
  <behavior>
    - Test 1: HistoricalEvalReport includes `evalRangeId`, `budget`, `effectiveFilters`, base/head refs, limit, mode, and repo id.
    - Test 2: Each commit eval includes changed-path labels with role, change kind, label scope, and excluded reason where applicable.
    - Test 3: Historical-only and deleted paths are labeled explicitly without leaking source or prompt text.
  </behavior>
  <action>Extend `HistoricalEvalOptions` with a fixed retrieval budget field defaulted at the CLI boundary later, and extend `HistoricalEvalReport`/`HistoricalCommitEval` with source-free range and label records per D-07/D-08. Build labels from the rich index sample records and classify against parent/head snapshots where needed; do not rely on current inventory alone. Re-export any new public eval structs from `ctxpack-compiler/src/lib.rs` when they appear in public report fields.</action>
  <verify>
    <automated>cargo test -p ctxpack-compiler historical_eval -- --nocapture</automated>
  </verify>
  <done>Historical eval JSON is reproducible, source-free, and explicit about role/status labels and effective filters.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack-index historical_commit -- --nocapture`
- `cargo test -p ctxpack-compiler historical_eval -- --nocapture`
- `cargo test -p ctxpack-index -p ctxpack-compiler`
</verification>

<success_criteria>
Plan 03 is complete when historical eval labels are status-aware, reproducible, and safe enough for fixed-budget ranking metrics.
</success_criteria>

<output>
After completion, create `.planning/phases/03-measured-retrieval-lift-eval-gates/03-measured-retrieval-lift-eval-gates-03-SUMMARY.md`
</output>
