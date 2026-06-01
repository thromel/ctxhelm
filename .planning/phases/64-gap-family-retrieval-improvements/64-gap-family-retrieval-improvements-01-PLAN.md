---
phase: 64
title: Gap-Family Retrieval Improvements
status: planned
requirements_addressed:
  - GAP-01
  - GAP-02
  - GAP-03
  - GAP-04
depends_on:
  - 61
  - 63
---

# Phase 64 Plan: Gap-Family Retrieval Improvements

<objective>
Fix at least one measured high-impact RefactoringMiner retrieval gap family with
before/after proof, without weakening protected evidence or test/source metric
separation.
</objective>

<threat_model>
- A broad ranking tweak hides candidate-generation gaps instead of fixing them.
- A RefactoringMiner improvement regresses ctxhelm or protected evidence.
- Test recall improvements mask source recall losses.
- Graph expansion recurses from weak semantic-only seeds and bloats context.
</threat_model>

<must_haves>
- Reuse the existing `ctxhelm eval benchmark` reports.
- Target a named gap family from Phase 63.
- Add focused tests for the retrieval behavior changed.
- Report source recall, test recall, protected evidence, runtime, and named gaps.
</must_haves>

<tasks>

<task id="64.1" name="Group measured gap families">
<read_first>
- `.planning/e2e/2026-05-30-phase63-reranker-fusion-promotion.md`
- `crates/ctxhelm-compiler/src/eval.rs`
- `docs/benchmarking.md`
</read_first>
<action>
Create a source-free gap-family work item list and choose the first target.
</action>
<verify>
- The chosen target maps to GAP-01 and GAP-02.
</verify>
<acceptance_criteria>
- At least one RefactoringMiner source gap family has a concrete fix hypothesis.
</acceptance_criteria>
</task>

<task id="64.2" name="Reproduce and inspect candidate absence">
<read_first>
- `.ctxhelm/e2e/phase63-default-report.json` if available
- `crates/ctxhelm-index/src/lib.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
</read_first>
<action>
Inspect why wrapper-family Java files are marked `no_candidate_signal`.
</action>
<verify>
- Use source-free query trace, path labels, and candidate signals.
</verify>
<acceptance_criteria>
- The fix targets candidate generation, not only final ordering.
</acceptance_criteria>
</task>

<task id="64.3" name="Implement targeted retrieval fix">
<read_first>
- `crates/ctxhelm-index/src/lib.rs`
- `crates/ctxhelm-compiler/src/planning.rs`
- `crates/ctxhelm-compiler/src/ranking.rs`
</read_first>
<action>
Add the smallest retrieval change that makes the selected gap family eligible
while preserving exact-seed and protected-evidence behavior.
</action>
<verify>
- Add focused unit tests for the new retrieval behavior.
- Check no MCP tool surface changed.
</verify>
<acceptance_criteria>
- The selected family improves in source-free before/after reports.
</acceptance_criteria>
</task>

<task id="64.4" name="Run before/after proof and update state">
<read_first>
- `.ctxhelm/e2e/phase62-default-config.json`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
</read_first>
<action>
Run the fixed two-repo corpus and commit a concise source-free E2E summary.
</action>
<verify>
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verify>
<acceptance_criteria>
- Phase 64 is completed only if the selected gap family improves and validation
  passes.
</acceptance_criteria>
</task>

</tasks>

<verification>
- focused retrieval tests
- two-repo benchmark before/after comparison
- `cargo test --workspace --no-fail-fast`
- `cargo run -p ctxhelm -- --help`
- `git diff --check`
</verification>

<success_criteria>
- A repeated RefactoringMiner gap family has measured improvement.
- Test recall remains separately reported.
- Protected evidence and seed-safe graph behavior remain intact.
</success_criteria>
